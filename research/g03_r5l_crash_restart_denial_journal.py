#!/usr/bin/env python3
"""NORDOI G0.3 R5L: real crash/restart persistence of deny-only research journal.

Uses SQLite transactions and actual killed/restarted OS processes. No native
authority, no independent guardian, no owner OOB enrollment, no production.
Restoring *all* same-admin snapshots remains globally undetectable.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time

import g03_r5h_signed_revocation_lab as rev
import g03_r5i_revocation_floor as floor
import g03_r5j_signed_recovery_quarantine as rec
import g03_r5k_concurrent_recovery_quarantine as concurrent

PROTOCOL="NORDOI-G03-R5L-CRASH-RESTART-DENIAL-JOURNAL-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="7a22152f5bc4c74f014601bfd56338195d78a647"
NAME="r5l_local_denial_journal.sqlite3"
GENESIS=hashlib.sha256((PROTOCOL+":GENESIS").encode("ascii")).hexdigest()
ROLES=("candidate_a","candidate_b")
SQLITE_HEADER=b"SQLite format 3\x00"

class Refused(ValueError):pass

def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")

def digest(prev,height,role,nonce,claim_sha):
    return hashlib.sha256(prev.encode("ascii")+b"\n"+canonical({
        "protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
        "height":height,"role":role,"nonce":nonce,
        "claim_sha256":claim_sha,"decision":"BLOCK","admission":"NOT_AUTHORIZED"
    })).hexdigest()

def check_claim(nonce,claim_sha,role):
    if not rev.is_sha(nonce) or not rev.is_sha(claim_sha):
        raise Refused("INVALID_DENY_EVENT_NONCE_OR_SHA")
    if type(role) is not str or role not in ROLES:
        raise Refused("INVALID_DENY_EVENT_ROLE")

def private_journal_dir(directory):
    p=Path(directory)
    if not p.is_absolute() or p.is_symlink() or not p.is_dir():
        raise Refused("JOURNAL_DIRECTORY_NOT_ABSOLUTE_PRIVATE")
    if os.name!="nt" and p.stat().st_mode & 0o077:
        raise Refused("JOURNAL_DIRECTORY_PERMISSIONS")
    path=p/NAME
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise Refused("JOURNAL_FILE_SYMLINK_OR_NONREGULAR")
    if path.exists() and os.name!="nt" and path.stat().st_mode & 0o077:
        raise Refused("JOURNAL_FILE_PERMISSIONS")
    return path

def block(reason,**extra):
    return {"status":"BLOCK","admission":"NOT_AUTHORIZED","reason":reason,
            "real_independent_guardian":"NOT_ENROLLED",
            "owner_oob_root":"NOT_ENROLLED",
            "global_monotonic_antirollback":"NOT_IMPLEMENTED",**extra}

class DenialJournal:
    def __init__(self,directory):
        self.path=private_journal_dir(directory)
        newly_created=not self.path.exists()
        self.db=sqlite3.connect(str(self.path),timeout=8.0,isolation_level=None)
        try:
            self.db.execute("PRAGMA busy_timeout=8000")
            self.db.execute("PRAGMA synchronous=FULL")
            self.db.execute("PRAGMA journal_mode=DELETE")
            self.db.execute("BEGIN IMMEDIATE")
            self.db.execute("CREATE TABLE IF NOT EXISTS state (id INTEGER PRIMARY KEY CHECK(id=1),height INTEGER NOT NULL,head TEXT NOT NULL)")
            self.db.execute("CREATE TABLE IF NOT EXISTS denials (seq INTEGER PRIMARY KEY,role TEXT NOT NULL,nonce TEXT NOT NULL UNIQUE,claim_sha TEXT NOT NULL,prev TEXT NOT NULL,head TEXT NOT NULL,decision TEXT NOT NULL,admission TEXT NOT NULL)")
            if newly_created:
                self.db.execute("INSERT INTO state VALUES(1,0,?)",(GENESIS,))
            self._verify()
            self.db.execute("COMMIT")
            if os.name!="nt":os.chmod(self.path,0o600)
        except BaseException:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            self.db.close()
            raise
    def __enter__(self):return self
    def __exit__(self,*_):self.db.close()
    def _verify(self):
        if self.db.execute("PRAGMA quick_check").fetchone()!=("ok",):
            raise Refused("LOCAL_DENIAL_JOURNAL_CORRUPT")
        state=self.db.execute("SELECT height,head FROM state WHERE id=1").fetchone()
        if state is None or self.db.execute("SELECT count(*) FROM state").fetchone()[0]!=1:
            raise Refused("LOCAL_DENIAL_STATE_MISSING")
        h,head=0,GENESIS
        events=self.db.execute("SELECT seq,role,nonce,claim_sha,prev,head,decision,admission FROM denials ORDER BY seq").fetchall()
        for seq,role,nonce,claim_sha,prev,updated,decision,admission in events:
            check_claim(nonce,claim_sha,role)
            if (seq!=h+1 or prev!=head or
                updated!=digest(head,seq,role,nonce,claim_sha) or
                decision!="BLOCK" or admission!="NOT_AUTHORIZED"):
                raise Refused("LOCAL_DENIAL_CHAIN_INVALID")
            h,head=seq,updated
        if state!=(h,head):raise Refused("LOCAL_DENIAL_METADATA_MISMATCH")
        return state
    def audit(self):
        self.db.execute("BEGIN IMMEDIATE")
        try:
            h,head=self._verify()
            self.db.execute("COMMIT")
            return block("LOCAL_DENIAL_HISTORY_NOT_EXTERNAL",local_height=h,local_head=head)
        except BaseException:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            raise
    def record(self,nonce,claim_sha,role,commit=True):
        check_claim(nonce,claim_sha,role)
        self.db.execute("BEGIN IMMEDIATE")
        try:
            height,head=self._verify()
            if self.db.execute("SELECT 1 FROM denials WHERE nonce=?",(nonce,)).fetchone():
                raise Refused("ALREADY_RECORDED_DENIAL_NONCE")
            seq=height+1
            updated=digest(head,seq,role,nonce,claim_sha)
            self.db.execute("INSERT INTO denials VALUES(?,?,?,?,?,?,?,?)",
                            (seq,role,nonce,claim_sha,head,updated,"BLOCK","NOT_AUTHORIZED"))
            self.db.execute("UPDATE state SET height=?,head=? WHERE id=1",(seq,updated))
            self._verify()
            if commit:
                self.db.execute("COMMIT")
                return block("LOCAL_DENIAL_DURABLY_COMMITTED_NOT_ADMISSION",
                             local_height=seq)
            return block("LOCAL_UNCOMMITTED_DENIAL_WILL_BE_KILLED",
                         pending_height=seq)
        except BaseException:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            raise

def execute(directory,action,nonce=None,claim_sha=None,role=None):
    try:
        with DenialJournal(directory) as journal:
            if action=="audit":return journal.audit()
            if action=="record":return journal.record(nonce,claim_sha,role)
            raise Refused("UNKNOWN_DENIAL_ACTION")
    except (Refused,rev.Denied,sqlite3.Error,ValueError,TypeError,OSError) as exc:
        return block("LOCAL_DENIAL_JOURNAL_REFUSED",
                     detail=str(exc) if isinstance(exc,(Refused,rev.Denied)) else type(exc).__name__)

def internal_command(directory,mode,nonce,claim_sha,role):
    """Only within private disposable R5L lab paths, deny only."""
    path=Path(directory)
    if (not path.is_absolute() or
        not any(part.startswith("nordoi-r5l-") for part in path.parts)):
        return block("NON_LAB_PATH_REFUSED")
    if mode=="audit":return execute(path,"audit")
    if mode=="record":return execute(path,"record",nonce,claim_sha,role)
    if mode=="hold":
        try:
            with DenialJournal(path) as journal:
                report=journal.record(nonce,claim_sha,role,commit=False)
                print("R5L_CHILD_TXN_UNCOMMITTED_READY",flush=True)
                time.sleep(90)
                return report
        except (Refused,sqlite3.Error,OSError,ValueError) as exc:
            return block("UNCOMMITTED_HOLDER_REFUSED",detail=type(exc).__name__)
    return block("UNKNOWN_INTERNAL_COMMAND")

def _process(mode,directory,nonce=None,claim_sha=None,role=None):
    cmd=[sys.executable,"-B",str(Path(__file__).resolve()),
         "--internal-mode",mode,"--directory",str(directory)]
    if nonce is not None:cmd+=["--nonce",nonce]
    if claim_sha is not None:cmd+=["--claim-sha",claim_sha]
    if role is not None:cmd+=["--role",role]
    return cmd

def _run(mode,directory,nonce=None,claim_sha=None,role=None):
    r=subprocess.run(_process(mode,directory,nonce,claim_sha,role),
                     text=True,capture_output=True,timeout=30,check=False)
    if r.returncode!=2:raise Refused("REAL_CHILD_EXIT_NOT_DENY_TWO")
    try:return json.loads(r.stdout)
    except json.JSONDecodeError as exc:
        raise Refused("CHILD_OUTPUT_NOT_JSON") from exc

def actual_kill(directory,nonce,claim_sha,role):
    if os.name=="nt":raise Refused("UNIX_KILL_TEST_ONLY")
    child=subprocess.Popen(_process("hold",directory,nonce,claim_sha,role),
                           stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE,text=True,bufsize=1)
    try:
        ready,_,_=select.select([child.stdout],[],[],15)
        if not ready:raise Refused("REAL_UNCOMMITTED_CHILD_NOT_READY")
        marker=child.stdout.readline().strip()
        if marker!="R5L_CHILD_TXN_UNCOMMITTED_READY":
            raise Refused("REAL_UNCOMMITTED_CHILD_BAD_MARKER")
        if child.poll() is not None:raise Refused("UNCOMMITTED_CHILD_EXITED_EARLY")
        child.kill()
        child.wait(timeout=20)
        if child.returncode in (0,2):
            raise Refused("UNCOMMITTED_CHILD_NOT_FORCIBLY_TERMINATED")
        return True
    finally:
        if child.poll() is None:
            child.kill()
            child.wait(timeout=20)
        child.stdout.close()
        child.stderr.close()

def native_exercise():
    if os.name=="nt" or not Path(rev.SSH).is_file():
        return block("NATIVE_UNIX_OPENSSH_NOT_AVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-native-") as d:
        root=Path(d);os.chmod(root,0o700)
        primary=root/"primary";secondary=root/"floor";local=root/"journal"
        key_a=root/"key_a";key_b=root/"key_b"
        for path in (primary,secondary,local,key_a,key_b):
            path.mkdir(mode=0o700);os.chmod(path,0o700)
        priv_a,pub_a=rev.keygen(key_a)
        priv_b,pub_b=rev.keygen(key_b)
        with rev.RevocationLedger(primary) as journal:
            if journal.audit()["local_height"]!=0:raise Refused("REVOCATION_GENESIS_ERROR")
        if floor.execute(primary,secondary,"audit").get("floor_height")!=0:
            raise Refused("FLOOR_GENESIS_ERROR")
        with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5h-packet-") as temp:
            signed_revoke=rev.make_packet(priv_a,pub_a,Path(temp))
        with rev.RevocationLedger(primary) as journal:
            if journal.revoke(signed_revoke,pub_a,1800000060).get("status")!="LAB_LOCAL_REVOKE_COMMIT":
                raise Refused("REAL_SIGNED_REVOCATION_NOT_COMMITTED")
        if floor.execute(primary,secondary,"checkpoint").get("status")!="LAB_LOCAL_CHECKPOINT_COMMIT":
            raise Refused("REAL_FLOOR_CHECKPOINT_NOT_COMMITTED")
        state=rec.observed(primary,secondary)
        intent_a=rec.intent(priv_a,pub_a,state,nonce="a"*64)
        intent_b=rec.intent(priv_b,pub_b,state,nonce="b"*64)
        batch=concurrent.envelope(intent_a,intent_b)
        decision=concurrent.decide(batch,{"candidate_a":pub_a,"candidate_b":pub_b},
                                   primary,secondary,1800000060)
        if decision.get("reason")!="CONCURRENT_VALID_SIGNED_INTENTS_NO_EXTERNAL_AUTHORITY":
            raise Refused("GENUINE_SIGNED_RECOVERY_NOT_BLOCKED")
        claim_sha=hashlib.sha256(batch).hexdigest()
        begin=_run("audit",local)
        if begin.get("local_height")!=0:raise Refused("DENIAL_GENESIS_ERROR")
        durable=_run("record",local,"1"*64,claim_sha,"candidate_a")
        if durable.get("local_height")!=1:raise Refused("DURABLE_DENIAL_NOT_COMMITTED")
        if not actual_kill(local,"2"*64,claim_sha,"candidate_b"):
            raise Refused("CRASH_NOT_OBSERVED")
        recovered=_run("audit",local)
        if recovered.get("local_height")!=1:
            raise Refused("PARTIAL_UNCOMMITTED_DENIAL_VISIBLE_AFTER_KILL")
        resumed=_run("record",local,"2"*64,claim_sha,"candidate_b")
        if resumed.get("local_height")!=2:
            raise Refused("REAL_RESTART_AFTER_KILL_FAILED")
        duplicate=_run("record",local,"2"*64,claim_sha,"candidate_b")
        if duplicate.get("reason")!="LOCAL_DENIAL_JOURNAL_REFUSED" or "ALREADY_RECORDED_DENIAL_NONCE" not in duplicate.get("detail",""):
            raise Refused("RESTARTED_SPENT_NONCE_NOT_DENIED")
        workers=[subprocess.Popen(_process("record",local,"3"*64,claim_sha,role),
                                  stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
                 for role in ("candidate_a","candidate_b")]
        outcomes=[]
        for worker in workers:
            stdout,stderr=worker.communicate(timeout=30)
            if worker.returncode!=2:
                raise Refused("CONCURRENT_DENIAL_CHILD_NOT_BLOCKED")
            try:outcomes.append(json.loads(stdout))
            except json.JSONDecodeError as exc:raise Refused("CONCURRENT_CHILD_NOT_JSON") from exc
        if sorted(x["reason"] for x in outcomes)!=[
            "LOCAL_DENIAL_DURABLY_COMMITTED_NOT_ADMISSION","LOCAL_DENIAL_JOURNAL_REFUSED"]:
            raise Refused("CONCURRENT_SAME_NONCE_NOT_EXACTLY_ONCE_LOCAL")
        end=_run("audit",local)
        if end.get("local_height")!=3:raise Refused("CONCURRENT_DENIAL_FINAL_HEIGHT_BAD")
        return block("REAL_PROCESS_KILL_AND_RESTART_RECOVERY_DENIED",
                     real_temp_signers=2,real_revocation_floor=1,
                     durable_precrash_denial="PRESERVED",
                     killed_uncommitted_denial="ROLLED_BACK",
                     restart_denial_committed="PASS_LOCAL_ONLY",
                     restarted_duplicate_nonce="BLOCK",
                     concurrent_same_nonce="ONE_LOCAL_DENY_COMMIT_ONE_REPLAY_BLOCK",
                     final_local_denial_height=3,
                     same_admin_entire_history_rollback="NOT_GLOBALLY_DETECTABLE",
                     native_permission_granted=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    group=p.add_mutually_exclusive_group(required=True)
    group.add_argument("--exercise",action="store_true")
    group.add_argument("--internal-mode",choices=("audit","record","hold"))
    p.add_argument("--directory")
    p.add_argument("--nonce")
    p.add_argument("--claim-sha")
    p.add_argument("--role")
    args=p.parse_args(argv)
    try:
        if args.exercise:result=native_exercise()
        elif args.directory:
            result=internal_command(args.directory,args.internal_mode,args.nonce,args.claim_sha,args.role)
        else:result=block("INTERNAL_DISPOSABLE_PATH_MISSING")
    except (Refused,rev.Denied,floor.Refused,sqlite3.Error,TypeError,ValueError,OSError,
            subprocess.SubprocessError) as exc:
        result=block("R5L_REAL_CRASH_RECOVERY_LAB_REFUSED",
                     detail=str(exc) if isinstance(exc,(Refused,rev.Denied,floor.Refused)) else type(exc).__name__)
    print(json.dumps(result,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
