#!/usr/bin/env python3
"""NORDOI G0.3 R5N: interrupted checkpoint hand-off, no native permission.

Real R5L SQLite denial journal and R5M same-admin witness are observed before,
between, and after controlled file replacement by actual separate OS processes.
A local deny-only journal records accepted evidence for a *refusal*, not an
authorization. A third read cannot create atomic filesystem custody. A full
same-admin restore of all evidence remains undetectable. Always BLOCK.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile

import g03_r5l_crash_restart_denial_journal as denial
import g03_r5m_file_integrity_quarantine as witness

PROTOCOL="NORDOI-G03-R5N-INTERRUPTED-CHECKPOINT-USE-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="e1073d35bb13a61ebe97d20968dbc2abbb28dba5"
FIELDS=frozenset(("protocol","project","parent","operation","nonce",
                  "height","head","db_sha256","witness_sha256"))
MAX_BYTES=1800

class Refused(ValueError):pass

def canonical(data):
    return (json.dumps(data,sort_keys=True,separators=(",",":"),ensure_ascii=True,
                       allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    d={}
    for k,v in pairs:
        if k in d:raise Refused("DUPLICATE_CHECKPOINT_FIELD")
        d[k]=v
    return d

def no_constant(_):raise Refused("NONFINITE_VALUE")

def parse(raw):
    if type(raw) is not bytes or not 0<len(raw)<=MAX_BYTES:
        raise Refused("MISSING_OR_OVERSIZED_CHECKPOINT_INTENT")
    try:
        obj=json.loads(raw.decode("utf-8"),object_pairs_hook=unique,
                       parse_constant=no_constant)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Refused("NOT_CANONICAL_CHECKPOINT_JSON") from exc
    if type(obj) is not dict or obj.keys()!=FIELDS or canonical(obj)!=raw:
        raise Refused("CHECKPOINT_FIELDS_OR_CANONICAL_FAILURE")
    if obj["protocol"]!=PROTOCOL or obj["project"]!=PROJECT or obj["parent"]!=PARENT:
        raise Refused("SOURCE_OR_PROTOCOL_NOT_PINNED")
    if obj["operation"]!="DENIAL_ONLY_EVIDENCE_USE":
        raise Refused("RECOVERY_OR_GRANT_OPERATION_NOT_ALLOWED")
    if type(obj["height"]) is not int or obj["height"]<1:
        raise Refused("GENESIS_OR_BOOLEAN_HEIGHT_NOT_ALLOWED")
    for key in ("nonce","head","db_sha256","witness_sha256"):
        if not witness.sha(obj[key]):raise Refused("CHECKPOINT_DIGEST_OR_NONCE_INVALID")
    return obj

def view(primary_dir,witness_dir):
    db,wp=witness.private_paths(primary_dir,witness_dir)
    audit=witness.audit(primary_dir,witness_dir)
    if audit.get("reason")!="PRESERVED_LOCAL_WITNESS_MATCH_NOT_INDEPENDENT":
        raise Refused("CHECKPOINT_QUARANTINED:"+audit.get("reason","UNKNOWN"))
    proof,raw=witness.read_witness(wp)
    present=witness.primary_state(db)
    if any(proof[k]!=present[k] for k in ("height","head","db_sha256")):
        raise Refused("CHECKPOINT_WITNESS_CHANGED")
    if present["height"]<1:raise Refused("CHECKPOINT_PRE_DENIAL")
    return {"height":present["height"],"head":present["head"],
            "db_sha256":present["db_sha256"],
            "witness_sha256":hashlib.sha256(raw).hexdigest()}

def prepare(primary_dir,witness_dir,nonce):
    if not witness.sha(nonce):raise Refused("INVALID_CHECKPOINT_USE_NONCE")
    state=view(primary_dir,witness_dir)
    return canonical({"protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
                      "operation":"DENIAL_ONLY_EVIDENCE_USE","nonce":nonce,**state})

def block(reason,detail=None,**kw):
    result=denial.block(reason,**kw)
    if detail is not None:result["detail"]=detail
    return result

def evaluate(raw,primary_dir,witness_dir,journal_dir,
             after_first_observation=None,before_local_deny_commit=None):
    """Laboratory use path: all outcomes BLOCK, no permission grant.

    A changed file after the final read can always escape these observations.
    Such races are not an admission vulnerability: NO ADMISSION is possible.
    """
    try:
        claim=parse(raw)
        # R5L journal must already be initialized. No silent creation by a use call.
        journal_path=denial.private_journal_dir(journal_dir)
        if not journal_path.is_file():
            raise Refused("DENIAL_JOURNAL_MISSING_NO_AUTO_CREATE")
        initial=view(primary_dir,witness_dir)
        if after_first_observation is not None:
            if not callable(after_first_observation):raise Refused("INVALID_INTERLEAVING_HOOK")
            after_first_observation()
        second=view(primary_dir,witness_dir)
        if initial!=second:raise Refused("CHECKPOINT_CHANGED_AFTER_FIRST_READ")
        for key in ("height","head","db_sha256","witness_sha256"):
            if claim[key]!=second[key]:
                raise Refused("CHECKPOINT_INTENT_STALE_"+key.upper())
        if before_local_deny_commit is not None:
            if not callable(before_local_deny_commit):raise Refused("INVALID_COMMIT_HOOK")
            before_local_deny_commit()
        last=view(primary_dir,witness_dir)
        if second!=last:raise Refused("CHECKPOINT_CHANGED_BEFORE_LOCAL_DENIAL")
        claim_digest=hashlib.sha256(raw).hexdigest()
        result=denial.execute(journal_dir,"record",claim["nonce"],claim_digest,
                              "candidate_a")
        if result.get("reason")!="LOCAL_DENIAL_DURABLY_COMMITTED_NOT_ADMISSION":
            raise Refused("LOCAL_DENIAL_REPLAY_OR_REFUSAL:"+str(result.get("detail",result.get("reason"))))
        return block("VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED",
                     local_denial_height=result["local_height"],
                     verified_local_evidence_only=True,native_authority_granted=False)
    except (Refused,witness.Refused,denial.Refused,OSError,ValueError,TypeError,
            sqlite3.Error,subprocess.SubprocessError) as exc:
        return block("CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED",
                     detail=str(exc) if isinstance(exc,(Refused,witness.Refused,denial.Refused)) else type(exc).__name__,
                     native_authority_granted=False)

def native_exercise():
    if os.name=="nt":
        return block("POSIX_REAL_PROCESS_EXERCISE_UNAVAILABLE",native_authority_granted=False)
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5n-native-") as td:
        root=Path(td)
        os.chmod(root,0o700)
        p,w,j=(root/"primary",root/"witness",root/"journal")
        for folder in (p,w,j):
            folder.mkdir(mode=0o700);os.chmod(folder,0o700)
        with denial.DenialJournal(p) as d:
            if d.record("a"*64,"b"*64,"candidate_a")["local_height"]!=1:
                raise Refused("R5L_ACTUAL_DENIAL_NOT_DURABLE")
        if witness.capture(p,w).get("local_height")!=1:
            raise Refused("R5M_WITNESS_NOT_CAPTURED")
        old_db=root/"old-primary.sqlite3"
        old_proof=root/"old-witness.json"
        shutil.copy2(p/denial.NAME,old_db)
        shutil.copy2(w/witness.WITNESS,old_proof)
        with denial.DenialJournal(j) as d:
            if d.audit()["local_height"]!=0:raise Refused("R5L_USE_JOURNAL_NOT_EMPTY")
        intent=prepare(p,w,"c"*64)
        good=evaluate(intent,p,w,j)
        if good.get("reason")!="VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED":
            raise Refused("LOCAL_VERIFIED_USE_WAS_NOT_BLOCKED")
        duplicate=evaluate(intent,p,w,j)
        if "ALREADY_RECORDED_DENIAL_NONCE" not in duplicate.get("detail",""):
            raise Refused("DUPLICATE_CHECKPOINT_USE_NOT_BLOCKED")
        # Real SIGKILL of an UNCOMMITTED denial in a second OS process. A
        # new OS process must see only the pre-crash committed denial.
        if not denial.actual_kill(j,"d"*64,hashlib.sha256(intent).hexdigest(),"candidate_b"):
            raise Refused("REAL_CRASH_NOT_OBSERVED")
        if denial._run("audit",j).get("local_height")!=1:
            raise Refused("UNCOMMITTED_DENIAL_VISIBLE_AFTER_KILL")
        # A distinct OS process physically truncates SQLite AFTER a verified
        # observation, just before this deny-only use would be published.
        stale=evaluate(prepare(p,w,"e"*64),p,w,j,
                       after_first_observation=lambda:witness._real_child_restore([
                           ("TRUNC","32",p/denial.NAME)]))
        if stale.get("reason")!="CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED":
            raise Refused("TRUNCATED_SQLITE_NOT_QUARANTINED")
        witness._real_child_restore([("COPY",old_db,p/denial.NAME)])
        # Move the journal to height 2, retain its earlier proof first.
        with denial.DenialJournal(p) as d:
            if d.record("f"*64,"1"*64,"candidate_b")["local_height"]!=2:
                raise Refused("SECOND_DURABLE_DENIAL_FAILED")
        if witness.advance(p,w).get("local_height")!=2:
            raise Refused("SECOND_LOCAL_WITNESS_ADVANCE_FAILED")
        latest=prepare(p,w,"2"*64)
        changed=evaluate(latest,p,w,j,
                         before_local_deny_commit=lambda:witness._real_child_restore([
                             ("COPY",old_proof,w/witness.WITNESS)]))
        if changed.get("reason")!="CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED":
            raise Refused("PROOF_SWAP_BEFORE_COMMIT_NOT_QUARANTINED")
        # Both old same-owner artifacts may be restored together without
        # globally proving that height 2 ever existed. Still no grant.
        witness._real_child_restore([("COPY",old_db,p/denial.NAME)])
        oldclaim=prepare(p,w,"3"*64)
        rollback=evaluate(oldclaim,p,w,j)
        if rollback.get("reason")!="VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED":
            raise Refused("OLD_CONSISTENT_FILES_MUST_STILL_DENY")
        return block("REAL_CHECKPOINT_USE_INTERRUPTION_ALWAYS_DENIED",
                     verified_local_checkpoint_use="DENIAL_COMMITTED",
                     local_replay="BLOCK",
                     genuine_os_sigkill_uncommitted_record="ROLLED_BACK",
                     second_os_process_truncation="QUARANTINED",
                     witness_swapped_before_denial_commit="QUARANTINED",
                     both_old_local_artifacts="LOCALLY_MATCH_NO_GLOBAL_ANTIROLLBACK",
                     native_authority_granted=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    g=p.add_mutually_exclusive_group(required=True)
    g.add_argument("--exercise",action="store_true")
    g.add_argument("--assess",action="store_true")
    p.add_argument("--intent-file")
    p.add_argument("--primary-dir")
    p.add_argument("--witness-dir")
    p.add_argument("--journal-dir")
    args=p.parse_args(argv)
    try:
        if args.exercise:r=native_exercise()
        elif args.intent_file:
            r=evaluate(Path(args.intent_file).read_bytes(),
                       args.primary_dir,args.witness_dir,args.journal_dir)
        else:r=block("MISSING_RECOVERY_EVIDENCE",native_authority_granted=False)
    except (Refused,witness.Refused,denial.Refused,OSError,TypeError,ValueError,
            sqlite3.Error,subprocess.SubprocessError) as exc:
        r=block("CHECKPOINT_USE_EXERCISE_REFUSED",
                detail=str(exc) if isinstance(exc,(Refused,witness.Refused,denial.Refused)) else type(exc).__name__,
                native_authority_granted=False)
    print(json.dumps(r,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
