#!/usr/bin/env python3
"""NORDOI R6F E9: actual event-vs-SQLite ledger mismatch, research deny only.

E8's caller-supplied local event hash is not a protected ledger receipt. A
forged event can hide E8's conflict *flag* while all outcomes still BLOCK.
E9 checks the event against the actual persisted row, but cannot attest the
administrator did not rewrite both rows or hide an entire fork.
"""
import argparse
from contextlib import closing
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
import bureau_001_e8_local_ledger_split_brain as e8
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E9_EVENT_LEDGER_CONTRACT.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E9-RECORDED-EVENT-LEDGER-CONSISTENCY-V1"
PARENT="bf4c0d66256ad3a871e44cae39808af5ea1418d4"
STEPS=[
    "two_real_fork_sqlite_commits_with_genuine_e6_ssh_signature",
    "actual_e8_split_brain_detected_before_forgery",
    "recompute_local_b_event_hash_for_different_nonce",
    "actual_e8_reconcile_false_negative_conflict_flag",
    "new_guard_rejects_event_to_sqlite_row_mismatch",
    "unchanged_authentic_records_still_conflict_block"]
ROLES=["ATLAS","FERRUM","BOUNDARY","CIPHER","LEDGER","CAPSTONE",
       "ADVERSARY","VERITY","GUARDIAN"]
class Blocked(ValueError): pass

def require(ok,why):
    if not ok:raise Blocked(why)

def outcome(reason,**extra):
    return e8.result(reason,**extra)

def audit(root,head):
    prior=e8.audit(root,head)
    c=e3.strictly_load(e3.real_file_bytes(Path(root)/CONTRACT))
    require(c["schema"]==SCHEMA and
            c["project"]=="eddarsglobal/nordoi" and
            c["parent_e8_exact_head"]==PARENT and
            c["prior_e8"]=={"specialized_run_id":44036977230,
                           "specialized_success":4,
                           "rust_run_id":44036977539,
                           "rust_success":5},
            "E9_PARENT_E8_OR_GREEN_C20_NOT_PINNED")
    require(c["research_steps"]==STEPS and
            c["bureau_functions"]==ROLES and
            c["independent_bureau_agents_executed"]==0 and
            c["independent_specialist_reviews"]==0,
            "E9_FALSE_BUREAU_INDEPENDENCE")
    require(c["human_testers_allowed"] is False and
            c["human_testers_executed"]==0 and
            c["external_witness_receipts"]==0 and
            c["no_network"] is True and
            c["no_export_external_keys"] is True and
            c["permission_to_merge"] is False and
            c["external_custodian"]=="NOT_ENROLLED" and
            c["owner_oob_root"]=="NOT_AUTHENTICATED" and
            c["global_monotonic_antirollback"]=="NOT_VERIFIED" and
            c["external_complete_fork_enumeration"]=="NOT_VERIFIED" and
            c["formal_r6f_closure"]=="HOLD_MISSING_INDEPENDENT_EVIDENCE" and
            c["native_admission_gates_unmet"]==12 and
            c["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
            c["r6g"]=="LOCKED_BY_SEQUENCE" and
            c["production"]=="NOT_AUTHORIZED" and
            c["research_policy"]==
            "LOCAL_NEGATIVE_PROOF_ONLY_ZERO_HUMANS_DENY_ALL_NATIVE_EFFECTS",
            "E9_MISSING_SECURITY_AND_OWNER_NO_HUMAN_LOCKS")
    require(prior["result"]=="BLOCK" and
            prior["native_admission_gates_unmet"]==12,
            "E9_PREDECESSOR_BLOCK_INVALID")
    return outcome("E9_EVENT_TO_PERSISTED_LEDGER_AUDIT_ONLY",
                   source_head=head,live_experiment_performed=False)

def record(root,fork):
    db=e8.fork_path(root,fork)
    require(db.is_file() and not db.is_symlink(),
            "E9_REAL_FORK_DATABASE_MISSING_OR_SYMLINK")
    # Read only SELECT statements on real disk DB, owned by disposable lab.
    with closing(sqlite3.connect(str(db),timeout=15)) as conn:
        rows=conn.execute(
            "SELECT namespace,challenge_nonce_sha256,receipt_sha256,source_head "
            "FROM research_local_spent ORDER BY namespace,challenge_nonce_sha256"
            ).fetchall()
    require(len(rows)==1,"E9_EXPECT_EXACTLY_ONE_REAL_LEDGER_RECORD")
    return tuple(rows[0])

def guard_record(root,event,nonce,receipt,head):
    """Correlates locally supplied event to a real row, NOT global history."""
    require(type(event) is dict and
            set(event)=={"fork","nonce","receipt","head",
                         "event_sha256","local_commit_only"},
            "E9_EVENT_SCHEMA_INVALID")
    fork=event["fork"]
    require(fork in e8.FORKS and
            event==e8.signed_event(fork,event["nonce"],
                                   event["receipt"],event["head"]),
            "E9_EVENT_HASH_OR_FORMAT_INVALID")
    require(all(type(x) is str and e8.e7.e6.SHA.fullmatch(x)
                for x in (nonce,receipt)) and
            type(head) is str and e8.e7.e6.SHA40.fullmatch(head),
            "E9_EXPECTED_SIGNED_CONTEXT_INVALID")
    expected=(e8.e7.NAMESPACE,nonce,receipt,head)
    require(record(root,fork)==expected and
            event["nonce"]==nonce and
            event["receipt"]==receipt and
            event["head"]==head,
            "E9_EVENT_PERSISTED_ROW_MISMATCH")
    return True

def guarded_reconcile(root,events,nonce,receipt,head):
    require(type(events) is list and len(events)==2,
            "E9_INCOMPLETE_KNOWN_BRANCH_PROOFS")
    require({x.get("fork") for x in events if type(x) is dict}==
            set(e8.FORKS),"E9_KNOWN_FORKS_NOT_BOTH_SUPPLIED")
    for event in events:
        guard_record(root,event,nonce,receipt,head)
    d=e8.reconcile(events)
    require(d["result"]=="BLOCK" and
            d.get("split_brain_detected") is True,
            "E9_KNOWN_LOCAL_FORK_CONFLICT_NOT_DETECTED")
    return outcome("E9_REAL_PERSISTED_ROWS_AND_EVENTS_CONFLICT_BLOCK",
        verified_known_local_forks=2,
        local_rows_verified=2,
        local_split_brain_detected=True,
        true_external_completeness=False)

def launch_real_e8_workers(root,head):
    procs=[]
    try:
        for fork in e8.FORKS:
            p=subprocess.Popen([
                sys.executable,"-B",str(Path(e8.__file__).resolve()),
                "--worker",fork,"--lab-dir",str(root),
                "--expected-head",head],
                cwd=ROOT,stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,text=True)
            procs.append(p)
        results=[]
        for p in procs:
            stdout,stderr=p.communicate(timeout=75)
            require(p.returncode==2,
                    "E9_REAL_E8_CHILD_FAILED:"+stderr[:160])
            d=json.loads(stdout)
            require(d["result"]=="BLOCK" and
                    d["true_local_ssh_signature_verified"] is True and
                    d["native_authority_granted"] is False and
                    d["native_admission_gates_unmet"]==12,
                    "E9_FALSE_CHILD_CRYPTO_OR_NATIVE_AUTHORITY")
            results.append(d)
        require({d["fork"] for d in results}==set(e8.FORKS) and
                len({d["process_pid"] for d in results})==2,
                "E9_TRUE_DISTINCT_CHILD_PROCESS_OR_FORK_MISSING")
        return results
    finally:
        for p in procs:
            if p.poll() is None:
                p.kill()
                p.communicate()

def exercise(root,head):
    audit(root,head)
    require(e8.e7.e6.r4.is_posix_ssh(),
            "E9_TRUE_POSIX_OPENSSH_REQUIRED")
    with tempfile.TemporaryDirectory(prefix=e8.LAB_PREFIX) as tmp:
        home=Path(tmp)
        os.chmod(home,0o700)
        fixture=e8.e7.e6.local_crypto_fixture(home,head)
        original=e8.e7.initialize(home)
        for fork in e8.FORKS:
            shutil.copy2(original,e8.fork_path(home,fork))
        outputs=launch_real_e8_workers(home,head)
        events=[x["event"] for x in outputs]
        nonce,receipt=e8.e7.local_signed_context(home,head)
        require(nonce==fixture["nonce"],"E9_TRUE_SIGNED_NONCE_MISMATCH")
        require(all(e8.e7.count_nonce(e8.fork_path(home,f),nonce)==1
                    for f in e8.FORKS),
                "E9_BOTH_REAL_DISK_LEDGER_SPENDS_NOT_CONFIRMED")
        honest=e8.reconcile(events)
        require(honest["result"]=="BLOCK" and
                honest["split_brain_detected"] is True,
                "E9_E8_HONEST_CONFLICT_NOT_REPRODUCED")
        before=guarded_reconcile(home,events,nonce,receipt,head)
        require(before["local_split_brain_detected"] is True,
                "E9_NEW_GUARD_HONEST_CONFLICT_NOT_FOUND")
        fake_nonce="f"*64 if nonce!="f"*64 else "0"*64
        forged=[dict(x) for x in events]
        bidx=next(i for i,x in enumerate(forged) if x["fork"]=="B")
        forged[bidx]=e8.signed_event("B",fake_nonce,receipt,head)
        old=e8.reconcile(forged)
        require(old["result"]=="BLOCK" and
                old.get("split_brain_detected") is False and
                old["reason"]==
                    "E8_LOCAL_EVENTS_NONMATCHING_NO_EXTERNAL_COMPLETENESS_BLOCK",
                "E9_OLD_LOCAL_RECONCILER_FALSE_NEGATIVE_NOT_OBSERVED")
        guarded_rejected=False
        try:
            guarded_reconcile(home,forged,nonce,receipt,head)
        except Blocked as err:
            guarded_rejected=str(err)=="E9_EVENT_PERSISTED_ROW_MISMATCH"
        require(guarded_rejected,
                "E9_NEW_GUARD_DID_NOT_REJECT_EVENT_ROW_SUBSTITUTION")
        require(all(e8.e7.count_nonce(e8.fork_path(home,f),nonce)==1
                    for f in e8.FORKS),
                "E9_PERSISTED_REAL_FORK_CONTENTS_UNEXPECTEDLY_CHANGED")
    require(e3.check_head(root,head)==head,"E9_GIT_HEAD_CHANGED")
    return outcome("E9_REAL_LOCAL_EVENT_LEDGER_SUBSTITUTION_CAUGHT_DENY_ONLY",
        exact_source_head=head,genuine_local_ssh_signatures_verified=True,
        actual_distinct_e8_worker_processes=2,
        real_independent_sqlite_forks=2,
        honest_e8_conflict=True,
        falsified_local_event_hash_recomputed=True,
        old_e8_split_brain_conflict_flag_false_negative=True,
        old_e8_result_remained_block=True,
        persisted_sqlite_rows_unchanged=True,
        new_event_to_sqlite_guard_rejected=True,
        genuine_unmodified_events_reconcile_to_block=True,
        external_complete_history_proven=False,
        real_external_witness_receipts=0)

def main(argv=None):
    ap=argparse.ArgumentParser(description=__doc__)
    mode=ap.add_mutually_exclusive_group(required=True)
    mode.add_argument("--audit",action="store_true")
    mode.add_argument("--exercise",action="store_true")
    ap.add_argument("--expected-head",required=True)
    ap.add_argument("--root",default=str(ROOT))
    a=ap.parse_args(argv)
    try:
        d=audit(a.root,a.expected_head) if a.audit else exercise(a.root,a.expected_head)
        code=2
    except (Blocked,e8.Blocked,e8.e7.Blocked,e8.e7.e6.Blocked,
            e3.Rejected,ValueError,KeyError,TypeError,sqlite3.Error,
            OSError,subprocess.SubprocessError) as ex:
        d=outcome("E9_SOURCE_OR_EXPERIMENT_NOT_QUALIFIED",
                  error_type=type(ex).__name__)
        code=3
    print(json.dumps(d,sort_keys=True))
    return code

if __name__=="__main__":raise SystemExit(main())
