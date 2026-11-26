#!/usr/bin/env python3
"""Recipient-issued challenge and one-time local acceptance of APP-044 archives.
A permanent marker blocks retries even after interruption. Stable trusted local
registry shared by cooperating processes; no distributed replay or rollback guarantee.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import secrets
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app044_signed_chain_archive"))
import signed_app044 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
native=prior.prior.prior.prior.native
need=prior.need
SCHEMA="NORDOI_APP045_ARCHIVE_NONCE_ADMISSION_V1"
MAX_RECORD=16384

def context(**trust):
    return {"schema":SCHEMA,"statement":prior.expected(**trust)}

def issue(challenges,sender,receiver,key_pin,archive_pin,source_plan_pin,anchor_pin,head_pin):
    root=registry.root(challenges)
    nonce=secrets.token_hex(32)
    ctx=context(sender=sender,receiver=receiver,nonce=nonce,key_pin=key_pin,
                archive_pin=archive_pin,source_plan_pin=source_plan_pin,anchor_pin=anchor_pin,head_pin=head_pin)
    folder=root/nonce
    folder.mkdir(mode=0o700)
    portable.safe_write(folder/"CHALLENGE.json",portable.serialize(ctx))
    return {"schema":SCHEMA,"status":"ARCHIVE_CHALLENGE_ISSUED","nonce":nonce,
            "challenge_sha256":transfer.sha(portable.serialize(ctx)),
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def folder_for(challenges,ctx,paths):
    root=registry.root(challenges)
    for p in paths:
        resolved=p.resolve(strict=True)
        need(root!=resolved and root not in resolved.parents,"CHALLENGE_INPUT_OVERLAP")
    folder=root/ctx["statement"]["nonce"]
    native.real_directory(folder,"ARCHIVE_CHALLENGE_NOT_ISSUED")
    need(registry.read(folder/"CHALLENGE.json",MAX_RECORD)==portable.serialize(ctx),
         "ARCHIVE_CHALLENGE_CONTEXT_MISMATCH")
    need({p.name for p in folder.iterdir()} in ({"CHALLENGE.json"},{"CHALLENGE.json","consumed"}),
         "CHALLENGE_LAYOUT_INVALID")
    return folder

def receipt(ctx,verified):
    return {"schema":SCHEMA,"status":"SIGNED_ARCHIVE_ACCEPTED_ONCE",
            "challenge_sha256":transfer.sha(portable.serialize(ctx)),
            "verification_sha256":transfer.sha(portable.serialize(verified)),
            "statement_sha256":verified["statement_sha256"],"signature_sha256":verified["signature_sha256"],
            "archive_sha256":verified["archive_sha256"],"head_sha256":verified["head_sha256"],
            "sender":verified["sender"],"receiver":verified["receiver"],"nonce":verified["nonce"],
            "transition_count":verified["transition_count"],"decisions_sha256":verified["decisions_sha256"],
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def stored(folder,ctx,verified):
    consumed=folder/"consumed"
    native.real_directory(consumed,"CONSUMPTION_MARKER_INVALID")
    need({p.name for p in consumed.iterdir()}=={"START.json","COMPLETE.json"},"CONSUMPTION_INCOMPLETE")
    start=portable.serialize(ctx)
    rec=receipt(ctx,verified)
    need(registry.read(consumed/"START.json",MAX_RECORD)==start
         and registry.read(consumed/"COMPLETE.json",MAX_RECORD)==portable.serialize(rec),
         "CONSUMPTION_RECEIPT_MISMATCH")
    need(registry.read(folder/"CHALLENGE.json",MAX_RECORD)==start,"CHALLENGE_CHANGED")
    return rec

def outcome(status,rec):
    return {"schema":SCHEMA,"status":status,"record":rec,
            "nonce_consumption":"PERMANENT_LOCAL_MARKER",
            "local_replay_scope":"COOPERATING_RECEIVERS_SAME_STABLE_REGISTRY",
            "distributed_anti_replay":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
            "freshness":"NOT_PROVEN","crash_durability":"UNCLAIMED",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}

def accept(challenges,archive,statement,signature,public_key,**trust):
    ctx=context(**trust)
    paths=(archive,statement,signature,public_key)
    folder=folder_for(challenges,ctx,paths)
    consumed=folder/"consumed"
    need(not registry.present(consumed),"NONCE_ALREADY_CONSUMED_OR_UNCERTAIN")
    verified=prior.verify(archive,statement,signature,public_key,**trust)
    need(verified["status"]=="SIGNED_CHAIN_ARCHIVE_VERIFIED","ARCHIVE_NOT_AUTHENTICATED")
    need(folder_for(challenges,ctx,paths)==folder,"CHALLENGE_LOCATION_CHANGED")
    try:
        consumed.mkdir(mode=0o700)
    except FileExistsError:
        need(False,"NONCE_ALREADY_CONSUMED_OR_UNCERTAIN")
    # No removal or retry if interrupted after the exclusive claim.
    portable.safe_write(consumed/"START.json",portable.serialize(ctx))
    rec=receipt(ctx,verified)
    portable.safe_write(consumed/"COMPLETE.json",portable.serialize(rec))
    need(stored(folder,ctx,verified)==rec,"CONSUMPTION_PUBLICATION_NOT_VERIFIED")
    return outcome("SIGNED_ARCHIVE_ACCEPTED_ONCE",rec)

def audit(challenges,archive,statement,signature,public_key,**trust):
    ctx=context(**trust)
    paths=(archive,statement,signature,public_key)
    folder=folder_for(challenges,ctx,paths)
    verified=prior.verify(archive,statement,signature,public_key,**trust)
    need(verified["status"]=="SIGNED_CHAIN_ARCHIVE_VERIFIED","ARCHIVE_NOT_AUTHENTICATED")
    rec=stored(folder,ctx,verified)
    need(folder_for(challenges,ctx,paths)==folder and stored(folder,ctx,verified)==rec,
         "CONSUMPTION_CHANGED_DURING_AUDIT")
    return outcome("ARCHIVE_ACCEPTANCE_VERIFIED",rec)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("issue","accept","audit"):
        a=sub.add_parser(action)
        a.add_argument("--challenges",type=Path,required=True)
        for n in ("sender","receiver","key-pin","archive-pin","source-plan-pin","anchor-pin","head-pin"):
            a.add_argument("--"+n,required=True)
        if action!="issue":
            a.add_argument("--nonce",required=True)
            for n in ("archive","statement","signature","public-key"): a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"issue":issue,"accept":accept,"audit":audit}[action](**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED","reason":getattr(exc,"code","ARCHIVE_ADMISSION_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
