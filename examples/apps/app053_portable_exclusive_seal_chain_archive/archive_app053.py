#!/usr/bin/env python3
"""Bounded canonical archive of APP-052 structural decisions, not live proofs."""
from __future__ import annotations
import argparse
import base64
import json
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app052_verified_exclusive_conflict_seal_chain"))
import chain_app052 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
native=prior.native
need=prior.need
SCHEMA="NORDOI_APP053_PORTABLE_EXCLUSIVE_SEAL_CHAIN_ARCHIVE_V1"
MAX_ARCHIVE=24*1024*1024
MAX_BYTES=16*1024*1024
LIMITS={**prior.prior.LIMITS,"START.json":prior.prior.MAX_RECORD,"COMPLETE.json":prior.prior.MAX_RECORD}

def collect(root,steps):
    expected={s["parent_sha256"] for s in steps}
    need(prior.inventory(root)==expected,"ARCHIVE_REGISTRY_INVENTORY_MISMATCH")
    files={}
    total=0
    for parent in sorted(expected):
        folder=root/parent
        need({p.name for p in folder.iterdir()}==set(LIMITS),"ARCHIVE_DECISION_LAYOUT_INVALID")
        members={}
        for name,limit in LIMITS.items():
            raw=registry.read(folder/name,limit)
            total+=len(raw)
            need(total<=MAX_BYTES,"ARCHIVE_BUDGET_EXCEEDED")
            members[name]=base64.b64encode(raw).decode("ascii")
        files[parent]=members
    need(prior.inventory(root)==expected,"ARCHIVE_REGISTRY_CHANGED")
    return files

def replay(doc,plan_pin,journal_id,anchor_pin,head_pin):
    for pin in (plan_pin,journal_id,anchor_pin,head_pin): transfer.pin(pin)
    need(type(doc) is dict and set(doc)=={"schema","source_plan","plan_sha256","journal_id",
         "anchor_sha256","head_sha256","decisions_sha256","transition_count","members"},
         "ARCHIVE_SCHEMA_INVALID")
    need(doc["schema"]==SCHEMA and doc["plan_sha256"]==plan_pin and doc["journal_id"]==journal_id
         and doc["anchor_sha256"]==anchor_pin and doc["head_sha256"]==head_pin,"ARCHIVE_TRUST_MISMATCH")
    transfer.pin(doc["decisions_sha256"])
    need(type(doc["transition_count"]) is int and 1<=doc["transition_count"]<=prior.MAX_STEPS,"ARCHIVE_COUNT_INVALID")
    raw_plan=portable.serialize(doc["source_plan"])
    need(len(raw_plan)<=prior.MAX_PLAN and transfer.sha(raw_plan)==plan_pin,"SOURCE_PLAN_PIN_MISMATCH")
    members=doc["members"]
    need(type(members) is dict and 1<=len(members)<=prior.MAX_STEPS,"ARCHIVE_MEMBER_COUNT_INVALID")
    with tempfile.TemporaryDirectory(prefix="nordoi_app053_") as tmp:
        root=Path(tmp).resolve()
        plan_file=root/"CHAIN.json"
        portable.safe_write(plan_file,raw_plan)
        _,steps=prior.plan(plan_file,plan_pin,journal_id,anchor_pin,head_pin)
        expected={s["parent_sha256"] for s in steps}
        need(set(members)==expected,"ARCHIVE_MISSING_OR_EXTRA_DECISION")
        successor=root/"successors"
        successor.mkdir(mode=0o700)
        total=0
        for parent in sorted(expected):
            transfer.pin(parent)
            group=members[parent]
            need(type(group) is dict and set(group)==set(LIMITS),"ARCHIVE_MEMBER_LAYOUT_INVALID")
            folder=successor/parent
            folder.mkdir(mode=0o700)
            for name,limit in LIMITS.items():
                value=group[name]
                need(type(value) is str and len(value)<=4*((limit+2)//3),"ARCHIVE_MEMBER_INVALID")
                data=base64.b64decode(value,validate=True)
                need(len(data)<=limit and base64.b64encode(data).decode("ascii")==value,"ARCHIVE_BASE64_INVALID")
                total+=len(data)
                need(total<=MAX_BYTES,"ARCHIVE_BUDGET_EXCEEDED")
                portable.safe_write(folder/name,data)
        audited=prior.audit(successor,plan_file,plan_pin,journal_id,anchor_pin,head_pin)
        need(audited["status"]=="EXCLUSIVE_CHAIN_VERIFIED"
             and audited["decisions_sha256"]==doc["decisions_sha256"]
             and audited["transition_count"]==doc["transition_count"]
             and collect(successor,steps)==members,"ARCHIVE_REPLAY_MISMATCH")
    return audited

def outcome(status,**extra):
    return {"schema":SCHEMA,"status":status,"original_locations_required":False,
            "head_inventory_verified":False,"historical_live_replay":"NOT_ATTESTED",
            "archive_encryption":"NONE","archive_signature":"NONE",
            "global_antirollback":"NOT_PROVEN","distributed_exclusivity":"NOT_PROVEN",
            "independent_historical_admission":"NOT_PROVEN","freshness":"NOT_PROVEN",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED",**extra}

def result(status,doc,pin):
    return outcome(status,archive_sha256=pin,plan_sha256=doc["plan_sha256"],
                   journal_id=doc["journal_id"],anchor_sha256=doc["anchor_sha256"],
                   head_sha256=doc["head_sha256"],decisions_sha256=doc["decisions_sha256"],
                   transition_count=doc["transition_count"],member_count=5*len(doc["members"]))

def archive(successor_registry,plan_file,plan_pin,journal_id,anchor_pin,head_pin,output):
    root=registry.root(successor_registry)
    native.real_directory(output.parent,"ARCHIVE_OUTPUT_PARENT_INVALID")
    dest=output.parent.resolve(strict=True)/output.name
    prior.prior.prior.outside(root,dest)
    need(not registry.present(dest),"ARCHIVE_OUTPUT_EXISTS")
    raw_plan,steps=prior.plan(plan_file,plan_pin,journal_id,anchor_pin,head_pin)
    audited=prior.audit(root,plan_file,plan_pin,journal_id,anchor_pin,head_pin)
    members=collect(root,steps)
    doc={"schema":SCHEMA,"source_plan":json.loads(raw_plan),"plan_sha256":plan_pin,
         "journal_id":journal_id,"anchor_sha256":anchor_pin,"head_sha256":head_pin,
         "decisions_sha256":audited["decisions_sha256"],"transition_count":audited["transition_count"],
         "members":members}
    replay(doc,plan_pin,journal_id,anchor_pin,head_pin)
    need(collect(root,steps)==members and prior.audit(root,plan_file,plan_pin,journal_id,anchor_pin,head_pin)==audited,
         "SOURCE_CHAIN_CHANGED")
    raw=portable.serialize(doc)
    need(len(raw)<=MAX_ARCHIVE,"ARCHIVE_TOO_LARGE")
    portable.safe_write(dest,raw)
    need(registry.read(dest,MAX_ARCHIVE)==raw,"ARCHIVE_PUBLICATION_MISMATCH")
    return result("EXCLUSIVE_CHAIN_ARCHIVED",doc,transfer.sha(raw))

def verify(output,archive_pin,plan_pin,journal_id,anchor_pin,head_pin):
    transfer.pin(archive_pin)
    raw=registry.read(output,MAX_ARCHIVE)
    need(transfer.sha(raw)==archive_pin,"ARCHIVE_PIN_MISMATCH")
    doc=json.loads(raw)
    need(raw==portable.serialize(doc),"ARCHIVE_NOT_CANONICAL")
    replay(doc,plan_pin,journal_id,anchor_pin,head_pin)
    need(registry.read(output,MAX_ARCHIVE)==raw,"ARCHIVE_CHANGED_DURING_REPLAY")
    return result("EXCLUSIVE_CHAIN_ARCHIVE_VERIFIED",doc,archive_pin)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("archive","verify"):
        a=sub.add_parser(action)
        a.add_argument("--output",type=Path,required=True)
        for n in ("plan-pin","journal-id","anchor-pin","head-pin"): a.add_argument("--"+n,required=True)
        if action=="archive":
            for n in ("successor-registry","plan-file"): a.add_argument("--"+n,type=Path,required=True)
        else: a.add_argument("--archive-pin",required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"archive":archive,"verify":verify}[action](**args)
        rc=0
    except Exception as exc:
        out=outcome("UNCERTAIN_BLOCKED",reason=getattr(exc,"code","ARCHIVE_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
