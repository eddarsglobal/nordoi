#!/usr/bin/env python3
"""Bounded read-only verification of the complete APP-051 local chain.
External plan, genesis, head and per-decision pins are required.
Historical structural verification does not attest historical live replay.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app051_exclusive_conflict_seal_successor"))
import successor_app051 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
native=prior.native
need=prior.need
SCHEMA="NORDOI_APP052_VERIFIED_EXCLUSIVE_CONFLICT_SEAL_CHAIN_V1"
PLAN="NORDOI_APP052_CHAIN_EXPECTATIONS_V1"
MAX_STEPS=64
MAX_PLAN=262144
FIELDS={"parent_sha256","child_sha256","plan_sha256","decision_sha256"}

def plan(path,pin,journal_id,anchor_pin,head_pin):
    for value in (pin,journal_id,anchor_pin,head_pin): transfer.pin(value)
    raw=registry.read(path,MAX_PLAN)
    need(transfer.sha(raw)==pin,"CHAIN_PLAN_PIN_MISMATCH")
    doc=json.loads(raw)
    need(type(doc) is dict and set(doc)=={"schema","journal_id","anchor_sha256","head_sha256","steps"}
         and doc["schema"]==PLAN and doc["journal_id"]==journal_id
         and doc["anchor_sha256"]==anchor_pin and doc["head_sha256"]==head_pin
         and raw==portable.serialize(doc),"CHAIN_PLAN_SCOPE_OR_CANONICAL_MISMATCH")
    steps=doc["steps"]
    need(type(steps) is list and 1<=len(steps)<=MAX_STEPS,"CHAIN_LENGTH_INVALID")
    current=anchor_pin
    seen={current}
    for step in steps:
        need(type(step) is dict and set(step)==FIELDS,"CHAIN_STEP_FIELDS_INVALID")
        for value in step.values(): transfer.pin(value)
        need(step["parent_sha256"]==current,"CHAIN_LINK_BROKEN")
        current=step["child_sha256"]
        need(current not in seen,"CHAIN_CYCLE")
        seen.add(current)
    need(current==head_pin,"CHAIN_HEAD_MISMATCH")
    return raw,steps

def inventory(root):
    names=set()
    for entry in root.iterdir():
        need(len(names)<MAX_STEPS,"CHAIN_REGISTRY_LIMIT")
        transfer.pin(entry.name)
        native.real_directory(entry,"CHAIN_ENTRY_INVALID")
        names.add(entry.name)
    return names

def snapshot(folder):
    need({p.name for p in folder.iterdir()}==set(prior.LIMITS)|{"START.json","COMPLETE.json"},
         "CHAIN_DECISION_LAYOUT_INVALID")
    files=prior.capture({name:folder/name for name in prior.LIMITS})
    for name in ("START.json","COMPLETE.json"):
        files[name]=registry.read(folder/name,prior.MAX_RECORD)
    return {name:transfer.sha(raw) for name,raw in files.items()}

def outcome(status,**extra):
    return {"schema":SCHEMA,"status":status,"scope":"SINGLE_STABLE_LOCAL_REGISTRY",
            "distributed_exclusivity":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
            "independent_historical_admission":"NOT_PROVEN","historical_live_replay":"NOT_ATTESTED",
            "freshness":"NOT_PROVEN","execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED",**extra}

def audit(successor_registry,plan_file,plan_pin,journal_id,anchor_pin,head_pin,journal=None):
    root=registry.root(successor_registry)
    prior.prior.outside(root,plan_file)
    live=None if journal is None else registry.root(journal)
    if live is not None:
        prior.prior.prior.separate(root,live)
        prior.prior.outside(live,plan_file)
    raw,steps=plan(plan_file,plan_pin,journal_id,anchor_pin,head_pin)
    expected={s["parent_sha256"] for s in steps}
    need(inventory(root)==expected,"CHAIN_ORPHAN_MISSING_OR_EXTRA_DECISION")
    records=[]
    observed={}
    previous=None
    for index,step in enumerate(steps):
        parent=step["parent_sha256"]
        child=step["child_sha256"]
        folder=root/parent
        before=snapshot(folder)
        need(before["COMPLETE.json"]==step["decision_sha256"],"CHAIN_DECISION_PIN_MISMATCH")
        _,old=prior.prior.load_seal(folder/"PARENT.json",parent,journal_id)
        _,new=prior.prior.load_seal(folder/"CHILD.json",child,journal_id)
        need(old["sequence"]==index and new["sequence"]==index+1,"CHAIN_SEQUENCE_MISMATCH")
        if index==0:
            need(old["parent_sha256"]==prior.prior.ZERO,"CHAIN_GENESIS_INVALID")
        else:
            need(old==previous,"CHAIN_SEAL_CONTINUITY_MISMATCH")
        rec=prior.stored(folder,prior.context(journal_id,parent),child,step["plan_sha256"],
                         live if index==len(steps)-1 else None)
        need(transfer.sha(portable.serialize(rec))==step["decision_sha256"],"CHAIN_DECISION_PIN_MISMATCH")
        need(snapshot(folder)==before,"CHAIN_DECISION_CHANGED")
        records.append(rec)
        observed[parent]=before
        previous=new
    need(inventory(root)==expected,"CHAIN_INVENTORY_CHANGED")
    for parent,before in observed.items():
        need(snapshot(root/parent)==before,"CHAIN_DECISION_CHANGED")
    need(registry.read(plan_file,MAX_PLAN)==raw,"CHAIN_PLAN_CHANGED")
    return outcome("EXCLUSIVE_CHAIN_VERIFIED",journal_id=journal_id,anchor_sha256=anchor_pin,
                   head_sha256=head_pin,plan_sha256=plan_pin,transition_count=len(records),
                   decisions_sha256=transfer.sha(portable.serialize(records)),
                   head_inventory_verified=live is not None)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    for n in ("successor-registry","plan-file"): p.add_argument("--"+n,type=Path,required=True)
    for n in ("plan-pin","journal-id","anchor-pin","head-pin"): p.add_argument("--"+n,required=True)
    p.add_argument("--journal",type=Path)
    try:
        out=audit(**vars(p.parse_args(argv)))
        rc=0
    except Exception as exc:
        out=outcome("UNCERTAIN_BLOCKED",reason=getattr(exc,"code","CHAIN_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
