#!/usr/bin/env python3
"""Portable evidence of an APP-045 acceptance, under independently supplied pins.
Restored markers are evidence in a disposable workspace, not live replay state.
No independent proof of historical execution, remote consumption or freshness.
"""
from __future__ import annotations
import argparse
import base64
import json
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app045_archive_nonce_admission"))
import admission_app045 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
SCHEMA="NORDOI_APP046_PORTABLE_ADMISSION_EVIDENCE_V1"
MAX_PACKAGE=48*1024*1024
LIMITS={"archive.json":prior.prior.prior.MAX_ARCHIVE,
        "statement.json":prior.prior.MAX_STATEMENT,
        "statement.sig":prior.prior.crypto.MAX_SIGNATURE,
        "sender.pub":prior.prior.crypto.MAX_PUBLIC_KEY,
        "CHALLENGE.json":prior.MAX_RECORD,"START.json":prior.MAX_RECORD,
        "COMPLETE.json":prior.MAX_RECORD}

def capture(challenges,archive,statement,signature,public_key,**trust):
    ctx=prior.context(**trust)
    folder=prior.folder_for(challenges,ctx,(archive,statement,signature,public_key))
    marker=folder/"consumed"
    prior.native.real_directory(marker,"ADMISSION_MARKER_INVALID")
    need({p.name for p in marker.iterdir()}=={"START.json","COMPLETE.json"},
         "ADMISSION_INCOMPLETE")
    paths={"archive.json":archive,"statement.json":statement,"statement.sig":signature,
           "sender.pub":public_key,"CHALLENGE.json":folder/"CHALLENGE.json",
           "START.json":marker/"START.json","COMPLETE.json":marker/"COMPLETE.json"}
    return {k:registry.read(p,LIMITS[k]) for k,p in paths.items()}

def decode(doc,receipt_pin):
    need(type(doc) is dict and set(doc)=={"schema","receipt_sha256","members"}
         and doc["schema"]==SCHEMA and doc["receipt_sha256"]==receipt_pin,"PACKAGE_SCHEMA_INVALID")
    members=doc["members"]
    need(type(members) is dict and set(members)==set(LIMITS),"PACKAGE_MEMBER_SET_INVALID")
    files={}
    for name,limit in LIMITS.items():
        v=members[name]
        need(type(v) is str and len(v)<=4*((limit+2)//3),"PACKAGE_MEMBER_TOO_LARGE")
        raw=base64.b64decode(v,validate=True)
        need(len(raw)<=limit and base64.b64encode(raw).decode("ascii")==v,"PACKAGE_BASE64_INVALID")
        files[name]=raw
    need(transfer.sha(files["COMPLETE.json"])==receipt_pin,"ACCEPTANCE_RECEIPT_PIN_MISMATCH")
    return files

def replay(files,receipt_pin,trust):
    ctx=prior.context(**trust)
    need(transfer.sha(files["COMPLETE.json"])==receipt_pin,"ACCEPTANCE_RECEIPT_PIN_MISMATCH")
    need(files["CHALLENGE.json"]==portable.serialize(ctx),"PACKAGE_CHALLENGE_MISMATCH")
    with tempfile.TemporaryDirectory(prefix="nordoi_app046_evidence_") as tmp:
        root=Path(tmp).resolve()
        challenges=root/"evidence-registry"
        marker=challenges/trust["nonce"]/"consumed"
        marker.mkdir(parents=True)
        paths={"archive":root/"archive.json","statement":root/"statement.json",
               "signature":root/"statement.sig","public_key":root/"sender.pub"}
        for p in paths.values(): portable.safe_write(p,files[p.name])
        portable.safe_write(marker.parent/"CHALLENGE.json",files["CHALLENGE.json"])
        for name in ("START.json","COMPLETE.json"): portable.safe_write(marker/name,files[name])
        verified=prior.audit(challenges=challenges,**paths,**trust)
        need(verified["status"]=="ARCHIVE_ACCEPTANCE_VERIFIED"
             and portable.serialize(verified["record"])==files["COMPLETE.json"],"ADMISSION_REPLAY_MISMATCH")
    return verified["record"]

def outcome(status,pin,receipt_pin,rec):
    return {"schema":SCHEMA,"status":status,"package_sha256":pin,
            "receipt_sha256":receipt_pin,"record":rec,"original_locations_required":False,
            "local_acceptance_trust":"EXTERNAL_RECEIPT_PIN_REQUIRED",
            "independent_historical_acceptance":"NOT_PROVEN",
            "live_nonce_state":"NOT_VERIFIED","distributed_anti_replay":"NOT_PROVEN",
            "global_antirollback":"NOT_PROVEN","freshness":"NOT_PROVEN",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}

def pack(output,receipt_pin,challenges,archive,statement,signature,public_key,**trust):
    transfer.pin(receipt_pin)
    prior.native.real_directory(output.parent,"PACKAGE_PARENT_INVALID")
    dest=output.parent.resolve(strict=True)/output.name
    root=registry.root(challenges)
    need(root!=dest and root not in dest.parents and not registry.present(dest),"PACKAGE_OUTPUT_INVALID")
    args=dict(challenges=root,archive=archive,statement=statement,signature=signature,
              public_key=public_key,**trust)
    files=capture(**args)
    checked=prior.audit(**args)
    need(checked["status"]=="ARCHIVE_ACCEPTANCE_VERIFIED" and
         portable.serialize(checked["record"])==files["COMPLETE.json"],"SOURCE_RECEIPT_MISMATCH")
    rec=replay(files,receipt_pin,trust)
    need(capture(**args)==files,"SOURCE_EVIDENCE_CHANGED")
    doc={"schema":SCHEMA,"receipt_sha256":receipt_pin,
         "members":{k:base64.b64encode(v).decode("ascii") for k,v in files.items()}}
    raw=portable.serialize(doc)
    need(len(raw)<=MAX_PACKAGE,"PACKAGE_TOO_LARGE")
    portable.safe_write(dest,raw)
    need(registry.read(dest,MAX_PACKAGE)==raw,"PACKAGE_PUBLICATION_MISMATCH")
    return outcome("ADMISSION_EVIDENCE_PACKED",transfer.sha(raw),receipt_pin,rec)

def verify(output,package_pin,receipt_pin,**trust):
    for pin in (package_pin,receipt_pin): transfer.pin(pin)
    raw=registry.read(output,MAX_PACKAGE)
    need(transfer.sha(raw)==package_pin,"PACKAGE_PIN_MISMATCH")
    doc=json.loads(raw)
    need(raw==portable.serialize(doc),"PACKAGE_NOT_CANONICAL")
    rec=replay(decode(doc,receipt_pin),receipt_pin,trust)
    need(registry.read(output,MAX_PACKAGE)==raw,"PACKAGE_CHANGED_DURING_REPLAY")
    return outcome("ADMISSION_EVIDENCE_VERIFIED",package_pin,receipt_pin,rec)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("pack","verify"):
        a=sub.add_parser(action)
        a.add_argument("--output",type=Path,required=True)
        for n in ("receipt-pin","sender","receiver","nonce","key-pin","archive-pin","source-plan-pin","anchor-pin","head-pin"):
            a.add_argument("--"+n,required=True)
        if action=="pack":
            for n in ("challenges","archive","statement","signature","public-key"):
                a.add_argument("--"+n,type=Path,required=True)
        else: a.add_argument("--package-pin",required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"pack":pack,"verify":verify}[action](**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED","reason":getattr(exc,"code","ADMISSION_EVIDENCE_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
