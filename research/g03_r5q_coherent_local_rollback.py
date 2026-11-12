#!/usr/bin/env python3
"""G0.3 R5Q: real two-history coherent local rollback indistinguishability.

Proof BY COUNTEREXAMPLE: an owner's three genuine persistent SQLite/JSON
evidence files, restored as a complete old set by a separate process, cannot
tell a newly started local examiner that a later history was ever committed.
No external operator, protected high-water, global replay defense, or access
authorization is inferred from local bytes, metadata, hashes or signatures.
All responses are BLOCK / NOT_AUTHORIZED; native CLI exits 2.
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
import g03_r5m_file_integrity_quarantine as witness
import g03_r5p_reopen_identity_continuity as reopen

PROTOCOL="NORDOI-G03-R5Q-COHERENT-LOCAL-ROLLBACK-V1"
PARENT="a9968141e628ec4ccfae560830414c9c8cb5dd92"
PROJECT="eddarsglobal/nordoi"
PARTS=("primary.sqlite3","local_witness.json","spent_nonces.sqlite3")

class Refused(ValueError):pass

def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")

def block(reason,**kw):
    return denial.block(reason,native_authority_granted=False,**kw)

def paths(primary_dir,witness_dir,journal_dir):
    db,wp=witness.private_paths(primary_dir,witness_dir)
    jd=denial.private_journal_dir(journal_dir)
    jroot=Path(journal_dir).resolve(strict=True)
    if jroot in (db.parent,wp.parent) or jroot in db.parent.parents or jroot in wp.parent.parents or db.parent in jroot.parents or wp.parent in jroot.parents:
        raise Refused("OVERLAPPING_EVIDENCE_ROOTS")
    for p in (db,wp,jd):
        if not p.is_file() or p.is_symlink():
            raise Refused("MISSING_REAL_EVIDENCE_NO_SILENT_REINITIALIZATION")
        if os.name!="nt" and p.stat().st_mode&0o077:
            raise Refused("EVIDENCE_FILE_NOT_PRIVATE")
    return db,wp,jd

def spent_nonce_records(db):
    if not db.is_file() or db.is_symlink():raise Refused("JOURNAL_NONCE_FILE_MISSING")
    uri="file:"+quote(str(db),safe="/")+"?mode=ro"
    try:
        with closing(sqlite3.connect(uri,uri=True,timeout=8.0)) as con:
            con.execute("PRAGMA query_only=ON")
            con.execute("BEGIN")
            if con.execute("PRAGMA quick_check").fetchone()!=("ok",):
                raise Refused("SPENT_NONCE_SQLITE_CORRUPT")
            rows=con.execute("SELECT seq,nonce,claim_sha,role,head,decision,admission FROM denials ORDER BY seq").fetchall()
            con.execute("COMMIT")
    except sqlite3.Error as exc:
        raise Refused("SPENT_NONCE_SQLITE_INVALID:"+type(exc).__name__) from exc
    for row in rows:
        if not (len(row)==7 and row[5]=="BLOCK" and row[6]=="NOT_AUTHORIZED"):
            raise Refused("NON_DENIAL_RECORD_PRESENT")
    return [[seq,nonce,claim_sha,role,head] for seq,nonce,claim_sha,role,head,_,_ in rows]

def projection(primary_dir,witness_dir,journal_dir):
    """Read-only identity of security-relevant local evidence, not inode/time.

    Reopened path metadata is a new local observation. Two coherent states
    may differ in ctime/inode yet remain indistinguishable *historically*:
    no independent historical root exists after one admin restores everything.
    """
    db,wp,jd=paths(primary_dir,witness_dir,journal_dir)
    proof=witness.audit(primary_dir,witness_dir)
    if proof.get("reason")!="PRESERVED_LOCAL_WITNESS_MATCH_NOT_INDEPENDENT":
        raise Refused("PRIMARY_AND_LOCAL_WITNESS_DISAGREE:"+proof.get("reason","UNKNOWN"))
    present=witness.primary_state(db)
    wobj,wbytes=witness.read_witness(wp)
    if any(wobj[key]!=present[key] for key in ("height","head","db_sha256")):
        raise Refused("PRIMARY_LOCAL_PROOF_INCONSISTENT")
    # R5L verifies all committed local deny records and the full SHA chain.
    journal=denial.execute(journal_dir,"audit")
    if journal.get("reason")!="LOCAL_DENIAL_HISTORY_NOT_EXTERNAL":
        raise Refused("LOCAL_DENIAL_AUDIT_FAILED:"+str(journal.get("detail",journal.get("reason"))))
    rows=spent_nonce_records(jd)
    if len(rows)!=journal["local_height"]:
        raise Refused("LOCAL_SPENT_NONCE_COUNT_CHANGED")
    if rows and rows[-1][-1]!=journal["local_head"]:
        raise Refused("LOCAL_SPENT_NONCE_CHAIN_HEAD_CHANGED")
    if hashlib.sha256(db.read_bytes()).hexdigest()!=present["db_sha256"] or wp.read_bytes()!=wbytes:
        raise Refused("LOCAL_BYTES_CHANGED_DURING_READ")
    return {
        "protocol":PROTOCOL,
        "parent":PARENT,
        "project":PROJECT,
        "primary_height":present["height"],
        "primary_head":present["head"],
        "primary_db_sha256":present["db_sha256"],
        "witness_bytes_sha256":hashlib.sha256(wbytes).hexdigest(),
        "local_spent_nonce_height":journal["local_height"],
        "local_spent_nonce_head":journal["local_head"],
        "spent_nonce_records":rows,
        "all_local_authority_only":True,
    }

def digest_projection(p):
    return hashlib.sha256(canonical(p)).hexdigest()

def report_audit(primary_dir,witness_dir,journal_dir):
    try:
        p=projection(primary_dir,witness_dir,journal_dir)
        return block("COHERENT_LOCAL_EVIDENCE_NOT_GLOBAL_HISTORY",
                     local_projection_sha256=digest_projection(p),
                     primary_height=p["primary_height"],
                     locally_spent_nonces=p["local_spent_nonce_height"],
                     externally_authenticated_history=False)
    except (Refused,witness.Refused,denial.Refused,reopen.Refused,
            OSError,ValueError,TypeError,sqlite3.Error) as exc:
        return block("LOCAL_EVIDENCE_QUARANTINED",
                     detail=str(exc) if isinstance(exc,(Refused,witness.Refused,
                              denial.Refused,reopen.Refused)) else type(exc).__name__)

def safe_lab_root(root):
    p=Path(root)
    if not p.is_absolute() or p.is_symlink() or not p.is_dir() or not p.name.startswith("nordoi-r5l-r5q-"):
        raise Refused("ONLY_DISPOSABLE_R5Q_LAB_ROOT_ALLOWED")
    if os.name!="nt" and p.stat().st_mode&0o077:
        raise Refused("LAB_ROOT_NOT_PRIVATE")
    return p.resolve(strict=True)

def file_snapshots(root,primary_dir,witness_dir,journal_dir,label):
    root=safe_lab_root(root)
    if label not in ("history_a","history_b"):
        raise Refused("UNRECOGNIZED_LAB_SNAPSHOT")
    sources=paths(primary_dir,witness_dir,journal_dir)
    destination=root/label
    destination.mkdir(mode=0o700)
    if os.name!="nt":os.chmod(destination,0o700)
    for source,name in zip(sources,PARTS):
        shutil.copy2(source,destination/name)
        if os.name!="nt":os.chmod(destination/name,0o600)
    return destination

def actual_second_process_restore(root,snapshot,primary_dir,witness_dir,journal_dir,names=PARTS):
    """Dangerous operation strictly limited to fresh disposable R5Q lab dirs."""
    root=safe_lab_root(root)
    if type(names) is not tuple or not names or len(set(names))!=len(names):
        raise Refused("INVALID_LAB_RESTORE_SUBSET")
    if any(name not in PARTS for name in names):
        raise Refused("UNLISTED_LAB_RESTORE_FILE")
    snapshot=Path(snapshot)
    if snapshot.is_symlink() or snapshot.resolve(strict=True).parent!=root or snapshot.name not in ("history_a","history_b"):
        raise Refused("RESTORE_SOURCE_OUTSIDE_DISPOSABLE_LAB")
    original=paths(primary_dir,witness_dir,journal_dir)
    for name,dst in zip(PARTS,original):
        if name in names and (root not in dst.resolve(strict=True).parents or dst.is_symlink()):
            raise Refused("RESTORE_TARGET_OUTSIDE_DISPOSABLE_LAB")
    actions=[("COPY",snapshot/name,dst)
             for name,dst in zip(PARTS,original) if name in names]
    for _,src,_ in actions:
        if not src.is_file() or src.is_symlink():
            raise Refused("RESTORE_SOURCE_MISSING_OR_SYMLINK")
    witness._real_child_restore(actions)
    return block("SAME_ADMIN_SECOND_PROCESS_RESTORED_LOCAL_FILES",
                 restored_files=list(names),
                 no_independent_external_custody=True)

def fixture(root):
    root=safe_lab_root(root)
    a,b,j=(root/"primary",root/"witness",root/"local_nonces")
    for d in (a,b,j):
        d.mkdir(mode=0o700)
        if os.name!="nt":os.chmod(d,0o700)
    with denial.DenialJournal(a) as d:
        x=d.record("a"*64,"b"*64,"candidate_a")
        if x.get("local_height")!=1:raise Refused("INITIAL_DENIAL_DID_NOT_COMMIT")
    if witness.capture(a,b).get("local_height")!=1:
        raise Refused("INITIAL_LOCAL_CHECKPOINT_DID_NOT_COMMIT")
    with denial.DenialJournal(j) as d:
        if d.audit().get("local_height")!=0:
            raise Refused("LOCAL_NONCE_GENESIS_FAILED")
    first=use_nonce(a,b,j,"c"*64)
    if first.get("reason")!="REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL":
        raise Refused("INITIAL_USE_DENIAL_NOT_RECORDED")
    return a,b,j

def use_nonce(a,b,j,nonce):
    ticket=reopen.prepare(a,b,nonce)
    return reopen.evaluate(ticket,a,b,j)

def advance_world(a,b,j,nonce="d"*64):
    with denial.DenialJournal(a) as journal:
        x=journal.record("e"*64,"f"*64,"candidate_b")
        if x.get("local_height")!=2:raise Refused("ADVANCED_WORLD_DENIAL_NOT_COMMITTED")
    if witness.advance(a,b).get("local_height")!=2:
        raise Refused("ADVANCED_WORLD_WITNESS_NOT_UPDATED")
    r=use_nonce(a,b,j,nonce)
    if r.get("reason")!="REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL" or r.get("local_denial_height")!=2:
        raise Refused("ADVANCED_WORLD_SPENT_NONCE_NOT_COMMITTED")
    return r

def native_exercise():
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-native-") as tmp:
        root=Path(tmp)
        if os.name!="nt":os.chmod(root,0o700)
        a,b,j=fixture(root)
        world_a=projection(a,b,j)
        a_hash=digest_projection(world_a)
        history_a=file_snapshots(root,a,b,j,"history_a")
        advance_world(a,b,j)
        world_b=projection(a,b,j)
        if (world_b["primary_height"]!=2 or
            world_b["local_spent_nonce_height"]!=2 or
            a_hash==digest_projection(world_b)):
            raise Refused("REAL_ADVANCED_WORLD_NOT_DISTINCT")
        # Preserve the advanced world too, for intentional symmetric tests.
        file_snapshots(root,a,b,j,"history_b")
        old_epoch=reopen.prepare(a,b,"1"*64)
        # An isolated primary rollback is detectable with newer witness retained.
        actual_second_process_restore(root,history_a,a,b,j,("primary.sqlite3",))
        single=report_audit(a,b,j)
        if single.get("reason")!="LOCAL_EVIDENCE_QUARANTINED":
            raise Refused("PRIMARY_ONLY_ROLLBACK_NOT_DETECTED")
        # Restore ALL same-admin state through another actual OS process.
        actual_second_process_restore(root,history_a,a,b,j)
        old_world=projection(a,b,j)
        if canonical(old_world)!=canonical(world_a):
            raise Refused("ALL_LOCAL_EVIDENCE_NOT_OBSERVATIONALLY_EQUAL")
        if digest_projection(old_world)!=a_hash:
            raise Refused("LOST_HISTORY_INCONSISTENT_DIGEST")
        # R5P stale epoch from advanced world correctly quarantines, while a
        # *new* epoch after rollback sees only an older coherent local state.
        old_ticket=reopen.evaluate(old_epoch,a,b,j)
        if old_ticket.get("reason")!="REOPEN_IDENTITY_QUARANTINED":
            raise Refused("ADVANCED_EPOCH_NOT_QUARANTINED")
        # Genuine NEW OS PROCESS must agree with the old local history, though
        # it has no means to know world B ever existed.
        cmd=[sys.executable,"-B",str(Path(__file__).resolve()),"--audit",
             "--primary-dir",str(a),"--witness-dir",str(b),"--journal-dir",str(j)]
        proc=subprocess.run(cmd,text=True,capture_output=True,timeout=35)
        if proc.returncode!=2:raise Refused("FRESH_EXAMINER_EXIT_NOT_DENY_TWO")
        child=json.loads(proc.stdout)
        if (child.get("reason")!="COHERENT_LOCAL_EVIDENCE_NOT_GLOBAL_HISTORY" or
            child.get("local_projection_sha256")!=a_hash):
            raise Refused("NEW_PROCESS_SEES_UNSUPPORTED_ADVANCED_WORLD")
        # This exact nonce was *actually spent* in advanced world B, but its
        # SQLite spent-nonce history was also reverted; a fresh local ticket
        # can now record it AGAIN. It is still BLOCK, never an actual grant.
        reissue=use_nonce(a,b,j,"d"*64)
        if reissue.get("reason")!="REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL" or reissue.get("local_denial_height")!=2:
            raise Refused("LOST_SPENT_NONCE_HISTORY_NOT_REPRODUCED")
        return block("REAL_TWO_HISTORIES_SAME_LOCAL_EVIDENCE_DENIED",
                     world_a_sha256=a_hash,
                     world_b_distinct=True,
                     same_admin_all_three_restored=True,
                     same_application_evidence_before_after=True,
                     fresh_os_process_observed_old_local_history=True,
                     advanced_epoch_ticket_quarantined=True,
                     primary_only_rollback_with_newer_witness="QUARANTINED",
                     globally_spent_nonce_reissued_after_full_rollback=True,
                     globally_spent_nonce_reissued_result="LOCAL_DENIAL_ONLY",
                     protected_independent_high_water=False,
                     all_native_admission="BLOCK")

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--exercise",action="store_true")
    group.add_argument("--audit",action="store_true")
    parser.add_argument("--primary-dir")
    parser.add_argument("--witness-dir")
    parser.add_argument("--journal-dir")
    args=parser.parse_args(argv)
    try:
        if args.exercise:result=native_exercise()
        else:result=report_audit(args.primary_dir,args.witness_dir,args.journal_dir)
    except (Refused,witness.Refused,denial.Refused,reopen.Refused,
            OSError,TypeError,ValueError,sqlite3.Error,subprocess.SubprocessError) as exc:
        result=block("R5Q_RESEARCH_EXERCISE_REFUSED",
                     detail=str(exc) if isinstance(exc,(Refused,witness.Refused,
                                denial.Refused,reopen.Refused)) else type(exc).__name__)
    print(json.dumps(result,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
