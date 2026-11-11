#!/usr/bin/env python3
"""NORDOI G0.3 R5M: detect actual SQLite damage against a preserved LOCAL proof.

No outside independently controlled witness. Valid matching old primary+proof
can be restored together by the same owner. Every output denies admission.
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
from urllib.parse import quote

import g03_r5l_crash_restart_denial_journal as denial

PROTOCOL="NORDOI-G03-R5M-LOCAL-SQLITE-INTEGRITY-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="db80d370bdd3ec6acebe01fb1f752b819412062b"
WITNESS="r5m_preserved_local_checkpoint.json"
FIELDS=frozenset(("protocol","project","r5l_parent","journal_name","height",
                  "head","db_sha256"))
MAX_BYTES=2048

class Refused(ValueError):pass

def sha(value):
    return type(value) is str and len(value)==64 and all(c in "0123456789abcdef" for c in value) and value!="0"*64

def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True,
                       allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    result={}
    for name,value in pairs:
        if name in result:raise Refused("DUPLICATE_WITNESS_FIELD")
        result[name]=value
    return result

def disallow_constant(_):raise Refused("NONFINITE_WITNESS")

def private_paths(primary_dir,witness_dir):
    a,b=Path(primary_dir),Path(witness_dir)
    for d in (a,b):
        if not d.is_absolute() or d.is_symlink() or not d.is_dir():
            raise Refused("ABSOLUTE_PRIVATE_DIRECTORY_REQUIRED")
        if os.name!="nt" and d.stat().st_mode & 0o077:
            raise Refused("DIRECTORY_NOT_PRIVATE")
    ap,bp=a.resolve(strict=True),b.resolve(strict=True)
    if ap==bp or ap in bp.parents or bp in ap.parents:
        raise Refused("PRIMARY_WITNESS_ROOTS_OVERLAP")
    db=ap/denial.NAME
    record=bp/WITNESS
    for target in (db,record):
        if target.is_symlink() or (target.exists() and not target.is_file()):
            raise Refused("SYMLINK_OR_NONREGULAR_EVIDENCE")
        if target.exists() and os.name!="nt" and target.stat().st_mode & 0o077:
            raise Refused("EVIDENCE_FILE_NOT_PRIVATE")
    return db,record

def read_witness(path):
    if not path.is_file() or path.is_symlink():raise Refused("PRESERVED_LOCAL_WITNESS_MISSING")
    raw=path.read_bytes()
    if not 0<len(raw)<=MAX_BYTES:raise Refused("WITNESS_MISSING_OR_OVERSIZED")
    try:
        obj=json.loads(raw.decode("utf-8"),object_pairs_hook=unique,
                       parse_constant=disallow_constant)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Refused("WITNESS_NOT_CANONICAL_JSON") from exc
    if type(obj) is not dict or obj.keys()!=FIELDS or canonical(obj)!=raw:
        raise Refused("WITNESS_FIELDS_OR_CANONICAL_FAILURE")
    if (obj["protocol"]!=PROTOCOL or obj["project"]!=PROJECT or
            obj["r5l_parent"]!=PARENT or obj["journal_name"]!=denial.NAME):
        raise Refused("WITNESS_IDENTITY_OR_SOURCE_MISMATCH")
    if type(obj["height"]) is not int or obj["height"]<1:
        raise Refused("WITNESS_PRE_DENIAL_OR_NONINTEGER_HEIGHT")
    if not sha(obj["head"]) or not sha(obj["db_sha256"]):
        raise Refused("WITNESS_DIGEST_INVALID")
    return obj,raw

def primary_state(path):
    """Real SQLite query_only mode=ro, verified SHA chain, no new DB creation."""
    if not path.is_file() or path.is_symlink():
        raise Refused("PRIMARY_SQLITE_MISSING_NO_REINITIALIZATION")
    raw=path.read_bytes()
    if len(raw)<100 or raw[:16]!=denial.SQLITE_HEADER:
        raise Refused("PRIMARY_SQLITE_TRUNCATED_OR_BAD_HEADER")
    digest=hashlib.sha256(raw).hexdigest()
    uri="file:"+quote(str(path),safe="/")+ "?mode=ro"
    try:
        with closing(sqlite3.connect(uri,uri=True,timeout=5.0)) as con:
            con.execute("PRAGMA query_only=ON")
            con.execute("BEGIN")
            if con.execute("PRAGMA quick_check").fetchone()!=("ok",):
                raise Refused("PRIMARY_SQLITE_INTEGRITY_CHECK_FAILED")
            state=con.execute("SELECT height,head FROM state WHERE id=1").fetchone()
            if state is None or con.execute("SELECT COUNT(*) FROM state").fetchone()[0]!=1:
                raise Refused("PRIMARY_EXISTING_STATE_MISSING")
            rows=con.execute("SELECT seq,role,nonce,claim_sha,prev,head,decision,admission FROM denials ORDER BY seq").fetchall()
            height,head=0,denial.GENESIS
            for seq,role,nonce,claim_sha,prev,updated,decision,admission in rows:
                denial.check_claim(nonce,claim_sha,role)
                if (seq!=height+1 or prev!=head or
                    updated!=denial.digest(head,seq,role,nonce,claim_sha) or
                    decision!="BLOCK" or admission!="NOT_AUTHORIZED"):
                    raise Refused("PRIMARY_DENIAL_CHAIN_INVALID")
                height,head=seq,updated
            if state!=(height,head):
                raise Refused("PRIMARY_DENIAL_STATE_CHAIN_MISMATCH")
            con.execute("COMMIT")
    except (sqlite3.Error,denial.Refused) as exc:
        raise Refused("PRIMARY_SQLITE_REFUSED:"+type(exc).__name__) from exc
    if path.read_bytes()!=raw:
        raise Refused("PRIMARY_SQLITE_CHANGED_DURING_OBSERVATION")
    return {"height":height,"head":head,"db_sha256":digest}

def proof_for(state):
    return {"protocol":PROTOCOL,"project":PROJECT,"r5l_parent":PARENT,
            "journal_name":denial.NAME,**state}

def write_exact(path,content,replace):
    """Real same-admin local fsync; not protected against owner replacement."""
    target=path
    if target.is_symlink() or (target.exists() and not target.is_file()):
        raise Refused("WITNESS_OUTPUT_IS_NOT_REGULAR")
    if replace:
        temp=target.parent/(WITNESS+".pending-"+hashlib.sha256(content).hexdigest()[:16])
        fd=os.open(str(temp),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        try:
            with os.fdopen(fd,"wb") as f:
                f.write(content);f.flush();os.fsync(f.fileno())
            os.replace(temp,target)
        finally:
            if temp.exists():temp.unlink()
    else:
        fd=os.open(str(target),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        with os.fdopen(fd,"wb") as f:
            f.write(content);f.flush();os.fsync(f.fileno())
    if os.name!="nt":
        dir_fd=os.open(str(target.parent),os.O_RDONLY)
        try:os.fsync(dir_fd)
        finally:os.close(dir_fd)

def capture(primary_dir,witness_dir):
    db,path=private_paths(primary_dir,witness_dir)
    if path.exists():raise Refused("WITNESS_ALREADY_EXISTS_NOT_REINITIALIZED")
    state=primary_state(db)
    if state["height"]<1:raise Refused("CANNOT_ANCHOR_PRE_DENIAL_GENESIS")
    write_exact(path,canonical(proof_for(state)),False)
    return denial.block("LOCAL_PRESERVED_WITNESS_CAPTURED_NOT_EXTERNAL",
                        local_height=state["height"])

def advance(primary_dir,witness_dir):
    db,path=private_paths(primary_dir,witness_dir)
    prior,_=read_witness(path)
    state=primary_state(db)
    if state["height"]<=prior["height"]:
        raise Refused("WITNESS_FLOOR_NOT_ADVANCING")
    # Prove the old hash-chain head remains at the old witness height.
    uri="file:"+quote(str(db),safe="/")+ "?mode=ro"
    with closing(sqlite3.connect(uri,uri=True,timeout=5.0)) as con:
        old=con.execute("SELECT head FROM denials WHERE seq=?",
                        (prior["height"],)).fetchone()
    if old!=(prior["head"],):
        raise Refused("REPLACED_FORK_BEFORE_PREVIOUS_WITNESS")
    write_exact(path,canonical(proof_for(state)),True)
    return denial.block("LOCAL_WITNESS_ADVANCED_NOT_EXTERNAL",
                        local_height=state["height"])

def audit(primary_dir,witness_dir,after_first_observation=None):
    """A retained local proof detects changed bytes; still NEVER authorizes."""
    try:
        db,path=private_paths(primary_dir,witness_dir)
        witness,old_bytes=read_witness(path)
        first=primary_state(db)
        if after_first_observation is not None:
            if not callable(after_first_observation):
                raise Refused("INVALID_LAB_INTERLEAVE")
            after_first_observation()
        second=primary_state(db)
        newest,now_bytes=read_witness(path)
        if old_bytes!=now_bytes or first!=second:
            raise Refused("EVIDENCE_CHANGED_DURING_TWO_READS")
        if any(witness[k]!=second[k] for k in ("height","head","db_sha256")):
            raise Refused("PRIMARY_REPLACED_WITH_PRESERVED_WITNESS")
        return denial.block("PRESERVED_LOCAL_WITNESS_MATCH_NOT_INDEPENDENT",
                            local_height=second["height"],
                            preserved_witness_sha=hashlib.sha256(now_bytes).hexdigest())
    except (Refused,denial.Refused,sqlite3.Error,OSError,TypeError,ValueError) as exc:
        return denial.block("INTEGRITY_OR_RECOVERY_EVIDENCE_QUARANTINED",
                            detail=str(exc) if isinstance(exc,(Refused,denial.Refused)) else type(exc).__name__)

def _real_child_restore(actions):
    """Real child process touching only disposable lab files."""
    program=("import os,shutil,sys; a=sys.argv[1:]; "
             "assert len(a)%3==0; "
             "[(shutil.copy2(a[i+1],a[i+2]) if a[i]=='COPY' "
             "else (os.unlink(a[i+2]) if a[i]=='DELETE' else open(a[i+2],'wb').write(b'x'*int(a[i+1])))) "
             "for i in range(0,len(a),3)]")
    cmd=[sys.executable,"-B","-c",program]
    for mode,source,target in actions:
        cmd.extend([mode,str(source),str(target)])
    result=subprocess.run(cmd,capture_output=True,text=True,timeout=20)
    if result.returncode:
        raise Refused("REAL_CHILD_FILE_DAMAGE_FAILED:"+result.stderr[-180:])

def native_exercise():
    with tempfile.TemporaryDirectory(prefix="nordoi-r5m-native-") as root_str:
        root=Path(root_str)
        if os.name!="nt":os.chmod(root,0o700)
        a=root/"primary";b=root/"witness"
        a.mkdir(mode=0o700);b.mkdir(mode=0o700)
        if os.name!="nt":os.chmod(a,0o700);os.chmod(b,0o700)
        with denial.DenialJournal(a) as journal:
            if journal.record("a"*64,"b"*64,"candidate_a")["local_height"]!=1:
                raise Refused("REAL_PREPARED_SQLITE_DENIAL_FAILED")
        captured=capture(a,b)
        if captured["local_height"]!=1:raise Refused("REAL_CAPTURE_FAILED")
        olddb=root/"old-db.sqlite3";oldw=root/"old-witness.json"
        shutil.copy2(a/denial.NAME,olddb)
        shutil.copy2(b/WITNESS,oldw)
        with denial.DenialJournal(a) as journal:
            if journal.record("c"*64,"d"*64,"candidate_b")["local_height"]!=2:
                raise Refused("REAL_SECOND_DENIAL_FAILED")
        if advance(a,b)["local_height"]!=2:
            raise Refused("LOCAL_ADVANCE_FAILED")
        before=audit(a,b)
        if before["reason"]!="PRESERVED_LOCAL_WITNESS_MATCH_NOT_INDEPENDENT":
            raise Refused("REAL_PRECHECK_FAILED")
        trunc=audit(a,b,after_first_observation=lambda:_real_child_restore([
            ("TRUNC","32",a/denial.NAME)]))
        if trunc["reason"]!="INTEGRITY_OR_RECOVERY_EVIDENCE_QUARANTINED":
            raise Refused("REAL_SECOND_PROCESS_TRUNCATE_NOT_BLOCKED")
        _real_child_restore([("COPY",olddb,a/denial.NAME)])
        rollback=audit(a,b)
        if "PRIMARY_REPLACED_WITH_PRESERVED_WITNESS" not in rollback.get("detail",""):
            raise Refused("PRESERVED_SECOND_PROOF_ROLLBACK_NOT_DETECTED")
        _real_child_restore([("COPY",oldw,b/WITNESS)])
        joint=audit(a,b)
        if joint["reason"]!="PRESERVED_LOCAL_WITNESS_MATCH_NOT_INDEPENDENT" or joint["local_height"]!=1:
            raise Refused("SAME_ADMIN_JOINT_OLD_SNAPSHOTS_NOT_REPRODUCED")
        return denial.block("REAL_CORRUPTION_RECOVERY_REFUSED_WITH_LOCAL_WITNESS_LIMITS",
                            actual_second_process_truncation="BLOCK",
                            preserved_witness_detected_primary_rollback=True,
                            two_restored_old_same_owner_artifacts="LOCALLY_MATCH_UNDETECTABLE",
                            native_authority_granted=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    mode=p.add_mutually_exclusive_group(required=True)
    mode.add_argument("--exercise",action="store_true")
    mode.add_argument("--audit",action="store_true")
    p.add_argument("--primary-dir");p.add_argument("--witness-dir")
    args=p.parse_args(argv)
    try:
        if args.exercise:report=native_exercise()
        else:report=audit(args.primary_dir,args.witness_dir)
    except (Refused,denial.Refused,sqlite3.Error,OSError,TypeError,ValueError) as exc:
        report=denial.block("R5M_LAB_REFUSED",
                            detail=str(exc) if isinstance(exc,(Refused,denial.Refused)) else type(exc).__name__)
    print(json.dumps(report,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
