#!/usr/bin/env python3
"""Exclusive local evidence journal for contradictory signed APP-036 ACKs.

Separate from APP-037. No mutation of the accepted ACK, no remote attestation,
no completeness or rollback guarantee, no native execution authority.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app037_sender_ack_registry"))
import registry_app037 as prior
transfer=prior.transfer
portable=prior.portable
native=prior.native
need=prior.need
SCHEMA="NORDOI_APP038_SIGNED_CONFLICT_JOURNAL_V1"
NAMES=("ACK.json","ACK.sig","RECEIVER.pub")

def journal_root(registry,journal):
    r=prior.root(registry)
    j=prior.root(journal)
    need(r!=j and r not in j.parents and j not in r.parents,"JOURNAL_REGISTRY_OVERLAP")
    return j

def result(status,nonce,**extra):
    return dict(prior.outcome(status,nonce),schema=SCHEMA,journal_scope="TRUSTED_LOCAL_FILESYSTEM",
                journal_completeness="NOT_PROVEN",**extra)

def evidence_record(ctx,original,record,conflict,verified):
    need(original["ACK.json"]!=conflict["ACK.json"],"ACK_NOT_CONTRADICTORY")
    need(record["ack_sha256"]==transfer.sha(original["ACK.json"]),"ORIGINAL_ACK_PIN_MISMATCH")
    return {"schema":SCHEMA,"status":"SIGNED_CONTRADICTION_RECORDED",
            "context_sha256":transfer.sha(portable.serialize(ctx)),
            "accepted_record_sha256":transfer.sha(portable.serialize(record)),
            "accepted_ack_sha256":record["ack_sha256"],
            "conflicting_ack_sha256":verified["ack_sha256"],
            "accepted_receipt_sha256":record["receipt_sha256"],
            "conflicting_receipt_sha256":verified["receipt_sha256"],
            "original_files":{k:transfer.sha(original[k]) for k in NAMES},
            "conflicting_files":{k:transfer.sha(conflict[k]) for k in NAMES},
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"production":"NOT_AUTHORIZED"}

def capture_side(directory,prefix):
    return prior.capture(directory/(prefix+"-ACK.json"),directory/(prefix+"-ACK.sig"),
                         directory/(prefix+"-RECEIVER.pub"))

def check_entry(directory,ctx,args,original,record,conflict_pin):
    native.real_directory(directory,"CONFLICT_ENTRY_INVALID")
    names={"START.json","COMPLETE.json"}|{p+"-"+n for p in ("original","conflict") for n in NAMES}
    need({p.name for p in directory.iterdir()}==names,"CONFLICT_INCOMPLETE_OR_INVALID")
    start=prior.read(directory/"START.json")
    complete=prior.read(directory/"COMPLETE.json")
    expected_start=portable.serialize({"schema":SCHEMA,"context":ctx,"conflicting_ack_sha256":conflict_pin})
    need(start==expected_start,"CONFLICT_START_MISMATCH")
    first=capture_side(directory,"original")
    second=capture_side(directory,"conflict")
    need(first==original,"ORIGINAL_EVIDENCE_MISMATCH")
    prior.verify_bytes(first,args)
    verified=prior.verify_bytes(second,args)
    need(verified["ack_sha256"]==conflict_pin,"CONFLICT_PIN_MISMATCH")
    rec=evidence_record(ctx,first,record,second,verified)
    need(complete==portable.serialize(rec),"CONFLICT_COMPLETION_MISMATCH")
    need(capture_side(directory,"original")==first and capture_side(directory,"conflict")==second
         and prior.read(directory/"START.json")==start and prior.read(directory/"COMPLETE.json")==complete
         and {p.name for p in directory.iterdir()}==names,"CONFLICT_CHANGED_DURING_AUDIT")
    return rec

def preserve(registry,journal,ack,ack_signature,receiver_public_key,sender,receiver,nonce,
             transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin):
    args=prior.verification_args(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin)
    ctx=prior.context(**args)
    j=journal_root(registry,journal)
    entry=prior.entry(registry,ctx,nonce)
    original,record=prior.stored(entry,ctx,args)
    conflict=prior.capture(ack,ack_signature,receiver_public_key)
    verified=prior.verify_bytes(conflict,args)
    if original["ACK.json"]==conflict["ACK.json"]:
        return result("DUPLICATE_ACK_BLOCKED",nonce,conflict_persisted=False)
    pin=verified["ack_sha256"]
    transfer.pin(pin)
    rec=evidence_record(ctx,original,record,conflict,verified)
    need(prior.stored(entry,ctx,args)==(original,record),"ORIGINAL_CHANGED_BEFORE_JOURNAL")
    folder=j/nonce
    folder.mkdir(mode=0o700,exist_ok=True)
    native.real_directory(folder,"CONFLICT_NONCE_DIRECTORY_INVALID")
    directory=folder/pin
    try:
        directory.mkdir(mode=0o700)
    except FileExistsError:
        old=check_entry(directory,ctx,args,original,record,pin)
        return result("CONTRADICTION_ALREADY_RECORDED",nonce,record=old,conflict_persisted=True)
    # No removal or retry of an incomplete entry.
    portable.safe_write(directory/"START.json",portable.serialize(
        {"schema":SCHEMA,"context":ctx,"conflicting_ack_sha256":pin}))
    for prefix,files in (("original",original),("conflict",conflict)):
        for name in NAMES:
            portable.safe_write(directory/(prefix+"-"+name),files[name])
    need(prior.stored(entry,ctx,args)==(original,record),"ORIGINAL_CHANGED_DURING_JOURNAL")
    portable.safe_write(directory/"COMPLETE.json",portable.serialize(rec))
    need(check_entry(directory,ctx,args,original,record,pin)==rec,"CONFLICT_PUBLICATION_NOT_VERIFIED")
    return result("SIGNED_CONTRADICTION_RECORDED",nonce,record=rec,conflict_persisted=True)

def audit(registry,journal,conflict_pin,sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin):
    args=prior.verification_args(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin)
    ctx=prior.context(**args)
    transfer.pin(conflict_pin)
    j=journal_root(registry,journal)
    entry=prior.entry(registry,ctx,nonce)
    original,record=prior.stored(entry,ctx,args)
    folder=j/nonce
    native.real_directory(folder,"CONFLICT_NONCE_DIRECTORY_INVALID")
    rec=check_entry(folder/conflict_pin,ctx,args,original,record,conflict_pin)
    need(prior.stored(entry,ctx,args)==(original,record),"ORIGINAL_CHANGED_DURING_AUDIT")
    return result("SIGNED_CONTRADICTION_VERIFIED",nonce,record=rec,conflict_persisted=True)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("preserve","audit"):
        a=sub.add_parser(action)
        for n in ("registry","journal"): a.add_argument("--"+n,type=Path,required=True)
        for n in ("sender","receiver","nonce","transfer-pin","archive-pin","key-pin","receiver-key-pin","challenge-pin"):
            a.add_argument("--"+n,required=True)
        if action=="preserve":
            for n in ("ack","ack-signature","receiver-public-key"):
                a.add_argument("--"+n,type=Path,required=True)
        else:
            a.add_argument("--conflict-pin",required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"preserve":preserve,"audit":audit}[action](**args)
        rc=0 if out["status"] in ("SIGNED_CONTRADICTION_RECORDED","SIGNED_CONTRADICTION_VERIFIED") else 3
    except Exception as exc:
        out=result("UNCERTAIN_BLOCKED",args.get("nonce"),reason=getattr(exc,"code","CONFLICT_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
