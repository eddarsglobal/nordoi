#!/usr/bin/env python3
"""APP-025: READ-ONLY recovery classification for interrupted NORDOI DAGs.

Reconcile an APP-014 portable checkpoint and an APP-024 captured native
suffix, both bound to host-supplied SHA-256 pins, against actual P2.7 files.
Never launch Rust, repair, delete, overwrite, admit new tasks, or claim that
an unlogged file can be recovered as a committed native operation.

All business decisions and native receipts come from genuine NORDOI Rust.
Coherence-only SHA-256 is not external authentication; this is NOT durable
antirollback, crash-safe replay, OS-level isolation or Bureau certification.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app014_portable_checkpoint"))
import portable_app014 as portable

audit=portable.independent
native=portable.audit
PIN=portable.pinned.PIN
SCHEMA="NORDOI_APP025_READONLY_INTERRUPTED_DAG_FORENSICS_V1"
MAX_SUFFIX_BYTES=audit.MAX_EVENTS_BYTES
ALLOWED={"COMMITTED","SKIPPED"}


class Blocked(Exception):
    def __init__(self,reason):
        self.code=str(reason)
        super().__init__(self.code)


def must(condition,reason):
    if not condition:
        raise Blocked(reason)


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def reconcile(checkpoint:Path,checkpoint_pin:str,root:Path,
              suffix:Path,suffix_pin:str)->dict:
    for pin in (checkpoint_pin,suffix_pin):
        must(type(pin) is str and PIN.fullmatch(pin) is not None,
             "TRUSTED_EXTERNAL_SHA256_PIN_REQUIRED")

    manifest,_,historical=portable.verify(checkpoint,checkpoint_pin)
    native.real_directory(root,"RESTORED_ROOT_NOT_REAL")
    prefix_file=checkpoint/"handoff"/"native-prefix.jsonl"
    plan=checkpoint/"handoff"/"plan.json"
    prefix=native.bounded_regular_file(
        prefix_file,audit.MAX_EVENTS_BYTES,"ORIGINAL_PREFIX_FILE_INVALID")
    must(sha256(prefix)==manifest["prefix_sha256"],
         "ORIGINAL_PREFIX_HASH_CHANGED")
    must(len(historical)==manifest["events"] and len(historical)>0,
         "ORIGINAL_PREFIX_INVALID")
    nodes,plan_sha=audit.plan_nodes(plan,manifest["plan_sha256"])
    must(len(historical)<len(nodes),
         "CHECKPOINT_NOT_INCOMPLETE")
    must(all(x["status"] in ALLOWED for x in historical),
         "UNCERTAIN_HISTORICAL_EVENT")

    raw=native.bounded_regular_file(suffix,MAX_SUFFIX_BYTES,
                                    "SUFFIX_NOT_REGULAR_OR_BOUNDED")
    must(sha256(raw)==suffix_pin,"SUFFIX_EXTERNAL_PIN_MISMATCH")
    must(len(prefix)+len(raw)<=audit.MAX_EVENTS_BYTES,
         "JOINED_EVENTS_EXCEED_BOUNDS")
    must(not raw or raw.endswith(b"\n"),"SUFFIX_TRUNCATED_OR_UNTERMINATED")
    if raw:
        # Prevent false prefix extension across LF/CRLF conventions.
        original_crlf=prefix.endswith(b"\r\n")
        lines=raw.split(b"\n")[:-1]
        must(len(lines)<=len(nodes)-len(historical),
             "SUFFIX_MORE_EVENTS_THAN_DAG")
        must(all(bool(line) and line.endswith(b"\r")==original_crlf
                 for line in lines),
             "SUFFIX_NEWLINE_STYLE_CHANGED")

    # The genuine APP-010 auditor verifies all native job bytes against each
    # receipt, exact topology/decisions, event shape and root contents.
    # It accepts no orphan directory and detects unexpected files/symlinks.
    with tempfile.TemporaryDirectory(prefix="nordoi_app025_readonly_") as d:
        joined=Path(d)/"joined-native-events.jsonl"
        joined.write_bytes(prefix+raw)
        records,_=audit.event_records(joined)
        must(records[:len(historical)]==historical,
             "HISTORICAL_EVENTS_CHANGED")
        added=records[len(historical):]
        must(all(x.get("status") in ALLOWED for x in added),
             "UNCERTAIN_OR_FAILED_NATIVE_SUFFIX")
        result,code=audit.audit(root,plan,joined,plan_sha)

    must(result["status"] in ("INCONCLUSIVE","PASS"),
         "NATIVE_DAG_STATE_UNEXPECTED")
    must(code in (0,3) and result["events_checked"]==len(records),
         "NATIVE_DAG_AUDIT_NOT_VERIFIED")
    must(result["committed_jobs_verified"]>=manifest["committed"],
         "HISTORICAL_COMMIT_LOST")
    must(result["skipped_nodes_verified"]>=manifest["skipped"],
         "HISTORICAL_SKIP_LOST")
    must(all(x["status"] in ALLOWED for x in records),
         "UNVERIFIED_EXECUTION_EVENT")

    # Conservative recovery policy:
    #  - Fully completed verified DAG is terminal.
    #  - No suffix, no new directory => original APP-024 checkpoint reusable.
    #  - Clean, incomplete extended prefix => a NEW externally pinned
    #    checkpoint must be issued first. Original APP-014 MUST NOT be replayed.
    #  - Any orphan/missing/unverified effect => BLOCK (the audit already did).
    if len(records)==len(nodes):
        state="COMPLETE_VERIFIED"
        next_action="DO_NOT_REPLAY_COMPLETED_DAG"
    elif not added:
        state="ORIGINAL_CHECKPOINT_REUSABLE"
        next_action="APP024_MAY_RESUME_FROM_UNCHANGED_ORIGINAL_CHECKPOINT"
    else:
        state="CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT"
        next_action="SEAL_NEW_CHECKPOINT_BEFORE_ANY_FURTHER_RESUME"

    return {
      "schema":SCHEMA,
      "status":state,
      "basis":"INDEPENDENT_APP010_REAL_P27_FILE_AND_RECEIPT_AUDIT",
      "checkpoint_sha256":checkpoint_pin,
      "plan_sha256":plan_sha,
      "original_prefix_sha256":manifest["prefix_sha256"],
      "suffix_sha256":suffix_pin,
      "combined_events_sha256":sha256(prefix+raw),
      "original_events":len(historical),
      "new_verified_events":len(added),
      "verified_events_total":len(records),
      "total_dag_nodes":len(nodes),
      "verified_native_jobs":result["committed_jobs_verified"],
      "skipped_nodes":result["skipped_nodes_verified"],
      "next_action":next_action,
      "actual_user_data_files_written":0,
      "native_rust_processes_started_by_auditor":0,
      "external_pin_authentication":"NOT_PROVEN",
      "global_antirollback":"NOT_PROVEN",
      "hostile_host_concurrency":"NOT_COVERED",
      "formal_bureau_certification":"NOT_CERTIFIED",
      "production":"NOT_AUTHORIZED"
    }


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--portable-dir",required=True)
    p.add_argument("--expected-portable-sha256",required=True)
    p.add_argument("--restore-root",required=True)
    p.add_argument("--suffix-events",required=True)
    p.add_argument("--expected-suffix-sha256",required=True)
    a=p.parse_args(argv)
    try:
        report=reconcile(Path(a.portable_dir),a.expected_portable_sha256,
                         Path(a.restore_root),Path(a.suffix_events),
                         a.expected_suffix_sha256)
        print(json.dumps(report,sort_keys=True,separators=(",",":")))
        return 0
    except (Blocked,portable.Refused,portable.inner.Refused,
            native.AuditRejected,portable.pinned.Denied,OSError,
            ValueError,TypeError,KeyError) as e:
        print(json.dumps({
            "schema":SCHEMA,"status":"UNCERTAIN_BLOCKED",
            "reason":getattr(e,"code","EVIDENCE_NOT_VERIFIED"),
            "next_action":"STOP_NO_AUTOMATIC_NATIVE_REPLAY",
            "actual_user_data_files_written":0,
            "native_rust_processes_started_by_auditor":0,
            "formal_bureau_certification":"NOT_CERTIFIED",
            "production":"NOT_AUTHORIZED"
        },sort_keys=True,separators=(",",":")))
        return 3


if __name__=="__main__":
    raise SystemExit(main())
