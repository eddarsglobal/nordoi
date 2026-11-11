#!/usr/bin/env python3
"""NORDOI G0.3 R5P — real close/reopen identity continuity, deny-only.

A tuple of local filesystem metadata is an observational fingerprint, NOT a
globally unique identity or cryptographic monotonic witness. Even successful
research checks grant no native privilege. Local same-admin rollback persists.
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
import g03_r5n_checkpoint_use_quarantine as use
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL="NORDOI-G03-R5P-CLOSE-REOPEN-IDENTITY-EPOCH-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="d97886abdf2b57324d7e05cd73182837dd112917"
OPERATION="LOCAL_DENIAL_AFTER_REOPEN_ONLY"
FIELDS=frozenset(("protocol","project","parent","operation","nonce",
                  "height","head","db_sha256","witness_sha256",
                  "db_identity","witness_identity"))
MAX_TICKET_BYTES=2400
REUSE_ATTEMPTS=48

class Refused(ValueError):pass

def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    result={}
    for key,value in pairs:
        if key in result:raise Refused("DUPLICATE_REOPEN_TICKET_KEY")
        result[key]=value
    return result

def no_constant(_):raise Refused("NONFINITE_TICKET")

def parse(ticket):
    if type(ticket) is not bytes or not 0<len(ticket)<=MAX_TICKET_BYTES:
        raise Refused("TICKET_NOT_BYTES_OR_OVERSIZED")
    try:
        value=json.loads(ticket.decode("utf-8"),object_pairs_hook=unique,
                         parse_constant=no_constant)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Refused("REOPEN_TICKET_NOT_JSON") from exc
    if type(value) is not dict or value.keys()!=FIELDS or canonical(value)!=ticket:
        raise Refused("REOPEN_TICKET_NOT_CANONICAL_OR_FIELDS")
    if (value["protocol"]!=PROTOCOL or value["project"]!=PROJECT or
        value["parent"]!=PARENT):
        raise Refused("REOPEN_TICKET_SOURCE_MISMATCH")
    if value["operation"]!=OPERATION:raise Refused("RECOVERY_GRANT_OP_FORBIDDEN")
    if type(value["height"]) is not int or value["height"]<1:
        raise Refused("REOPEN_HEIGHT_INVALID")
    for key in ("nonce","head","db_sha256","witness_sha256"):
        if not witness.sha(value[key]):raise Refused("REOPEN_SHA_INVALID_"+key)
    for key in ("db_identity","witness_identity"):
        ident=value[key]
        if type(ident) is not list or len(ident)!=6 or any(
                type(item) is not int or item<0 for item in ident):
            raise Refused("REOPEN_IDENTITY_NOT_SIX_NONNEGATIVE_INTS_"+key)
    return value

def epoch(primary_dir,witness_dir):
    """One actual open/close epoch. Re-opened descriptors will be fresh."""
    db,wp=witness.private_paths(primary_dir,witness_dir)
    with fdgate.OpenSnapshot(db) as d,fdgate.OpenSnapshot(wp) as w:
        seen=fdgate.observed(db,wp)
        if (d.digest!=seen["db_sha256"] or
            w.digest!=seen["witness_sha256"] or
            tuple(d.before[:2])!=(seen["db_dev"],seen["db_ino"]) or
            tuple(w.before[:2])!=(seen["witness_dev"],seen["witness_ino"])):
            raise Refused("OPEN_FILE_DIFFERS_FROM_REOPEN_EPOCH")
        d.check();w.check()
        return {"height":seen["height"],"head":seen["head"],
                "db_sha256":seen["db_sha256"],"witness_sha256":seen["witness_sha256"],
                "db_identity":list(d.before),"witness_identity":list(w.before)}

def prepare(primary_dir,witness_dir,nonce):
    if not witness.sha(nonce):raise Refused("BAD_REOPEN_NONCE")
    state=epoch(primary_dir,witness_dir)
    return canonical({"protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
                      "operation":OPERATION,"nonce":nonce,**state})

def block(reason,detail=None,**items):
    record=denial.block(reason,native_authority_granted=False,**items)
    if detail is not None:record["detail"]=detail
    return record

def evaluate(ticket,primary_dir,witness_dir,journal_dir,
             after_close=None,before_denial=None):
    """The ticket never grants. File identity is necessarily reopened and rechecked."""
    try:
        claim=parse(ticket)
        journal=denial.private_journal_dir(journal_dir)
        if not journal.is_file():raise Refused("DENIAL_JOURNAL_MISSING_NO_REINITIALIZATION")
        db,wp=witness.private_paths(primary_dir,witness_dir)
        first=epoch(primary_dir,witness_dir)
        for key,value in first.items():
            if claim[key]!=value:raise Refused("CLOSED_DESCRIPTOR_TICKET_STALE_"+key.upper())
        # All first-epoch FDs are closed. A real second process can swap paths.
        if after_close is not None:
            if not callable(after_close):raise Refused("INVALID_AFTER_CLOSE_INTERLEAVE")
            after_close()
        reopened=epoch(primary_dir,witness_dir)
        if reopened!=first:raise Refused("PATH_IDENTITY_CHANGED_ACROSS_CLOSE_REOPEN")
        if before_denial is not None:
            if not callable(before_denial):raise Refused("INVALID_BEFORE_DENIAL_INTERLEAVE")
            before_denial()
        last=epoch(primary_dir,witness_dir)
        if last!=first:raise Refused("PATH_IDENTITY_CHANGED_BEFORE_LOCAL_DENIAL")
        # Fresh R5O held-descriptor check; all subsequent effects are deny-only.
        r5o_ticket=fdgate.prepare(primary_dir,witness_dir,claim["nonce"])
        receipt=fdgate.evaluate(r5o_ticket,primary_dir,witness_dir,journal_dir)
        if receipt.get("reason")!="OPEN_FD_IDENTITY_MATCHED_ONLY_DENIAL_COMMITTED":
            raise Refused("LOCAL_DENIAL_REPLAY_OR_REFUSAL:"+str(
                receipt.get("detail",receipt.get("reason"))))
        return block("REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL",
                     local_denial_height=receipt["local_denial_height"],
                     close_reopen_checked=True,
                     independent_identity_proof=False)
    except (Refused,witness.Refused,denial.Refused,use.Refused,fdgate.Refused,
            OSError,sqlite3.Error,ValueError,TypeError,subprocess.SubprocessError) as exc:
        return block("REOPEN_IDENTITY_QUARANTINED",
                     detail=str(exc) if isinstance(exc,(
                         Refused,witness.Refused,denial.Refused,use.Refused,
                         fdgate.Refused)) else type(exc).__name__)

def replace_second_process(source,target):
    """Actual os.replace by another local OS process; byte-preserving swap."""
    fdgate.real_atomic_replace(source,target)

def actual_reuse_probe(source,target,attempts=REUSE_ATTEMPTS):
    """Bounded genuine inode reuse observation. Never assert reuse MUST occur."""
    if type(attempts) is not int or not 1<=attempts<=64:
        raise Refused("INODE_REUSE_PROBE_ATTEMPTS_INVALID")
    seen=set();reused=False
    for _ in range(attempts):
        current=os.lstat(target)
        identity=(current.st_dev,current.st_ino)
        if identity in seen:reused=True
        seen.add(identity)
        replace_second_process(source,target)
    current=os.lstat(target)
    if (current.st_dev,current.st_ino) in seen:reused=True
    return "OBSERVED_NO_AUTHORITY" if reused else "NOT_OBSERVED_BOUNDED_NO_AUTHORITY"

def native_exercise():
    if os.name=="nt":return block("POSIX_REOPEN_LAB_NOT_AVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5p-native-") as tmp:
        root=Path(tmp);os.chmod(root,0o700)
        a,b,j=(root/"primary",root/"witness",root/"journal")
        for d in (a,b,j):d.mkdir(mode=0o700);os.chmod(d,0o700)
        with denial.DenialJournal(a) as ledger:
            if ledger.record("a"*64,"b"*64,"candidate_a")["local_height"]!=1:
                raise Refused("GENUINE_R5L_DENIAL_FAILED")
        if witness.capture(a,b).get("local_height")!=1:
            raise Refused("GENUINE_R5M_LOCAL_WITNESS_FAILED")
        with denial.DenialJournal(j) as journal:
            if journal.audit().get("local_height")!=0:
                raise Refused("LOCAL_USE_JOURNAL_MISSING")
        db,wp=a/denial.NAME,b/witness.WITNESS
        source_db,source_wp=root/"copy.sqlite3",root/"copy.json"
        shutil.copy2(db,source_db);shutil.copy2(wp,source_wp)
        baseline=evaluate(prepare(a,b,"c"*64),a,b,j)
        if baseline.get("reason")!="REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL":
            raise Refused("BASELINE_LOCAL_DENIAL_FAILED")
        duplicate=evaluate(prepare(a,b,"c"*64),a,b,j)
        if "ALREADY_RECORDED_DENIAL_NONCE" not in duplicate.get("detail",""):
            raise Refused("SPENT_LOCAL_NONCE_NOT_DENIED")
        old=prepare(a,b,"d"*64)
        switched=evaluate(old,a,b,j,
                          after_close=lambda:replace_second_process(source_db,db))
        if switched.get("reason")!="REOPEN_IDENTITY_QUARANTINED" or \
           "PATH_IDENTITY_CHANGED_ACROSS_CLOSE_REOPEN" not in switched.get("detail",""):
            raise Refused("REAL_CLOSED_FD_PRIMARY_SWAP_NOT_REFUSED")
        old_proof=prepare(a,b,"e"*64)
        proof_changed=evaluate(old_proof,a,b,j,
                               before_denial=lambda:replace_second_process(source_wp,wp))
        if "PATH_IDENTITY_CHANGED_BEFORE_LOCAL_DENIAL" not in proof_changed.get("detail",""):
            raise Refused("REAL_WITNESS_SWAP_BEFORE_DENIAL_NOT_REFUSED")
        old_twice=prepare(a,b,"f"*64)
        def twice():
            replace_second_process(source_db,db)
            replace_second_process(source_db,db)
        twice_changed=evaluate(old_twice,a,b,j,after_close=twice)
        if "PATH_IDENTITY_CHANGED_ACROSS_CLOSE_REOPEN" not in twice_changed.get("detail",""):
            raise Refused("TWO_ATOMIC_PRIMARY_SWAPS_NOT_REFUSED")
        reuse=actual_reuse_probe(source_db,db)
        if reuse not in ("OBSERVED_NO_AUTHORITY","NOT_OBSERVED_BOUNDED_NO_AUTHORITY"):
            raise Refused("UNEXPECTED_INODE_REUSE_OBSERVATION")
        refreshed=evaluate(prepare(a,b,"1"*64),a,b,j)
        if refreshed.get("reason")!="REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL":
            raise Refused("NEW_CONSISTENT_FILES_MUST_STILL_DENY")
        if denial._run("audit",j).get("local_height")!=2:
            raise Refused("LOCAL_DENIAL_COUNT_NOT_TWO")
        return block("REAL_CLOSE_REOPEN_IDENTITY_CONTINUITY_DENIED",
                     closed_descriptor_primary_swap="QUARANTINED",
                     witness_swap_before_denial="QUARANTINED",
                     two_successive_atomic_swaps="QUARANTINED",
                     bounded_real_inode_reuse_probe=reuse,
                     fresh_matching_new_file_identity="ONLY_LOCAL_DENIAL",
                     spent_local_nonce="BLOCK",
                     all_same_admin_history_restore="NOT_GLOBALLY_DETECTABLE")

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    g=parser.add_mutually_exclusive_group(required=True)
    g.add_argument("--exercise",action="store_true")
    g.add_argument("--assess",action="store_true")
    parser.add_argument("--ticket-file")
    parser.add_argument("--primary-dir")
    parser.add_argument("--witness-dir")
    parser.add_argument("--journal-dir")
    args=parser.parse_args(argv)
    try:
        if args.exercise:result=native_exercise()
        elif args.ticket_file:
            result=evaluate(Path(args.ticket_file).read_bytes(),args.primary_dir,
                            args.witness_dir,args.journal_dir)
        else:result=block("TICKET_REQUIRED")
    except (Refused,witness.Refused,denial.Refused,use.Refused,fdgate.Refused,
            OSError,sqlite3.Error,ValueError,TypeError,subprocess.SubprocessError) as exc:
        result=block("R5P_NATIVE_EXERCISE_REFUSED",
                     detail=str(exc) if isinstance(exc,(
                         Refused,witness.Refused,denial.Refused,use.Refused,
                         fdgate.Refused)) else type(exc).__name__)
    print(json.dumps(result,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
