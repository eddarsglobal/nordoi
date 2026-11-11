#!/usr/bin/env python3
"""G0.3 R5O: open FD/path identity vs real atomic replace; deny-only.

All evidence belongs to the same local admin. No external protected root,
atomic cross-file authorization, permission grant or native admission.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import stat
import subprocess
import sys
import tempfile

import g03_r5l_crash_restart_denial_journal as denial
import g03_r5m_file_integrity_quarantine as witness
import g03_r5n_checkpoint_use_quarantine as use

PROTOCOL="NORDOI-G03-R5O-FD-PATH-IDENTITY-QUARANTINE-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="b997d84e3a17dec010a3ec801e67ac0ce7f3b47f"
FIELDS=frozenset(("protocol","project","parent","operation","nonce","height","head",
                  "db_sha256","witness_sha256","db_dev","db_ino","witness_dev","witness_ino"))
LIMIT=2200
FILE_LIMIT=8*1024*1024

class Refused(ValueError):pass

def canonical(x):
    return (json.dumps(x,sort_keys=True,separators=(",",":"),ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    out={}
    for k,v in pairs:
        if k in out:raise Refused("DUPLICATE_IDENTITY_FIELD")
        out[k]=v
    return out

def constant(_):raise Refused("NONFINITE_JSON")

def parse(raw):
    if type(raw) is not bytes or not 0<len(raw)<=LIMIT:
        raise Refused("MISSING_OR_OVERSIZED_IDENTITY_TICKET")
    try:
        value=json.loads(raw.decode("utf-8"),object_pairs_hook=unique,parse_constant=constant)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Refused("BAD_IDENTITY_JSON") from exc
    if type(value) is not dict or value.keys()!=FIELDS or canonical(value)!=raw:
        raise Refused("WRONG_IDENTITY_FIELDS_OR_CANONICAL")
    if (value["protocol"]!=PROTOCOL or value["project"]!=PROJECT or value["parent"]!=PARENT):
        raise Refused("WRONG_IDENTITY_SOURCE")
    if value["operation"]!="DENY_ONLY_FD_IDENTITY_USE":
        raise Refused("GRANT_OR_RECOVERY_FORBIDDEN")
    for name in ("nonce","head","db_sha256","witness_sha256"):
        if not witness.sha(value[name]):raise Refused("INVALID_IDENTITY_DIGEST_"+name)
    for name in ("height","db_dev","db_ino","witness_dev","witness_ino"):
        if type(value[name]) is not int or value[name]<(1 if name=="height" else 0):
            raise Refused("INVALID_INTEGER_FILE_IDENTITY_"+name)
    return value

class OpenSnapshot:
    """Opened descriptor stays live during path rechecks, never an authority lock."""
    def __init__(self,path):
        self.path=Path(path)
        self.fd=None;self.before=None;self.digest=None
        self.open()
    @staticmethod
    def identity(s):
        return (s.st_dev,s.st_ino,s.st_mode,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
    def open(self):
        if self.path.is_symlink() or not self.path.is_file():
            raise Refused("IDENTITY_PATH_MISSING_OR_SYMLINK")
        flags=os.O_RDONLY|getattr(os,"O_CLOEXEC",0)|getattr(os,"O_NOFOLLOW",0)
        try:
            self.fd=os.open(str(self.path),flags)
            st=os.fstat(self.fd)
            if not stat.S_ISREG(st.st_mode) or st.st_size>FILE_LIMIT:
                raise Refused("FD_NOT_REGULAR_OR_OVERSIZED")
            if self.identity(os.lstat(self.path))!=self.identity(st):
                raise Refused("FD_PATH_DIFFER_AT_OPEN")
            chunks=[];remain=FILE_LIMIT+1
            while remain:
                data=os.read(self.fd,min(65536,remain))
                if not data:break
                chunks.append(data);remain-=len(data)
            if remain==0:raise Refused("FD_READ_EXCEEDS_BUDGET")
            self.digest=hashlib.sha256(b"".join(chunks)).hexdigest()
            self.before=self.identity(st)
            self.check()
        except BaseException:
            self.close()
            raise
    def check(self):
        if self.fd is None:raise Refused("FD_CLOSED")
        if self.path.is_symlink() or not self.path.is_file():
            raise Refused("PATH_DISAPPEARED_DURING_OBSERVATION")
        # Atomic replacement may change old inode ctime/link metadata even
        # while its already-open FD still points to the same old inode.
        # Qualify a pathname swap first, then check surviving FD metadata.
        if self.identity(os.lstat(self.path))[:2]!=self.before[:2]:
            raise Refused("OPEN_FD_AND_CURRENT_PATH_DIVERGED")
        if self.identity(os.fstat(self.fd))!=self.before:
            raise Refused("OPEN_FD_MODIFIED_DURING_OBSERVATION")
        if self.identity(os.lstat(self.path))!=self.before:
            raise Refused("PATH_METADATA_CHANGED_DURING_OBSERVATION")
        if os.name!="nt" and self.before[2]&0o077:
            raise Refused("FILE_MODE_NOT_PRIVATE")
    def close(self):
        if self.fd is not None:os.close(self.fd);self.fd=None
    def __enter__(self):return self
    def __exit__(self,*_):self.close()

def observed(db,proof):
    with OpenSnapshot(db) as a,OpenSnapshot(proof) as b:
        current=witness.primary_state(db)
        p,raw=witness.read_witness(proof)
        if (p["height"],p["head"],p["db_sha256"])!=(
            current["height"],current["head"],current["db_sha256"]):
            raise Refused("PRIMARY_WITNESS_CONTENT_INCONSISTENT")
        if a.digest!=current["db_sha256"] or b.digest!=hashlib.sha256(raw).hexdigest():
            raise Refused("OPEN_FD_VS_REOPENED_FILE_BYTES_DIFFER")
        a.check();b.check()
        return {"height":current["height"],"head":current["head"],
                "db_sha256":a.digest,"witness_sha256":b.digest,
                "db_dev":a.before[0],"db_ino":a.before[1],
                "witness_dev":b.before[0],"witness_ino":b.before[1]}

def prepare(primary_dir,witness_dir,nonce):
    if not witness.sha(nonce):raise Refused("INVALID_FD_USE_NONCE")
    db,proof=witness.private_paths(primary_dir,witness_dir)
    state=observed(db,proof)
    return canonical({"protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
                      "operation":"DENY_ONLY_FD_IDENTITY_USE","nonce":nonce,**state})

def block(reason,detail=None,**items):
    value=denial.block(reason,native_authority_granted=False,**items)
    if detail is not None:value["detail"]=detail
    return value

def evaluate(raw,primary_dir,witness_dir,journal_dir,after_open=None,before_deny=None):
    try:
        claim=parse(raw)
        db,proof=witness.private_paths(primary_dir,witness_dir)
        journal=denial.private_journal_dir(journal_dir)
        if not journal.is_file():raise Refused("DENIAL_JOURNAL_MISSING_NO_CREATE")
        with OpenSnapshot(db) as dbfd,OpenSnapshot(proof) as proof_fd:
            def check_matches():
                dbfd.check();proof_fd.check()
                actual=observed(db,proof)
                for key in ("height","head","db_sha256","witness_sha256",
                            "db_dev","db_ino","witness_dev","witness_ino"):
                    if claim[key]!=actual[key]:raise Refused("EVIDENCE_USE_CHANGED_"+key.upper())
                dbfd.check();proof_fd.check()
            check_matches()
            if after_open is not None:
                if not callable(after_open):raise Refused("INVALID_INTERLEAVE_HOOK")
                after_open()
            check_matches()
            if before_deny is not None:
                if not callable(before_deny):raise Refused("INVALID_DENY_HOOK")
                before_deny()
            check_matches()
            # Intent cannot grant. Unchanged R5N checks again and records only a BLOCK.
            ticket=use.prepare(primary_dir,witness_dir,claim["nonce"])
            result=use.evaluate(ticket,primary_dir,witness_dir,journal_dir)
            if result.get("reason")!="VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED":
                raise Refused("LOCAL_DENIAL_REPLAY_OR_REFUSAL:"+str(
                    result.get("detail",result.get("reason"))))
            return block("OPEN_FD_IDENTITY_MATCHED_ONLY_DENIAL_COMMITTED",
                         local_denial_height=result["local_denial_height"],
                         fd_path_coincide=True)
    except (Refused,witness.Refused,denial.Refused,use.Refused,
            sqlite3.Error,OSError,TypeError,ValueError,subprocess.SubprocessError) as exc:
        return block("FILE_IDENTITY_CHANGED_OR_UNVERIFIED_QUARANTINE",
                     detail=str(exc) if isinstance(exc,(
                         Refused,witness.Refused,denial.Refused,use.Refused))
                            else type(exc).__name__)

def real_atomic_replace(source,destination):
    """Second OS process, real same-owner os.replace in disposable test files."""
    program=("import os,shutil,sys,tempfile; src,dst=sys.argv[1:]; "
             "fd,tmp=tempfile.mkstemp(prefix='r5o-swap-',dir=os.path.dirname(dst)); "
             "os.close(fd); shutil.copyfile(src,tmp); os.chmod(tmp,0o600); "
             "os.replace(tmp,dst)")
    r=subprocess.run([sys.executable,"-B","-c",program,str(source),str(destination)],
                     capture_output=True,text=True,timeout=25)
    if r.returncode:
        raise Refused("REAL_ATOMIC_CHILD_REPLACEMENT_FAILED:"+r.stderr[-220:])

def native_exercise():
    if os.name=="nt":return block("POSIX_OPEN_FD_REPLACE_EXPERIMENT_ONLY")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5o-real-") as temp:
        root=Path(temp);os.chmod(root,0o700)
        a,b,j=(root/"primary",root/"witness",root/"journal")
        for d in (a,b,j):d.mkdir(mode=0o700);os.chmod(d,0o700)
        with denial.DenialJournal(a) as x:
            if x.record("a"*64,"b"*64,"candidate_a")["local_height"]!=1:
                raise Refused("REAL_SQLITE_INITIAL_DENIAL_FAILED")
        if witness.capture(a,b)["local_height"]!=1:
            raise Refused("REAL_LOCAL_WITNESS_CAPTURE_FAILED")
        with denial.DenialJournal(j) as x:
            if x.audit()["local_height"]!=0:
                raise Refused("REAL_USE_DENIAL_JOURNAL_NOT_INITIALIZED")
        db=a/denial.NAME;proof=b/witness.WITNESS
        old_db,old_proof=root/"same-bytes.sqlite3",root/"same-bytes.json"
        shutil.copy2(db,old_db);shutil.copy2(proof,old_proof)
        ok=evaluate(prepare(a,b,"c"*64),a,b,j)
        if ok.get("reason")!="OPEN_FD_IDENTITY_MATCHED_ONLY_DENIAL_COMMITTED":
            raise Refused("REAL_INITIAL_DENIAL_ONLY_USE_FAILED")
        replay=evaluate(prepare(a,b,"c"*64),a,b,j)
        if "ALREADY_RECORDED_DENIAL_NONCE" not in replay.get("detail",""):
            raise Refused("REAL_DUPLICATE_NONCE_NOT_REFUSED")
        primary=evaluate(prepare(a,b,"d"*64),a,b,j,
                         after_open=lambda:real_atomic_replace(old_db,db))
        if "OPEN_FD_AND_CURRENT_PATH_DIVERGED" not in primary.get("detail",""):
            raise Refused("SAME_BYTE_NEW_INODE_PRIMARY_NOT_BLOCKED")
        second=evaluate(prepare(a,b,"e"*64),a,b,j,
                        before_deny=lambda:real_atomic_replace(old_proof,proof))
        if "OPEN_FD_AND_CURRENT_PATH_DIVERGED" not in second.get("detail",""):
            raise Refused("SAME_BYTE_NEW_INODE_WITNESS_NOT_BLOCKED")
        with OpenSnapshot(db) as opened:
            previous_inode,previous_hash=opened.before[1],opened.digest
            real_atomic_replace(old_db,db)
            if previous_inode==os.lstat(db).st_ino or opened.digest!=previous_hash:
                raise Refused("REAL_OPEN_FD_DID_NOT_SURVIVE_PATH_REPLACE")
            try:opened.check()
            except Refused as exc:
                if "OPEN_FD_AND_CURRENT_PATH_DIVERGED" not in str(exc):raise
            else:raise Refused("OLD_OPEN_FD_NEW_PATH_MISMATCH_NOT_FOUND")
        return block("REAL_ATOMIC_FILE_IDENTITY_SWAPS_DENIED",
                     matched_path_can_record_only_denial="PASS_LOCAL_ONLY",
                     repeated_local_nonce="BLOCK",
                     same_bytes_new_inode_primary="QUARANTINED",
                     same_bytes_new_inode_witness="QUARANTINED",
                     old_open_fd_survived_atomic_path_change=True,
                     all_same_admin_old_artifacts_rollback="NOT_GLOBALLY_DETECTABLE")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    group=p.add_mutually_exclusive_group(required=True)
    group.add_argument("--exercise",action="store_true")
    group.add_argument("--assess",action="store_true")
    p.add_argument("--intent-file")
    p.add_argument("--primary-dir")
    p.add_argument("--witness-dir")
    p.add_argument("--journal-dir")
    args=p.parse_args(argv)
    try:
        if args.exercise:result=native_exercise()
        elif args.intent_file:
            result=evaluate(Path(args.intent_file).read_bytes(),args.primary_dir,
                            args.witness_dir,args.journal_dir)
        else:result=block("INTENT_FILE_MISSING")
    except (Refused,witness.Refused,denial.Refused,use.Refused,
            sqlite3.Error,OSError,TypeError,ValueError,subprocess.SubprocessError) as exc:
        result=block("FD_IDENTITY_EXERCISE_REFUSED",
                     detail=str(exc) if isinstance(exc,(
                         Refused,witness.Refused,denial.Refused,use.Refused))
                            else type(exc).__name__)
    print(json.dumps(result,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
