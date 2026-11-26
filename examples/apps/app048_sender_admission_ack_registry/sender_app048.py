#!/usr/bin/env python3
"""Sender-side exclusive receipt of authenticated APP-047 acceptance evidence.
Trust: cooperating processes, stable local registry, externally trusted candidate pins.
No distributed anti-replay, independent remote attestation or rollback protection.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app047_signed_admission_evidence"))
import ack_app047 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
native=prior.prior.prior.native
SCHEMA="NORDOI_APP048_SENDER_ADMISSION_ACK_REGISTRY_V1"
MAX_RECORD=32768
LIMITS={"PACKAGE.json":prior.prior.MAX_PACKAGE,"ACK.json":prior.MAX_ACK,
        "ACK.sig":prior.crypto.MAX_SIGNATURE,"RECEIVER.pub":prior.crypto.MAX_PUBLIC_KEY}

def context(receiver_key_pin,**trust):
    transfer.pin(receiver_key_pin)
    need(receiver_key_pin!=trust["key_pin"],"RECIPIENT_KEY_MUST_DIFFER")
    return {"schema":SCHEMA,"challenge":prior.prior.prior.context(**trust),
            "receiver_key_sha256":receiver_key_pin}

def register(ack_registry,receiver_key_pin,**trust):
    ctx=context(receiver_key_pin,**trust)
    entry=registry.root(ack_registry)/trust["nonce"]
    entry.mkdir(mode=0o700)
    portable.safe_write(entry/"EXPECTED.json",portable.serialize(ctx))
    return outcome("ACK_EXPECTATION_REGISTERED",trust["nonce"])

def entry_for(ack_registry,ctx,nonce):
    entry=registry.root(ack_registry)/nonce
    native.real_directory(entry,"ACK_EXPECTATION_NOT_REGISTERED")
    need(registry.read(entry/"EXPECTED.json",MAX_RECORD)==portable.serialize(ctx),"ACK_REGISTERED_CONTEXT_MISMATCH")
    need({p.name for p in entry.iterdir()} in ({"EXPECTED.json"},{"EXPECTED.json","accepted"}),
         "ACK_REGISTRY_LAYOUT_INVALID")
    return entry

def capture(paths):
    return {n:registry.read(paths[n],limit) for n,limit in LIMITS.items()}

def verify_bytes(files,package_pin,receipt_pin,receiver_key_pin,trust):
    with tempfile.TemporaryDirectory(prefix="nordoi_app048_") as tmp:
        root=Path(tmp).resolve()
        for name,raw in files.items(): portable.safe_write(root/name,raw)
        verified=prior.verify(package=root/"PACKAGE.json",ack=root/"ACK.json",
            ack_signature=root/"ACK.sig",receiver_public_key=root/"RECEIVER.pub",
            package_pin=package_pin,receipt_pin=receipt_pin,receiver_key_pin=receiver_key_pin,**trust)
    need(verified["status"]=="RECIPIENT_ACCEPTANCE_SIGNATURE_VERIFIED","ACK_NOT_AUTHENTICATED")
    return verified

def receipt(ctx,files,verified):
    return {"schema":SCHEMA,"status":"ACK_ACCEPTED_ONCE",
            "context_sha256":transfer.sha(portable.serialize(ctx)),
            "package_sha256":verified["package_sha256"],"receipt_sha256":verified["receipt_sha256"],
            "ack_sha256":verified["ack_sha256"],"signature_sha256":verified["ack_signature_sha256"],
            "receiver_key_sha256":verified["receiver_key_sha256"],
            "verification_sha256":transfer.sha(portable.serialize(verified)),
            "members":{name:transfer.sha(raw) for name,raw in files.items()},
            "execution_authorized":False,"production":"NOT_AUTHORIZED"}

def stored(entry,ctx,receiver_key_pin,trust):
    accepted=entry/"accepted"
    native.real_directory(accepted,"ACK_MARKER_INVALID")
    need({p.name for p in accepted.iterdir()}==set(LIMITS)|{"START.json","COMPLETE.json"},
         "ACK_CONSUMPTION_INCOMPLETE")
    start=registry.read(accepted/"START.json",MAX_RECORD)
    complete=registry.read(accepted/"COMPLETE.json",MAX_RECORD)
    need(start==portable.serialize(ctx),"ACK_START_CONTEXT_MISMATCH")
    rec=json.loads(complete)
    need(type(rec) is dict,"ACK_RECEIPT_INVALID")
    package_pin=rec["package_sha256"]
    receipt_pin=rec["receipt_sha256"]
    for pin in (package_pin,receipt_pin): transfer.pin(pin)
    paths={name:accepted/name for name in LIMITS}
    files=capture(paths)
    verified=verify_bytes(files,package_pin,receipt_pin,receiver_key_pin,trust)
    expected=receipt(ctx,files,verified)
    need(complete==portable.serialize(expected),"ACK_COMPLETION_RECEIPT_MISMATCH")
    need(capture(paths)==files and registry.read(accepted/"START.json",MAX_RECORD)==start
         and registry.read(accepted/"COMPLETE.json",MAX_RECORD)==complete
         and registry.read(entry/"EXPECTED.json",MAX_RECORD)==start,"ACK_REGISTRY_CHANGED_DURING_AUDIT")
    return files,expected

def outcome(status,nonce,**extra):
    return {"schema":SCHEMA,"status":status,"nonce":nonce,
            "local_replay_scope":"COOPERATING_SENDERS_SAME_STABLE_REGISTRY",
            "distributed_anti_replay":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
            "independent_historical_acceptance":"NOT_PROVEN","live_nonce_state":"NOT_VERIFIED",
            "crash_durability":"UNCLAIMED","execution_authorized":False,
            "reservation_release_authorized":False,"nonce_reset_authorized":False,
            "automatic_retry":False,"production":"NOT_AUTHORIZED",**extra}

def accept(ack_registry,package,ack,ack_signature,receiver_public_key,package_pin,receipt_pin,
           receiver_key_pin,**trust):
    ctx=context(receiver_key_pin,**trust)
    nonce=trust["nonce"]
    root=registry.root(ack_registry)
    paths={"PACKAGE.json":package,"ACK.json":ack,"ACK.sig":ack_signature,"RECEIVER.pub":receiver_public_key}
    for p in paths.values():
        resolved=p.resolve(strict=True)
        need(root!=resolved and root not in resolved.parents,"ACK_INPUT_OVERLAP")
    entry=entry_for(root,ctx,nonce)
    files=capture(paths)
    verified=verify_bytes(files,package_pin,receipt_pin,receiver_key_pin,trust)
    need(capture(paths)==files and entry_for(root,ctx,nonce)==entry,"ACK_INPUT_CHANGED")
    accepted=entry/"accepted"
    try:
        accepted.mkdir(mode=0o700)
    except FileExistsError:
        old,record=stored(entry,ctx,receiver_key_pin,trust)
        if old["ACK.json"]==files["ACK.json"]:
            return outcome("DUPLICATE_ACK_BLOCKED",nonce,accepted_ack_sha256=record["ack_sha256"])
        return outcome("CONTRADICTORY_SIGNED_ACK_BLOCKED",nonce,
                       accepted_ack_sha256=record["ack_sha256"],conflicting_ack_sha256=verified["ack_sha256"],
                       conflict_persisted=False)
    # Permanent marker: no cleanup or retry after an incomplete write.
    portable.safe_write(accepted/"START.json",portable.serialize(ctx))
    for name,raw in files.items(): portable.safe_write(accepted/name,raw)
    record=receipt(ctx,files,verified)
    portable.safe_write(accepted/"COMPLETE.json",portable.serialize(record))
    need(stored(entry,ctx,receiver_key_pin,trust)==(files,record),"ACK_PUBLICATION_NOT_VERIFIED")
    return outcome("ACK_ACCEPTED_ONCE",nonce,record=record)

def audit(ack_registry,receiver_key_pin,**trust):
    ctx=context(receiver_key_pin,**trust)
    nonce=trust["nonce"]
    entry=entry_for(ack_registry,ctx,nonce)
    if not registry.present(entry/"accepted"):
        need(entry_for(ack_registry,ctx,nonce)==entry and not registry.present(entry/"accepted"),"ACK_STATE_CHANGED")
        return outcome("ACK_PENDING",nonce)
    _,record=stored(entry,ctx,receiver_key_pin,trust)
    return outcome("ACK_ACCEPTED_VERIFIED",nonce,record=record)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("register","accept","audit"):
        a=sub.add_parser(action)
        a.add_argument("--ack-registry",type=Path,required=True)
        for n in ("sender","receiver","nonce","key-pin","archive-pin","source-plan-pin","anchor-pin","head-pin","receiver-key-pin"):
            a.add_argument("--"+n,required=True)
        if action=="accept":
            for n in ("package","ack","ack-signature","receiver-public-key"): a.add_argument("--"+n,type=Path,required=True)
            for n in ("package-pin","receipt-pin"): a.add_argument("--"+n,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"register":register,"accept":accept,"audit":audit}[action](**args)
        rc=0 if out["status"] in ("ACK_EXPECTATION_REGISTERED","ACK_ACCEPTED_ONCE","ACK_PENDING","ACK_ACCEPTED_VERIFIED") else 3
    except Exception as exc:
        out=outcome("UNCERTAIN_BLOCKED",args.get("nonce"),reason=getattr(exc,"code","ACK_NOT_VERIFIED"))
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
