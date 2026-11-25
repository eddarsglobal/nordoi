#!/usr/bin/env python3
"""Read-only audit of APP-033 challenge states on a trusted stable local host.
Temporary verification files only. No historical destination identity claim.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app033_local_nonce_consumption"))
import nonce_app033 as previous
signed=previous.signed
transfer=previous.transfer
portable=previous.portable
native=previous.native
need=previous.need
SCHEMA="NORDOI_APP034_RECEPTION_FORENSICS_V1"

def read(path):
    return transfer.prior.read(path,8192)

def present(path):
    return path.exists() or path.is_symlink()

def report(status,nonce,**extra):
    return {"schema":SCHEMA,"status":status,"nonce":nonce,
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,
            "historical_destination_identity":"NOT_RECORDED_BY_APP033",
            "distributed_anti_replay":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
            "production":"NOT_AUTHORIZED",**extra}

def audit(registry,packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,public_key,signed_statement,signature,target):
    expected=previous.context(sender,receiver,nonce,transfer_pin,archive_pin,key_pin)
    expected_bytes=portable.serialize(expected)
    root=previous.registry_root(registry)
    directory=root/nonce
    native.real_directory(directory,"CHALLENGE_NOT_ISSUED")
    need(read(directory/"CHALLENGE.json")==expected_bytes,"CHALLENGE_CONTEXT_MISMATCH")
    claimed=directory/"consumed"
    def snapshot():
        need(read(directory/"CHALLENGE.json")==expected_bytes,"CHALLENGE_CONTEXT_CHANGED")
        entries={p.name for p in directory.iterdir()}
        need(entries in ({"CHALLENGE.json"},{"CHALLENGE.json","consumed"}),"UNEXPECTED_CHALLENGE_ENTRY")
        if not present(claimed):
            return None
        native.real_directory(claimed,"CONSUMPTION_MARKER_INVALID")
        names={p.name for p in claimed.iterdir()}
        need(names in (set(),{"START.json"},{"START.json","COMPLETE.json"}),"CONSUMPTION_RECEIPT_LAYOUT_INVALID")
        values={name:read(claimed/name) for name in names}
        if "START.json" in values:
            need(values["START.json"]==expected_bytes,"CONSUMPTION_START_MISMATCH")
        return values
    state=snapshot()
    if state is None:
        need(snapshot() is None,"REGISTRY_CHANGED_DURING_AUDIT")
        return report("ISSUED",nonce,observed_consumed=False,signature_reverified=False)
    if "COMPLETE.json" not in state:
        need(snapshot()==state,"REGISTRY_CHANGED_DURING_AUDIT")
        return report("CONSUMED_INCOMPLETE",nonce,observed_consumed=True,signature_reverified=False)
    native.real_directory(target,"PUBLISHED_EVIDENCE_MISSING")
    dest=target.resolve(strict=True)
    need(root!=dest and root not in dest.parents and dest not in root.parents,"EVIDENCE_REGISTRY_OVERLAP")
    with tempfile.TemporaryDirectory(prefix="nordoi_app034_") as tmp:
        staged=Path(tmp)/"verified"
        verified=signed.receive(packet,transfer_pin,archive_pin,sender,receiver,nonce,key_pin,
                                public_key,signed_statement,signature,staged)
        expected_files=transfer.collect(staged,archive_pin)
        need(transfer.collect(target,archive_pin)==expected_files,"PUBLISHED_EVIDENCE_MISMATCH")
        final=transfer.prior.verify(target,archive_pin)
        receipt={"schema":previous.SCHEMA,"status":"SIGNED_TRANSFER_ACCEPTED_ONCE",
                 "sender":sender,"receiver":receiver,"nonce":nonce,"sender_key_sha256":key_pin,
                 "archive_sha256":archive_pin,"transfer_sha256":transfer_pin,
                 "crypto_signature":verified["crypto_signature"],"verified_native_jobs":final["verified_native_jobs"],
                 "nonce_consumption":"PERMANENT_LOCAL_MARKER","automatic_retry":False,
                 "execution_authorized":False,"reservation_release_authorized":False,
                 "local_replay_scope":"COOPERATING_RECEIVERS_SAME_STABLE_REGISTRY",
                 "distributed_anti_replay":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
                 "crash_durability":"UNCLAIMED","transport_encrypted":False,"production":"NOT_AUTHORIZED"}
        need(state["COMPLETE.json"]==portable.serialize(receipt),"COMPLETION_RECEIPT_MISMATCH")
        need(transfer.collect(target,archive_pin)==expected_files,"PUBLISHED_EVIDENCE_CHANGED")
        need(snapshot()==state,"REGISTRY_CHANGED_DURING_AUDIT")
    return report("CONSUMED_VERIFIED",nonce,observed_consumed=True,signature_reverified=True,
                  verified_native_jobs=final["verified_native_jobs"],archive_sha256=archive_pin,
                  transfer_sha256=transfer_pin,matching_supplied_evidence=True)

def classify(**args):
    try:
        return audit(**args)
    except Exception as exc:
        return report("UNCERTAIN_BLOCKED",args.get("nonce"),reason=getattr(exc,"code","EVIDENCE_NOT_VERIFIED"),
                      signature_reverified=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    for n in ("registry","packet","public-key","signed-statement","signature","target"):
        p.add_argument("--"+n,type=Path,required=True)
    for n in ("sender","receiver","nonce","transfer-pin","archive-pin","key-pin"):
        p.add_argument("--"+n,required=True)
    result=classify(**vars(p.parse_args(argv)))
    print(json.dumps(result,sort_keys=True))
    return 3 if result["status"]=="UNCERTAIN_BLOCKED" else 0

if __name__=="__main__":
    raise SystemExit(main())
