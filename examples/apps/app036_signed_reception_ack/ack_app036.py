#!/usr/bin/env python3
"""Recipient-signed acknowledgment of a completed APP-035 reception.
Sender verification is a signed assertion, not independent remote FS attestation.
No secret key accepted, no reset, no execution, no transport implemented.
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
sys.path.insert(0,str(HERE.parent/"app035_destination_bound_reception"))
import bound_app035 as bound
signed=bound.signed
transfer=bound.transfer
portable=bound.portable
native=bound.native
need=bound.need
crypto=signed.crypto
SCHEMA="NORDOI_APP036_SIGNED_RECEPTION_ACK_V1"
NAMESPACE="nordoi-reception-ack@eddarsglobal"
MAX_ACK=4096

def expected(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin):
    statement=signed.statement(sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    transfer.pin(receiver_key_pin)
    transfer.pin(challenge_pin)
    return {"schema":SCHEMA,"status":"RECEPTION_VERIFIED_AT_SIGNING_PREPARATION",
            "transfer_statement":statement,"receiver_key_sha256":receiver_key_pin,
            "challenge_sha256":challenge_pin,"purpose":"EVIDENCE_ACK_ONLY",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"production":"NOT_AUTHORIZED"}

def prepare(registry,packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,public_key,
            signed_statement,signature,target,receiver_key_pin,challenge_pin,ack):
    core=expected(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin)
    native.real_directory(ack.parent,"ACK_PARENT_INVALID")
    output=ack.parent.resolve(strict=True)/ack.name
    for p in (registry,target):
        root=p.resolve(strict=True)
        need(root!=output and root not in output.parents and output not in root.parents,"ACK_OVERLAPS_EVIDENCE")
    need(not ack.exists() and not ack.is_symlink(),"ACK_DESTINATION_EXISTS")
    args=dict(registry=registry,packet=packet,transfer_pin=transfer_pin,archive_pin=archive_pin,
              sender=sender,receiver=receiver,nonce=nonce,key_pin=key_pin,public_key=public_key,
              signed_statement=signed_statement,signature=signature,target=target)
    verified=bound.audit(**args)
    need(verified["status"]=="DESTINATION_BOUND_VERIFIED","RECEPTION_NOT_VERIFIED")
    _,context=bound.context(registry,target,sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    need(transfer.sha(portable.serialize(context))==challenge_pin,"ACK_CHALLENGE_PIN_MISMATCH")
    original=dict(verified,status="DESTINATION_BOUND_ACCEPTED_ONCE")
    receipt_pin=transfer.sha(portable.serialize(original))
    # Re-audit before exclusive publication. No local path is disclosed in ACK.
    need(bound.audit(**args)==verified,"RECEPTION_CHANGED_BEFORE_ACK")
    doc=dict(core,receipt_sha256=receipt_pin,verified_native_jobs=verified["verified_native_jobs"])
    raw=portable.serialize(doc)
    need(len(raw)<=MAX_ACK,"ACK_TOO_LARGE")
    portable.safe_write(ack,raw)
    return {"schema":SCHEMA,"status":"ACK_PREPARED_UNSIGNED","ack_sha256":transfer.sha(raw),
            "receipt_sha256":receipt_pin,"execution_authorized":False,"production":"NOT_AUTHORIZED"}

def verify(ack,ack_signature,receiver_public_key,sender,receiver,nonce,transfer_pin,archive_pin,
           key_pin,receiver_key_pin,challenge_pin):
    core=expected(sender,receiver,nonce,transfer_pin,archive_pin,key_pin,receiver_key_pin,challenge_pin)
    key=crypto.check_pubkey(receiver_public_key,receiver_key_pin)
    raw=transfer.prior.read(ack,MAX_ACK)
    obj=transfer.prior.prior.forensic.audit.decode_json(raw,"ACK_JSON_INVALID")
    need(type(obj) is dict and set(obj)==set(core)|{"receipt_sha256","verified_native_jobs"},
         "ACK_SCHEMA_INVALID")
    transfer.pin(obj["receipt_sha256"])
    count=obj["verified_native_jobs"]
    need(type(count) is int and 1<=count<=8,"ACK_NATIVE_COUNT_INVALID")
    need(raw==portable.serialize(dict(core,receipt_sha256=obj["receipt_sha256"],verified_native_jobs=count)),
         "ACK_EXPECTATION_MISMATCH")
    sig=transfer.prior.read(ack_signature,crypto.MAX_SIGNATURE)
    need(sig.startswith(b"-----BEGIN SSH SIGNATURE-----\n") and
         sig.rstrip().endswith(b"-----END SSH SIGNATURE-----"),"ACK_SIGNATURE_FORMAT_INVALID")
    binary=shutil.which("ssh-keygen")
    need(binary is not None,"SSHSIG_VERIFIER_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi_app036_") as tmp:
        p=Path(tmp)
        allowed=p/"allowed_signers"
        captured=p/"ack.sig"
        portable.safe_write(allowed,(receiver+" "+key).encode("ascii"))
        portable.safe_write(captured,sig)
        verdict=subprocess.run([binary,"-Y","verify","-f",str(allowed),"-I",receiver,
                                "-n",NAMESPACE,"-s",str(captured)],
                               input=raw,capture_output=True,timeout=20)
        need(verdict.returncode==0,"ACK_SIGNATURE_VERIFICATION_FAILED")
    return {"schema":SCHEMA,"status":"RECIPIENT_ACK_SIGNATURE_VERIFIED",
            "ack_sha256":transfer.sha(raw),"receipt_sha256":obj["receipt_sha256"],
            "sender":sender,"receiver":receiver,"nonce":nonce,"challenge_sha256":challenge_pin,
            "transfer_sha256":transfer_pin,"archive_sha256":archive_pin,
            "receiver_key_sha256":receiver_key_pin,"recipient_asserted_native_jobs":count,
            "crypto_signature":"SSHSIG_ED25519_VERIFIED","remote_filesystem_independently_verified":False,
            "current_remote_state_proven":False,"ack_replay_prevention":"NOT_IMPLEMENTED",
            "independent_key_custody":"NOT_PROVEN","execution_authorized":False,
            "reservation_release_authorized":False,"nonce_reset_authorized":False,
            "automatic_retry":False,"production":"NOT_AUTHORIZED"}

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("prepare","verify"):
        a=sub.add_parser(action)
        for n in ("sender","receiver","nonce","transfer-pin","archive-pin","key-pin","receiver-key-pin","challenge-pin"):
            a.add_argument("--"+n,required=True)
        a.add_argument("--ack",type=Path,required=True)
        paths=("registry","packet","public-key","signed-statement","signature","target") if action=="prepare" else ("ack-signature","receiver-public-key")
        for n in paths: a.add_argument("--"+n,type=Path,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out=prepare(**args) if action=="prepare" else verify(**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"BLOCK","reason":getattr(exc,"code","ACK_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
