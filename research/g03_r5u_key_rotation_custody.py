#!/usr/bin/env python3
"""G0.3 R5U — real two-key SSHSIG rotation/revocation, local custody only.

True Ed25519 signatures and a durable SQLite key lineage are local facts.
Neither local signer allowlists, a second process, chmod, or local revocation
can prove independent administrative custody or global antirollback.
ALL evidence outcomes remain BLOCK / NOT_AUTHORIZED; no enrollment or GRANT.
"""
import argparse
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile
from urllib.parse import quote

import g03_r5l_crash_restart_denial_journal as denial
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5q_coherent_local_rollback as history
import g03_r5s_challenge_bound_handoff as handoff
import g03_r5t_signed_receipt_provenance as signed

PROTOCOL="NORDOI-G03-R5U-LOCAL-KEY-CUSTODY-CONTINUITY-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="3372ddbd691d5c6102bd395b4589c7a27ec030b3"
NAMESPACE="nordoi-r5u"
ROTATE="ROTATE_LOCAL_RESEARCH_KEY_BLOCK_ONLY"
REVOKE="REVOKE_LOCAL_RESEARCH_KEY_BLOCK_ONLY"
LEDGER_NAME="r5u_local_key_lineage.sqlite3"
INTENT_FIELDS=frozenset(("protocol","project","parent","operation","epoch",
                         "prior_key_sha256","next_key_sha256","receipt_sha256","nonce"))
GENESIS=hashlib.sha256((PROTOCOL+":GENESIS").encode()).hexdigest()

class Refused(ValueError):pass

def sha(b):return hashlib.sha256(b).hexdigest()
def canonical(x):return (json.dumps(x,sort_keys=True,separators=(",",":"),ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")
def check_sha(x):return type(x) is str and re.fullmatch("[a-f0-9]{64}",x) is not None
def pairs(items):
    result={}
    for name,v in items:
        if name in result:raise Refused("DUPLICATE_SIGNED_INTENT_KEY")
        result[name]=v
    return result
def no_nan(_):raise Refused("NONFINITE_SIGNED_INTENT")
def block(reason,**extra):
    return denial.block(reason,native_authority_granted=False,
                        independent_custodian="NOT_ENROLLED",
                        owner_oob_root="NOT_ENROLLED",
                        external_monotonic_key_history="NOT_VERIFIED",
                        globally_protected_revocation_floor="NOT_VERIFIED",**extra)

def parse_intent(raw):
    if type(raw) is not bytes or not 0<len(raw)<=1800:
        raise Refused("SIGNED_INTENT_INVALID_SIZE")
    try:
        x=json.loads(raw.decode("utf-8"),object_pairs_hook=pairs,parse_constant=no_nan)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Refused("SIGNED_INTENT_INVALID_UTF8_OR_JSON") from exc
    if type(x) is not dict or x.keys()!=INTENT_FIELDS or canonical(x)!=raw:
        raise Refused("SIGNED_INTENT_SCHEMA_NOT_CANONICAL")
    if (x["protocol"],x["project"],x["parent"])!=(PROTOCOL,PROJECT,PARENT):
        raise Refused("SIGNED_INTENT_SOURCE_OR_PROTOCOL_NOT_PINNED")
    if x["operation"] not in (ROTATE,REVOKE):raise Refused("GRANT_OR_UNKNOWN_OPERATION_DENIED")
    if type(x["epoch"]) is not int or x["epoch"]<=0:raise Refused("EPOCH_INVALID")
    for k in ("prior_key_sha256","next_key_sha256","receipt_sha256","nonce"):
        if not check_sha(x[k]):raise Refused("MALFORMED_PIN_OR_NONCE_"+k)
    if x["operation"]==ROTATE and x["prior_key_sha256"]==x["next_key_sha256"]:
        raise Refused("IDENTICAL_KEY_ROTATION_DENIED")
    if x["operation"]==REVOKE and x["prior_key_sha256"]!=x["next_key_sha256"]:
        raise Refused("REVOCATION_CANNOT_CHANGE_KEY")
    return x

def intent(operation,epoch,prior,new,receipt_sha,nonce):
    msg=canonical({"protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
                   "operation":operation,"epoch":epoch,
                   "prior_key_sha256":prior,"next_key_sha256":new,
                   "receipt_sha256":receipt_sha,"nonce":nonce})
    parse_intent(msg)
    return msg

def public_sha(path):
    raw=Path(path).read_bytes()
    columns=raw.strip().split()
    if len(columns)<2 or columns[0]!=b"ssh-ed25519" or len(raw)>2048:
        raise Refused("REAL_ED25519_PUBLIC_KEY_REQUIRED")
    return sha(b" ".join(columns[:2])+b"\n")

def verify_local(raw,signature,allowlist):
    """Genuine installed OpenSSH, *untrusted same-admin* allowed-signers file."""
    if os.name=="nt" or not Path("/usr/bin/ssh-keygen").is_file():
        raise Refused("REAL_POSIX_OPENSSH_REQUIRED")
    for path in (signature,allowlist):
        p=Path(path)
        if not p.is_file() or p.is_symlink():
            raise Refused("LOCAL_SIGNER_FILE_MISSING_OR_SYMLINK")
    args=["-Y","verify","-f",str(allowlist),"-I",signed.IDENTITY,
          "-n",NAMESPACE,"-s",str(signature)]
    if signed.run_ssh(args,raw)!=0:
        raise Refused("REAL_LOCAL_SSHSIG_INVALID_OR_WRONG_PIN")
    return True

def sign_intent(path,key):
    target=Path(str(path)+".sig")
    if target.exists():raise Refused("REAL_SIGNATURE_WOULD_OVERWRITE")
    if signed.run_ssh(["-Y","sign","-f",str(key),"-n",NAMESPACE,str(path)])!=0 or not target.is_file():
        raise Refused("REAL_ED25519_SIGN_INTENT_FAILED")
    if os.name!="nt":os.chmod(target,0o600)
    return target

def private_lab_dir(root):
    root=Path(root)
    if not root.is_absolute() or root.is_symlink() or not root.is_dir() or \
       not root.name.startswith("nordoi-r5l-r5q-r5s-r5t-r5u-") or \
       (os.name!="nt" and root.stat().st_mode&0o077):
        raise Refused("ONLY_REAL_DISPOSABLE_PRIVATE_R5U_LAB_DIR")
    return root.resolve(strict=True)

def ledger_path(directory):
    root=private_lab_dir(directory)
    path=root/LEDGER_NAME
    if path.is_symlink():raise Refused("LOCAL_LEDGER_SYMLINK")
    return path

def bootstrap(directory,key_sha):
    if not check_sha(key_sha):raise Refused("BOOTSTRAP_KEY_SHA_INVALID")
    path=ledger_path(directory)
    fd=os.open(str(path),os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600)
    os.close(fd)
    try:
        with closing(sqlite3.connect(str(path),timeout=15)) as con:
            con.execute("PRAGMA synchronous=FULL")
            con.execute("CREATE TABLE meta (initial TEXT NOT NULL, protocol TEXT NOT NULL)")
            con.execute("INSERT INTO meta VALUES (?,?)",(key_sha,PROTOCOL))
            con.execute("""CREATE TABLE events(
                seq INTEGER PRIMARY KEY,operation TEXT NOT NULL,
                prior_key TEXT NOT NULL,next_key TEXT NOT NULL,
                receipt TEXT NOT NULL,nonce TEXT NOT NULL UNIQUE,
                prev TEXT NOT NULL,digest TEXT NOT NULL)""")
            con.commit()
        if os.name!="nt":os.chmod(path,0o600)
    except Exception:
        path.unlink(missing_ok=True)
        raise
    return audit(directory)

def audit_rows(path):
    if not path.is_file():raise Refused("LOCAL_KEY_LEDGER_MISSING_NO_AUTOGENESIS")
    if os.name!="nt" and path.stat().st_mode&0o077:raise Refused("LOCAL_KEY_LEDGER_NOT_PRIVATE")
    uri="file:"+quote(str(path),safe="/")+"?mode=ro"
    try:
        with closing(sqlite3.connect(uri,uri=True,timeout=10)) as con:
            con.execute("PRAGMA query_only=ON")
            con.execute("BEGIN")
            if con.execute("PRAGMA quick_check").fetchone()!=("ok",):
                raise Refused("LOCAL_KEY_LEDGER_SQLITE_CORRUPT")
            meta=con.execute("SELECT initial,protocol FROM meta").fetchall()
            rows=con.execute("SELECT seq,operation,prior_key,next_key,receipt,nonce,prev,digest FROM events ORDER BY seq").fetchall()
            con.execute("COMMIT")
    except sqlite3.Error as e:
        raise Refused("LOCAL_KEY_LEDGER_SQLITE_INVALID:"+type(e).__name__) from e
    if len(meta)!=1 or meta[0][1]!=PROTOCOL or not check_sha(meta[0][0]):
        raise Refused("LOCAL_KEY_LEDGER_GENESIS_INVALID")
    return meta[0][0],rows

def state_from(initial,rows):
    current=initial;revoked=False;head=GENESIS;seen=set()
    for i,row in enumerate(rows,1):
        seq,op,old,new,receipt,nonce,prev,digest=row
        if (seq!=i or op not in (ROTATE,REVOKE) or
            not all(check_sha(x) for x in (old,new,receipt,nonce,prev,digest)) or
            nonce in seen or prev!=head or old!=current or revoked):
            raise Refused("LOCAL_KEY_LINEAGE_INVALID_OR_ROLLED_BACK_PARTIAL")
        expected=sha(canonical({"seq":seq,"operation":op,"prior_key":old,
                                "next_key":new,"receipt":receipt,
                                "nonce":nonce,"prev":prev}))
        if digest!=expected:raise Refused("LOCAL_KEY_LINEAGE_HASH_MISMATCH")
        if op==ROTATE:
            if new==old:raise Refused("LOCAL_KEY_ROTATION_NOT_DISTINCT")
            current=new
        else:
            if new!=old:raise Refused("LOCAL_REVOCATION_KEY_CHANGED")
            revoked=True
        head=digest;seen.add(nonce)
    return {"initial":initial,"active":current,"revoked":revoked,
            "height":len(rows),"head":head,"spent_nonces":sorted(seen),
            "independent_history_authority":False}

def audit(directory):
    path=ledger_path(directory)
    return state_from(*audit_rows(path))

def append(directory,payload,signature_old,allow_old,signature_next=None,allow_next=None):
    """Genuine crypto BEFORE local SQLite transaction; never a permission."""
    try:
        msg=parse_intent(payload)
        path=ledger_path(directory)
        state=audit(directory)
        if msg["epoch"]!=state["height"]+1 or state["revoked"]:
            raise Refused("STALE_OR_REVOKED_KEY_EPOCH")
        if msg["prior_key_sha256"]!=state["active"]:
            raise Refused("OLD_ACTIVE_KEY_PIN_MISMATCH")
        verify_local(payload,signature_old,allow_old)
        if msg["operation"]==ROTATE:
            if signature_next is None or allow_next is None:
                raise Refused("TWO_GENUINE_SIGNATURES_REQUIRED")
            verify_local(payload,signature_next,allow_next)
        # Both signers' real public keys must be pinned to the message,
        # not merely self-authored identity claims.
        old_pub=Path(str(allow_old)).read_text("ascii").split()
        if len(old_pub)<3 or sha((" ".join(old_pub[1:3])+"\n").encode())!=msg["prior_key_sha256"]:
            raise Refused("OLD_SIGNER_KEY_NOT_BOUND_TO_PIN")
        if msg["operation"]==ROTATE:
            new_pub=Path(str(allow_next)).read_text("ascii").split()
            if len(new_pub)<3 or sha((" ".join(new_pub[1:3])+"\n").encode())!=msg["next_key_sha256"]:
                raise Refused("NEW_SIGNER_KEY_NOT_BOUND_TO_PIN")
        if msg["nonce"] in state["spent_nonces"]:
            raise Refused("REPLAYED_LOCAL_ROTATION_OR_REVOCATION_NONCE")
        row={"seq":msg["epoch"],"operation":msg["operation"],
             "prior_key":msg["prior_key_sha256"],"next_key":msg["next_key_sha256"],
             "receipt":msg["receipt_sha256"],"nonce":msg["nonce"],"prev":state["head"]}
        digest=sha(canonical(row))
        with closing(sqlite3.connect(str(path),timeout=15)) as con:
            con.execute("PRAGMA synchronous=FULL")
            con.execute("BEGIN IMMEDIATE")
            # Hold an SQLite writer lock and reject external state changes.
            initial,rows=audit_rows(path)
            locked=state_from(initial,rows)
            if locked!=state:
                con.rollback()
                raise Refused("SQLITE_LINEAGE_CHANGED_BEFORE_COMMIT")
            con.execute("INSERT INTO events VALUES (?,?,?,?,?,?,?,?)",
                        (row["seq"],row["operation"],row["prior_key"],
                         row["next_key"],row["receipt"],row["nonce"],row["prev"],digest))
            con.commit()
        new_state=audit(directory)
        if new_state["head"]!=digest or new_state["height"]!=msg["epoch"]:
            raise Refused("LOCAL_LINEAGE_COMMIT_NOT_VISIBLE")
        return block("REAL_LOCAL_DUAL_SSHSIG_ROTATION_DENIED" if msg["operation"]==ROTATE else
                     "REAL_LOCAL_SIGNED_REVOCATION_DENIED",
                     cryptographic_key_possession_verified=True,
                     local_height=new_state["height"],
                     active_local_key_sha256=new_state["active"],
                     locally_revoked=new_state["revoked"],
                     independent_signer_provenance=False)
    except (Refused,sqlite3.Error,OSError,UnicodeError,ValueError) as e:
        return block("LOCAL_KEY_CHANGE_QUARANTINED",detail=str(e) if isinstance(e,Refused) else type(e).__name__,
                     independent_signer_provenance=False)

def active_key_check(directory,pub,receipt,signature,allowed):
    """Even an intact active signer is only a locally accepted DENIAL."""
    try:
        state=audit(directory)
        if state["revoked"]:raise Refused("LOCAL_SIGNER_REVOKED")
        if public_sha(pub)!=state["active"]:raise Refused("LOCAL_SIGNER_NOT_ACTIVE")
        raw=Path(receipt).read_bytes()
        signed.parse(raw)
        # R5T signed receipt uses namespace NORDOI-R5T, not the key-change
        # namespace. Exactly verify real signature, not only public-key SHA.
        if signed.run_ssh(["-Y","verify","-f",str(allowed),"-I",signed.IDENTITY,
                           "-n",signed.NAMESPACE,"-s",str(signature)],raw)!=0:
            raise Refused("REAL_LOCAL_ACTIVE_KEY_SSHSIG_INVALID")
        return block("ACTIVE_LOCAL_KEY_VERIFIED_NOT_INDEPENDENT",
                     local_key_valid=True,local_height=state["height"])
    except (Refused,sqlite3.Error,OSError,UnicodeError,ValueError) as exc:
        return block("LOCAL_KEY_AUTHORITY_QUARANTINED",
                     detail=str(exc) if isinstance(exc,Refused) else type(exc).__name__)

def native_exercise():
    if os.name=="nt":return block("POSIX_NATIVE_KEY_PROVENANCE_LAB_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-r5u-native-") as tmp:
        root=Path(tmp);os.chmod(root,0o700)
        a,b,j=history.fixture(root)
        zipfile=root/"handoff.zip"
        handoff.prepare(a,b,j,"a"*64,1000,1200,zipfile)
        receipt=root/"receipt.json"
        payload=signed.prepare_receipt(zipfile,"a"*64,1100)
        receipt.write_bytes(payload);os.chmod(receipt,0o600)
        old_key,old_allow=signed.create_same_admin_key(root,"prior_local")
        new_key,new_allow=signed.create_same_admin_key(root,"next_local")
        old_pub=root/"prior_local.pub";new_pub=root/"next_local.pub"
        old_sha=public_sha(old_pub);new_sha=public_sha(new_pub)
        first=signed.sign_local(receipt,old_key)
        init=bootstrap(root,old_sha)
        if init["height"]!=0 or init["active"]!=old_sha:raise Refused("GENESIS_NOT_COMMITTED")
        old_snapshot=root/"old_key_state.sqlite3"
        shutil.copy2(ledger_path(root),old_snapshot)
        rotation=root/"rotation.json"
        rotation.write_bytes(intent(ROTATE,1,old_sha,new_sha,sha(payload),"1"*64))
        old_rot=sign_intent(rotation,old_key)
        next_rot=root/"rotation_other.json";next_rot.write_bytes(rotation.read_bytes())
        next_sig=sign_intent(next_rot,new_key)
        wrong=append(root,rotation.read_bytes(),old_rot,old_allow)
        if "TWO_GENUINE_SIGNATURES_REQUIRED" not in wrong.get("detail",""):
            raise Refused("ONE_SIGNATURE_ROTATION_NOT_QUARANTINED")
        good=append(root,rotation.read_bytes(),old_rot,old_allow,next_sig,new_allow)
        if good.get("reason")!="REAL_LOCAL_DUAL_SSHSIG_ROTATION_DENIED":
            raise Refused("REAL_DUAL_SIGNATURE_ROTATION_FAILED:"+repr(good))
        old_now=active_key_check(root,old_pub,receipt,first,old_allow)
        if "LOCAL_SIGNER_NOT_ACTIVE" not in old_now.get("detail",""):
            raise Refused("OLD_KEY_NOT_LOCAL_QUARANTINED")
        new_receipt=root/"new_receipt.json";new_receipt.write_bytes(payload)
        new_sig=signed.sign_local(new_receipt,new_key)
        current=active_key_check(root,new_pub,new_receipt,new_sig,new_allow)
        if current["reason"]!="ACTIVE_LOCAL_KEY_VERIFIED_NOT_INDEPENDENT":
            raise Refused("NEW_LOCAL_SIGNER_NOT_VERIFIABLE")
        preremove=root/"pre_revocation.sqlite3"
        shutil.copy2(ledger_path(root),preremove)
        rev=root/"revoke.json"
        rev.write_bytes(intent(REVOKE,2,new_sha,new_sha,sha(payload),"2"*64))
        rev_sig=sign_intent(rev,new_key)
        revoke=append(root,rev.read_bytes(),rev_sig,new_allow)
        if revoke.get("reason")!="REAL_LOCAL_SIGNED_REVOCATION_DENIED" or not revoke["locally_revoked"]:
            raise Refused("LOCAL_SIGNED_REVOCATION_FAILED")
        revoked=active_key_check(root,new_pub,new_receipt,new_sig,new_allow)
        if "LOCAL_SIGNER_REVOKED" not in revoked.get("detail",""):
            raise Refused("REVOKED_SUCCESSOR_NOT_QUARANTINED")
        # Real separate OS process atomically restores the previous local
        # SQLite snapshot; signed revocation is LOST LOCALLY.
        fdgate.real_atomic_replace(preremove,ledger_path(root))
        after_rev_rollback=active_key_check(root,new_pub,new_receipt,new_sig,new_allow)
        if after_rev_rollback["reason"]!="ACTIVE_LOCAL_KEY_VERIFIED_NOT_INDEPENDENT":
            raise Refused("REVOCATION_ROLLBACK_LIMIT_NOT_REPRODUCED")
        fdgate.real_atomic_replace(old_snapshot,ledger_path(root))
        after_full_rollback=active_key_check(root,old_pub,receipt,first,old_allow)
        if after_full_rollback["reason"]!="ACTIVE_LOCAL_KEY_VERIFIED_NOT_INDEPENDENT":
            raise Refused("FULL_KEY_CHAIN_ROLLBACK_LIMIT_NOT_REPRODUCED")
        repeat=append(root,rotation.read_bytes(),old_rot,old_allow,next_sig,new_allow)
        if repeat["reason"]!="REAL_LOCAL_DUAL_SSHSIG_ROTATION_DENIED":
            raise Refused("ROTATION_NONCE_ROLLBACK_REPLAY_NOT_REPRODUCED")
        return block("REAL_KEY_ROTATION_REVOCATION_CUSTODY_LIMITS_QUALIFIED",
                     genuine_distinct_old_new_ed25519_signatures="PASS_KEY_POSSESSION_ONLY",
                     one_signature_rotation="QUARANTINED",
                     rotated_out_old_key="LOCAL_QUARANTINED",
                     rotated_in_new_key="LOCAL_DENIAL_ONLY",
                     signed_successor_revocation="LOCAL_QUARANTINED",
                     revocation_after_real_sqlite_rollback="LOST_LOCALLY",
                     old_key_after_full_sqlite_rollback="LOCALLY_VALID_AGAIN_DENIAL_ONLY",
                     old_rotation_nonce_after_full_rollback="LOCAL_DENIAL_AGAIN",
                     independent_admin_domain="NOT_PROVEN",
                     authentic_external_guardian="NOT_ENROLLED",
                     native_admission="BLOCK")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    g=p.add_mutually_exclusive_group(required=True)
    g.add_argument("--exercise",action="store_true")
    g.add_argument("--audit",action="store_true")
    p.add_argument("--ledger-dir")
    args=p.parse_args(argv)
    try:
        if args.exercise:out=native_exercise()
        else:
            s=audit(args.ledger_dir)
            out=block("LOCAL_KEY_LEDGER_AUDITED_NOT_EXTERNAL",
                      local_height=s["height"],locally_revoked=s["revoked"])
    except (Refused,OSError,sqlite3.Error,ValueError,TypeError,subprocess.SubprocessError) as exc:
        out=block("REAL_KEY_CUSTODY_EXERCISE_REFUSED",
                  detail=str(exc) if isinstance(exc,Refused) else type(exc).__name__)
    print(json.dumps(out,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
