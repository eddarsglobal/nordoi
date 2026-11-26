#!/usr/bin/env python3
"""Exclusive local successor decision for an APP-040 verified transition.

All participating processes must share one stable trusted registry.
No distributed consensus, global head, rollback or crash durability guarantee.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app040_journal_seal_continuity"))
import continuity_app040 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
native=prior.prior.native
need=prior.need
SCHEMA="NORDOI_APP041_EXCLUSIVE_SEAL_SUCCESSOR_V1"
MAX_RECORD=65536

def root(successors,args):
    dest=registry.root(successors)
    for key in ("registry_path","journal"):
        p=registry.root(args[key])
        need(p!=dest and p not in dest.parents and dest not in p.parents,"SUCCESSOR_REGISTRY_OVERLAP")
    for key in ("old_seal","new_seal","old_contexts","new_contexts"):
        p=args[key].resolve(strict=True)
        need(dest!=p and dest not in p.parents,"SUCCESSOR_INPUT_OVERLAP")
    return dest

def result(status,old_pin,**extra):
    return {"schema":SCHEMA,"status":status,"previous_seal_sha256":old_pin,
            "scope":"SINGLE_TRUSTED_LOCAL_REGISTRY","distributed_fork_prevention":"NOT_PROVEN",
            "global_antirollback":"NOT_PROVEN","global_head":"NOT_IMPLEMENTED",
            "crash_durability":"UNCLAIMED","execution_authorized":False,
            "reservation_release_authorized":False,"nonce_reset_authorized":False,
            "automatic_retry":False,"production":"NOT_AUTHORIZED",**extra}

def decision(doc,raw):
    return {"schema":SCHEMA,"status":"SUCCESSOR_ACCEPTED_ONCE",
            "previous_seal_sha256":doc["previous_seal_sha256"],
            "successor_seal_sha256":doc["successor_seal_sha256"],
            "transition_sha256":transfer.sha(raw),"execution_authorized":False,
            "production":"NOT_AUTHORIZED"}

def start(old_pin,new_pin,transition_pin):
    return portable.serialize({"schema":SCHEMA,"previous_seal_sha256":old_pin,
           "successor_seal_sha256":new_pin,"transition_sha256":transition_pin})

def stored(directory,old_pin):
    native.real_directory(directory,"SUCCESSOR_MARKER_INVALID")
    need({p.name for p in directory.iterdir()}=={"START.json","TRANSITION.json","COMPLETE.json"},
         "SUCCESSOR_INCOMPLETE_OR_INVALID")
    raw=registry.read(directory/"TRANSITION.json",prior.MAX_TRANSITION_BYTES)
    first=registry.read(directory/"START.json",MAX_RECORD)
    last=registry.read(directory/"COMPLETE.json",MAX_RECORD)
    doc=json.loads(raw)
    need(type(doc) is dict and doc.get("schema")==prior.SCHEMA
         and doc.get("status")=="APPEND_ONLY_TRANSITION"
         and doc.get("previous_seal_sha256")==old_pin and raw==portable.serialize(doc),
         "STORED_TRANSITION_INVALID")
    transfer.pin(doc["successor_seal_sha256"])
    rec=decision(doc,raw)
    need(first==start(old_pin,rec["successor_seal_sha256"],rec["transition_sha256"])
         and last==portable.serialize(rec),"SUCCESSOR_RECORD_MISMATCH")
    need(registry.read(directory/"TRANSITION.json",prior.MAX_TRANSITION_BYTES)==raw
         and registry.read(directory/"START.json",MAX_RECORD)==first
         and registry.read(directory/"COMPLETE.json",MAX_RECORD)==last,
         "SUCCESSOR_RECORD_CHANGED")
    return rec,raw

def candidate(transition_pin,args):
    transfer.pin(transition_pin)
    doc,raw=prior.transition(**args)
    need(transfer.sha(raw)==transition_pin,"CANDIDATE_TRANSITION_PIN_MISMATCH")
    return doc,raw

def accept(successors,transition_pin,**args):
    dest=root(successors,args)
    doc,raw=candidate(transition_pin,args)
    old_pin=args["old_pin"]
    new_pin=args["new_pin"]
    need(root(successors,args)==dest,"SUCCESSOR_REGISTRY_CHANGED")
    directory=dest/old_pin
    try:
        directory.mkdir(mode=0o700)
    except FileExistsError:
        existing,old_raw=stored(directory,old_pin)
        if existing["successor_seal_sha256"]==new_pin:
            need(old_raw==raw,"SUCCESSOR_TRANSITION_CONTRADICTION")
            return result("DUPLICATE_SUCCESSOR_BLOCKED",old_pin,record=existing)
        return result("ALTERNATIVE_SUCCESSOR_BLOCKED",old_pin,record=existing,
                      rejected_successor_sha256=new_pin)
    # Permanent exclusive marker precedes writes; no release even on failure.
    portable.safe_write(directory/"START.json",start(old_pin,new_pin,transition_pin))
    portable.safe_write(directory/"TRANSITION.json",raw)
    rec=decision(doc,raw)
    portable.safe_write(directory/"COMPLETE.json",portable.serialize(rec))
    need(stored(directory,old_pin)==(rec,raw),"SUCCESSOR_PUBLICATION_NOT_VERIFIED")
    return result("SUCCESSOR_ACCEPTED_ONCE",old_pin,record=rec)

def audit(successors,transition_pin,**args):
    dest=root(successors,args)
    doc,raw=candidate(transition_pin,args)
    rec,captured=stored(dest/args["old_pin"],args["old_pin"])
    need(captured==raw and rec==decision(doc,raw),"AUDIT_EXPECTED_SUCCESSOR_MISMATCH")
    return result("SUCCESSOR_DECISION_VERIFIED",args["old_pin"],record=rec)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("accept","audit"):
        a=sub.add_parser(action)
        for n in ("successors","registry-path","journal","old-seal","new-seal","old-contexts","new-contexts"):
            a.add_argument("--"+n,type=Path,required=True)
        for n in ("old-pin","new-pin","old-contexts-pin","new-contexts-pin","transition-pin"):
            a.add_argument("--"+n,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"accept":accept,"audit":audit}[action](**args)
        rc=0 if out["status"] in ("SUCCESSOR_ACCEPTED_ONCE","SUCCESSOR_DECISION_VERIFIED") else 3
    except Exception as exc:
        out=result("UNCERTAIN_BLOCKED",args.get("old_pin"),reason=getattr(exc,"code","SUCCESSOR_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
