#!/usr/bin/env python3
"""Recipient-pinned Ed25519 SSHSIG transfer verification.
The sender signs the canonical statement externally. No private key is
accepted by this program. A matching nonce is not a consumed replay token.
Trusted local OpenSSH binary and stable recipient trust inputs required.
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
sys.path.insert(0,str(HERE.parent/"app031_pinned_evidence_transfer"))
import transfer_app031 as transfer
sys.path.insert(0,str(HERE.parent/"app020_signed_head_provenance"))
import receiver_app020 as crypto

need=transfer.need
portable=transfer.portable
native=transfer.native
SCHEMA="NORDOI_APP032_SIGNED_EVIDENCE_TRANSFER_V1"
NAMESPACE="nordoi-transfer@eddarsglobal"
MAX_STATEMENT=2048

def statement(sender,receiver,nonce,transfer_pin,archive_pin,key_pin):
    need(crypto.valid_id(sender) and crypto.valid_id(receiver),"EXPECTED_IDENTITY_INVALID")
    for p in (nonce,transfer_pin,archive_pin,key_pin): transfer.pin(p)
    return {"schema":SCHEMA,"sender":sender,"receiver":receiver,"nonce":nonce,
            "transfer_sha256":transfer_pin,"archive_sha256":archive_pin,
            "sender_key_sha256":key_pin,"purpose":"EVIDENCE_ONLY",
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def prepare(packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,target):
    expected=statement(sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    transfer.decode(transfer.prior.read(packet,transfer.MAX_PACKAGE),transfer_pin,archive_pin)
    native.real_directory(target.parent,"STATEMENT_PARENT_INVALID")
    portable.safe_write(target,portable.serialize(expected))
    return expected

def receive(packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,public_key,signed_statement,signature,target):
    expected=statement(sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    native.real_directory(target.parent,"RECEIVER_PARENT_INVALID")
    need(not target.exists() and not target.is_symlink(),"RECEIVER_DESTINATION_EXISTS")
    key=crypto.check_pubkey(public_key,key_pin)
    raw=transfer.prior.read(signed_statement,MAX_STATEMENT)
    need(raw==portable.serialize(expected),"SIGNED_STATEMENT_EXPECTATION_MISMATCH")
    sig=transfer.prior.read(signature,crypto.MAX_SIGNATURE)
    need(sig.startswith(b"-----BEGIN SSH SIGNATURE-----\n") and
         sig.rstrip().endswith(b"-----END SSH SIGNATURE-----"),"SIGNATURE_FORMAT_INVALID")
    packet_raw=transfer.prior.read(packet,transfer.MAX_PACKAGE)
    need(transfer.sha(packet_raw)==transfer_pin,"TRANSFER_PIN_MISMATCH")
    binary=shutil.which("ssh-keygen")
    need(binary is not None,"SSHSIG_VERIFIER_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi_app032_") as tmp:
        p=Path(tmp)
        allowed=p/"allowed_signers"
        sigpath=p/"statement.sig"
        captured=p/"packet.json"
        portable.safe_write(allowed,(sender+" "+key).encode("ascii"))
        portable.safe_write(sigpath,sig)
        portable.safe_write(captured,packet_raw)
        verdict=subprocess.run([binary,"-Y","verify","-f",str(allowed),"-I",sender,
                                "-n",NAMESPACE,"-s",str(sigpath)],
                               input=raw,capture_output=True,timeout=20)
        need(verdict.returncode==0,"SSHSIG_SIGNATURE_VERIFICATION_FAILED")
        # Verify native evidence from the same captured bytes bound by signature.
        verified=transfer.receive(captured,transfer_pin,archive_pin,target)
    return {"schema":SCHEMA,"status":"SIGNED_TRANSFER_VERIFIED",
            "sender":sender,"receiver":receiver,"nonce":nonce,"sender_key_sha256":key_pin,
            "transfer_sha256":transfer_pin,"archive_sha256":archive_pin,
            "crypto_signature":"SSHSIG_ED25519_VERIFIED","nonce_matches_receiver_expectation":True,
            "verified_native_jobs":verified["verified_native_jobs"],"execution_authorized":False,
            "reservation_release_authorized":False,"automatic_retry":False,
            "external_identity_enrollment":"NOT_PROVEN","independent_key_custody":"NOT_PROVEN",
            "nonce_consumption":"NOT_IMPLEMENTED","distributed_anti_replay":"NOT_PROVEN",
            "transport_encrypted":False,"crash_durability":"UNCLAIMED","production":"NOT_AUTHORIZED"}

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("prepare","receive"):
        a=sub.add_parser(action)
        for n in ("packet","target"): a.add_argument("--"+n,type=Path,required=True)
        for n in ("transfer-pin","archive-pin","sender","receiver","nonce","key-pin"):
            a.add_argument("--"+n,required=True)
        if action=="receive":
            for n in ("public-key","signed-statement","signature"):
                a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out=prepare(**args) if action=="prepare" else receive(**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"BLOCK","reason":getattr(exc,"code","SIGNED_TRANSFER_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
