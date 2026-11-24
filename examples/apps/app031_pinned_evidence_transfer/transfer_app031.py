#!/usr/bin/env python3
"""Bounded canonical evidence transfer, independent recipient pins.
No network, cryptographic sender identity, confidentiality or replay authority.
"""
from __future__ import annotations
import argparse
import base64
import hashlib
import json
from pathlib import Path
import re
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app030_verified_reservation_archive"))
import archive_app030 as prior
portable=prior.portable
native=prior.native
need=prior.need
SCHEMA="NORDOI_APP031_PINNED_EVIDENCE_TRANSFER_V1"
MAX_PACKAGE=16*1024*1024
MAX_ENTRIES=prior.MAX_FILES+2
MAX_TOTAL=8*1024*1024
FIXED={"ARCHIVE.json","ARCHIVE.sha256","partial-suffix.jsonl",
       "reservation/START.json","reservation/COMPLETE.json",
       "reservation/native-attempt/START.json","reservation/native-attempt/COMPLETE.json",
       "reservation/native-attempt/native-suffix.jsonl"}
PACKAGE_FIXED=re.compile(r"(old|new)/(portable\.json|PORTABLE\.sha256|handoff/(manifest\.json|SEAL\.sha256|plan\.json|native-prefix\.jsonl))")
NATIVE_FIXED=re.compile(r"((old|new)/)?native/job-000[1-8]/quality-decision/(decision\.txt|report\.txt)")

def allowed(path):
    return type(path) is str and (path in FIXED or PACKAGE_FIXED.fullmatch(path) is not None or NATIVE_FIXED.fullmatch(path) is not None)

def pin(value):
    need(type(value) is str and prior.PIN.fullmatch(value) is not None,"EXTERNAL_PIN_REQUIRED")

def sha(data):
    return hashlib.sha256(data).hexdigest()

def limit(path):
    return prior.MAX_MANIFEST if path=="ARCHIVE.json" else prior.MAX_FILE

def encode(files,archive_pin):
    need(0<len(files)<=MAX_ENTRIES and sum(map(len,files.values()))<=MAX_TOTAL,"TRANSFER_LIMIT")
    entries=[]
    for path,data in sorted(files.items()):
        need(allowed(path) and len(data)<=limit(path),"TRANSFER_PATH_OR_SIZE_INVALID")
        entries.append({"path":path,"bytes":len(data),"sha256":sha(data),
                        "base64":base64.b64encode(data).decode("ascii")})
    raw=portable.serialize({"schema":SCHEMA,"archive_sha256":archive_pin,"files":entries})
    need(len(raw)<=MAX_PACKAGE,"TRANSFER_TOO_LARGE")
    return raw

def decode(raw,transfer_pin,archive_pin):
    pin(transfer_pin)
    pin(archive_pin)
    need(len(raw)<=MAX_PACKAGE and sha(raw)==transfer_pin,"TRANSFER_PIN_MISMATCH")
    obj=prior.prior.forensic.audit.decode_json(raw,"TRANSFER_JSON_INVALID")
    need(type(obj) is dict and set(obj)=={"schema","archive_sha256","files"} and
         obj["schema"]==SCHEMA and obj["archive_sha256"]==archive_pin,"TRANSFER_ARCHIVE_PIN_MISMATCH")
    entries=obj["files"]
    need(type(entries) is list and 0<len(entries)<=MAX_ENTRIES,"TRANSFER_ENTRY_COUNT")
    files={}
    total=0
    for item in entries:
        need(type(item) is dict and set(item)=={"path","bytes","sha256","base64"},"TRANSFER_ENTRY_SCHEMA")
        path=item["path"]
        need(allowed(path) and path not in files,"TRANSFER_PATH_OR_DUPLICATE")
        size=item["bytes"]
        need(type(size) is int and 0<=size<=limit(path) and type(item["base64"]) is str
             and len(item["base64"])==4*((size+2)//3),"TRANSFER_ENTRY_SIZE")
        total+=size
        need(total<=MAX_TOTAL,"TRANSFER_TOTAL_LIMIT")
        data=base64.b64decode(item["base64"],validate=True)
        need(len(data)==size and sha(data)==item["sha256"],"TRANSFER_ENTRY_DIGEST")
        files[path]=data
    need(raw==encode(files,archive_pin),"NONCANONICAL_TRANSFER")
    need(FIXED<=files.keys(),"TRANSFER_REQUIRED_FILES_MISSING")
    return files

def collect(archive,archive_pin):
    prior.verify(archive,archive_pin)
    files=prior.contents(archive)
    files["ARCHIVE.json"]=prior.read(archive/"ARCHIVE.json",prior.MAX_MANIFEST)
    files["ARCHIVE.sha256"]=prior.read(archive/"ARCHIVE.sha256",65)
    need(sha(files["ARCHIVE.json"])==archive_pin,"ARCHIVE_PIN_CHANGED")
    return files

def result(status,transfer_pin,archive_pin,jobs):
    return {"schema":SCHEMA,"status":status,"transfer_sha256":transfer_pin,"archive_sha256":archive_pin,
            "verified_native_jobs":jobs,"execution_authorized":False,"reservation_release_authorized":False,
            "automatic_retry":False,"sender_identity_authenticated":False,"transport_encrypted":False,
            "distributed_anti_replay":"NOT_PROVEN","crash_durability":"UNCLAIMED","production":"NOT_AUTHORIZED"}

def export(archive,archive_pin,packet):
    pin(archive_pin)
    native.real_directory(packet.parent,"TRANSFER_PARENT_INVALID")
    a=archive.resolve(strict=True)
    p=packet.parent.resolve(strict=True)/packet.name
    need(a!=p and a not in p.parents,"TRANSFER_OVERLAPS_SOURCE")
    need(not packet.exists() and not packet.is_symlink(),"TRANSFER_DESTINATION_EXISTS")
    files=collect(archive,archive_pin)
    raw=encode(files,archive_pin)
    verified=prior.verify(archive,archive_pin)
    need(files==collect(archive,archive_pin),"SOURCE_CHANGED_DURING_TRANSFER")
    portable.safe_write(packet,raw)
    need(prior.read(packet,MAX_PACKAGE)==raw,"TRANSFER_PUBLICATION_UNKNOWN")
    return result("TRANSFER_EXPORTED",sha(raw),archive_pin,verified["verified_native_jobs"])

def populate(directory,files):
    for rel,data in sorted(files.items()):
        need(allowed(rel),"TRANSFER_PATH_INVALID")
        p=directory/rel
        p.parent.mkdir(parents=True,exist_ok=True)
        portable.safe_write(p,data)

def receive(packet,transfer_pin,archive_pin,target):
    pin(transfer_pin)
    pin(archive_pin)
    native.real_directory(target.parent,"RECEIVER_PARENT_INVALID")
    need(not target.exists() and not target.is_symlink(),"RECEIVER_DESTINATION_EXISTS")
    raw=prior.read(packet,MAX_PACKAGE)
    files=decode(raw,transfer_pin,archive_pin)
    # All untrusted data is parsed and audited in private temporary storage.
    # No destination publication until semantic verification of the native DAG.
    with tempfile.TemporaryDirectory(prefix="nordoi_app031_") as tmp:
        staged=Path(tmp)
        populate(staged,files)
        verified=prior.verify(staged,archive_pin)
        need(prior.read(packet,MAX_PACKAGE)==raw,"PACKET_CHANGED_DURING_VERIFY")
        target.mkdir(mode=0o700)
        # Publish completion marker last, preserving partial output on failure.
        try:
            marker=files["ARCHIVE.sha256"]
            populate(target,{k:v for k,v in files.items() if k!="ARCHIVE.sha256"})
            portable.safe_write(target/"ARCHIVE.sha256",marker)
            final=prior.verify(target,archive_pin)
            need(final==verified,"RECEIVER_VERIFICATION_CHANGED")
        except Exception as exc:
            raise prior.prior.forensic.Blocked("RECEIVER_PUBLICATION_UNKNOWN_NO_REPLAY") from exc
    return result("TRANSFER_RECEIVED_VERIFIED",transfer_pin,archive_pin,verified["verified_native_jobs"])

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    e=sub.add_parser("export")
    e.add_argument("--archive",type=Path,required=True)
    e.add_argument("--packet",type=Path,required=True)
    e.add_argument("--archive-pin",required=True)
    v=sub.add_parser("receive")
    v.add_argument("--packet",type=Path,required=True)
    v.add_argument("--target",type=Path,required=True)
    v.add_argument("--archive-pin",required=True)
    v.add_argument("--transfer-pin",required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out=export(**args) if action=="export" else receive(**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"BLOCK","reason":getattr(exc,"code","TRANSFER_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,"automatic_retry":False,
             "production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
