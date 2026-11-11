#!/usr/bin/env python3
"""R5H: signed local revocation and reversible history, denial-only.

A real OpenSSH Ed25519 SSHSIG verifies test-key possession, NOT independent
custody, owner OOB enrollment or an externally protected revocation authority.
No production, no native admission, no real guardian integration.
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
import stat
import subprocess
import sys
import tempfile

PROTOCOL = "NORDOI-G03-R5H-SIGNED-REVOCATION-LAB-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "c4356e24aeb1638dcd110f10d3439f1d471b3667"
NAMESPACE = "nordoi-r5h-revocation-fixture@eddarsglobal"
PRINCIPAL = "nordoi-r5h-lab-guardian"
SSH = "/usr/bin/ssh-keygen"
DB_NAME = "r5h_local_revocations.sqlite3"
MAX_BYTES = 10000
GENESIS = hashlib.sha256((PROTOCOL+":GENESIS").encode("ascii")).hexdigest()
HEX = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
PAYLOAD_FIELDS = frozenset(("protocol", "project", "parent_head", "operation",
                            "sequence", "epoch", "nonce", "prev", "resource_sha256",
                            "guardian_pubkey_sha256", "issued_unix", "expires_unix"))
PACKET_FIELDS = frozenset(("payload", "guardian_pubkey", "signature_b64"))

class Denied(ValueError): pass

def is_sha(s):
    return type(s) is str and bool(HEX.fullmatch(s)) and s != "0"*64

def canonical(obj):
    return (json.dumps(obj, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    d={}
    for k,v in pairs:
        if k in d: raise Denied("DUPLICATE_KEY")
        d[k]=v
    return d

def nonfinite(_): raise Denied("NONFINITE")

def decode(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_BYTES:
        raise Denied("MISSING_OR_OVERSIZED")
    try:
        packet=json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                          parse_constant=nonfinite)
    except (UnicodeError, json.JSONDecodeError) as e:
        raise Denied("MALFORMED_UTF8_JSON") from e
    if type(packet) is not dict or packet.keys() != PACKET_FIELDS or canonical(packet)!=raw:
        raise Denied("PACKET_NONCANONICAL_OR_FIELDS")
    payload=packet["payload"]
    if type(payload) is not dict or payload.keys()!=PAYLOAD_FIELDS:
        raise Denied("PAYLOAD_FIELDS")
    if payload["protocol"]!=PROTOCOL or payload["project"]!=PROJECT or payload["parent_head"]!=PARENT:
        raise Denied("WRONG_PROTOCOL_OR_PARENT")
    if payload["operation"]!="REVOKE":
        raise Denied("NEVER_AUTHORIZE_OR_UNREVOKE")
    if type(payload["sequence"]) is not int or payload["sequence"]<1:
        raise Denied("SEQUENCE_INVALID")
    if type(payload["epoch"]) is not int or payload["epoch"]<4:
        raise Denied("EPOCH_INVALID")
    for name in ("nonce","prev","resource_sha256","guardian_pubkey_sha256"):
        if not is_sha(payload[name]): raise Denied("DIGEST_INVALID_"+name)
    for name in ("issued_unix","expires_unix"):
        if type(payload[name]) is not int or payload[name]<1:
            raise Denied("TIME_INVALID")
    if not 1<=payload["expires_unix"]-payload["issued_unix"]<=300:
        raise Denied("TTL_TOO_LONG_OR_INVALID")
    key=packet["guardian_pubkey"]
    if type(key) is not str or not key.startswith("ssh-ed25519 ") or len(key)>180 or "\n" in key:
        raise Denied("PUBLIC_KEY_NOT_ED25519")
    components=key.split(" ")
    if len(components)!=2: raise Denied("PUBLIC_KEY_NONCANONICAL")
    try: wire=base64.b64decode(components[1],validate=True)
    except (binascii.Error,ValueError) as e: raise Denied("PUBLIC_KEY_INVALID_BASE64") from e
    if (len(wire)!=51 or wire[:19]!=b"\x00\x00\x00\x0bssh-ed25519\x00\x00\x00\x20" or
        base64.b64encode(wire).decode("ascii")!=components[1]):
        raise Denied("PUBLIC_KEY_NOT_ED25519_WIRE")
    if hashlib.sha256(key.encode("ascii")).hexdigest()!=payload["guardian_pubkey_sha256"]:
        raise Denied("KEY_PIN_MISMATCH")
    sig_b64=packet["signature_b64"]
    if type(sig_b64) is not str or not 20<len(sig_b64)<6000:
        raise Denied("SIGNATURE_MISSING_OR_OVERSIZED")
    try: signature=base64.b64decode(sig_b64,validate=True)
    except (binascii.Error,ValueError) as e: raise Denied("SIGNATURE_INVALID_B64") from e
    if not signature.startswith(b"-----BEGIN SSH SIGNATURE-----\n") or not signature.rstrip().endswith(b"-----END SSH SIGNATURE-----"):
        raise Denied("SIGNATURE_WRONG_FORMAT")
    if base64.b64encode(signature).decode("ascii")!=sig_b64:
        raise Denied("SIGNATURE_NONCANONICAL_BASE64")
    return packet, canonical(payload), signature

def verify_native(payload, signature, public_key):
    if not Path(SSH).is_file() or os.name=="nt": return False
    try:
        with tempfile.TemporaryDirectory(prefix="nordoi-r5h-verify-") as temp:
            root=Path(temp)
            allow=root/"allowed_signers"
            sig=root/"signature"
            allow.write_text(PRINCIPAL+" "+public_key+"\n",encoding="ascii")
            sig.write_bytes(signature)
            p=subprocess.run([SSH,"-Y","verify","-f",str(allow),"-I",PRINCIPAL,
                              "-n",NAMESPACE,"-s",str(sig)],input=payload,
                             capture_output=True,timeout=25,check=False)
            return p.returncode==0
    except (OSError, subprocess.SubprocessError): return False

def evaluate(raw, expected_fixture_key, now):
    try:
        packet, message, sig=decode(raw)
        payload=packet["payload"]
        if type(now) is not int or not payload["issued_unix"]<=now<=payload["expires_unix"]:
            raise Denied("EXPIRED_OR_FUTURE")
        if packet["guardian_pubkey"]!=expected_fixture_key:
            raise Denied("UNTRUSTED_SIGNER_KEY_SUBSTITUTION")
        if not verify_native(message,sig,packet["guardian_pubkey"]):
            raise Denied("SSHSIG_NOT_VERIFIED")
        return {"status":"BLOCK", "admission":"NOT_AUTHORIZED",
                "reason":"SIGNED_REVOCATION_KEY_POSSESSION_ONLY",
                "signature_verified":True,"guardian_identity":"NOT_AUTHENTICATED",
                "owner_oob_root":"NOT_ENROLLED", "external_revocation_authority":"NOT_ENROLLED"}
    except (Denied,TypeError,ValueError,OSError,OverflowError) as exc:
        return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"INVALID_OR_UNVERIFIED_REVOCATION",
                "detail":str(exc) if isinstance(exc,Denied) else type(exc).__name__}

def _safe_db_dir(folder):
    p=Path(folder)
    if not p.is_absolute() or p.is_symlink() or not p.is_dir(): raise Denied("INVALID_LOCAL_LAB_DIR")
    if os.name!="nt" and p.stat().st_mode & 0o077: raise Denied("LOCAL_LAB_DIR_NOT_PRIVATE")
    db=p/DB_NAME
    if db.is_symlink() or (db.exists() and not db.is_file()): raise Denied("NONREGULAR_SQLITE")
    if os.name!="nt" and db.exists() and db.stat().st_mode & 0o077: raise Denied("SQLITE_NOT_PRIVATE")
    return db

class RevocationLedger:
    """Same-admin SQLite: no global rollback guarantees. Persist REVOKE only."""
    def __init__(self,folder):
        self.path=_safe_db_dir(folder)
        self.db=sqlite3.connect(str(self.path),isolation_level=None,timeout=10.0)
        try:
            self.db.execute("PRAGMA busy_timeout=10000")
            self.db.execute("PRAGMA synchronous=FULL")
            self.db.execute("PRAGMA journal_mode=DELETE")
            self.db.execute("BEGIN IMMEDIATE")
            self.db.execute("CREATE TABLE IF NOT EXISTS state (id INTEGER PRIMARY KEY CHECK(id=1),height INTEGER NOT NULL,head TEXT NOT NULL,epoch INTEGER NOT NULL,revoked INTEGER NOT NULL)")
            self.db.execute("CREATE TABLE IF NOT EXISTS events (seq INTEGER PRIMARY KEY,epoch INTEGER NOT NULL,nonce TEXT NOT NULL UNIQUE,packet_sha TEXT NOT NULL,prev TEXT NOT NULL,head TEXT NOT NULL)")
            self.db.execute("INSERT OR IGNORE INTO state VALUES(1,0,?,4,0)",(GENESIS,))
            self._verify()
            self.db.execute("COMMIT")
            os.chmod(self.path,0o600)
        except BaseException:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            self.db.close()
            raise
    def __enter__(self):return self
    def __exit__(self,*_):self.db.close()
    def _verify(self):
        if self.db.execute("PRAGMA quick_check").fetchone() != ("ok",):raise Denied("SQLITE_CORRUPTION")
        meta=self.db.execute("SELECT height,head,epoch,revoked FROM state WHERE id=1").fetchone()
        if meta is None or self.db.execute("SELECT count(*) FROM state").fetchone()[0]!=1:
            raise Denied("STATE_MISSING")
        h,head,epoch,revoked=0,GENESIS,4,0
        for seq,e,nonce,packet_sha,prev,this_head in self.db.execute("SELECT seq,epoch,nonce,packet_sha,prev,head FROM events ORDER BY seq"):
            if (seq!=h+1 or e<epoch or not is_sha(nonce) or not is_sha(packet_sha) or prev!=head or
                this_head!=hashlib.sha256((prev+"\n"+str(seq)+"\n"+str(e)+"\n"+nonce+"\n"+packet_sha).encode("ascii")).hexdigest()):
                raise Denied("REVOCATION_CHAIN_INVALID")
            h,head,epoch,revoked=seq,this_head,e,1
        if meta!=(h,head,epoch,revoked):raise Denied("REVOCATION_METADATA_MISMATCH")
        return meta
    def audit(self):
        self.db.execute("BEGIN IMMEDIATE")
        try:
            state=self._verify()
            self.db.execute("COMMIT")
            return {"status":"BLOCK","admission":"NOT_AUTHORIZED","reason":"LOCAL_REVOCATION_NOT_EXTERNAL",
                    "local_height":state[0],"local_revoked":bool(state[3]),
                    "global_antirollback":"NOT_IMPLEMENTED"}
        except BaseException:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            raise
    def revoke(self,raw,key,now):
        r=evaluate(raw,key,now)
        if not r.get("signature_verified"):
            return r
        payload=decode(raw)[0]["payload"]
        self.db.execute("BEGIN IMMEDIATE")
        try:
            h,head,e,revoked=self._verify()
            if revoked:raise Denied("ALREADY_REVOKED_LOCAL_HISTORY")
            if payload["sequence"]!=h+1 or payload["prev"]!=head or payload["epoch"]<e:
                raise Denied("SEQUENCE_OR_PREDECESSOR_ROLLBACK")
            if self.db.execute("SELECT 1 FROM events WHERE nonce=?",(payload["nonce"],)).fetchone():
                raise Denied("NONCE_ALREADY_SPENT")
            packet_sha=hashlib.sha256(raw).hexdigest()
            new=hashlib.sha256((head+"\n"+str(h+1)+"\n"+str(payload["epoch"])+"\n"+payload["nonce"]+"\n"+packet_sha).encode("ascii")).hexdigest()
            self.db.execute("INSERT INTO events VALUES(?,?,?,?,?,?)",(h+1,payload["epoch"],payload["nonce"],packet_sha,head,new))
            self.db.execute("UPDATE state SET height=?,head=?,epoch=?,revoked=1 WHERE id=1",(h+1,new,payload["epoch"]))
            self._verify()
            self.db.execute("COMMIT")
            return {"status":"LAB_LOCAL_REVOKE_COMMIT","admission":"NOT_AUTHORIZED",
                    "reason":"SIGNED_REVOCATION_SAME_ADMIN_LOCAL_ONLY",
                    "signature_verified":True,"local_revoked":True,"local_height":h+1,
                    "external_revocation_authority":"NOT_ENROLLED","global_antirollback":"NOT_IMPLEMENTED"}
        except BaseException as exc:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            if isinstance(exc,Denied):
                return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                        "reason":"REVOCATION_LOCAL_REPLAY_OR_FORK_REFUSED","detail":str(exc)}
            raise

def keygen(folder):
    if not Path(SSH).is_file():raise Denied("NATIVE_SSH_KEYGEN_UNAVAILABLE")
    private=folder/"fixture_guardian"
    result=subprocess.run([SSH,"-q","-t","ed25519","-N","","-f",str(private)],
                          capture_output=True,timeout=30,check=False)
    if result.returncode!=0:raise Denied("TEST_KEYGEN_FAILED")
    pub_file=Path(str(private)+".pub")
    components=pub_file.read_text("ascii").split()
    return private, " ".join(components[:2])

def sign(private,payload,folder):
    msg=folder/"payload.txt"
    msg.write_bytes(canonical(payload))
    result=subprocess.run([SSH,"-Y","sign","-f",str(private),"-n",NAMESPACE,str(msg)],
                          capture_output=True,timeout=30,check=False)
    if result.returncode!=0:raise Denied("FIXTURE_SIGN_FAILED")
    signature=Path(str(msg)+".sig").read_bytes()
    return base64.b64encode(signature).decode("ascii")

def make_packet(private,pub,folder,nonce="a"*64,epoch=4,issued=1800000000,prev=GENESIS):
    payload={"protocol":PROTOCOL,"project":PROJECT,"parent_head":PARENT,
             "operation":"REVOKE","sequence":1,"epoch":epoch,"nonce":nonce,
             "prev":prev,"resource_sha256":"b"*64,
             "guardian_pubkey_sha256":hashlib.sha256(pub.encode("ascii")).hexdigest(),
             "issued_unix":issued,"expires_unix":issued+120}
    return canonical({"payload":payload,"guardian_pubkey":pub,
                      "signature_b64":sign(private,payload,folder)})

def exercise():
    if os.name=="nt" or not Path(SSH).is_file():
        return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"OPENSSH_NATIVE_REVOCATION_UNAVAILABLE",
                "independent_guardian":"NOT_ENROLLED"}
    with tempfile.TemporaryDirectory(prefix="nordoi-r5h-revoke-") as directory:
        root=Path(directory)
        os.chmod(root,0o700)
        private,pub=keygen(root)
        dbroot=root/"revocations"
        dbroot.mkdir(mode=0o700)
        os.chmod(dbroot,0o700)
        with RevocationLedger(dbroot) as ledger:
            initial=ledger.audit()
        old=root/"old.sqlite3"
        shutil.copy2(dbroot/DB_NAME,old)
        packet=make_packet(private,pub,root)
        with RevocationLedger(dbroot) as ledger:
            committed=ledger.revoke(packet,pub,1800000060)
            duplicate=ledger.revoke(packet,pub,1800000060)
            after=ledger.audit()
        if (committed.get("status")!="LAB_LOCAL_REVOKE_COMMIT" or
                duplicate.get("status")!="BLOCK" or
                after.get("local_revoked") is not True):
            raise Denied("REAL_SIGNED_REVOKE_LAB_FAILED")
        shutil.copy2(old,dbroot/DB_NAME)
        with RevocationLedger(dbroot) as ledger:
            restored=ledger.audit()
        if restored.get("local_revoked") is not False or restored.get("local_height")!=0:
            raise Denied("REVOCATION_SNAPSHOT_ROLLBACK_EXPERIMENT_FAILED")
        return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"SIGNED_LOCAL_REVOCATION_LOST_AFTER_SAME_ADMIN_SNAPSHOT_RESTORE",
                "native_real_ed25519_sshsig":True,"verified_fixture_key_possession":True,
                "local_revoked_before_rollback":True,"local_revoked_after_rollback":False,
                "local_nonce_replay_before_rollback":"BLOCK",
                "independent_guardian":"NOT_ENROLLED","owner_authenticated_oob_root":"NOT_ENROLLED",
                "global_monotonic_antirollback":"NOT_IMPLEMENTED"}

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--exercise",action="store_true",required=True)
    args=p.parse_args(argv)
    try: result=exercise()
    except (Denied,OSError,ValueError,TypeError,sqlite3.Error,subprocess.SubprocessError) as exc:
        result={"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"REVOCATION_EXPERIMENT_REFUSED",
                "detail":str(exc) if isinstance(exc,Denied) else type(exc).__name__}
    print(json.dumps(result,sort_keys=True,separators=(",", ":")))
    return 2

if __name__=="__main__":sys.exit(main())
