#!/usr/bin/env python3
"""NORDOI G0.3 R5T: real Ed25519 SSHSIG handoff receipts; no native admission.

A real local signature verifies KEY POSSESSION over a bound R5S manifest,
challenge and exact historical projection. The signer and allowed-signers
trust source are generated under the SAME administrator and therefore do
not prove independent custody, a trusted clock, or a globally monotonic
spent-nonce ledger. Every exit and receipt decision is BLOCK.
"""
import argparse
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

import g03_r5l_crash_restart_denial_journal as denial
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5q_coherent_local_rollback as history
import g03_r5s_challenge_bound_handoff as handoff

PROTOCOL="NORDOI-G03-R5T-SIGNED-LOCAL-RECEIPT-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="ebe695680e1992261f55a7d534fa9505c9133bc5"
OPERATION="SIGNED_LOCAL_RECEIPT_BLOCK_ONLY"
IDENTITY="same-admin-research-fixture"
NAMESPACE="nordoi-r5t"
MAX_RECEIPT_BYTES=3072
FIELDS=frozenset(("protocol","project","parent","operation","signer",
                  "namespace","challenge","archive_sha256",
                  "projection_sha256","manifest_sha256",
                  "local_primary_height","received_at","expires_at"))

class Refused(ValueError):pass

def sha(b):return hashlib.sha256(b).hexdigest()
def canonical(x):
    return (json.dumps(x,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")
def pairs_unique(rows):
    out={}
    for k,v in rows:
        if k in out:raise Refused("DUPLICATE_RECEIPT_JSON_KEY")
        out[k]=v
    return out
def nonfinite(_):raise Refused("NONFINITE_RECEIPT")
def issha(s):
    return type(s) is str and re.fullmatch(r"[0-9a-f]{64}",s) is not None
def block(reason,**extra):
    return denial.block(reason,native_authority_granted=False,
                        independent_custodian="NOT_ENROLLED",
                        owner_oob_trust_root="NOT_ENROLLED",
                        external_signed_guardian_receipt="NONE",
                        global_monotonic_spent_challenge="NOT_VERIFIED",
                        **extra)

def parse(raw):
    if type(raw) is not bytes or not 0<len(raw)<=MAX_RECEIPT_BYTES:
        raise Refused("RECEIPT_NOT_BYTES_OR_OVERSIZED")
    try:
        obj=json.loads(raw.decode("utf-8"),object_pairs_hook=pairs_unique,
                       parse_constant=nonfinite)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Refused("RECEIPT_NOT_CANONICAL_UTF8_JSON") from exc
    if type(obj) is not dict or obj.keys()!=FIELDS or canonical(obj)!=raw:
        raise Refused("RECEIPT_FIELDS_OR_CANONICAL_MISMATCH")
    if (obj["protocol"],obj["project"],obj["parent"],obj["operation"])!=(
        PROTOCOL,PROJECT,PARENT,OPERATION):
        raise Refused("RECEIPT_PARENT_OR_GRANT_OPERATION_FORBIDDEN")
    if obj["signer"]!=IDENTITY or obj["namespace"]!=NAMESPACE:
        raise Refused("RECEIPT_SIGNER_OR_NAMESPACE_MISMATCH")
    for k in ("challenge","archive_sha256","projection_sha256","manifest_sha256"):
        if not issha(obj[k]):raise Refused("RECEIPT_BAD_SHA256_"+k)
    for k in ("local_primary_height","received_at","expires_at"):
        if type(obj[k]) is not int or obj[k]<0:
            raise Refused("RECEIPT_COUNTER_OR_UNTRUSTED_CLOCK_INVALID_"+k)
    if obj["local_primary_height"]<1:
        raise Refused("RECEIPT_LOCAL_PRIMARY_HEIGHT_NOT_POSITIVE")
    if not obj["received_at"]<=obj["expires_at"]:
        raise Refused("RECEIPT_LOCAL_TIMEBOX_BACKWARDS")
    return obj

def prepare_receipt(bundle,challenge,now):
    """Source is the real R5S ZIP, but never a remote trust anchor."""
    raw=Path(bundle).read_bytes()
    manifest,bundle_sha,state=handoff.decode_package(raw,challenge,now)
    return canonical({
        "protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
        "operation":OPERATION,"signer":IDENTITY,"namespace":NAMESPACE,
        "challenge":challenge,"archive_sha256":bundle_sha,
        "projection_sha256":manifest["projection_sha256"],
        "manifest_sha256":sha(handoff.canonical(manifest)),
        "local_primary_height":state["primary_height"],
        "received_at":now,"expires_at":manifest["expires_at"],
    })

def run_ssh(command,input_bytes=None):
    if os.name=="nt" or not Path("/usr/bin/ssh-keygen").is_file():
        raise Refused("REAL_POSIX_OPENSSH_SSHSIG_NOT_AVAILABLE")
    p=subprocess.run(["/usr/bin/ssh-keygen"]+command,input=input_bytes,
                     capture_output=True,timeout=35,check=False)
    return p.returncode

def create_same_admin_key(root,key_name="self_signer"):
    """Genuine local Ed25519 test key, not external operator enrollment."""
    root=Path(root)
    key=root/key_name
    if not root.is_dir() or not root.name.startswith("nordoi-r5l-r5q-r5t-") or \
       (os.name!="nt" and root.stat().st_mode&0o077) or \
       key.exists() or (root/(key_name+".pub")).exists():
        raise Refused("ONLY_NEW_DISPOSABLE_SAME_ADMIN_FIXTURE_KEYS")
    if run_ssh(["-q","-t","ed25519","-N","","-f",str(key)])!=0:
        raise Refused("REAL_ED25519_KEYGEN_FAILED")
    allowed=root/(key_name+".allowed_signers")
    allowed.write_text(IDENTITY+" "+(root/(key_name+".pub")).read_text("ascii"),
                       encoding="ascii")
    if os.name!="nt":os.chmod(allowed,0o600)
    return key,allowed

def sign_local(receipt_path,key,namespace=NAMESPACE):
    if namespace not in (NAMESPACE,"wrong-r5t-namespace"):
        raise Refused("INVALID_FIXTURE_NAMESPACE")
    sig=Path(str(receipt_path)+".sig")
    if sig.exists():raise Refused("SIGNATURE_ALREADY_PRESENT")
    if run_ssh(["-Y","sign","-f",str(key),"-n",namespace,
                str(receipt_path)])!=0 or not sig.is_file():
        raise Refused("REAL_ED25519_SSHSIG_SIGN_FAILED")
    if os.name!="nt":os.chmod(sig,0o600)
    return sig

def check_crypto(raw,signature,allowed_signers):
    """Cryptographic verification with local allowlist; provenance NOT verified."""
    if run_ssh(["-Y","verify","-f",str(allowed_signers),"-I",IDENTITY,
                "-n",NAMESPACE,"-s",str(signature)],raw)!=0:
        raise Refused("REAL_SSHSIG_SIGNATURE_INVALID_OR_UNTRUSTED_LOCAL_KEY")
    return True

def inspect(receipt_path,signature_path,allowed_signers,bundle,challenge,now,
            ledger_dir=None,after_first_observation=None):
    try:
        with fdgate.OpenSnapshot(Path(receipt_path)) as held:
            raw=Path(receipt_path).read_bytes()
            if sha(raw)!=held.digest:raise Refused("RECEIPT_BYTES_CHANGED_DURING_READ")
            doc=parse(raw)
            if after_first_observation is not None:
                if not callable(after_first_observation):
                    raise Refused("INVALID_NATIVE_INTERLEAVE")
                after_first_observation()
            held.check()
        if now is None or type(now) is not int:
            raise Refused("NO_TRUSTED_RECEIPT_CLOCK")
        if not doc["received_at"]<=now<=doc["expires_at"]:
            raise Refused("RECEIPT_LOCAL_TIMEBOX_EXPIRED_OR_IN_FUTURE")
        bundle_bytes=Path(bundle).read_bytes()
        manifest,bundle_sha,state=handoff.decode_package(bundle_bytes,challenge,now)
        if (doc["challenge"]!=challenge or
            doc["archive_sha256"]!=bundle_sha or
            doc["manifest_sha256"]!=sha(handoff.canonical(manifest)) or
            doc["projection_sha256"]!=manifest["projection_sha256"] or
            doc["local_primary_height"]!=state["primary_height"] or
            doc["expires_at"]!=manifest["expires_at"]):
            raise Refused("RECEIPT_SIGNED_CONTEXT_NOT_BOUND_TO_TRANSFER")
        with fdgate.OpenSnapshot(Path(signature_path)) as sig_held:
            check_crypto(raw,signature_path,allowed_signers)
            sig_held.check()
        if ledger_dir is not None:
            ledger=denial.private_journal_dir(ledger_dir)
            if not ledger.is_file():
                raise Refused("RECEIPT_LOCAL_SPENT_LEDGER_MISSING")
            record=denial.execute(ledger_dir,"record",doc["challenge"],
                                  sha(raw),"candidate_a")
            if record.get("reason")!="LOCAL_DENIAL_DURABLY_COMMITTED_NOT_ADMISSION":
                raise Refused("SIGNED_RECEIPT_NONCE_ALREADY_SPENT_OR_REFUSED:"+
                              str(record.get("detail",record.get("reason"))))
            height=record["local_height"]
        else:height=None
        return block("REAL_LOCAL_ED25519_SIGNATURE_VALID_NO_EXTERNAL_AUTHORITY",
                     cryptographic_signature_verified=True,
                     independently_authenticated_signer=False,
                     accepted_transfer_sha256=bundle_sha,
                     accepted_history_sha256=doc["projection_sha256"],
                     local_denial_height=height,
                     externally_protected_nonce=False)
    except (Refused,handoff.Refused,history.Refused,fdgate.Refused,denial.Refused,
            OSError,ValueError,TypeError,sqlite3.Error,subprocess.SubprocessError) as exc:
        return block("SIGNED_RECEIPT_QUARANTINED",
                     detail=str(exc) if isinstance(exc,(
                         Refused,handoff.Refused,history.Refused,fdgate.Refused,
                         denial.Refused)) else type(exc).__name__,
                     independently_authenticated_signer=False)

def second_process_check(receipt,sig,allowed,bundle,challenge,now,ledger=None):
    cmd=[sys.executable,"-B",str(Path(__file__).resolve()),"--inspect",
         "--receipt",str(receipt),"--signature",str(sig),
         "--allowed-signers",str(allowed),"--bundle",str(bundle),
         "--challenge",str(challenge),"--now",str(now)]
    if ledger is not None:cmd+=["--ledger-dir",str(ledger)]
    p=subprocess.run(cmd,capture_output=True,text=True,timeout=55)
    if p.returncode!=2:raise Refused("RECEIVER_PROCESS_DID_NOT_DENY")
    return json.loads(p.stdout)

def native_exercise():
    if os.name=="nt":return block("POSIX_REAL_RECEIPT_CRYPTO_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5t-native-") as tmp:
        root=Path(tmp);os.chmod(root,0o700)
        a,b,j=history.fixture(root)
        package=root/"handoff.zip"
        handoff.prepare(a,b,j,"a"*64,1000,1200,package)
        receipt=root/"local_receipt.json"
        payload=prepare_receipt(package,"a"*64,1100)
        receipt.write_bytes(payload);os.chmod(receipt,0o600)
        key,allowed=create_same_admin_key(root)
        sig=sign_local(receipt,key)
        local=second_process_check(receipt,sig,allowed,package,"a"*64,1100)
        if local.get("reason")!="REAL_LOCAL_ED25519_SIGNATURE_VALID_NO_EXTERNAL_AUTHORITY":
            raise Refused("REAL_SECOND_PROCESS_CRYPTO_VERIFY_FAILED")
        wrong=inspect(receipt,sig,allowed,package,"b"*64,1100)
        if wrong.get("reason")!="SIGNED_RECEIPT_QUARANTINED":
            raise Refused("RECEIPT_CHALLENGE_MISMATCH_NOT_REFUSED")
        changed=receipt.read_bytes()
        receipt.write_bytes(changed+b"tampered")
        if inspect(receipt,sig,allowed,package,"a"*64,1100)["reason"]!="SIGNED_RECEIPT_QUARANTINED":
            raise Refused("TAMPERED_RECEIPT_NOT_REFUSED")
        receipt.write_bytes(changed)
        # New local denial register is still a same-admin file.
        nonce_dir=root/"spent_receipts"
        nonce_dir.mkdir(mode=0o700)
        with denial.DenialJournal(nonce_dir) as d:
            if d.audit()["local_height"]!=0:raise Refused("NONCE_LEDGER_NOT_GENESIS")
        saved=root/"old_spent_receipts.sqlite3"
        shutil.copy2(nonce_dir/denial.NAME,saved)
        first=second_process_check(receipt,sig,allowed,package,"a"*64,1100,nonce_dir)
        if first.get("local_denial_height")!=1:raise Refused("FIRST_SIGNED_LOCAL_NONCE_NOT_DENIED")
        replay=second_process_check(receipt,sig,allowed,package,"a"*64,1100,nonce_dir)
        if "ALREADY_RECORDED_DENIAL_NONCE" not in replay.get("detail",""):
            raise Refused("SIGNED_RECEIPT_REPLAY_NOT_REFUSED")
        # Actual changed history no longer authenticated by old signed
        # offline bundle; the bundle itself can still pass local verification.
        history.advance_world(a,b,j)
        old=second_process_check(receipt,sig,allowed,package,"a"*64,1100)
        if old.get("reason")!="REAL_LOCAL_ED25519_SIGNATURE_VALID_NO_EXTERNAL_AUTHORITY":
            raise Refused("OLD_SIGNED_BUNDLE_LOCAL_VERIFY_FAILED")
        if history.projection(a,b,j)["primary_height"]!=2 or \
           handoff.decode_package(package.read_bytes(),"a"*64,1100)[2]["primary_height"]!=1:
            raise Refused("REAL_A_B_HISTORY_COMPARISON_FAILED")
        fdgate.real_atomic_replace(saved,nonce_dir/denial.NAME)
        rolled=second_process_check(receipt,sig,allowed,package,"a"*64,1100,nonce_dir)
        if rolled.get("local_denial_height")!=1:
            raise Refused("SIGNED_CHALLENGE_AFTER_ROLLBACK_NOT_ONLY_LOCAL_DENIAL")
        # Signature verified with another genuine locally generated key and
        # a separately authored local signer list; cannot prove independence.
        other_key,other_allowed=create_same_admin_key(root,"replacement_signer")
        other_receipt=root/"rotated_local_receipt.json"
        other_receipt.write_bytes(payload);os.chmod(other_receipt,0o600)
        other_sig=sign_local(other_receipt,other_key)
        alt=second_process_check(other_receipt,other_sig,other_allowed,package,"a"*64,1100)
        if alt["reason"]!="REAL_LOCAL_ED25519_SIGNATURE_VALID_NO_EXTERNAL_AUTHORITY":
            raise Refused("REAL_SAME_ADMIN_REKEY_PROVENANCE_TEST_FAILED")
        # Attempt an actual byte-identical atomic rename during opened FD.
        duplicate=root/"identical_receipt.json"
        shutil.copy2(receipt,duplicate);os.chmod(duplicate,0o600)
        swap=inspect(receipt,sig,allowed,package,"a"*64,1100,
                     after_first_observation=lambda:
                         fdgate.real_atomic_replace(duplicate,receipt))
        if "OPEN_FD_AND_CURRENT_PATH_DIVERGED" not in swap.get("detail",""):
            raise Refused("REAL_SIGNED_RECEIPT_ATOMIC_REPLACE_NOT_QUARANTINED")
        return block("REAL_SIGNED_RECEIPT_LOCAL_PROVENANCE_LIMITS_QUALIFIED",
                     real_open_ssh_ed25519_sign_and_verify="PASS_LOCAL_KEY_POSSESSION_ONLY",
                     tampered_receipt="QUARANTINED",
                     mismatched_challenge="QUARANTINED",
                     spent_signed_receipt_replay_with_intact_ledger="QUARANTINED",
                     spent_signed_receipt_after_same_admin_ledger_restore="LOCAL_DENIAL_AGAIN",
                     signed_old_bundle_after_new_history="LOCALLY_VERIFIABLE_NOT_MONOTONIC",
                     second_real_local_signing_key="VALID_NOT_INDEPENDENT",
                     real_receipt_inode_swap="QUARANTINED",
                     real_external_signed_guardian_receipt="NONE",
                     native_admission="BLOCK")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    group=p.add_mutually_exclusive_group(required=True)
    group.add_argument("--exercise",action="store_true")
    group.add_argument("--inspect",action="store_true")
    for arg in ("receipt","signature","allowed-signers","bundle","challenge","ledger-dir"):
        p.add_argument("--"+arg)
    p.add_argument("--now",type=int)
    args=p.parse_args(argv)
    try:
        if args.exercise:result=native_exercise()
        else:result=inspect(args.receipt,args.signature,args.allowed_signers,
                            args.bundle,args.challenge,args.now,args.ledger_dir)
    except (Refused,denial.Refused,fdgate.Refused,handoff.Refused,history.Refused,
            OSError,ValueError,TypeError,sqlite3.Error,subprocess.SubprocessError) as exc:
        result=block("REAL_SIGNED_RECEIPT_LAB_REFUSED",
                     detail=str(exc) if isinstance(exc,(
                         Refused,denial.Refused,fdgate.Refused,handoff.Refused,
                         history.Refused)) else type(exc).__name__)
    print(json.dumps(result,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
