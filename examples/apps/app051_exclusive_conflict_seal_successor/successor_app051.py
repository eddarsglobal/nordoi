#!/usr/bin/env python3
"""Exclusive local admission of a verified APP-050 successor.
Only cooperating processes using the same stable registry are covered.
Offline audit proves the retained structural transition, not current journal state.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app050_conflict_journal_seal"))
import seal_app050 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
native=prior.native
need=prior.need
SCHEMA="NORDOI_APP051_EXCLUSIVE_CONFLICT_SEAL_SUCCESSOR_V1"
MAX_RECORD=32768
LIMITS={"PARENT.json":prior.MAX_SEAL,"CHILD.json":prior.MAX_SEAL,"PLAN.json":prior.MAX_PLAN}

def context(journal_id,parent_pin):
    for pin in (journal_id,parent_pin): transfer.pin(pin)
    need(parent_pin!=prior.ZERO,"INVALID_PARENT_PIN")
    return {"schema":SCHEMA,"journal_id":journal_id,"parent_sha256":parent_pin}

def capture(paths):
    return {name:registry.read(paths[name],limit) for name,limit in LIMITS.items()}

def checked(files,ctx,child_pin,plan_pin,journal=None):
    for pin in (child_pin,plan_pin): transfer.pin(pin)
    with tempfile.TemporaryDirectory(prefix="nordoi_app051_") as tmp:
        root=Path(tmp).resolve()
        for name,raw in files.items(): portable.safe_write(root/name,raw)
        _,child=prior.load_seal(root/"CHILD.json",child_pin,ctx["journal_id"])
        need(child["plan_sha256"]==plan_pin,"CHILD_PLAN_MISMATCH")
        _,expected=prior.external_plan(root/"PLAN.json",plan_pin,ctx["journal_id"])
        need(set(expected)==set(child["entries"]),"CHILD_PLAN_INVENTORY_MISMATCH")
        link=prior.transition(root/"PARENT.json",ctx["parent_sha256"],root/"CHILD.json",child_pin,ctx["journal_id"])
        if journal is not None:
            audited=prior.audit(journal,root/"PLAN.json",plan_pin,ctx["journal_id"],root/"CHILD.json",child_pin)
            need(audited["status"]=="JOURNAL_SEAL_VERIFIED" and audited["record"]==child,"LIVE_CHILD_NOT_VERIFIED")
    need(link["status"]=="SEAL_TRANSITION_VERIFIED","TRANSITION_NOT_VERIFIED")
    return {"schema":SCHEMA,"status":"SUCCESSOR_ADMITTED_ONCE","context":ctx,
            "child_sha256":child_pin,"plan_sha256":plan_pin,"sequence":child["sequence"],
            "added":link["added"],"members":{n:transfer.sha(v) for n,v in files.items()},
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def stored(folder,ctx,child_pin=None,plan_pin=None,journal=None):
    native.real_directory(folder,"SUCCESSOR_MARKER_INVALID")
    need({p.name for p in folder.iterdir()}==set(LIMITS)|{"START.json","COMPLETE.json"},"SUCCESSOR_INCOMPLETE")
    start=registry.read(folder/"START.json",MAX_RECORD)
    raw=registry.read(folder/"COMPLETE.json",MAX_RECORD)
    need(start==portable.serialize(ctx),"SUCCESSOR_CONTEXT_MISMATCH")
    rec=json.loads(raw)
    need(type(rec) is dict,"SUCCESSOR_RECEIPT_INVALID")
    if child_pin is not None: need(rec["child_sha256"]==child_pin,"ADMITTED_CHILD_PIN_MISMATCH")
    if plan_pin is not None: need(rec["plan_sha256"]==plan_pin,"ADMITTED_PLAN_PIN_MISMATCH")
    paths={name:folder/name for name in LIMITS}
    files=capture(paths)
    verified=checked(files,ctx,rec["child_sha256"],rec["plan_sha256"],journal)
    need(raw==portable.serialize(verified),"SUCCESSOR_RECEIPT_MISMATCH")
    need(capture(paths)==files and registry.read(folder/"START.json",MAX_RECORD)==start
         and registry.read(folder/"COMPLETE.json",MAX_RECORD)==raw,"SUCCESSOR_CHANGED_DURING_AUDIT")
    return verified

def outcome(status,**extra):
    return {"schema":SCHEMA,"status":status,"local_exclusivity_scope":"SAME_STABLE_REGISTRY_COOPERATING_PROCESSES",
            "distributed_exclusivity":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
            "independent_historical_admission":"NOT_PROVEN","crash_durability":"UNCLAIMED",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED",**extra}

def admit(successor_registry,journal,parent,parent_pin,child,child_pin,plan,plan_pin,journal_id):
    ctx=context(journal_id,parent_pin)
    root=registry.root(successor_registry)
    live=registry.root(journal)
    prior.prior.separate(root,live)
    paths={"PARENT.json":parent,"CHILD.json":child,"PLAN.json":plan}
    for p in paths.values():
        prior.outside(root,p)
        prior.outside(live,p)
    files=capture(paths)
    record=checked(files,ctx,child_pin,plan_pin,live)
    need(capture(paths)==files,"SUCCESSOR_SOURCE_CHANGED")
    folder=root/parent_pin
    try:
        folder.mkdir(mode=0o700)
    except FileExistsError:
        old=stored(folder,ctx)
        status="DUPLICATE_SUCCESSOR_BLOCKED" if old["child_sha256"]==child_pin else "COMPETING_SUCCESSOR_BLOCKED"
        return outcome(status,accepted_child_sha256=old["child_sha256"],candidate_child_sha256=child_pin)
    portable.safe_write(folder/"START.json",portable.serialize(ctx))
    for name,raw in files.items(): portable.safe_write(folder/name,raw)
    portable.safe_write(folder/"COMPLETE.json",portable.serialize(record))
    need(stored(folder,ctx,child_pin,plan_pin,live)==record and capture(paths)==files,"SUCCESSOR_PUBLICATION_NOT_VERIFIED")
    return outcome("SUCCESSOR_ADMITTED_ONCE",record=record,live_inventory_verified=True)

def audit(successor_registry,parent_pin,child_pin,plan_pin,journal_id,journal=None):
    ctx=context(journal_id,parent_pin)
    root=registry.root(successor_registry)
    live=None if journal is None else registry.root(journal)
    if live is not None: prior.prior.separate(root,live)
    rec=stored(root/parent_pin,ctx,child_pin,plan_pin,live)
    return outcome("SUCCESSOR_VERIFIED",record=rec,live_inventory_verified=live is not None)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("admit","audit"):
        a=sub.add_parser(action)
        a.add_argument("--successor-registry",type=Path,required=True)
        for n in ("parent-pin","child-pin","plan-pin","journal-id"): a.add_argument("--"+n,required=True)
        a.add_argument("--journal",type=Path,required=action=="admit")
        if action=="admit":
            for n in ("parent","child","plan"): a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"admit":admit,"audit":audit}[action](**args)
        rc=0 if out["status"] in ("SUCCESSOR_ADMITTED_ONCE","SUCCESSOR_VERIFIED") else 3
    except Exception as exc:
        out=outcome("UNCERTAIN_BLOCKED",reason=getattr(exc,"code","SUCCESSOR_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
