#!/usr/bin/env python3
"""Local sender registry for APP-036 signed acknowledgments.

The sender independently provisions expected context and a trusted stable local
registry. Only cooperating callers sharing that registry are covered.
No distributed replay prevention, rollback protection or power-loss guarantee.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app036_signed_reception_ack"))
import ack_app036 as previous
transfer=previous.transfer
portable=previous.portable
native=previous.native
need=previous.need
SCHEMA="NORDOI_APP037_SENDER_ACK_REGISTRY_V1"
MAX_RECORD=16384

def context(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin):
    return {"schema":SCHEMA,"expected_ack":previous.expected(sender,receiver,nonce,transfer_pin,
                   archive_pin,key_pin,receiver_key_pin,challenge_pin)}

def root(registry):
    native.real_directory(registry,"ACK_REGISTRY_INVALID")
    return registry.resolve(strict=True)

def present(path):
    return path.exists() or path.is_symlink()

def read(path,limit=MAX_RECORD):
    return transfer.prior.read(path,limit)

def outcome(status,nonce,**extra):
    return {"schema":SCHEMA,"status":status,"nonce":nonce,
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,
            "remote_filesystem_independently_verified":False,
            "current_remote_state_proven":False,
            "distributed_anti_replay":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
            "crash_durability":"UNCLAIMED","production":"NOT_AUTHORIZED",**extra}

def register(registry,sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin):
    ctx=context(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin)
    directory=root(registry)/nonce
    directory.mkdir(mode=0o700)
    portable.safe_write(directory/"EXPECTED.json",portable.serialize(ctx))
    return outcome("ACK_EXPECTATION_REGISTERED",nonce)

def entry(registry,ctx,nonce):
    directory=root(registry)/nonce
    native.real_directory(directory,"ACK_EXPECTATION_NOT_REGISTERED")
    need(read(directory/"EXPECTED.json")==portable.serialize(ctx),"ACK_REGISTERED_CONTEXT_MISMATCH")
    need({p.name for p in directory.iterdir()} in ({"EXPECTED.json"},{"EXPECTED.json","accepted"}),
         "ACK_REGISTRY_LAYOUT_INVALID")
    return directory

def verification_args(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin):
    return dict(sender=sender,receiver=receiver,nonce=nonce,transfer_pin=transfer_pin,
                archive_pin=archive_pin,key_pin=key_pin,receiver_key_pin=receiver_key_pin,challenge_pin=challenge_pin)

def capture(ack,ack_signature,receiver_public_key):
    return {"ACK.json":read(ack,previous.MAX_ACK),
            "ACK.sig":read(ack_signature,previous.crypto.MAX_SIGNATURE),
            "RECEIVER.pub":read(receiver_public_key,previous.crypto.MAX_PUBLIC_KEY)}

def verify_bytes(files,args):
    # The exact bytes verified are the bytes later stored.
    with tempfile.TemporaryDirectory(prefix="nordoi_app037_") as tmp:
        p=Path(tmp)
        for name,raw in files.items():
            portable.safe_write(p/name,raw)
        return previous.verify(ack=p/"ACK.json",ack_signature=p/"ACK.sig",
                               receiver_public_key=p/"RECEIVER.pub",**args)

def receipt(ctx,files,verified):
    return {"schema":SCHEMA,"status":"ACK_ACCEPTED_ONCE",
            "context_sha256":transfer.sha(portable.serialize(ctx)),
            "ack_sha256":verified["ack_sha256"],"receipt_sha256":verified["receipt_sha256"],
            "signature_sha256":transfer.sha(files["ACK.sig"]),
            "receiver_key_sha256":verified["receiver_key_sha256"],
            "recipient_asserted_native_jobs":verified["recipient_asserted_native_jobs"],
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def stored(directory,ctx,args):
    accepted=directory/"accepted"
    native.real_directory(accepted,"ACK_CONSUMPTION_MARKER_INVALID")
    need({p.name for p in accepted.iterdir()}=={"START.json","ACK.json","ACK.sig","RECEIVER.pub","COMPLETE.json"},
         "ACK_CONSUMPTION_INCOMPLETE_OR_INVALID")
    start=read(accepted/"START.json")
    need(start==portable.serialize(ctx),"ACK_START_CONTEXT_MISMATCH")
    files=capture(accepted/"ACK.json",accepted/"ACK.sig",accepted/"RECEIVER.pub")
    complete=read(accepted/"COMPLETE.json")
    verified=verify_bytes(files,args)
    expected=receipt(ctx,files,verified)
    need(complete==portable.serialize(expected),"ACK_COMPLETION_RECEIPT_MISMATCH")
    need(capture(accepted/"ACK.json",accepted/"ACK.sig",accepted/"RECEIVER.pub")==files
         and read(accepted/"START.json")==start and read(accepted/"COMPLETE.json")==complete
         and read(directory/"EXPECTED.json")==portable.serialize(ctx),"ACK_REGISTRY_CHANGED_DURING_AUDIT")
    return files,expected

def accept(registry,ack,ack_signature,receiver_public_key,sender,receiver,nonce,
           transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin):
    args=verification_args(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin)
    ctx=context(**args)
    directory=entry(registry,ctx,nonce)
    files=capture(ack,ack_signature,receiver_public_key)
    verified=verify_bytes(files,args)
    need(entry(registry,ctx,nonce)==directory,"ACK_REGISTRY_LOCATION_CHANGED")
    accepted=directory/"accepted"
    try:
        accepted.mkdir(mode=0o700)
    except FileExistsError:
        old,record=stored(directory,ctx,args)
        if old["ACK.json"]==files["ACK.json"]:
            return outcome("DUPLICATE_ACK_BLOCKED",nonce,accepted_ack_sha256=record["ack_sha256"])
        return outcome("CONTRADICTORY_SIGNED_ACK_BLOCKED",nonce,
                       accepted_ack_sha256=record["ack_sha256"],conflicting_ack_sha256=verified["ack_sha256"],
                       conflict_persisted=False)
    # Exclusive permanent marker. A failure never releases it.
    portable.safe_write(accepted/"START.json",portable.serialize(ctx))
    for name,raw in files.items():
        portable.safe_write(accepted/name,raw)
    record=receipt(ctx,files,verified)
    portable.safe_write(accepted/"COMPLETE.json",portable.serialize(record))
    need(stored(directory,ctx,args)==(files,record),"ACK_PUBLICATION_NOT_VERIFIED")
    return outcome("ACK_ACCEPTED_ONCE",nonce,record=record)

def audit(registry,sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin):
    args=verification_args(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin)
    ctx=context(**args)
    directory=entry(registry,ctx,nonce)
    accepted=directory/"accepted"
    if not present(accepted):
        need(entry(registry,ctx,nonce)==directory and not present(accepted),"ACK_STATE_CHANGED")
        return outcome("ACK_PENDING",nonce)
    _,record=stored(directory,ctx,args)
    return outcome("ACK_ACCEPTED_VERIFIED",nonce,record=record)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("register","accept","audit"):
        a=sub.add_parser(action)
        a.add_argument("--registry",type=Path,required=True)
        for n in ("sender","receiver","nonce","transfer-pin","archive-pin","key-pin","receiver-key-pin","challenge-pin"):
            a.add_argument("--"+n,required=True)
        if action=="accept":
            for n in ("ack","ack-signature","receiver-public-key"):
                a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"register":register,"accept":accept,"audit":audit}[action](**args)
        rc=0 if out["status"] in ("ACK_EXPECTATION_REGISTERED","ACK_ACCEPTED_ONCE","ACK_PENDING","ACK_ACCEPTED_VERIFIED") else 3
    except Exception as exc:
        out=outcome("UNCERTAIN_BLOCKED",args.get("nonce"),reason=getattr(exc,"code","ACK_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
