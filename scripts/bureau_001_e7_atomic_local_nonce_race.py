#!/usr/bin/env python3
"""NORDOI R6F BUREAU-001 E7: REAL simultaneous local nonce check/consume.

Actual subprocesses verify a genuine local OpenSSH Ed25519 E6 receipt.
Two concurrent check-then-act readers can both see 'unspent'. A SQLite
BEGIN IMMEDIATE + UNIQUE(namespace, nonce) transaction guarantees exactly
one local record. A real same-admin rollback of that closed disposable
SQLite file re-enables the local nonce, as does R5Q's genuine three-file
restore: NONE of this authenticates an external witness.

Every CLI result remains BLOCK, with exit 2 for expected denial and
exit 3 for a failed scientific experiment/source check. No humans, external
service, native admission, production, new privileges or Git writes.
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
import time

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e6_signed_receipt_rebinding as e6
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E7_LOCAL_NONCE_RACE_CONTRACT.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E7-ATOMIC-LOCAL-NONCE-CONSUMPTION-V1"
PARENT="ab3858d2c909b91c997782993b5c1739f49286b8"
NAMESPACE=e6.NAMESPACE
TABLE="research_local_spent"
LAB_PREFIX="nordoi-r5l-r5q-r5s-r5t-e7-native-"

class Blocked(ValueError):pass

def require(ok,why):
    if not ok:raise Blocked(why)

def proof(reason,**items):
    fixed=dict(result="BLOCK",reason=reason,
        real_human_testers=0,independent_specialist_reviews=0,
        external_witness_receipts=0,
        independent_custodian="NOT_ENROLLED",
        owner_oob_root="NOT_AUTHENTICATED",
        globally_monotonic_antirollback="NOT_VERIFIED",
        native_admission_gates_unmet=12,
        r6f="IN_PROGRESS_UNCERTIFIED",
        r6g="LOCKED_BY_SEQUENCE",production="NOT_AUTHORIZED")
    fixed.update({k:v for k,v in items.items() if k not in fixed})
    return fixed

def audit(root,head):
    root=Path(root).resolve(strict=True)
    original=e6.audit(root,head)
    p=e3.strictly_load(e3.real_file_bytes(root/CONTRACT))
    require(p["schema"]==SCHEMA and
         p["project"]=="eddarsglobal/nordoi" and
         p["parent_e6_exact_head"]==PARENT and
         p["prior_e6"]=={
             "specialized_run_id":44030442674,
             "specialized_jobs_completed_success":4,
             "rust_run_id":44030442683,
             "rust_jobs_completed_success":5,
             "actual_e6_findings":[
                "GENUINE_LOCAL_ED25519_SSHSIG_VALID_WITHOUT_INDEPENDENT_ADMIN",
                "CROSS_PROCESS_REPLAY_ACCEPTED_WITHOUT_SHARED_SPENT_STATE",
                "SAME_ADMIN_CAN_GENUINELY_RESIGN_ANOTHER_MANIFEST",
                "R5Q_REAL_THREE_FILE_COHERENT_ROLLBACK_REMAINS"]},
         "E7_SOURCE_OR_PREDECESSOR_E6_NOT_PINNED")
    methods=p["replay_research_design"]
    require([x["id"] for x in methods]==[
        "naive","atomic","restart","restored","historic_r5q"] and
        p["sqlite_table"]==TABLE and
        p["protocol_namespace"]==NAMESPACE and
        p["local_nonce_primary_key"]==
               ["namespace","challenge_nonce_sha256"] and
        p["sqlite_transaction"]==
               "BEGIN IMMEDIATE_AND_PRIMARY_KEY_CONFLICT_REFUSE",
        "E7_LOCAL_TRANSACTION_OR_COUNTEREXAMPLE_MISDESCRIBED")
    roles=p["automated_bureau_functions"]
    require([x["id"] for x in roles]==[
         "ATLAS","FERRUM","BOUNDARY","CIPHER","LEDGER",
         "CAPSTONE","ADVERSARY","VERITY","GUARDIAN"] and
         p["bureau_roles_actual_independent_agents_executed"]==0 and
         p["independent_specialist_reviews"]==0,
         "E7_FABRICATED_NINE_INDEPENDENT_BUREAU_REVIEWS")
    require(p["physical_test_humans"]==0 and
         p["human_testers_allowed"] is False and
         p["no_credential_export"] is True and p["no_network"] is True and
         p["actual_independent_witness_receipts"]==0 and
         p["actual_independent_witness_keys"]==0 and
         p["independent_operator"]=="NOT_ENROLLED" and
         p["owner_oob_root"]=="NOT_AUTHENTICATED" and
         p["external_global_antirollback"]=="NOT_VERIFIED" and
         p["formal_r6f_closure"]=="HOLD_MISSING_INDEPENDENT_EVIDENCE" and
         p["native_admission_gates_unmet"]==12 and
         p["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
         p["r6g"]=="LOCKED_BY_SEQUENCE" and
         p["production"]=="NOT_AUTHORIZED" and
         p["permission_to_merge"] is False and
         p["policy"]=="RESEARCH_ONLY_TRUE_MACHINE_RESULTS_NEVER_EXTERNAL_CERTIFICATION",
         "E7_FALSE_INDEPENDENT_AUTHORITY_OR_PRIVILEGE")
    require(original["result"]=="BLOCK" and
            original["native_admission_gates_unmet"]==12,
            "E7_PREVIOUS_DENY_GUARD_MISSING")
    return proof("E7_LOCAL_ATOMIC_NONCE_RACE_PROTOCOL_RESEARCH_ONLY",
                 source_head=head,
                 source_pin=e3.check_head(root,head),
                 local_race_executed=False,
                 external_protection_demonstrated=False)

def lab(root):
    r=Path(root).resolve(strict=True)
    require(r.name.startswith(LAB_PREFIX) and r.is_dir() and
            not r.is_symlink() and
            (os.name=="nt" or r.stat().st_mode & 0o077==0),
            "E7_ONLY_DISPOSABLE_PRIVATE_REAL_LAB_ALLOWED")
    return r

def ledger_path(root):
    return lab(root)/"e7_local_spent.sqlite3"

def connect(db):
    c=sqlite3.connect(str(db),timeout=15,isolation_level=None)
    c.execute("PRAGMA busy_timeout=15000")
    c.execute("PRAGMA journal_mode=DELETE")
    c.execute("PRAGMA synchronous=FULL")
    return c

def initialize(root):
    db=ledger_path(root)
    require(not db.exists(),"E7_DISPOSABLE_DB_ALREADY_EXISTS")
    with closing(connect(db)) as c:
        c.execute("BEGIN IMMEDIATE")
        c.execute("CREATE TABLE research_local_spent("
                  "namespace TEXT NOT NULL, "
                  "challenge_nonce_sha256 TEXT NOT NULL, "
                  "receipt_sha256 TEXT NOT NULL, "
                  "source_head TEXT NOT NULL, "
                  "PRIMARY KEY(namespace,challenge_nonce_sha256)) "
                  "WITHOUT ROWID")
        c.commit()
    if os.name=="posix":os.chmod(db,0o600)
    return db

def count_nonce(db,nonce):
    require(type(nonce) is str and e6.SHA.fullmatch(nonce),
            "E7_INVALID_NONCE_SHA")
    with closing(connect(db)) as c:
        row=c.execute(
          "SELECT COUNT(*) FROM research_local_spent "
          "WHERE namespace=? AND challenge_nonce_sha256=?",
          (NAMESPACE,nonce)).fetchone()
        return int(row[0])

def consume_atomic(db,nonce,receipt_sha,head):
    require(all(type(v) is str and e6.SHA.fullmatch(v)
                for v in (nonce,receipt_sha)) and
            type(head) is str and e6.SHA40.fullmatch(head),
            "E7_ATOMIC_RECEIPT_CONTEXT_INVALID")
    with closing(connect(db)) as c:
        try:
            c.execute("BEGIN IMMEDIATE")
            count=c.execute(
                "SELECT COUNT(*) FROM research_local_spent "
                "WHERE namespace=? AND challenge_nonce_sha256=?",
                (NAMESPACE,nonce)).fetchone()[0]
            if count:
                c.rollback()
                return "LOCAL_DUPLICATE_BLOCKED_NO_NATIVE_ADMISSION"
            c.execute(
                "INSERT INTO research_local_spent"
                "(namespace,challenge_nonce_sha256,receipt_sha256,source_head) "
                "VALUES(?,?,?,?)",(NAMESPACE,nonce,receipt_sha,head))
            c.commit()
            return "LOCAL_SPENT_RECORD_COMMITTED_NO_NATIVE_ADMISSION"
        except BaseException:
            c.rollback()
            raise

def make_barrier(folder,mode,worker_id):
    require(mode in ("naive","atomic") and worker_id in ("0","1"),
            "E7_BARRIER_ARGUMENTS_INVALID")
    path=Path(folder)/("E7_"+mode+"_"+worker_id+".ready")
    with path.open("x",encoding="ascii") as f:
        f.write(str(os.getpid())+"\n")
        f.flush()
        os.fsync(f.fileno())
    ready=[Path(folder)/("E7_"+mode+"_"+i+".ready") for i in ("0","1")]
    cutoff=time.monotonic()+20.0
    while not all(x.is_file() for x in ready):
        if time.monotonic()>cutoff:raise Blocked("E7_REAL_TWO_PROCESS_BARRIER_TIMEOUT")
        time.sleep(0.015)

def local_signed_context(root,head):
    root=lab(root)
    raw=e3.real_file_bytes(root/"e6_receipt.json",limit=4096)
    data=e6.parse(raw,head)
    sig=root/"e6_receipt.json.sig"
    public=root/"e6_local_candidate.pub"
    allowed=root/"e6_local_candidate.allowed_signers"
    result=e6.verify_local(raw,sig,allowed,public,head,
               data["manifest_sha256"],data["previous_receipt_sha256"],
               data["challenge_nonce_sha256"],1200)
    require(result["reason"]=="E6_REAL_LOCAL_SSHSIG_VALID_NO_INDEPENDENT_WITNESS"
            and result["cryptographic_signature_valid"] is True,
            "E7_TRUE_LOCAL_SIGNED_RECEIPT_NOT_VERIFIED")
    return data["challenge_nonce_sha256"],hashlib.sha256(raw).hexdigest()

def worker(root,head,mode,worker_id,synchronized):
    root=lab(root)
    require(mode in ("naive","atomic"),"E7_UNRECOGNIZED_RESEARCH_MODE")
    nonce,receipt_sha=local_signed_context(root,head)
    db=ledger_path(root)
    require(db.is_file() and not db.is_symlink(),
            "E7_REAL_SQLITE_DB_MISSING_OR_SYMLINK")
    if mode=="naive":
        before=count_nonce(db,nonce)  # intentionally UNSAFE check-before-act
        if synchronized:make_barrier(root,mode,worker_id)
        # No native authorization nor DB write; demonstrate two parallel
        # stale local checks that WOULD both authorize a naive consumer.
        return proof("E7_NAIVE_SNAPSHOT_CHECK_BEFORE_ACT_UNSAFE",
             process_pid=os.getpid(),
             local_ssh_signature_verified=True,local_before_count=before,
             stale_eligibility_observed=(before==0),
             native_authority_granted=False)
    if synchronized:make_barrier(root,mode,worker_id)
    outcome=consume_atomic(db,nonce,receipt_sha,head)
    return proof("E7_ATOMIC_LOCAL_SQLITE_ONE_NONCE_CONSUMER",
             process_pid=os.getpid(),local_ssh_signature_verified=True,
             local_spend_outcome=outcome,native_authority_granted=False)

def launch(root,head,mode,count=2):
    cmd_base=[sys.executable,"-B",str(Path(__file__).resolve()),
              "--worker",mode,"--lab-dir",str(root),
              "--expected-head",head]
    procs=[]
    try:
        for i in range(count):
            cmd=cmd_base+["--worker-id",str(i)]
            if count==2:cmd+=["--synchronized"]
            procs.append(subprocess.Popen(cmd,cwd=ROOT,stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,text=True))
        results=[]
        for p in procs:
            out,err=p.communicate(timeout=75)
            require(p.returncode==2,
                    "E7_TRUE_CHILD_FAILED:"+str(p.returncode)+":"+err[:150])
            d=json.loads(out)
            require(d["result"]=="BLOCK" and
                    d["local_ssh_signature_verified"] is True and
                    d["native_admission_gates_unmet"]==12 and
                    d["r6g"]=="LOCKED_BY_SEQUENCE",
                    "E7_CHILD_UNEXPECTED_AUTHORITY_OR_UNVERIFIED_SIGNATURE")
            results.append(d)
        require(len(set(d["process_pid"] for d in results))==len(results),
                "E7_REAL_DISTINCT_OS_PROCESSES_NOT_OBSERVED")
        return results
    finally:
        for p in procs:
            if p.poll() is None:
                p.kill()
                p.communicate()

def exercise(root,head):
    audit(root,head)
    require(e6.r4.is_posix_ssh(),
            "E7_NATIVE_POSIX_OPENSSH_REAL_PROCESS_REQUIRED")
    with tempfile.TemporaryDirectory(prefix=LAB_PREFIX) as tmp:
        home=Path(tmp)
        os.chmod(home,0o700)
        fixture=e6.local_crypto_fixture(home,head)
        db=initialize(home)
        nonce=fixture["nonce"]
        # Two real OS processes validate the true same-admin SSHSIG,
        # check the same empty database BEFORE synchronizing, then both
        # report stale local eligibility without changing permissions.
        naive=launch(home,head,"naive")
        require([x["local_before_count"] for x in naive]==[0,0] and
                all(x["stale_eligibility_observed"] for x in naive),
                "E7_NAIVE_REAL_CONCURRENT_STALE_READ_NOT_REPRODUCED")
        # Hold an authentic but SAME-ADMIN local history snapshot before
        # spending. It is deliberately not protected by an external witness.
        old_copy=home/"e7_unprotected_old_db.sqlite3"
        shutil.copy2(db,old_copy)
        # Atomic worker transactions in genuinely distinct OS processes:
        # at most one will insert the unique namespace+nonce key.
        atomic=launch(home,head,"atomic")
        outcomes=[x["local_spend_outcome"] for x in atomic]
        require(sorted(outcomes)==sorted([
          "LOCAL_SPENT_RECORD_COMMITTED_NO_NATIVE_ADMISSION",
          "LOCAL_DUPLICATE_BLOCKED_NO_NATIVE_ADMISSION"]),
          "E7_ATOMIC_SQLITE_FAILED_EXACT_ONE_WINNER")
        require(count_nonce(db,nonce)==1,
                "E7_SQLITE_DID_NOT_DURABLY_RECORD_ONE_LOCAL_NONCE")
        # A third real, restarted process sees the durable spent record.
        replay=launch(home,head,"atomic",count=1)[0]
        require(replay["local_spend_outcome"]==
              "LOCAL_DUPLICATE_BLOCKED_NO_NATIVE_ADMISSION",
              "E7_PERSISTED_LOCAL_NONCE_REPLAY_NOT_DENIED")
        # All child processes exited, SQLite DELETE journal mode. The same
        # admin can restore its own old DB snapshot, wiping the nonce floor.
        restored=home/"e7_restore_staging.sqlite3"
        shutil.copy2(old_copy,restored)
        os.replace(restored,db)
        require(count_nonce(db,nonce)==0,
                "E7_SAME_ADMIN_LOCAL_SQLITE_RESTORE_NOT_REPRODUCED")
        after_restore=launch(home,head,"atomic",count=1)[0]
        require(after_restore["local_spend_outcome"]==
              "LOCAL_SPENT_RECORD_COMMITTED_NO_NATIVE_ADMISSION" and
              count_nonce(db,nonce)==1,
              "E7_LOCAL_REPLAY_AFTER_SAME_ADMIN_ROLLBACK_NOT_OBSERVED")
        r5q=e6.r6.inspect_real_rollback()
        require(r5q["same_admin_all_three_restored"] and
                r5q["globally_spent_nonce_reissued_after_full_rollback"] and
                r5q["protected_independent_high_water"] is False,
                "E7_R5Q_GENUINE_FULL_HISTORY_ROLLBACK_NOT_CONFIRMED")
    require(e3.check_head(root,head)==head,"E7_GIT_HEAD_CHANGED")
    return proof("E7_REAL_TWO_PROCESS_LOCAL_NONCE_RACE_AND_ATOMIC_FIX_WITH_ROLLBACK_LIMIT",
        exact_source_head=head,local_ssh_signatures_verified_in_each_child=True,
        two_real_concurrent_naive_unspent_reads=2,
        naive_granted_native_authority=False,
        atomic_distinct_processes=2,
        atomic_local_committed=1,atomic_local_duplicates_blocked=1,
        persisted_local_nonce_replay_blocked_after_restart=True,
        same_admin_old_sqlite_snapshot_restored=True,
        local_replay_after_rollback_committed_again=True,
        original_r5q_three_file_rollback_reproduced=True,
        externally_protected_monotonic_floor=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    modes=p.add_mutually_exclusive_group(required=True)
    modes.add_argument("--audit",action="store_true")
    modes.add_argument("--exercise",action="store_true")
    modes.add_argument("--worker",choices=("naive","atomic"))
    p.add_argument("--expected-head",required=True)
    p.add_argument("--root",type=Path,default=ROOT)
    p.add_argument("--lab-dir",type=Path)
    p.add_argument("--worker-id",default="0")
    p.add_argument("--synchronized",action="store_true")
    args=p.parse_args(argv)
    try:
        if args.audit:out=audit(args.root,args.expected_head)
        elif args.exercise:out=exercise(args.root,args.expected_head)
        else:
            require(args.lab_dir is not None,"E7_WORKER_LAB_REQUIRED")
            out=worker(args.lab_dir,args.expected_head,
                       args.worker,args.worker_id,args.synchronized)
        code=2
    except (Blocked,e6.Blocked,e3.Rejected,ValueError,KeyError,
            TypeError,sqlite3.Error,OSError,subprocess.SubprocessError) as ex:
        out=proof("E7_SOURCE_OR_REAL_CONCURRENCY_EXERCISE_NOT_QUALIFIED",
             error_type=type(ex).__name__,
             detail=str(ex) if isinstance(ex,Blocked) else None)
        code=3
    print(json.dumps(out,sort_keys=True))
    return code

if __name__=="__main__":raise SystemExit(main())
