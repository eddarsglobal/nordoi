#!/usr/bin/env python3
"""Recipient-signed acceptance evidence, not independent remote state attestation.
Private keys are not accepted. Preparation audits the live local record; verification
replays the portable evidence and authenticates the recipient's declaration.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app046_portable_admission_evidence"))
import evidence_app046 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
crypto=prior.prior.prior.crypto
SCHEMA="NORDOI_APP047_SIGNED_ADMISSION_EVIDENCE_V1"
NAMESPACE="nordoi-archive-admission-ack@eddarsglobal"
MAX_ACK=16384

def expected(package_pin,receipt_pin,receiver_key_pin,**trust):
    for pin in (package_pin,receipt_pin,receiver_key_pin): transfer.pin(pin)
    ctx=prior.prior.context(**trust)
    need(receiver_key_pin!=trust["key_pin"],"RECIPIENT_KEY_MUST_DIFFER")
    return {"schema":SCHEMA,"purpose":"RECIPIENT_ACCEPTANCE_ASSERTION_ONLY",
            "challenge":ctx,"challenge_sha256":transfer.sha(portable.serialize(ctx)),
            "package_sha256":package_pin,"receipt_sha256":receipt_pin,
            "receiver_key_sha256":receiver_key_pin,"execution_authorized":False,
            "production":"NOT_AUTHORIZED"}

def capture(package,pin):
    raw=registry.read(package,prior.MAX_PACKAGE)
    need(transfer.sha(raw)==pin,"PACKAGE_PIN_MISMATCH")
    return raw

def replay(raw,core,trust):
    with tempfile.TemporaryDirectory(prefix="nordoi_app047_package_") as tmp:
        path=Path(tmp)/"package.json"
        portable.safe_write(path,raw)
        result=prior.verify(path,core["package_sha256"],core["receipt_sha256"],**trust)
    need(result["status"]=="ADMISSION_EVIDENCE_VERIFIED" and
         result["record"]["challenge_sha256"]==core["challenge_sha256"],"ACK_EVIDENCE_MISMATCH")
    return result["record"]

def prepare(package,ack,challenges,archive,statement,signature,public_key,
            package_pin,receipt_pin,receiver_key_pin,**trust):
    core=expected(package_pin,receipt_pin,receiver_key_pin,**trust)
    prior.prior.native.real_directory(ack.parent,"ACK_PARENT_INVALID")
    dest=ack.parent.resolve(strict=True)/ack.name
    root=registry.root(challenges)
    need(root!=dest and root not in dest.parents and not registry.present(dest),"ACK_OUTPUT_INVALID")
    raw=capture(package,package_pin)
    rec=replay(raw,core,trust)
    args=dict(challenges=root,archive=archive,statement=statement,signature=signature,
              public_key=public_key,**trust)
    audited=prior.prior.audit(**args)
    need(audited["status"]=="ARCHIVE_ACCEPTANCE_VERIFIED" and audited["record"]==rec,
         "LOCAL_ACCEPTANCE_NOT_MATCHED")
    need(capture(package,package_pin)==raw and prior.prior.audit(**args)==audited,
         "SOURCE_ACCEPTANCE_CHANGED")
    msg=portable.serialize(core)
    portable.safe_write(dest,msg)
    need(registry.read(dest,MAX_ACK)==msg,"ACK_PUBLICATION_MISMATCH")
    return {"schema":SCHEMA,"status":"ACK_PREPARED_UNSIGNED","ack_sha256":transfer.sha(msg),
            "receipt_sha256":receipt_pin,"execution_authorized":False,"production":"NOT_AUTHORIZED"}

def verify(package,ack,ack_signature,receiver_public_key,package_pin,receipt_pin,receiver_key_pin,**trust):
    core=expected(package_pin,receipt_pin,receiver_key_pin,**trust)
    key=crypto.check_pubkey(receiver_public_key,receiver_key_pin)
    msg=registry.read(ack,MAX_ACK)
    need(msg==portable.serialize(core),"ACK_EXPECTATION_MISMATCH")
    sig=registry.read(ack_signature,crypto.MAX_SIGNATURE)
    need(sig.startswith(b"-----BEGIN SSH SIGNATURE-----\n") and
         sig.rstrip().endswith(b"-----END SSH SIGNATURE-----"),"ACK_SIGNATURE_FORMAT_INVALID")
    raw=capture(package,package_pin)
    binary=shutil.which("ssh-keygen")
    need(binary is not None,"SSHSIG_VERIFIER_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi_app047_signature_") as tmp:
        p=Path(tmp)
        allowed=p/"allowed_signers"
        captured=p/"ack.sig"
        portable.safe_write(allowed,(trust["receiver"]+" "+key).encode("ascii"))
        portable.safe_write(captured,sig)
        result=subprocess.run([binary,"-Y","verify","-f",str(allowed),"-I",trust["receiver"],
                               "-n",NAMESPACE,"-s",str(captured)],input=msg,capture_output=True,timeout=20)
        need(result.returncode==0,"RECIPIENT_SIGNATURE_VERIFICATION_FAILED")
    rec=replay(raw,core,trust)
    need(capture(package,package_pin)==raw and registry.read(ack,MAX_ACK)==msg
         and registry.read(ack_signature,crypto.MAX_SIGNATURE)==sig
         and crypto.check_pubkey(receiver_public_key,receiver_key_pin)==key,"ACK_INPUT_CHANGED")
    return {"schema":SCHEMA,"status":"RECIPIENT_ACCEPTANCE_SIGNATURE_VERIFIED",
            "ack_sha256":transfer.sha(msg),"ack_signature_sha256":transfer.sha(sig),
            "package_sha256":package_pin,"receipt_sha256":receipt_pin,
            "receiver_key_sha256":receiver_key_pin,"record":rec,
            "crypto_signature":"SSHSIG_ED25519_VERIFIED","recipient_assertion":True,
            "independent_historical_acceptance":"NOT_PROVEN","live_nonce_state":"NOT_VERIFIED",
            "external_identity_enrollment":"NOT_PROVEN","independent_key_custody":"NOT_PROVEN",
            "freshness":"NOT_PROVEN","ack_replay_prevention":"NOT_IMPLEMENTED",
            "global_antirollback":"NOT_PROVEN","execution_authorized":False,
            "reservation_release_authorized":False,"nonce_reset_authorized":False,
            "automatic_retry":False,"production":"NOT_AUTHORIZED"}

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("prepare","verify"):
        a=sub.add_parser(action)
        for n in ("package","ack"): a.add_argument("--"+n,type=Path,required=True)
        for n in ("package-pin","receipt-pin","receiver-key-pin","sender","receiver","nonce","key-pin",
                  "archive-pin","source-plan-pin","anchor-pin","head-pin"): a.add_argument("--"+n,required=True)
        paths=("challenges","archive","statement","signature","public-key") if action=="prepare" else ("ack-signature","receiver-public-key")
        for n in paths: a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"prepare":prepare,"verify":verify}[action](**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED","reason":getattr(exc,"code","RECIPIENT_ACK_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
