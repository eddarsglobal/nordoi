#!/usr/bin/env python3
"""E8 real independent SQLite forks: valid local atomic commits, global conflict.

Research-only, disposable local fixture and real OS subprocesses. All CLI
success paths return BLOCK (exit 2), errors return 3. Never native admission.
"""
import argparse
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e7_atomic_local_nonce_race as e7
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E8_LEDGER_SPLIT_BRAIN_CONTRACT.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E8-LOCAL-LEDGER-SPLIT-BRAIN-V1"
PARENT="81c84f116b3ae6003b84c335e47af4d1b84637ef"
FORKS=("A","B")
LAB_PREFIX=e7.LAB_PREFIX
class Blocked(ValueError):pass

def check(ok,reason):
    if not ok:raise Blocked(reason)

def result(reason,**values):
    d=e7.proof(reason)
    d.update({k:v for k,v in values.items() if k not in d})
    return d

def audit(root,head):
    original=e7.audit(root,head)
    c=e3.strictly_load(e3.real_file_bytes(Path(root)/CONTRACT))
    check(c["schema"]==SCHEMA and c["parent_e7_exact_head"]==PARENT
        and c["project"]=="eddarsglobal/nordoi"
        and c["prior_e7"]=={"specialized_run_id":44033036777,
                           "specialized_success":4,"rust_run_id":44033036774,
                           "rust_success":5}, "E8_PRIOR_E7_NOT_PINNED")
    check(c["experiment_stages"]==[
        "fork_two_closed_sqlite_files",
        "true_distinct_process_signed_receipt_verification",
        "one_atomic_local_commit_per_fork",
        "local_reconciliation_detects_double_branch_event",
        "withheld_branch_evidence_fail_closed"],"E8_STAGES_NOT_PINNED")
    check(c["automated_bureau_functions"]==[
       "ATLAS","FERRUM","BOUNDARY","CIPHER","LEDGER","CAPSTONE",
       "ADVERSARY","VERITY","GUARDIAN"] and
       c["actual_independent_bureau_agents_executed"]==0 and
       c["independent_specialist_reviews"]==0,
       "E8_FAKE_INDEPENDENT_BUREAU")
    check(c["human_testers_allowed"] is False and
          c["physical_test_humans"]==0 and c["no_network"] is True and
          c["no_credentials_export"] is True and
          c["independent_external_receipts"]==0 and
          c["external_custodian"]=="NOT_ENROLLED" and
          c["owner_oob_root"]=="NOT_AUTHENTICATED" and
          c["global_monotonic_antirollback"]=="NOT_VERIFIED" and
          c["formal_r6f_closure"]=="HOLD_MISSING_INDEPENDENT_EVIDENCE" and
          c["native_admission_gates_unmet"]==12 and
          c["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
          c["r6g"]=="LOCKED_BY_SEQUENCE" and
          c["production"]=="NOT_AUTHORIZED" and
          c["permission_to_merge"] is False and
          c["verification_claim"]==
          "LOCAL_FORK_DETECTION_RESEARCH_ONLY_NOT_PREVENTION_OF_ALL_BRANCHES",
          "E8_EXTERNAL_AUTHORITY_OR_NATIVE_GATE_FALSIFIED")
    check(original["result"]=="BLOCK" and
          original["native_admission_gates_unmet"]==12,
          "E8_PREDECESSOR_AUDIT_NOT_BLOCKED")
    return result("E8_LOCAL_FORK_PROTOCOL_ONLY",source_head=head)

def fork_path(root,fork):
    root=e7.lab(root)
    check(fork in FORKS,"E8_UNKNOWN_FORK")
    p=root/("e8_fork_"+fork+".sqlite3")
    check(p.parent==root and not p.is_symlink(),"E8_INVALID_FORK_PATH")
    return p

def signed_event(fork,nonce,receipt,head):
    check(fork in FORKS and
          all(isinstance(x,str) and e7.e6.SHA.fullmatch(x)
              for x in (nonce,receipt)) and
          e7.e6.SHA40.fullmatch(head),
          "E8_EVENT_CONTEXT_INVALID")
    raw=("E8-LOCAL-"+fork+"|"+nonce+"|"+receipt+"|"+head).encode("ascii")
    return {"fork":fork,"nonce":nonce,"receipt":receipt,"head":head,
            "event_sha256":hashlib.sha256(raw).hexdigest(),
            "local_commit_only":True}

def reconcile(events,expected_forks=FORKS):
    """Detect conflict only in supplied evidence; does NOT attest completeness."""
    check(tuple(expected_forks)==FORKS,"E8_FORK_SCOPE_INVALID")
    if len(events)!=2:
        return result("E8_INCOMPLETE_LOCAL_FORK_EVIDENCE_BLOCK",
                      supplied_forks=len(events),externally_complete=False)
    check(all(type(e) is dict and set(e)=={
        "fork","nonce","receipt","head","event_sha256","local_commit_only"}
        and e.get("local_commit_only") is True for e in events),
        "E8_INVALID_LOCAL_EVENT_SCHEMA")
    for e in events:
        check(e==signed_event(e["fork"],e["nonce"],e["receipt"],e["head"]),
              "E8_EVENT_DIGEST_MISMATCH")
    check({e["fork"] for e in events}==set(FORKS),
          "E8_FORK_EVIDENCE_DUPLICATED")
    overlap=(events[0]["nonce"]==events[1]["nonce"] and
             events[0]["receipt"]==events[1]["receipt"] and
             events[0]["head"]==events[1]["head"])
    if overlap:
        return result("E8_SAME_NONCE_TWO_LOCAL_BRANCH_COMMITS_DETECTED_BLOCK",
              local_forks=2,local_commits=2,split_brain_detected=True,
              external_witnessed_completeness=False)
    return result("E8_LOCAL_EVENTS_NONMATCHING_NO_EXTERNAL_COMPLETENESS_BLOCK",
          split_brain_detected=False,external_witnessed_completeness=False)

def worker(root,head,fork):
    root=e7.lab(root)
    nonce,receipt=e7.local_signed_context(root,head)
    db=fork_path(root,fork)
    check(db.is_file(),"E8_FORK_DATABASE_MISSING")
    # Both forks were copied from the same CLOSED empty SQLite file.
    outcome=e7.consume_atomic(db,nonce,receipt,head)
    check(outcome=="LOCAL_SPENT_RECORD_COMMITTED_NO_NATIVE_ADMISSION",
          "E8_EXPECTED_SINGLE_LOCAL_COMMIT_FAILED")
    check(e7.count_nonce(db,nonce)==1,"E8_LOCAL_RECORD_MISSING")
    event=signed_event(fork,nonce,receipt,head)
    return result("E8_ONE_LOCAL_BRANCH_COMMITTED_NO_NATIVE_AUTHORITY",
                  fork=fork,event=event,process_pid=os.getpid(),
                  true_local_ssh_signature_verified=True,
                  local_nonce_committed=True,
                  native_authority_granted=False)

def exercise(root,head):
    audit(root,head)
    check(e7.e6.r4.is_posix_ssh(),"E8_REAL_POSIX_OPENSSH_REQUIRED")
    with tempfile.TemporaryDirectory(prefix=LAB_PREFIX) as tmp:
        home=Path(tmp)
        os.chmod(home,0o700)
        fixture=e7.e6.local_crypto_fixture(home,head)
        db=e7.initialize(home)
        check(e7.count_nonce(db,fixture["nonce"])==0,
              "E8_NOT_FORKING_EMPTY_LEDGER")
        # Closed database copied into two ACTUAL separate disk files.
        for fork in FORKS:
            shutil.copy2(db,fork_path(home,fork))
        processes=[]
        try:
            for fork in FORKS:
                p=subprocess.Popen([sys.executable,"-B",
                    str(Path(__file__).resolve()),"--worker",fork,
                    "--lab-dir",str(home),"--expected-head",head],
                    cwd=ROOT,stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,text=True)
                processes.append(p)
            outputs=[]
            for p in processes:
                out,err=p.communicate(timeout=75)
                check(p.returncode==2,"E8_CHILD_FAILED:"+err[:150])
                d=json.loads(out)
                check(d["result"]=="BLOCK" and
                      d["native_admission_gates_unmet"]==12 and
                      d["true_local_ssh_signature_verified"] is True and
                      d["native_authority_granted"] is False,
                      "E8_CHILD_FALSE_AUTHORITY")
                outputs.append(d)
            check(len({d["process_pid"] for d in outputs})==2,
                  "E8_DISTINCT_REAL_PROCESS_CHECK_FAILED")
        finally:
            for p in processes:
                if p.poll() is None:
                    p.kill()
                    p.communicate()
        check(all(e7.count_nonce(fork_path(home,fork),
                    fixture["nonce"])==1 for fork in FORKS),
              "E8_FORK_LOCAL_ATOMIC_CONSUMPTION_NOT_PROVEN")
        events=[x["event"] for x in outputs]
        divergence=reconcile(events)
        check(divergence["split_brain_detected"] is True,
              "E8_BOTH_LOCAL_COMMITS_NOT_FLAGGED")
        absent=reconcile(events[:1])
        check(absent["reason"]=="E8_INCOMPLETE_LOCAL_FORK_EVIDENCE_BLOCK",
              "E8_WITHHELD_BRANCH_NOT_DENIED")
        check(e7.count_nonce(db,fixture["nonce"])==0,
              "E8_ORIGINAL_LEDGER_UNEXPECTEDLY_CHANGED")
        check(e7.e3.check_head(root,head)==head,
              "E8_GIT_HEAD_CHANGED")
        return result("E8_REAL_TWO_SQLITE_FORKS_BOTH_LOCALLY_SPENT_RECONCILIATION_DENY",
             exact_source_head=head,real_signed_receipt_verified_by_both=True,
             distinct_os_processes=2,disjoint_sqlite_database_files=2,
             same_nonce_locally_committed_on_each_fork=True,
             local_split_brain_detected_on_reconciliation=True,
             missing_branch_evidence_blocked=True,
             original_pre_fork_ledger_still_unspent=True,
             independent_external_witness_receipts=0,
             globally_monotonic_antirollback_verified=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    opts=p.add_mutually_exclusive_group(required=True)
    opts.add_argument("--audit",action="store_true")
    opts.add_argument("--exercise",action="store_true")
    opts.add_argument("--worker",choices=FORKS)
    p.add_argument("--expected-head",required=True)
    p.add_argument("--root",default=str(ROOT))
    p.add_argument("--lab-dir")
    args=p.parse_args(argv)
    try:
        if args.audit:d=audit(args.root,args.expected_head)
        elif args.exercise:d=exercise(args.root,args.expected_head)
        else:
            check(args.lab_dir is not None,"E8_WORKER_LAB_REQUIRED")
            d=worker(args.lab_dir,args.expected_head,args.worker)
        status=2
    except (Blocked,e7.Blocked,e7.e6.Blocked,e3.Rejected,
            ValueError,KeyError,TypeError,sqlite3.Error,OSError,
            subprocess.SubprocessError) as ex:
        d=result("E8_SOURCE_OR_REAL_SPLIT_BRAIN_EXPERIMENT_NOT_QUALIFIED",
                 error_type=type(ex).__name__)
        status=3
    print(json.dumps(d,sort_keys=True))
    return status

if __name__=="__main__":raise SystemExit(main())
