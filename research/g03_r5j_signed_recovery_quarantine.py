#!/usr/bin/env python3
"""NORDOI G0.3 R5J: signed recovery *intent*, read-only checkpoint quarantine.

The two real R5H/R5I SQLite files and fixture SSHSIG key are controlled by one
lab administrator. A verified signature does NOT authorize recovery, un-revoke,
native admission, or independently authenticate an operator. Always BLOCK.
"""
import argparse
import base64
import binascii
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

import g03_r5h_signed_revocation_lab as rev
import g03_r5i_revocation_floor as floor

PROTOCOL="NORDOI-G03-R5J-SIGNED-RECOVERY-QUARANTINE-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="32cdb98628f35cf08a965e1a2efe37bdd64e4d7b"
NAMESPACE="nordoi-r5j-recovery-intent-fixture@eddarsglobal"
PRINCIPAL="nordoi-r5j-lab-intent"
SSH="/usr/bin/ssh-keygen"
FIELDS=frozenset(("protocol","project","parent_commit","operation","floor_height",
                  "floor_head","primary_head","floor_epoch","nonce","issued_unix",
                  "expires_unix","fixture_key_sha256"))
ENVELOPE=frozenset(("payload","fixture_public_key","signature_b64"))
HEX=re.compile(r"[0-9a-f]{64}\Z",re.ASCII)
MAX_BYTES=11000
GENESIS=floor.GENESIS

class Invalid(ValueError): pass

def sha(s):
    return type(s) is str and bool(HEX.fullmatch(s)) and s!="0"*64

def canon(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True,
                       allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    result={}
    for key,value in pairs:
        if key in result: raise Invalid("DUPLICATE_FIELD")
        result[key]=value
    return result

def finite(_): raise Invalid("NONFINITE_NUMBER")

def decode(raw):
    if type(raw) is not bytes or not 0<len(raw)<=MAX_BYTES:
        raise Invalid("MISSING_OR_OVERSIZED_JSON")
    try:
        value=json.loads(raw.decode("utf-8"),object_pairs_hook=unique,parse_constant=finite)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Invalid("NOT_CANONICAL_UTF8_JSON") from exc
    if type(value) is not dict or value.keys()!=ENVELOPE or canon(value)!=raw:
        raise Invalid("WRONG_ENVELOPE_OR_NONCANONICAL")
    pl=value["payload"]
    if type(pl) is not dict or pl.keys()!=FIELDS:
        raise Invalid("WRONG_RECOVERY_FIELDS")
    if pl["protocol"]!=PROTOCOL or pl["project"]!=PROJECT or pl["parent_commit"]!=PARENT:
        raise Invalid("PROTOCOL_SOURCE_BINDING_FAILED")
    if pl["operation"]!="RECOVERY_INTENT_DENY_ONLY":
        raise Invalid("GRANT_UNREVOKE_OR_RECOVERY_OPERATION_FORBIDDEN")
    if type(pl["floor_height"]) is not int or pl["floor_height"]<1:
        raise Invalid("PRE_REVOCATION_FLOOR_NOT_ACCEPTABLE")
    if type(pl["floor_epoch"]) is not int or pl["floor_epoch"]<4:
        raise Invalid("FLOOR_EPOCH_INVALID")
    for name in ("floor_head","primary_head","nonce","fixture_key_sha256"):
        if not sha(pl[name]):raise Invalid("HASH_BINDING_INVALID_"+name)
    for name in ("issued_unix","expires_unix"):
        if type(pl[name]) is not int or pl[name]<1:raise Invalid("TIMESTAMP_INVALID")
    if not 1<=pl["expires_unix"]-pl["issued_unix"]<=300:
        raise Invalid("RECOVERY_INTENT_TTL_INVALID")
    pub=value["fixture_public_key"]
    if type(pub) is not str or not pub.startswith("ssh-ed25519 ") or len(pub)>180:
        raise Invalid("NOT_AN_ED25519_FIXTURE_KEY")
    parts=pub.split(" ")
    if len(parts)!=2:raise Invalid("PUBLIC_KEY_NOT_CANONICAL")
    try: wire=base64.b64decode(parts[1],validate=True)
    except (binascii.Error,ValueError) as exc:
        raise Invalid("PUBLIC_KEY_BASE64_INVALID") from exc
    if (len(wire)!=51 or wire[:19]!=b"\x00\x00\x00\x0bssh-ed25519\x00\x00\x00\x20" or
            base64.b64encode(wire).decode("ascii")!=parts[1]):
        raise Invalid("ED25519_PUBLIC_KEY_WIRE_INVALID")
    if hashlib.sha256(pub.encode("ascii")).hexdigest()!=pl["fixture_key_sha256"]:
        raise Invalid("KEY_FINGERPRINT_CLAIM_MISMATCH")
    sig=value["signature_b64"]
    if type(sig) is not str or not 20<len(sig)<6000:
        raise Invalid("FIXTURE_SSHSIG_MISSING")
    try: buf=base64.b64decode(sig,validate=True)
    except (binascii.Error,ValueError) as exc:
        raise Invalid("SSHSIG_BASE64_INVALID") from exc
    if (not buf.startswith(b"-----BEGIN SSH SIGNATURE-----\n") or
            not buf.rstrip().endswith(b"-----END SSH SIGNATURE-----") or
            base64.b64encode(buf).decode("ascii")!=sig):
        raise Invalid("FIXTURE_SSHSIG_NONCANONICAL")
    return value,canon(pl),buf

def verify(msg,sig,public_key):
    if os.name=="nt" or not Path(SSH).is_file():return False
    try:
        with tempfile.TemporaryDirectory(prefix="nordoi-r5j-verify-") as d:
            directory=Path(d)
            (directory/"allowed_signers").write_text(PRINCIPAL+" "+public_key+"\n",encoding="ascii")
            (directory/"sig").write_bytes(sig)
            result=subprocess.run([SSH,"-Y","verify","-f",str(directory/"allowed_signers"),
                                   "-I",PRINCIPAL,"-n",NAMESPACE,"-s",str(directory/"sig")],
                                  input=msg,capture_output=True,check=False,timeout=25)
            return result.returncode==0
    except (OSError,subprocess.SubprocessError):return False

def _ro_state(path,query):
    uri="file:"+quote(str(path),safe="/")+ "?mode=ro"
    with sqlite3.connect(uri,uri=True,timeout=10.0) as con:
        return con.execute(query).fetchone()

def observed(primary,floor_dir):
    """Read genuine files; no creation from absent database, no grant."""
    a,b=floor.private_dirs(primary,floor_dir)
    p=a/rev.DB_NAME
    w=b
    if not p.is_file() or p.is_symlink() or not w.is_file() or w.is_symlink():
        raise Invalid("REAL_SQLITE_FILES_MISSING")
    check=floor.execute(a,w.parent,"audit")
    if check.get("reason")!="LOCAL_REVOCATION_FLOOR_MATCH_NOT_INDEPENDENT":
        raise Invalid("STALE_DIVERGENT_OR_UNVERIFIED_REVOCATION_HISTORY:"+str(check.get("reason")))
    prim=_ro_state(p,"SELECT height,head,epoch,revoked FROM state WHERE id=1")
    secondary=_ro_state(w,"SELECT height,epoch,primary_head,floor_head FROM state WHERE id=1")
    if prim is None or secondary is None:
        raise Invalid("STATE_INCOMPLETE")
    height,head,epoch,revoked=prim
    f_height,f_epoch,f_primary,f_head=secondary
    if (height<1 or revoked!=1 or height!=f_height or epoch!=f_epoch or
            head!=f_primary or not sha(head) or not sha(f_head)):
        raise Invalid("PRE_REVOCATION_OR_UNALIGNED_HISTORY")
    return {"floor_height":f_height,"floor_epoch":f_epoch,"floor_head":f_head,
            "primary_head":head}

def sign_fixture(private,payload):
    if not Path(SSH).is_file():raise Invalid("NATIVE_OPENSSH_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5j-sign-") as temp:
        filepath=Path(temp)/"intent.txt"
        filepath.write_bytes(canon(payload))
        p=subprocess.run([SSH,"-Y","sign","-f",str(private),"-n",NAMESPACE,str(filepath)],
                         capture_output=True,check=False,timeout=30)
        if p.returncode!=0:raise Invalid("NATIVE_FIXTURE_SIGN_FAILURE")
        return base64.b64encode(Path(str(filepath)+".sig").read_bytes()).decode("ascii")

def intent(private,fixture_public_key,state,nonce="a"*64,issued=1800000000):
    payload={"protocol":PROTOCOL,"project":PROJECT,"parent_commit":PARENT,
             "operation":"RECOVERY_INTENT_DENY_ONLY","floor_height":state["floor_height"],
             "floor_head":state["floor_head"],"primary_head":state["primary_head"],
             "floor_epoch":state["floor_epoch"],"nonce":nonce,
             "issued_unix":issued,"expires_unix":issued+120,
             "fixture_key_sha256":hashlib.sha256(fixture_public_key.encode("ascii")).hexdigest()}
    return canon({"payload":payload,"fixture_public_key":fixture_public_key,
                  "signature_b64":sign_fixture(private,payload)})

def assessment(raw,fixture_public_key,primary_dir,floor_dir,now):
    """Signed and apparently aligned LOCAL recovery intent still always denied."""
    try:
        packet,msg,sig=decode(raw)
        pl=packet["payload"]
        if type(now) is not int or not pl["issued_unix"]<=now<=pl["expires_unix"]:
            raise Invalid("STALE_OR_NOT_YET_VALID_INTENT")
        if packet["fixture_public_key"]!=fixture_public_key:
            raise Invalid("UNTRUSTED_SIGNER_SUBSTITUTION")
        if not verify(msg,sig,fixture_public_key):
            raise Invalid("NATIVE_SSHSIG_NOT_VERIFIED")
        state=observed(primary_dir,floor_dir)
        for key in ("floor_height","floor_head","primary_head","floor_epoch"):
            if pl[key]!=state[key]:
                raise Invalid("SIGNED_RECOVERY_INTENT_HISTORY_CHANGED_"+key.upper())
        return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"VALID_LOCAL_FIXTURE_RECOVERY_INTENT_NO_EXTERNAL_AUTHORITY",
                "fixture_sshsig_verified":True,"local_floor_height":state["floor_height"],
                "real_external_guardian":"NOT_ENROLLED","owner_oob_root":"NOT_ENROLLED",
                "protected_remote_recovery":"NOT_IMPLEMENTED"}
    except (Invalid,rev.Denied,floor.Refused,OSError,ValueError,sqlite3.Error,
            subprocess.SubprocessError,TypeError,OverflowError) as exc:
        return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"UNVERIFIED_OR_DIVERGENT_RECOVERY_INTENT_REFUSED",
                "detail":str(exc) if isinstance(exc,(Invalid,rev.Denied,floor.Refused)) else type(exc).__name__,
                "real_external_guardian":"NOT_ENROLLED",
                "protected_remote_recovery":"NOT_IMPLEMENTED"}

def exercise():
    if os.name=="nt" or not Path(SSH).is_file():
        return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"NATIVE_UNIX_OPENSSH_NOT_AVAILABLE",
                "real_external_guardian":"NOT_ENROLLED"}
    with tempfile.TemporaryDirectory(prefix="nordoi-r5j-native-") as td:
        root=Path(td)
        os.chmod(root,0o700)
        a,b=root/"primary",root/"floor"
        a.mkdir(mode=0o700);b.mkdir(mode=0o700)
        os.chmod(a,0o700);os.chmod(b,0o700)
        priv,pub=rev.keygen(root)
        with rev.RevocationLedger(a) as ledger:
            assert ledger.audit()["local_height"]==0
        result0=floor.execute(a,b,"audit")
        if result0.get("floor_height")!=0:raise Invalid("GENESIS_PRECHECK_FAILED")
        snap_a=root/"pre-revoke.sqlite3"
        snap_b=root/"pre-floor.sqlite3"
        shutil.copy2(a/rev.DB_NAME,snap_a)
        shutil.copy2(b/floor.NAME,snap_b)
        packet=rev.make_packet(priv,pub,root)
        with rev.RevocationLedger(a) as ledger:
            committed=ledger.revoke(packet,pub,1800000060)
        if committed.get("status")!="LAB_LOCAL_REVOKE_COMMIT":
            raise Invalid("ACTUAL_SIGNED_REVOCATION_FAILED")
        checkpoint=floor.execute(a,b,"checkpoint")
        if checkpoint.get("status")!="LAB_LOCAL_CHECKPOINT_COMMIT":
            raise Invalid("ACTUAL_REVOCATION_FLOOR_FAILED")
        state=observed(a,b)
        request=intent(priv,pub,state)
        good=assessment(request,pub,a,b,1800000060)
        if good.get("reason")!="VALID_LOCAL_FIXTURE_RECOVERY_INTENT_NO_EXTERNAL_AUTHORITY":
            raise Invalid("REAL_SIGNED_INTENT_NOT_VERIFIED")
        shutil.copy2(snap_a,a/rev.DB_NAME)
        stale=assessment(request,pub,a,b,1800000060)
        if "STALE_DIVERGENT" not in stale.get("detail",""):
            raise Invalid("PRESERVED_FLOOR_RECOVERY_QUARANTINE_FAILED")
        shutil.copy2(snap_b,b/floor.NAME)
        both=assessment(request,pub,a,b,1800000060)
        if "PRE_REVOCATION_OR_UNALIGNED" not in both.get("detail",""):
            raise Invalid("TWO_OLD_SNAPSHOTS_NOT_QUARANTINED")
        return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"SIGNED_LOCAL_RECOVERY_INTENT_NEVER_REENABLES_AUTHORITY",
                "real_fixture_ed25519_sshsig":True,
                "current_matching_local_floor":"STILL_BLOCKED",
                "preserved_floor_primary_rollback":"DETECTED_AND_BLOCKED",
                "both_old_snapshots_with_preserved_signed_intent":"BLOCKED_PRE_REVOCATION",
                "both_old_snapshots_without_external_anchor":"NOT_GLOBALLY_DETECTABLE",
                "real_independent_guardian":"NOT_ENROLLED",
                "owner_authentic_oob_root":"NOT_ENROLLED",
                "globally_protected_recovery_floor":"NOT_IMPLEMENTED"}

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--exercise",action="store_true",required=True)
    p.parse_args(argv)
    try:r=exercise()
    except (Invalid,rev.Denied,floor.Refused,OSError,ValueError,sqlite3.Error,
            subprocess.SubprocessError,TypeError) as exc:
        r={"status":"BLOCK","admission":"NOT_AUTHORIZED",
           "reason":"R5J_REAL_LAB_EXPERIMENT_REFUSED",
           "detail":str(exc) if isinstance(exc,(Invalid,rev.Denied,floor.Refused)) else type(exc).__name__}
    print(json.dumps(r,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
