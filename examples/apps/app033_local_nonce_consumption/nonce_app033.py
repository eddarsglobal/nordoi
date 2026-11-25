#!/usr/bin/env python3
"""Local recipient challenge registry; permanent exclusive consumption.
Only cooperating APP-033 receivers on a stable trusted local filesystem.
No distributed replay, rollback or power-loss guarantee. No native execution.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import secrets
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app032_signed_evidence_transfer"))
import signed_app032 as signed
transfer=signed.transfer
portable=signed.portable
native=signed.native
need=signed.need
SCHEMA="NORDOI_APP033_LOCAL_NONCE_CONSUMPTION_V1"

def registry_root(registry):
    native.real_directory(registry,"REGISTRY_NOT_REAL")
    return registry.resolve(strict=True)

def context(sender,receiver,nonce,transfer_pin,archive_pin,key_pin):
    return {"schema":SCHEMA,"statement":signed.statement(sender,receiver,nonce,transfer_pin,archive_pin,key_pin)}

def issue(registry,sender,receiver,transfer_pin,archive_pin,key_pin):
    root=registry_root(registry)
    nonce=secrets.token_hex(32)
    expected=context(sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    directory=root/nonce
    directory.mkdir(mode=0o700)
    portable.safe_write(directory/"CHALLENGE.json",portable.serialize(expected))
    return {"schema":SCHEMA,"status":"CHALLENGE_ISSUED","nonce":nonce,"challenge":expected,
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def challenge(registry,expected):
    root=registry_root(registry)
    nonce=expected["statement"]["nonce"]
    transfer.pin(nonce)
    directory=root/nonce
    native.real_directory(directory,"CHALLENGE_NOT_ISSUED")
    raw=transfer.prior.read(directory/"CHALLENGE.json",8192)
    need(raw==portable.serialize(expected),"CHALLENGE_CONTEXT_MISMATCH")
    consumed=directory/"consumed"
    need(not consumed.exists() and not consumed.is_symlink(),"NONCE_ALREADY_CONSUMED_OR_UNCERTAIN")
    return directory

def consume(directory,expected):
    # Atomic exclusive mkdir is the irreversible local acceptance boundary.
    # Even an empty marker (owner died before writing receipt) blocks replay.
    claimed=directory/"consumed"
    try:
        claimed.mkdir(mode=0o700)
    except FileExistsError:
        need(False,"NONCE_ALREADY_CONSUMED_OR_UNCERTAIN")
    portable.safe_write(claimed/"START.json",portable.serialize(expected))
    return claimed

def receive(registry,packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,public_key,signed_statement,signature,target):
    expected=context(sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    root=registry_root(registry)
    native.real_directory(target.parent,"RECEIVER_PARENT_INVALID")
    destination=target.parent.resolve(strict=True)/target.name
    need(root!=destination and root not in destination.parents and destination not in root.parents,
         "RECEIVER_REGISTRY_OVERLAP")
    need(not target.exists() and not target.is_symlink(),"RECEIVER_DESTINATION_EXISTS")
    directory=challenge(registry,expected)
    # Reject malformed or unauthenticated inputs before consuming a challenge.
    with tempfile.TemporaryDirectory(prefix="nordoi_app033_") as tmp:
        staged=Path(tmp)/"verified"
        verified=signed.receive(packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,
                                public_key,signed_statement,signature,staged)
        files=transfer.collect(staged,archive_pin)
        # Recheck the issued context, then race for the exclusive marker.
        need(challenge(registry,expected)==directory,"CHALLENGE_LOCATION_CHANGED")
        claimed=consume(directory,expected)
        # Any subsequent error leaves the nonce permanently consumed.
        target.mkdir(mode=0o700)
        transfer.populate(target,{k:v for k,v in files.items() if k!="ARCHIVE.sha256"})
        portable.safe_write(target/"ARCHIVE.sha256",files["ARCHIVE.sha256"])
        final=transfer.prior.verify(target,archive_pin)
        need(final["verified_native_jobs"]==verified["verified_native_jobs"],"PUBLISHED_EVIDENCE_MISMATCH")
        result={"schema":SCHEMA,"status":"SIGNED_TRANSFER_ACCEPTED_ONCE",
                "sender":sender,"receiver":receiver,"nonce":nonce,"sender_key_sha256":key_pin,
                "archive_sha256":archive_pin,"transfer_sha256":transfer_pin,
                "crypto_signature":verified["crypto_signature"],"verified_native_jobs":final["verified_native_jobs"],
                "nonce_consumption":"PERMANENT_LOCAL_MARKER","automatic_retry":False,
                "execution_authorized":False,"reservation_release_authorized":False,
                "local_replay_scope":"COOPERATING_RECEIVERS_SAME_STABLE_REGISTRY",
                "distributed_anti_replay":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
                "crash_durability":"UNCLAIMED","transport_encrypted":False,"production":"NOT_AUTHORIZED"}
        portable.safe_write(claimed/"COMPLETE.json",portable.serialize(result))
        return result

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("issue","receive"):
        a=sub.add_parser(action)
        a.add_argument("--registry",type=Path,required=True)
        for n in ("sender","receiver","transfer-pin","archive-pin","key-pin"):
            a.add_argument("--"+n,required=True)
        if action=="receive":
            a.add_argument("--nonce",required=True)
            for n in ("packet","public-key","signed-statement","signature","target"):
                a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out=issue(**args) if action=="issue" else receive(**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"BLOCK","reason":getattr(exc,"code","CHALLENGE_OR_PUBLICATION_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
