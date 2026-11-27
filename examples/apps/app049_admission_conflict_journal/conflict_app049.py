#!/usr/bin/env python3
"""Local authenticated admission conflict journal. No fraud or distributed-state claim."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app048_sender_admission_ack_registry"))
import sender_app048 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
native=prior.native
SCHEMA="NORDOI_APP049_ADMISSION_CONFLICT_JOURNAL_V1"
MAX_RECORD=65536

def separate(a,b):
    need(a!=b and a not in b.parents and b not in a.parents,"JOURNAL_ROOT_OVERLAP")

def original(entry,ctx,receiver_key_pin,trust,original_record_pin):
    transfer.pin(original_record_pin)
    files,record=prior.stored(entry,ctx,receiver_key_pin,trust)
    need(transfer.sha(portable.serialize(record))==original_record_pin,"ORIGINAL_RECORD_PIN_MISMATCH")
    return files,record

def statement(ctx,original_record_pin,package_pin,receipt_pin,conflicting_ack_pin):
    for pin in (original_record_pin,package_pin,receipt_pin,conflicting_ack_pin): transfer.pin(pin)
    return {"schema":SCHEMA,"context":ctx,"original_record_sha256":original_record_pin,
            "conflicting_package_sha256":package_pin,"conflicting_receipt_sha256":receipt_pin,
            "conflicting_ack_sha256":conflicting_ack_pin}

def identity(start):
    return transfer.sha(portable.serialize(start))

def prove(start,first,record,other,receiver_key_pin,trust):
    ctx=prior.context(receiver_key_pin,**trust)
    need(start["context"]==ctx,"CONFLICT_CONTEXT_MISMATCH")
    need(transfer.sha(portable.serialize(record))==start["original_record_sha256"],"ORIGINAL_RECORD_PIN_MISMATCH")
    a=prior.verify_bytes(first,record["package_sha256"],record["receipt_sha256"],receiver_key_pin,trust)
    need(prior.receipt(ctx,first,a)==record,"ORIGINAL_RECORD_MISMATCH")
    b=prior.verify_bytes(other,start["conflicting_package_sha256"],start["conflicting_receipt_sha256"],receiver_key_pin,trust)
    need(b["ack_sha256"]==start["conflicting_ack_sha256"],"CONFLICT_ACK_PIN_MISMATCH")
    need(first["ACK.json"]!=other["ACK.json"] and a["ack_sha256"]!=b["ack_sha256"],"IDENTICAL_ACK_NOT_CONFLICT")
    return {"schema":SCHEMA,"status":"AUTHENTICATED_DECLARATION_CONFLICT",
            "conflict_id":identity(start),"original_record_sha256":start["original_record_sha256"],
            "original_ack_sha256":a["ack_sha256"],"conflicting_ack_sha256":b["ack_sha256"],
            "original_members":{n:transfer.sha(v) for n,v in first.items()},
            "conflicting_members":{n:transfer.sha(v) for n,v in other.items()},
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def paths(folder,prefix):
    return {n:folder/(prefix+n) for n in prior.LIMITS}

def stored(folder,start,receiver_key_pin,trust):
    native.real_directory(folder,"CONFLICT_ENTRY_INVALID")
    a=paths(folder,"FIRST-")
    b=paths(folder,"OTHER-")
    need({p.name for p in folder.iterdir()}=={p.name for p in (*a.values(),*b.values())}|{"START.json","ORIGINAL.json","COMPLETE.json"},"CONFLICT_ENTRY_INCOMPLETE")
    initial=registry.read(folder/"START.json",MAX_RECORD)
    raw_record=registry.read(folder/"ORIGINAL.json",prior.MAX_RECORD)
    complete=registry.read(folder/"COMPLETE.json",MAX_RECORD)
    need(initial==portable.serialize(start),"CONFLICT_START_MISMATCH")
    record=json.loads(raw_record)
    need(type(record) is dict and raw_record==portable.serialize(record),"ORIGINAL_RECORD_NOT_CANONICAL")
    first,other=prior.capture(a),prior.capture(b)
    expected=prove(start,first,record,other,receiver_key_pin,trust)
    need(complete==portable.serialize(expected),"CONFLICT_COMPLETION_MISMATCH")
    need(prior.capture(a)==first and prior.capture(b)==other
         and registry.read(folder/"START.json",MAX_RECORD)==initial
         and registry.read(folder/"ORIGINAL.json",prior.MAX_RECORD)==raw_record
         and registry.read(folder/"COMPLETE.json",MAX_RECORD)==complete,"CONFLICT_CHANGED_DURING_AUDIT")
    return expected

def outcome(status,**extra):
    return {"schema":SCHEMA,"status":status,"fraud_proven":False,"remote_state_proven":False,
            "journal_completeness":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
            "distributed_anti_replay":"NOT_PROVEN","crash_durability":"UNCLAIMED",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED",**extra}

def preserve(journal,ack_registry,package,ack,ack_signature,receiver_public_key,package_pin,receipt_pin,
             receiver_key_pin,original_record_pin,conflicting_ack_pin,**trust):
    root=registry.root(journal)
    source=registry.root(ack_registry)
    separate(root,source)
    ctx=prior.context(receiver_key_pin,**trust)
    start=statement(ctx,original_record_pin,package_pin,receipt_pin,conflicting_ack_pin)
    folder=root/identity(start)
    entry=prior.entry_for(source,ctx,trust["nonce"])
    inputs={"PACKAGE.json":package,"ACK.json":ack,"ACK.sig":ack_signature,"RECEIVER.pub":receiver_public_key}
    for p in inputs.values():
        resolved=p.resolve(strict=True)
        separate(root,resolved)
        separate(source,resolved)
    first,record=original(entry,ctx,receiver_key_pin,trust,original_record_pin)
    other=prior.capture(inputs)
    expected=prove(start,first,record,other,receiver_key_pin,trust)
    need(prior.capture(inputs)==other and original(entry,ctx,receiver_key_pin,trust,original_record_pin)==(first,record),"SOURCE_EVIDENCE_CHANGED")
    try:
        folder.mkdir(mode=0o700)
    except FileExistsError:
        need(stored(folder,start,receiver_key_pin,trust)==expected,"CONFLICT_EXISTING_MISMATCH")
        return outcome("DUPLICATE_CONFLICT_BLOCKED",record=expected)
    portable.safe_write(folder/"START.json",portable.serialize(start))
    portable.safe_write(folder/"ORIGINAL.json",portable.serialize(record))
    for prefix,files in (("FIRST-",first),("OTHER-",other)):
        for name,raw in files.items(): portable.safe_write(folder/(prefix+name),raw)
    portable.safe_write(folder/"COMPLETE.json",portable.serialize(expected))
    need(stored(folder,start,receiver_key_pin,trust)==expected,"CONFLICT_PUBLICATION_NOT_VERIFIED")
    need(original(entry,ctx,receiver_key_pin,trust,original_record_pin)==(first,record),"ORIGINAL_CHANGED_DURING_PUBLICATION")
    return outcome("CONFLICT_PRESERVED",record=expected)

def audit(journal,package_pin,receipt_pin,receiver_key_pin,original_record_pin,conflicting_ack_pin,**trust):
    ctx=prior.context(receiver_key_pin,**trust)
    start=statement(ctx,original_record_pin,package_pin,receipt_pin,conflicting_ack_pin)
    folder=registry.root(journal)/identity(start)
    return outcome("CONFLICT_VERIFIED",record=stored(folder,start,receiver_key_pin,trust))

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("preserve","audit"):
        a=sub.add_parser(action)
        a.add_argument("--journal",type=Path,required=True)
        for n in ("sender","receiver","nonce","key-pin","archive-pin","source-plan-pin","anchor-pin","head-pin",
                  "receiver-key-pin","package-pin","receipt-pin","original-record-pin","conflicting-ack-pin"):
            a.add_argument("--"+n,required=True)
        if action=="preserve":
            for n in ("ack-registry","package","ack","ack-signature","receiver-public-key"):
                a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"preserve":preserve,"audit":audit}[action](**args)
        rc=0 if out["status"] in ("CONFLICT_PRESERVED","CONFLICT_VERIFIED") else 3
    except Exception as exc:
        out=outcome("UNCERTAIN_BLOCKED",reason=getattr(exc,"code","CONFLICT_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
