#!/usr/bin/env python3
"""Read-only audit of an APP-028 reservation. Never grants execution.
Requires a stable trusted local host, and independently supplied expected
pins. A missing receipt or incomplete capture is uncertainty, not a retry grant.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app028_local_exclusive_resume"))
import exclusive_app028 as exclusive

previous=exclusive.previous
forensic=previous.forensic
portable=forensic.portable
native=forensic.native
need=forensic.must
SCHEMA="NORDOI_APP029_RESERVATION_FORENSICS_V1"
MAX_RECEIPT=65536


def receipt(path):
    raw=native.bounded_regular_file(path,MAX_RECEIPT,"RECEIPT_NOT_REGULAR_OR_BOUNDED")
    value=forensic.audit.decode_json(raw,"RESERVATION_RECEIPT_INVALID_JSON")
    need(type(value) is dict,"RESERVATION_RECEIPT_NOT_OBJECT")
    return value


def exact(actual,expected,reason):
    # Canonical JSON comparison distinguishes booleans from integer counters.
    need(json.dumps(actual,sort_keys=True,separators=(",",":")) ==
         json.dumps(expected,sort_keys=True,separators=(",",":")),reason)


def audit(old,old_pin,root,suffix,suffix_pin,new,new_pin,engine_pin,source_pin,final_suffix_pin):
    for pin in (old_pin,suffix_pin,new_pin,engine_pin,source_pin,final_suffix_pin):
        need(type(pin) is str and forensic.PIN.fullmatch(pin) is not None,"EXTERNAL_PIN_REQUIRED")
    canonical,key,claim=exclusive.location(root)
    native.real_directory(claim,"RESERVATION_MISSING_OR_NOT_REAL")
    need(previous.separate(old,new) and previous.separate(root,old) and
         previous.separate(root,new),"EVIDENCE_OVERLAP")
    start=receipt(claim/"START.json")
    context=start.get("context")
    need(type(context) is dict and type(context.get("workers")) is int and
         context["workers"] in (1,2),"RESERVATION_CONTEXT_INVALID")
    workers=context["workers"]
    exact(start,{"schema":exclusive.SCHEMA,"workspace_identity":key,
        "status":"RESERVED","context":{"old_pin":old_pin,"suffix_pin":suffix_pin,
          "new_pin":new_pin,"engine_pin":engine_pin,"source_pin":source_pin,"workers":workers},
        "automatic_retry":False,"crash_durability":"UNCLAIMED"},"RESERVATION_START_MISMATCH")
    attempt=claim/"native-attempt"
    native.real_directory(attempt,"NATIVE_ATTEMPT_MISSING_OR_NOT_REAL")
    native_start=receipt(attempt/"START.json")
    outer_final=receipt(claim/"COMPLETE.json")
    inner_final=receipt(attempt/"COMPLETE.json")
    # Old prefix plus captured partial suffix must agree with the independently
    # pinned reconstructed package, not the now-completed working directory.
    old_m,_,old_events=portable.verify(old,old_pin)
    new_m,_,events=portable.verify(new,new_pin)
    partial=forensic.reconcile(old,old_pin,new/"native",suffix,suffix_pin)
    need(partial["status"]=="CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT" and
         old_pin!=new_pin and new_m["plan_sha256"]==old_m["plan_sha256"] and
         new_m["prefix_sha256"]==partial["combined_events_sha256"] and
         len(events)==partial["verified_events_total"] and
         len(old_events)<len(events)<partial["total_dag_nodes"] and
         events[:len(old_events)]==old_events,"RECONSTRUCTED_HISTORY_MISMATCH")
    inventory={x["path"]:x for x in new_m["native_files"]}
    need(all(inventory.get(x["path"])==x for x in old_m["native_files"]),"OLD_NATIVE_INVENTORY_CHANGED")
    exact(native_start,{"schema":previous.SCHEMA,"status":"ATTEMPT_STARTED",
        "original_pin":old_pin,"reconstructed_pin":new_pin,"captured_suffix_pin":suffix_pin,
        "engine_pin":engine_pin,"source_pin":source_pin,"historical_events":len(events),
        "workers":workers,"automatic_retry":False},"NATIVE_START_MISMATCH")
    full=forensic.reconcile(new,new_pin,canonical,attempt/"native-suffix.jsonl",final_suffix_pin)
    need(full["status"]=="COMPLETE_VERIFIED" and
         full["verified_events_total"]==partial["total_dag_nodes"] and
         full["new_verified_events"]==partial["total_dag_nodes"]-len(events),
         "FULL_NATIVE_DAG_NOT_VERIFIED")
    expected={"schema":previous.SCHEMA,"status":"COMPLETE_VERIFIED",
        "original_checkpoint_sha256":old_pin,"reconstructed_checkpoint_sha256":new_pin,
        "new_suffix_sha256":final_suffix_pin,"historical_events":len(events),
        "new_verified_events":full["new_verified_events"],"verified_events_total":full["verified_events_total"],
        "verified_native_jobs":full["verified_native_jobs"],"automatic_retry":False,
        "global_antirollback":"NOT_PROVEN","external_pin_authentication":"NOT_PROVEN",
        "crash_durability":"UNCLAIMED","formal_bureau_certification":"NOT_CERTIFIED","production":"NOT_AUTHORIZED"}
    exact(inner_final,expected,"NATIVE_COMPLETION_RECEIPT_MISMATCH")
    exact(outer_final,{"schema":exclusive.SCHEMA,"status":"COMPLETE_VERIFIED",
        "workspace_identity":key,"continuation":expected,"reservation_retained":True,
        "automatic_retry":False,"global_antirollback":"NOT_PROVEN","crash_durability":"UNCLAIMED",
        "formal_bureau_certification":"NOT_CERTIFIED","production":"NOT_AUTHORIZED"},
        "RESERVATION_COMPLETION_RECEIPT_MISMATCH")
    need(exclusive.identity(root)==(canonical,key),"WORKSPACE_IDENTITY_CHANGED")
    return {"schema":SCHEMA,"status":"COMPLETE_EVIDENCE_VERIFIED","workspace_identity":key,
        "historical_events":len(events),"new_verified_events":full["new_verified_events"],
        "verified_events_total":full["verified_events_total"],"verified_native_jobs":full["verified_native_jobs"],
        "final_suffix_sha256":final_suffix_pin,"execution_authorized":False,"reservation_release_authorized":False,
        "automatic_retry":False,"native_processes_started":0,"global_antirollback":"NOT_PROVEN",
        "external_pin_authentication":"NOT_PROVEN","engine_execution_identity_attested":False,
        "crash_durability":"UNCLAIMED","formal_bureau_certification":"NOT_CERTIFIED","production":"NOT_AUTHORIZED"}


def classify(**args):
    try:
        return audit(**args)
    except Exception as exc:
        return {"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED",
            "reason":getattr(exc,"code","EVIDENCE_NOT_VERIFIED"),"execution_authorized":False,
            "reservation_release_authorized":False,"automatic_retry":False,"native_processes_started":0,
            "production":"NOT_AUTHORIZED","next_action":"STOP_NO_AUTOMATIC_NATIVE_REPLAY"}


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    for name in ("old","root","suffix","new"):
        p.add_argument("--"+name,type=Path,required=True)
    for name in ("old-pin","suffix-pin","new-pin","engine-pin","source-pin","final-suffix-pin"):
        p.add_argument("--"+name,required=True)
    result=classify(**vars(p.parse_args(argv)))
    print(json.dumps(result,sort_keys=True))
    return 0 if result["status"]=="COMPLETE_EVIDENCE_VERIFIED" else 3

if __name__=="__main__":
    raise SystemExit(main())
