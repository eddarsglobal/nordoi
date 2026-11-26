#!/usr/bin/env python3
"""Externally signed portable chain archive; recipient-pinned trust inputs.
No private key accepted, no replay consumption, freshness or execution authority.
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
sys.path.insert(0,str(HERE.parent/"app043_portable_chain_archive"))
import archive_app043 as prior
sys.path.insert(0,str(HERE.parent/"app020_signed_head_provenance"))
import receiver_app020 as crypto
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
SCHEMA="NORDOI_APP044_SIGNED_CHAIN_ARCHIVE_V1"
NAMESPACE="nordoi-chain-archive@eddarsglobal"
MAX_STATEMENT=4096

def expected(sender,receiver,nonce,key_pin,archive_pin,source_plan_pin,anchor_pin,head_pin):
    need(crypto.valid_id(sender) and crypto.valid_id(receiver),"EXPECTED_IDENTITY_INVALID")
    for pin in (nonce,key_pin,archive_pin,source_plan_pin,anchor_pin,head_pin): transfer.pin(pin)
    return {"schema":SCHEMA,"sender":sender,"receiver":receiver,"nonce":nonce,
            "sender_key_sha256":key_pin,"archive_sha256":archive_pin,
            "source_plan_sha256":source_plan_pin,"anchor_sha256":anchor_pin,
            "head_sha256":head_pin,"purpose":"CHAIN_EVIDENCE_ONLY",
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def capture_archive(archive,core):
    raw=registry.read(archive,prior.MAX_ARCHIVE)
    need(transfer.sha(raw)==core["archive_sha256"],"ARCHIVE_PIN_MISMATCH")
    return raw

def replay(raw,core):
    with tempfile.TemporaryDirectory(prefix="nordoi_app044_archive_") as tmp:
        path=Path(tmp)/"archive.json"
        portable.safe_write(path,raw)
        verified=prior.verify(path,core["archive_sha256"],core["source_plan_sha256"],core["anchor_sha256"])
    need(verified["status"]=="CHAIN_ARCHIVE_VERIFIED" and
         verified["head_sha256"]==core["head_sha256"],"EXPECTED_CHAIN_HEAD_MISMATCH")
    return verified

def prepare(archive,statement,**trust):
    core=expected(**trust)
    raw=capture_archive(archive,core)
    verified=replay(raw,core)
    need(capture_archive(archive,core)==raw,"ARCHIVE_CHANGED")
    portable.safe_write(statement,portable.serialize(core))
    return {"schema":SCHEMA,"status":"STATEMENT_PREPARED_UNSIGNED",
            "statement_sha256":transfer.sha(portable.serialize(core)),
            "head_sha256":verified["head_sha256"],"execution_authorized":False,
            "production":"NOT_AUTHORIZED"}

def verify(archive,statement,signature,public_key,**trust):
    core=expected(**trust)
    key=crypto.check_pubkey(public_key,core["sender_key_sha256"])
    message=registry.read(statement,MAX_STATEMENT)
    need(message==portable.serialize(core),"STATEMENT_EXPECTATION_MISMATCH")
    sig=registry.read(signature,crypto.MAX_SIGNATURE)
    need(sig.startswith(b"-----BEGIN SSH SIGNATURE-----\n") and
         sig.rstrip().endswith(b"-----END SSH SIGNATURE-----"),"SIGNATURE_FORMAT_INVALID")
    raw=capture_archive(archive,core)
    binary=shutil.which("ssh-keygen")
    need(binary is not None,"SSHSIG_VERIFIER_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi_app044_signature_") as tmp:
        p=Path(tmp)
        allowed=p/"allowed_signers"
        captured=p/"statement.sig"
        portable.safe_write(allowed,(core["sender"]+" "+key).encode("ascii"))
        portable.safe_write(captured,sig)
        verdict=subprocess.run([binary,"-Y","verify","-f",str(allowed),"-I",core["sender"],
                                "-n",NAMESPACE,"-s",str(captured)],
                               input=message,capture_output=True,timeout=20)
        need(verdict.returncode==0,"ARCHIVE_SIGNATURE_VERIFICATION_FAILED")
    verified=replay(raw,core)
    need(capture_archive(archive,core)==raw
         and registry.read(statement,MAX_STATEMENT)==message
         and registry.read(signature,crypto.MAX_SIGNATURE)==sig
         and crypto.check_pubkey(public_key,core["sender_key_sha256"])==key,"TRUST_INPUT_CHANGED")
    return dict(core,status="SIGNED_CHAIN_ARCHIVE_VERIFIED",
                statement_sha256=transfer.sha(message),signature_sha256=transfer.sha(sig),
                crypto_signature="SSHSIG_ED25519_VERIFIED",
                transition_count=verified["transition_count"],decisions_sha256=verified["decisions_sha256"],
                external_identity_enrollment="NOT_PROVEN",independent_key_custody="NOT_PROVEN",
                nonce_consumption="NOT_IMPLEMENTED",freshness="NOT_PROVEN",
                global_antirollback="NOT_PROVEN",transport_encrypted=False,
                reservation_release_authorized=False,nonce_reset_authorized=False,automatic_retry=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("prepare","verify"):
        a=sub.add_parser(action)
        for n in ("archive","statement"): a.add_argument("--"+n,type=Path,required=True)
        for n in ("sender","receiver","nonce","key-pin","archive-pin","source-plan-pin","anchor-pin","head-pin"):
            a.add_argument("--"+n,required=True)
        if action=="verify":
            for n in ("signature","public-key"): a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"prepare":prepare,"verify":verify}[action](**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED","reason":getattr(exc,"code","SIGNED_ARCHIVE_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
