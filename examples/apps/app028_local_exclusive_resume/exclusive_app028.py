#!/usr/bin/env python3
"""One-shot local reservation for cooperating APP-028 continuation processes.

An exclusive directory is a permanent refusal marker, NOT an OS advisory
lock, a cross-host ledger, a crash-durable transaction or a recovery grant.
Requires a trusted, stable local filesystem and a quiescent host except
for cooperating APP-028 callers. Never remove or reclaim a reservation.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app027_verified_reconstructed_resume"))
import resume_app027 as previous

native=previous.native
need=previous.need
SCHEMA="NORDOI_APP028_LOCAL_EXCLUSIVE_RESUME_V1"


def identity(root):
    native.real_directory(root,"WORKSPACE_NOT_REAL")
    canonical=root.resolve(strict=True)
    info=canonical.stat()
    need(stat.S_ISDIR(info.st_mode) and info.st_ino>0,"WORKSPACE_IDENTITY_UNAVAILABLE")
    # No lexical path or checkpoint pin in the key: case/alias and different
    # pins do not partition the same local directory identity.
    key=hashlib.sha256(f"{info.st_dev}:{info.st_ino}".encode("ascii")).hexdigest()
    return canonical,key


def location(root):
    canonical,key=identity(root)
    native.real_directory(canonical.parent,"WORKSPACE_PARENT_NOT_REAL")
    need(canonical.parent!=canonical,"FILESYSTEM_ROOT_NOT_SUPPORTED")
    return canonical,key,canonical.parent/(".nordoi-app028-"+key)


def inspect(root):
    _,key,claim=location(root)
    if not claim.exists() and not claim.is_symlink():
        observed="NO_LOCAL_RESERVATION_OBSERVED"
    elif claim.is_symlink() or not claim.is_dir():
        observed="RESERVATION_OBSTACLE_BLOCKED"
    elif (claim/"COMPLETE.json").exists() or (claim/"COMPLETE.json").is_symlink():
        observed="COMPLETION_RECEIPT_PRESENT_UNVERIFIED_BLOCKED"
    else:
        observed="RESERVATION_PRESENT_ACTIVE_OR_INTERRUPTED_BLOCKED"
    return {"schema":SCHEMA,"workspace_identity":key,"status":observed,
            "execution_authorized":False,"automatic_retry":False,
            "global_antirollback":"NOT_PROVEN","crash_durability":"UNCLAIMED",
            "production":"NOT_AUTHORIZED"}


def reserve(root,context,protected=()):
    canonical,key,claim=location(root)
    for item in protected:
        p=item.resolve(strict=True)
        need(claim!=p and p not in claim.parents and claim not in p.parents,
             "RESERVATION_OVERLAPS_PROTECTED_EVIDENCE")
    # mkdir is the linearization point on a supported local filesystem.
    # No exists-then-create sequence; do not remove this directory on error.
    try:
        claim.mkdir(mode=0o700)
    except FileExistsError as exc:
        raise previous.forensic.Blocked("LOCAL_RESERVATION_ALREADY_EXISTS_STOP_NO_REPLAY") from exc
    except OSError as exc:
        raise previous.forensic.Blocked("LOCAL_RESERVATION_NOT_CONFIRMED_STOP_NO_REPLAY") from exc
    try:
        need(identity(root)==(canonical,key),"WORKSPACE_IDENTITY_CHANGED")
        previous.write_receipt(claim/"START.json",{"schema":SCHEMA,"workspace_identity":key,
            "status":"RESERVED","context":context,"automatic_retry":False,
            "crash_durability":"UNCLAIMED"})
    except Exception as exc:
        raise previous.forensic.Blocked("RESERVATION_OUTCOME_UNKNOWN_STOP_NO_REPLAY") from exc
    return canonical,key,claim


def execute(old,old_pin,root,suffix,suffix_pin,new,new_pin,engine,engine_pin,source,source_pin,workers,consent):
    need(consent==previous.CONSENT,"EXPLICIT_RESEARCH_EXECUTION_REQUIRED")
    previous.preflight(old,old_pin,root,suffix,suffix_pin,new,new_pin,
                       engine,engine_pin,source,source_pin,workers)
    canonical,key,claim=reserve(root,{"old_pin":old_pin,"suffix_pin":suffix_pin,
        "new_pin":new_pin,"engine_pin":engine_pin,"source_pin":source_pin,
        "workers":workers},(old,new,suffix,engine,source))
    try:
        need(identity(root)==(canonical,key),"WORKSPACE_IDENTITY_CHANGED")
        # Reuse APP-027, which rechecks the entire input chain before effects.
        result=previous.execute(old,old_pin,canonical,suffix,suffix_pin,new,new_pin,
                                engine,engine_pin,source,source_pin,workers,
                                claim/"native-attempt",consent)
        need(result["status"]=="COMPLETE_VERIFIED","NATIVE_COMPLETION_NOT_CONFIRMED")
        final={"schema":SCHEMA,"status":"COMPLETE_VERIFIED","workspace_identity":key,
            "continuation":result,"reservation_retained":True,"automatic_retry":False,
            "global_antirollback":"NOT_PROVEN","crash_durability":"UNCLAIMED",
            "formal_bureau_certification":"NOT_CERTIFIED","production":"NOT_AUTHORIZED"}
        previous.write_receipt(claim/"COMPLETE.json",final)
        return final
    except Exception as exc:
        raise previous.forensic.Blocked("RESERVED_ATTEMPT_UNKNOWN_STOP_NO_REPLAY") from exc


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    status=sub.add_parser("status")
    status.add_argument("--root",type=Path,required=True)
    run=sub.add_parser("run")
    for name in ("old","root","suffix","new","engine","source"):
        run.add_argument("--"+name,type=Path,required=True)
    for name in ("old-pin","suffix-pin","new-pin","engine-pin","source-pin","consent"):
        run.add_argument("--"+name,required=True)
    run.add_argument("--workers",type=int,choices=(1,2),required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        result=inspect(**args) if action=="status" else execute(**args)
        rc=0
    except Exception as exc:
        result={"schema":SCHEMA,"status":"BLOCK","reason":getattr(exc,"code","INPUT_OR_RESERVATION_NOT_VERIFIED"),
                "next_action":"STOP_NO_AUTOMATIC_NATIVE_REPLAY","production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(result,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
