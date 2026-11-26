#!/usr/bin/env python3
"""Read-only audit of a bounded local successor chain under external trust pins.

Each historical step requires its corresponding evidence snapshot. No consensus,
freshness, hostile filesystem or global antirollback guarantee.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app041_exclusive_seal_successor"))
import successor_app041 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
SCHEMA="NORDOI_APP042_VERIFIED_SEAL_CHAIN_V1"
PLAN_SCHEMA="NORDOI_APP042_CHAIN_PLAN_V1"
MAX_STEPS=64
MAX_PLAN=262144
PATHS={"registry_path","journal","old_seal","new_seal","old_contexts","new_contexts"}
PINS={"old_pin","new_pin","old_contexts_pin","new_contexts_pin","transition_pin"}

def evidence_path(root,value):
    need(type(value) is str and 0<len(value)<=1024 and "\\" not in value and ":" not in value,
         "EVIDENCE_PATH_INVALID")
    parts=value.split("/")
    need(all(p not in ("",".","..") for p in parts),"EVIDENCE_PATH_INVALID")
    p=root
    for part in parts:
        p=p/part
        need(not p.is_symlink(),"EVIDENCE_SYMLINK_INVALID")
    resolved=p.resolve(strict=True)
    need(root in resolved.parents,"EVIDENCE_PATH_ESCAPE")
    return resolved

def plan(path,pin):
    transfer.pin(pin)
    raw=registry.read(path,MAX_PLAN)
    need(transfer.sha(raw)==pin,"CHAIN_PLAN_PIN_MISMATCH")
    obj=json.loads(raw)
    need(type(obj) is dict and set(obj)=={"schema","steps"} and obj["schema"]==PLAN_SCHEMA,
         "CHAIN_PLAN_SCHEMA_INVALID")
    steps=obj["steps"]
    need(type(steps) is list and 1<=len(steps)<=MAX_STEPS,"CHAIN_LENGTH_INVALID")
    need(raw==portable.serialize(obj),"CHAIN_PLAN_NOT_CANONICAL")
    for step in steps:
        need(type(step) is dict and set(step)==PATHS|PINS,"CHAIN_STEP_FIELDS_INVALID")
        for k in PINS: transfer.pin(step[k])
    return steps,raw

def inventory(root):
    entries=prior.prior.prior.children(root,MAX_STEPS)
    names=[]
    for p in entries:
        transfer.pin(p.name)
        prior.native.real_directory(p,"CHAIN_DECISION_DIRECTORY_INVALID")
        names.append(p.name)
    return names

def audit(successors,evidence_root,plan_file,plan_pin,anchor_pin):
    transfer.pin(anchor_pin)
    root=registry.root(successors)
    evidence=registry.root(evidence_root)
    need(root!=evidence and root not in evidence.parents,"CHAIN_ROOT_OVERLAP")
    steps,raw=plan(plan_file,plan_pin)
    current=anchor_pin
    seen={current}
    expected=[]
    previous_context=None
    for step in steps:
        need(step["old_pin"]==current,"CHAIN_LINK_BROKEN")
        need(previous_context is None or step["old_contexts_pin"]==previous_context,"CHAIN_CONTEXT_LINK_BROKEN")
        need(step["new_pin"] not in seen,"CHAIN_CYCLE")
        expected.append(current)
        current=step["new_pin"]
        previous_context=step["new_contexts_pin"]
        seen.add(current)
    need(inventory(root)==sorted(expected),"CHAIN_ORPHAN_MISSING_OR_EXTRA_DECISION")
    records=[]
    for step in steps:
        args={k:evidence_path(evidence,step[k]) if k in PATHS else step[k] for k in step}
        verified=prior.audit(successors=root,**args)
        need(verified["status"]=="SUCCESSOR_DECISION_VERIFIED","CHAIN_STEP_NOT_VERIFIED")
        rec=verified["record"]
        need(rec["previous_seal_sha256"]==step["old_pin"]
             and rec["successor_seal_sha256"]==step["new_pin"]
             and rec["transition_sha256"]==step["transition_pin"],"CHAIN_DECISION_MISMATCH")
        records.append(rec)
    need(inventory(root)==sorted(expected),"CHAIN_INVENTORY_CHANGED")
    for rec in records:
        captured,_=prior.stored(root/rec["previous_seal_sha256"],rec["previous_seal_sha256"])
        need(captured==rec,"CHAIN_DECISION_CHANGED")
    need(plan(plan_file,plan_pin)==(steps,raw),"CHAIN_PLAN_CHANGED")
    return {"schema":SCHEMA,"status":"LOCAL_CHAIN_VERIFIED","anchor_sha256":anchor_pin,
            "head_sha256":current,"transition_count":len(records),
            "decisions_sha256":transfer.sha(portable.serialize(records)),
            "plan_sha256":plan_pin,"scope":"SINGLE_TRUSTED_LOCAL_REGISTRY",
            "historical_snapshots_required":True,"global_antirollback":"NOT_PROVEN",
            "distributed_consensus":"NOT_PROVEN","freshness":"NOT_PROVEN",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    for n in ("successors","evidence-root","plan-file"):
        p.add_argument("--"+n,type=Path,required=True)
    for n in ("plan-pin","anchor-pin"): p.add_argument("--"+n,required=True)
    try:
        out=audit(**vars(p.parse_args(argv)))
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED","reason":getattr(exc,"code","CHAIN_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
