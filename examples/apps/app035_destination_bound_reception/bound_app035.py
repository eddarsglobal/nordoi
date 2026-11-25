#!/usr/bin/env python3
"""Destination-bound reception on a stable trusted local filesystem.

A local device/inode identity is not a cryptographic hardware identity.
No protection against hostile concurrent filesystem changes or inode reuse.
"""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import secrets
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app034_reception_forensics"))
import audit_app034 as prior
previous=prior.previous
signed=prior.signed
transfer=prior.transfer
portable=prior.portable
native=prior.native
need=prior.need
SCHEMA="NORDOI_APP035_DESTINATION_BOUND_RECEPTION_V1"

def identity(path):
    native.real_directory(path,"LOCAL_DIRECTORY_NOT_REAL")
    st=path.stat()
    need(st.st_ino>0,"LOCAL_DIRECTORY_IDENTITY_UNAVAILABLE")
    return {"device":st.st_dev,"inode":st.st_ino}

def binding(registry,target):
    root=previous.registry_root(registry)
    native.real_directory(target.parent,"RECEIVER_PARENT_INVALID")
    parent=target.parent.resolve(strict=True)
    need(target.name not in ("",".",".."),"DESTINATION_NAME_INVALID")
    dest=parent/target.name
    need(root!=dest and root not in dest.parents and dest not in root.parents,
         "RECEIVER_REGISTRY_OVERLAP")
    return dest,{"path":os.path.normcase(str(dest)),"parent_identity":identity(parent)}

def context(registry,target,sender,receiver,nonce,transfer_pin,archive_pin,key_pin):
    dest,bound=binding(registry,target)
    return dest,{"schema":SCHEMA,
                 "statement":signed.statement(sender,receiver,nonce,transfer_pin,archive_pin,key_pin),
                 "destination":bound}

def issue(registry,target,sender,receiver,transfer_pin,archive_pin,key_pin):
    nonce=secrets.token_hex(32)
    dest,expected=context(registry,target,sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    need(not prior.present(dest),"RECEIVER_DESTINATION_EXISTS")
    directory=previous.registry_root(registry)/nonce
    directory.mkdir(mode=0o700)
    portable.safe_write(directory/"CHALLENGE.json",portable.serialize(expected))
    return {"schema":SCHEMA,"status":"DESTINATION_CHALLENGE_ISSUED",
            "nonce":nonce,"challenge":expected,"execution_authorized":False,
            "production":"NOT_AUTHORIZED"}

def receipt(expected,local_identity,jobs):
    s=expected["statement"]
    return {"schema":SCHEMA,"status":"DESTINATION_BOUND_ACCEPTED_ONCE",
            "sender":s["sender"],"receiver":s["receiver"],"nonce":s["nonce"],
            "sender_key_sha256":s["sender_key_sha256"],
            "archive_sha256":s["archive_sha256"],"transfer_sha256":s["transfer_sha256"],
            "destination":expected["destination"],"destination_identity":local_identity,
            "crypto_signature":"SSHSIG_ED25519_VERIFIED","verified_native_jobs":jobs,
            "nonce_consumption":"PERMANENT_LOCAL_MARKER","automatic_retry":False,
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"destination_binding":"LOCAL_DEVICE_INODE_AND_PATH",
            "destination_signed_by_sender":False,"distributed_anti_replay":"NOT_PROVEN",
            "global_antirollback":"NOT_PROVEN","crash_durability":"UNCLAIMED",
            "production":"NOT_AUTHORIZED"}

def receive(registry,packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,public_key,signed_statement,signature,target):
    dest,expected=context(registry,target,sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    need(not prior.present(dest),"RECEIVER_DESTINATION_EXISTS")
    directory=previous.challenge(registry,expected)
    with tempfile.TemporaryDirectory(prefix="nordoi_app035_") as tmp:
        staged=Path(tmp)/"verified"
        verified=signed.receive(packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,
                                public_key,signed_statement,signature,staged)
        files=transfer.collect(staged,archive_pin)
        need(context(registry,target,sender,receiver,nonce,transfer_pin,archive_pin,key_pin)==(dest,expected),
             "DESTINATION_PARENT_CHANGED")
        need(not prior.present(dest),"RECEIVER_DESTINATION_EXISTS")
        need(previous.challenge(registry,expected)==directory,"CHALLENGE_LOCATION_CHANGED")
        claimed=previous.consume(directory,expected)
        # The permanent nonce marker precedes creation of the destination.
        dest.mkdir(mode=0o700)
        created=identity(dest)
        portable.safe_write(claimed/"DESTINATION.json",portable.serialize(created))
        transfer.populate(dest,{k:v for k,v in files.items() if k!="ARCHIVE.sha256"})
        portable.safe_write(dest/"ARCHIVE.sha256",files["ARCHIVE.sha256"])
        final=transfer.prior.verify(dest,archive_pin)
        need(transfer.collect(dest,archive_pin)==files,"PUBLISHED_EVIDENCE_MISMATCH")
        need(final["verified_native_jobs"]==verified["verified_native_jobs"],"NATIVE_COUNT_MISMATCH")
        need(identity(dest)==created and binding(registry,target)==(dest,expected["destination"]),
             "DESTINATION_IDENTITY_CHANGED")
        result=receipt(expected,created,final["verified_native_jobs"])
        portable.safe_write(claimed/"COMPLETE.json",portable.serialize(result))
        return result

def audit(registry,packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,public_key,signed_statement,signature,target):
    dest,expected=context(registry,target,sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    raw=portable.serialize(expected)
    directory=previous.registry_root(registry)/nonce
    native.real_directory(directory,"CHALLENGE_NOT_ISSUED")
    claimed=directory/"consumed"
    def snapshot():
        need(prior.read(directory/"CHALLENGE.json")==raw,"CHALLENGE_CONTEXT_MISMATCH")
        need({p.name for p in directory.iterdir()}=={"CHALLENGE.json","consumed"},
             "CHALLENGE_LAYOUT_INVALID")
        native.real_directory(claimed,"CONSUMPTION_MARKER_INVALID")
        names={"START.json","DESTINATION.json","COMPLETE.json"}
        need({p.name for p in claimed.iterdir()}==names,"COMPLETE_RECEIPT_REQUIRED")
        return {name:prior.read(claimed/name) for name in names}
    state=snapshot()
    need(state["START.json"]==raw,"CONSUMPTION_START_MISMATCH")
    current=identity(dest)
    need(state["DESTINATION.json"]==portable.serialize(current),"DESTINATION_IDENTITY_MISMATCH")
    with tempfile.TemporaryDirectory(prefix="nordoi_app035_audit_") as tmp:
        staged=Path(tmp)/"verified"
        verified=signed.receive(packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,
                                public_key,signed_statement,signature,staged)
        files=transfer.collect(staged,archive_pin)
        need(transfer.collect(dest,archive_pin)==files,"PUBLISHED_EVIDENCE_MISMATCH")
        final=transfer.prior.verify(dest,archive_pin)
        need(final["verified_native_jobs"]==verified["verified_native_jobs"],"NATIVE_COUNT_MISMATCH")
        expected_receipt=receipt(expected,current,final["verified_native_jobs"])
        need(state["COMPLETE.json"]==portable.serialize(expected_receipt),"COMPLETION_RECEIPT_MISMATCH")
        need(identity(dest)==current and binding(registry,target)==(dest,expected["destination"]),
             "DESTINATION_IDENTITY_CHANGED")
        need(transfer.collect(dest,archive_pin)==files and snapshot()==state,"EVIDENCE_CHANGED_DURING_AUDIT")
    return dict(expected_receipt,status="DESTINATION_BOUND_VERIFIED")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("issue","receive","audit"):
        a=sub.add_parser(action)
        for n in ("registry","target"):
            a.add_argument("--"+n,type=Path,required=True)
        for n in ("sender","receiver","transfer-pin","archive-pin","key-pin"):
            a.add_argument("--"+n,required=True)
        if action!="issue":
            a.add_argument("--nonce",required=True)
            for n in ("packet","public-key","signed-statement","signature"):
                a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"issue":issue,"receive":receive,"audit":audit}[action](**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"BLOCK","reason":getattr(exc,"code","DESTINATION_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
