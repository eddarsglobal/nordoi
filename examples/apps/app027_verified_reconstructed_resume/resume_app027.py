#!/usr/bin/env python3
"""Explicit continuation of a reconstructed checkpoint on a quiescent host.
No automatic restoration or retry, no new native privileges. A local attempt
record is NOT a global anti-replay ledger or a crash-durability guarantee.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app025_interrupted_dag_forensics"))
import audit_app025 as forensic
sys.path.insert(0,str(HERE.parent/"app024_verified_concurrent_resume"))
import run_app024 as resume

portable=forensic.portable
native=forensic.native
need=forensic.must
SCHEMA="NORDOI_APP027_VERIFIED_RECONSTRUCTED_RESUME_V1"
CONSENT="EXECUTE_REMAINING_NATIVE_RESEARCH_TASKS"


def separate(a,b):
    a,b=a.resolve(strict=True),b.resolve(strict=True)
    return a!=b and a not in b.parents and b not in a.parents


def preflight(old,old_pin,root,suffix,suffix_pin,new,new_pin,engine,engine_pin,source,source_pin,workers):
    need(type(workers) is int and workers in (1,2),"INVALID_WORKERS")
    for pin in (old_pin,suffix_pin,new_pin,engine_pin,source_pin):
        need(type(pin) is str and forensic.PIN.fullmatch(pin) is not None,"EXTERNAL_PIN_REQUIRED")
    for a,b in ((root,old),(root,new),(old,new),(root,source)):
        need(separate(a,b),"EVIDENCE_OR_WORKSPACE_OVERLAP")
    report=forensic.reconcile(old,old_pin,root,suffix,suffix_pin)
    need(report["status"]=="CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT","NOT_VERIFIED_PROPER_EXTENSION")
    previous,_,old_events=portable.verify(old,old_pin)
    current,_,events=portable.verify(new,new_pin)
    need(old_pin!=new_pin and current["plan_sha256"]==previous["plan_sha256"]
         and current["prefix_sha256"]==report["combined_events_sha256"]
         and len(events)==report["verified_events_total"]
         and len(old_events)<len(events)<report["total_dag_nodes"]
         and events[:len(old_events)]==old_events
         and current["committed"]==report["verified_native_jobs"]
         and current["skipped"]==report["skipped_nodes"],"RECONSTRUCTED_CHAIN_MISMATCH")
    inv={x["path"]:x for x in current["native_files"]}
    need(all(inv.get(x["path"])==x for x in previous["native_files"]),"HISTORICAL_BYTES_CHANGED")
    portable.restored_root_matches(root,new,events,current)
    need(resume.pinned.engine_sha256(engine)==engine_pin,"ENGINE_PIN_MISMATCH")
    need(resume.pinned.source_tree_sha256(source)==source_pin,"SOURCE_PIN_MISMATCH")
    return report,current


def write_receipt(path,data):
    portable.inner.exclusive_file(path, (json.dumps(data,sort_keys=True)+"\n").encode("utf-8"))


def execute(old,old_pin,root,suffix,suffix_pin,new,new_pin,engine,engine_pin,source,source_pin,workers,attempt,consent):
    need(consent==CONSENT,"EXPLICIT_RESEARCH_EXECUTION_REQUIRED")
    report,current=preflight(old,old_pin,root,suffix,suffix_pin,new,new_pin,engine,engine_pin,source,source_pin,workers)
    attempt=attempt.absolute()
    native.real_directory(attempt.parent,"ATTEMPT_PARENT_NOT_REAL")
    for item in (root,old,new,source):
        need(separate(attempt.parent,item) or attempt.parent.resolve(strict=True) in item.resolve(strict=True).parents,
             "ATTEMPT_OVERLAPS_PROTECTED_TREE")
        # Reject paths nested within any protected tree; common parent is fine.
        p=attempt.parent.resolve(strict=True)
        x=item.resolve(strict=True)
        need(p!=x and x not in p.parents,"ATTEMPT_OVERLAPS_PROTECTED_TREE")
    need(not attempt.exists() and not attempt.is_symlink(),"ATTEMPT_ALREADY_EXISTS")
    attempt.mkdir(mode=0o700)
    # Once the attempt exists, any unexpected result is UNKNOWN; never auto-retry.
    try:
        write_receipt(attempt/"START.json",{"schema":SCHEMA,"status":"ATTEMPT_STARTED",
            "original_pin":old_pin,"reconstructed_pin":new_pin,"captured_suffix_pin":suffix_pin,
            "engine_pin":engine_pin,"source_pin":source_pin,"historical_events":current["events"],
            "workers":workers,"automatic_retry":False})
        output=attempt/"native-suffix.jsonl"
        cmd=[sys.executable,"-B",str(HERE.parent/"app024_verified_concurrent_resume"/"run_app024.py"),
            "--portable-dir",str(new),"--expected-portable-sha256",new_pin,"--restore-root",str(root),
            "--engine",str(engine),"--source-root",str(source),"--expected-engine-sha256",engine_pin,
            "--expected-source-sha256",source_pin,"--max-in-flight",str(workers)]
        with output.open("xb") as out, (attempt/"stderr.log").open("xb") as err:
            proc=subprocess.run(cmd,stdout=out,stderr=err,timeout=240,check=False)
            out.flush()
            os.fsync(out.fileno())
        need(proc.returncode==0,"NATIVE_RESUME_NOT_CONFIRMED")
        raw=native.bounded_regular_file(output,forensic.MAX_SUFFIX_BYTES,"CAPTURED_SUFFIX_INVALID")
        pin=hashlib.sha256(raw).hexdigest()
        audit=forensic.reconcile(new,new_pin,root,output,pin)
        need(audit["status"]=="COMPLETE_VERIFIED"
             and audit["verified_events_total"]==report["total_dag_nodes"]
             and audit["new_verified_events"]==report["total_dag_nodes"]-current["events"],
             "FULL_DAG_AUDIT_NOT_CONFIRMED")
        portable.verify(old,old_pin)
        portable.verify(new,new_pin)
        result={"schema":SCHEMA,"status":"COMPLETE_VERIFIED","original_checkpoint_sha256":old_pin,
            "reconstructed_checkpoint_sha256":new_pin,"new_suffix_sha256":pin,
            "historical_events":current["events"],"new_verified_events":audit["new_verified_events"],
            "verified_events_total":audit["verified_events_total"],"verified_native_jobs":audit["verified_native_jobs"],
            "automatic_retry":False,"global_antirollback":"NOT_PROVEN","external_pin_authentication":"NOT_PROVEN",
            "crash_durability":"UNCLAIMED","formal_bureau_certification":"NOT_CERTIFIED","production":"NOT_AUTHORIZED"}
        write_receipt(attempt/"COMPLETE.json",result)
        return result
    except Exception as exc:
        raise forensic.Blocked("ATTEMPT_OUTCOME_UNKNOWN_STOP_NO_REPLAY") from exc


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    for name in ("old","root","suffix","new","engine","source","attempt"):
        p.add_argument("--"+name,type=Path,required=True)
    for name in ("old-pin","suffix-pin","new-pin","engine-pin","source-pin","consent"):
        p.add_argument("--"+name,required=True)
    p.add_argument("--workers",type=int,choices=(1,2),required=True)
    try:
        result=execute(**vars(p.parse_args(argv)))
        rc=0
    except Exception as exc:
        result={"schema":SCHEMA,"status":"BLOCK","reason":getattr(exc,"code","EVIDENCE_OR_ATTEMPT_NOT_VERIFIED"),
                "next_action":"STOP_NO_AUTOMATIC_NATIVE_REPLAY","production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(result,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
