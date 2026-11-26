#!/usr/bin/env python3
"""Canonical pinned snapshot of a complete visible APP-038 journal.

The caller provisions the expected contexts and preserves the seal digest
independently. Stable trusted local filesystem; no hostile-writer, global
completeness, rollback, or power-loss guarantee. No native execution.
"""
from __future__ import annotations
import argparse
from itertools import islice
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app038_signed_conflict_journal"))
import conflict_app038 as prior
registry=prior.prior
transfer=prior.transfer
portable=prior.portable
native=prior.native
need=prior.need
SCHEMA="NORDOI_APP039_CONFLICT_JOURNAL_SEAL_V1"
CONTEXT_SCHEMA="NORDOI_APP039_EXPECTED_CONTEXTS_V1"
MAX_ENTRIES=64
MAX_CONTEXTS=64
MAX_CONTEXT_BYTES=131072
MAX_SEAL_BYTES=262144
FIELDS={"sender","receiver","nonce","transfer_pin","archive_pin","key_pin","receiver_key_pin","challenge_pin"}
NAMES={"START.json","COMPLETE.json"}|{p+"-"+n for p in ("original","conflict") for n in prior.NAMES}

def children(path,limit):
    native.real_directory(path,"JOURNAL_DIRECTORY_INVALID")
    entries=list(islice(path.iterdir(),limit+1))
    need(len(entries)<=limit,"JOURNAL_LIMIT_EXCEEDED")
    return sorted(entries,key=lambda p:p.name)

def load_contexts(path,pin):
    transfer.pin(pin)
    raw=registry.read(path,MAX_CONTEXT_BYTES)
    need(transfer.sha(raw)==pin,"CONTEXTS_PIN_MISMATCH")
    obj=json.loads(raw)
    need(type(obj) is dict and set(obj)=={"schema","contexts"} and obj["schema"]==CONTEXT_SCHEMA,
         "CONTEXTS_SCHEMA_INVALID")
    need(type(obj["contexts"]) is list and len(obj["contexts"])<=MAX_CONTEXTS,"CONTEXTS_LIMIT_EXCEEDED")
    need(raw==portable.serialize(obj),"CONTEXTS_NOT_CANONICAL")
    by_nonce={}
    for args in obj["contexts"]:
        need(type(args) is dict and set(args)==FIELDS and all(type(v) is str for v in args.values()),
             "CONTEXT_FIELDS_INVALID")
        registry.context(**args)
        nonce=args["nonce"]
        need(nonce not in by_nonce,"DUPLICATE_CONTEXT_NONCE")
        by_nonce[nonce]=args
    need(list(by_nonce)==sorted(by_nonce),"CONTEXT_ORDER_INVALID")
    return by_nonce

def outside(registry_path,journal,output):
    native.real_directory(output.parent,"SEAL_PARENT_INVALID")
    dest=output.parent.resolve(strict=True)/output.name
    for root in (registry.root(registry_path),registry.root(journal)):
        need(root!=dest and root not in dest.parents and dest not in root.parents,"SEAL_OVERLAPS_EVIDENCE")
    return dest

def snapshot(registry_path,journal,contexts):
    root=prior.journal_root(registry_path,journal)
    folders=children(root,MAX_CONTEXTS)
    need({p.name for p in folders}==set(contexts),"JOURNAL_CONTEXT_SET_MISMATCH")
    rows=[]
    for folder in folders:
        nonce=folder.name
        transfer.pin(nonce)
        entries=children(folder,MAX_ENTRIES-len(rows))
        need(bool(entries),"EMPTY_NONCE_DIRECTORY")
        for directory in entries:
            pin=directory.name
            transfer.pin(pin)
            paths=children(directory,len(NAMES))
            need({p.name for p in paths}==NAMES,"JOURNAL_ENTRY_LAYOUT_INVALID")
            files={p.name:transfer.sha(registry.read(p,registry.MAX_RECORD)) for p in paths}
            verified=prior.audit(registry=registry_path,journal=journal,conflict_pin=pin,**contexts[nonce])
            need(verified["status"]=="SIGNED_CONTRADICTION_VERIFIED","CONTRADICTION_NOT_VERIFIED")
            need({p.name:transfer.sha(registry.read(p,registry.MAX_RECORD)) for p in children(directory,len(NAMES))}==files,
                 "JOURNAL_ENTRY_CHANGED")
            rows.append({"nonce":nonce,"conflict_sha256":pin,
                         "record_sha256":transfer.sha(portable.serialize(verified["record"])),
                         "files":files})
    return rows

def manifest(registry_path,journal,contexts_file,contexts_pin):
    contexts=load_contexts(contexts_file,contexts_pin)
    rows=snapshot(registry_path,journal,contexts)
    need(snapshot(registry_path,journal,contexts)==rows,"JOURNAL_CHANGED_DURING_SNAPSHOT")
    need(load_contexts(contexts_file,contexts_pin)==contexts,"CONTEXTS_CHANGED")
    doc={"schema":SCHEMA,"contexts_sha256":contexts_pin,"entry_count":len(rows),
         "entries":rows,"inventory_sha256":transfer.sha(portable.serialize(rows)),
         "scope":"VISIBLE_JOURNAL_SNAPSHOT","execution_authorized":False,"production":"NOT_AUTHORIZED"}
    raw=portable.serialize(doc)
    need(len(raw)<=MAX_SEAL_BYTES,"SEAL_TOO_LARGE")
    return doc,raw

def outcome(status,doc,pin):
    return {"schema":SCHEMA,"status":status,"seal_sha256":pin,
            "inventory_sha256":doc["inventory_sha256"],"entry_count":doc["entry_count"],
            "external_pin_required_for_verification":True,"journal_completeness":"NOT_PROVEN",
            "global_antirollback":"NOT_PROVEN","crash_durability":"UNCLAIMED",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}

def seal(registry_path,journal,contexts_file,contexts_pin,output):
    dest=outside(registry_path,journal,output)
    need(not registry.present(dest),"SEAL_ALREADY_EXISTS")
    doc,raw=manifest(registry_path,journal,contexts_file,contexts_pin)
    portable.safe_write(dest,raw)
    need(registry.read(dest,MAX_SEAL_BYTES)==raw,"SEAL_PUBLICATION_NOT_VERIFIED")
    return outcome("JOURNAL_SEAL_CREATED",doc,transfer.sha(raw))

def verify(registry_path,journal,contexts_file,contexts_pin,output,seal_pin):
    transfer.pin(seal_pin)
    dest=outside(registry_path,journal,output)
    raw=registry.read(dest,MAX_SEAL_BYTES)
    need(transfer.sha(raw)==seal_pin,"SEAL_PIN_MISMATCH")
    doc,expected=manifest(registry_path,journal,contexts_file,contexts_pin)
    need(raw==expected,"SEALED_INVENTORY_MISMATCH")
    need(registry.read(dest,MAX_SEAL_BYTES)==raw,"SEAL_CHANGED_DURING_AUDIT")
    return outcome("JOURNAL_SEAL_VERIFIED",doc,seal_pin)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("seal","verify"):
        a=sub.add_parser(action)
        for n in ("registry-path","journal","contexts-file","output"):
            a.add_argument("--"+n,type=Path,required=True)
        a.add_argument("--contexts-pin",required=True)
        if action=="verify": a.add_argument("--seal-pin",required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"seal":seal,"verify":verify}[action](**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED","reason":getattr(exc,"code","SEAL_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
