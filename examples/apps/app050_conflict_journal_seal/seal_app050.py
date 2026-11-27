#!/usr/bin/env python3
"""Canonical, externally pinned APP-049 inventory seals and append-only transitions.
Stable local filesystem, cooperating processes, trusted external plan and seal pins.
No signature, trusted clock, independent completeness or global rollback guarantee.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app049_admission_conflict_journal"))
import conflict_app049 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
native=prior.native
SCHEMA="NORDOI_APP050_CONFLICT_JOURNAL_SEAL_V1"
PLAN="NORDOI_APP050_EXTERNAL_EXPECTATIONS_V1"
MAX_ENTRIES=128
MAX_PLAN=2*1024*1024
MAX_SEAL=1024*1024
ZERO="0"*64
TRUST={"sender","receiver","nonce","key_pin","archive_pin","source_plan_pin","anchor_pin","head_pin"}
PINS={"receiver_key_pin","original_record_pin","package_pin","receipt_pin","conflicting_ack_pin"}

def canonical(path,pin,limit):
    transfer.pin(pin)
    raw=registry.read(path,limit)
    need(transfer.sha(raw)==pin,"EXTERNAL_PIN_MISMATCH")
    doc=json.loads(raw)
    need(type(doc) is dict and raw==portable.serialize(doc),"NONCANONICAL_DOCUMENT")
    return raw,doc

def external_plan(path,pin,journal_id):
    transfer.pin(journal_id)
    raw,doc=canonical(path,pin,MAX_PLAN)
    need(set(doc)=={"schema","journal_id","entries"} and doc["schema"]==PLAN
         and doc["journal_id"]==journal_id,"PLAN_SCOPE_MISMATCH")
    entries=doc["entries"]
    need(type(entries) is dict and len(entries)<=MAX_ENTRIES,"PLAN_ENTRY_LIMIT")
    validated={}
    for ident,args in entries.items():
        transfer.pin(ident)
        need(type(args) is dict and set(args)==TRUST|PINS,"PLAN_EXPECTATION_INVALID")
        trust={k:args[k] for k in TRUST}
        ctx=prior.prior.context(args["receiver_key_pin"],**trust)
        start=prior.statement(ctx,args["original_record_pin"],args["package_pin"],
                              args["receipt_pin"],args["conflicting_ack_pin"])
        need(prior.identity(start)==ident,"PLAN_IDENTITY_MISMATCH")
        validated[ident]=(start,args["receiver_key_pin"],trust)
    return raw,validated

def names(root):
    result=set()
    for entry in root.iterdir():
        need(len(result)<MAX_ENTRIES,"JOURNAL_ENTRY_LIMIT")
        transfer.pin(entry.name)
        native.real_directory(entry,"JOURNAL_ENTRY_NOT_DIRECTORY")
        result.add(entry.name)
    return result

def inventory(root,expected):
    need(names(root)==set(expected),"JOURNAL_INVENTORY_MISMATCH")
    result={}
    for ident in sorted(expected):
        start,key,trust=expected[ident]
        record=prior.stored(root/ident,start,key,trust)
        need(record["conflict_id"]==ident,"JOURNAL_RECORD_ID_MISMATCH")
        result[ident]=transfer.sha(portable.serialize(record))
    need(names(root)==set(expected),"JOURNAL_INVENTORY_CHANGED")
    for ident,pin in result.items():
        need(transfer.sha(registry.read(root/ident/"COMPLETE.json",prior.MAX_RECORD))==pin,
             "JOURNAL_RECORD_CHANGED")
    return result

def load_seal(path,pin,journal_id):
    raw,doc=canonical(path,pin,MAX_SEAL)
    need(set(doc)=={"schema","journal_id","sequence","parent_sha256","plan_sha256","entries","execution_authorized","production"}
         and doc["schema"]==SCHEMA and doc["journal_id"]==journal_id
         and doc["execution_authorized"] is False and doc["production"]=="NOT_AUTHORIZED","SEAL_SCOPE_INVALID")
    transfer.pin(journal_id)
    transfer.pin(doc["parent_sha256"])
    transfer.pin(doc["plan_sha256"])
    need(type(doc["sequence"]) is int and 0<=doc["sequence"]<2**63,"SEAL_SEQUENCE_INVALID")
    need((doc["sequence"]==0)==(doc["parent_sha256"]==ZERO),"SEAL_PARENT_INVALID")
    need(type(doc["entries"]) is dict and len(doc["entries"])<=MAX_ENTRIES,"SEAL_ENTRY_LIMIT")
    for ident,digest in doc["entries"].items():
        transfer.pin(ident)
        transfer.pin(digest)
    return raw,doc

def extension(previous,child,parent_pin):
    need(child["journal_id"]==previous["journal_id"] and child["parent_sha256"]==parent_pin
         and child["sequence"]==previous["sequence"]+1,"SEAL_LINK_MISMATCH")
    old,new=previous["entries"],child["entries"]
    need(len(new)>len(old) and all(new.get(k)==v for k,v in old.items()),"NOT_STRICT_APPEND_ONLY")

def outcome(status,**extra):
    return {"schema":SCHEMA,"status":status,"global_antirollback":"NOT_PROVEN",
            "independent_completeness":"NOT_PROVEN","trusted_timestamp":"NOT_PROVEN",
            "external_anchor_custody":"NOT_PROVEN","exclusive_successor":"NOT_IMPLEMENTED",
            "fraud_proven":False,"execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED",**extra}

def outside(root,path):
    prior.separate(root,path.resolve(strict=False))

def seal(journal,plan,plan_pin,journal_id,output,parent=None,parent_pin=None):
    root=registry.root(journal)
    native.real_directory(output.parent,"SEAL_OUTPUT_PARENT_INVALID")
    dest=output.parent.resolve(strict=True)/output.name
    for path in (plan,dest): outside(root,path)
    need(not registry.present(dest),"SEAL_OUTPUT_EXISTS")
    need((parent is None)==(parent_pin is None),"PARENT_PAIR_REQUIRED")
    if parent is not None:
        outside(root,parent)
        old_raw,old=load_seal(parent,parent_pin,journal_id)
    plan_raw,expected=external_plan(plan,plan_pin,journal_id)
    members=inventory(root,expected)
    doc={"schema":SCHEMA,"journal_id":journal_id,"sequence":0 if parent is None else old["sequence"]+1,
         "parent_sha256":ZERO if parent is None else parent_pin,"plan_sha256":plan_pin,
         "entries":members,"execution_authorized":False,"production":"NOT_AUTHORIZED"}
    if parent is not None:
        extension(old,doc,parent_pin)
        need(registry.read(parent,MAX_SEAL)==old_raw,"PARENT_CHANGED")
    need(registry.read(plan,MAX_PLAN)==plan_raw and names(root)==set(expected),"SEAL_INPUT_CHANGED")
    raw=portable.serialize(doc)
    need(len(raw)<=MAX_SEAL and doc["sequence"]<2**63,"SEAL_LIMIT")
    portable.safe_write(dest,raw)
    need(registry.read(dest,MAX_SEAL)==raw,"SEAL_PUBLICATION_MISMATCH")
    return outcome("JOURNAL_SEALED",seal_sha256=transfer.sha(raw),record=doc)

def audit(journal,plan,plan_pin,journal_id,seal_file,seal_pin):
    root=registry.root(journal)
    for path in (plan,seal_file): outside(root,path)
    raw,doc=load_seal(seal_file,seal_pin,journal_id)
    need(doc["plan_sha256"]==plan_pin,"SEALED_PLAN_PIN_MISMATCH")
    plan_raw,expected=external_plan(plan,plan_pin,journal_id)
    need(inventory(root,expected)==doc["entries"],"SEALED_INVENTORY_MISMATCH")
    need(registry.read(plan,MAX_PLAN)==plan_raw and registry.read(seal_file,MAX_SEAL)==raw
         and names(root)==set(expected),"AUDIT_INPUT_CHANGED")
    return outcome("JOURNAL_SEAL_VERIFIED",seal_sha256=seal_pin,record=doc)

def transition(parent,parent_pin,child,child_pin,journal_id):
    a,old=load_seal(parent,parent_pin,journal_id)
    b,new=load_seal(child,child_pin,journal_id)
    extension(old,new,parent_pin)
    need(registry.read(parent,MAX_SEAL)==a and registry.read(child,MAX_SEAL)==b,"TRANSITION_INPUT_CHANGED")
    return outcome("SEAL_TRANSITION_VERIFIED",parent_sha256=parent_pin,child_sha256=child_pin,
                   added=len(new["entries"])-len(old["entries"]),snapshot_authenticity="REQUIRES_LIVE_AUDIT")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("seal","audit","transition"):
        a=sub.add_parser(action)
        a.add_argument("--journal-id",required=True)
        if action in ("seal","audit"):
            for n in ("journal","plan"): a.add_argument("--"+n,type=Path,required=True)
            a.add_argument("--plan-pin",required=True)
        if action=="seal":
            a.add_argument("--output",type=Path,required=True)
            a.add_argument("--parent",type=Path)
            a.add_argument("--parent-pin")
        elif action=="audit":
            a.add_argument("--seal-file",type=Path,required=True)
            a.add_argument("--seal-pin",required=True)
        else:
            for n in ("parent","child"): a.add_argument("--"+n,type=Path,required=True)
            for n in ("parent-pin","child-pin"): a.add_argument("--"+n,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"seal":seal,"audit":audit,"transition":transition}[action](**args)
        rc=0
    except Exception as exc:
        out=outcome("UNCERTAIN_BLOCKED",reason=getattr(exc,"code","SEAL_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
