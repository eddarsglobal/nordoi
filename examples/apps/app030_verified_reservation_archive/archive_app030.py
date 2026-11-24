#!/usr/bin/env python3
"""Additive evidence archive. Never removes or releases a reservation.
Trusted quiescent local filesystem only. A pin is not an identity signature.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app029_reservation_forensics"))
import audit_app029 as prior
portable=prior.portable
native=prior.native
need=prior.need
SCHEMA="NORDOI_APP030_VERIFIED_RESERVATION_ARCHIVE_V1"
MAX_FILE=max(65536,prior.forensic.MAX_SUFFIX_BYTES,native.MAX_FILES_BYTES,portable.MAX_MANIFEST)
MAX_MANIFEST=262144
MAX_FILES=80
PIN=prior.forensic.PIN

def read(path,limit=MAX_FILE):
    return native.bounded_regular_file(path,limit,"ARCHIVE_FILE_INVALID")

def digest(raw):
    return hashlib.sha256(raw).hexdigest()

def encoded(data):
    return portable.serialize(data)

def add_tree(files,label,root,paths):
    for rel in paths:
        p=root
        for part in rel.split("/")[:-1]:
            p=p/part
            native.real_directory(p,"SOURCE_DIRECTORY_INVALID")
        files[label+"/"+rel]=read(root/rel)

def package_paths(manifest):
    return ["portable.json","PORTABLE.sha256","handoff/manifest.json",
            "handoff/SEAL.sha256","handoff/plan.json","handoff/native-prefix.jsonl"] + [
            "native/"+x["path"] for x in manifest["native_files"]]

def snapshot(old,old_pin,root,suffix,suffix_pin,new,new_pin,engine_pin,source_pin,final_suffix_pin):
    args=locals().copy()
    report=prior.audit(**args)
    om,_,_=portable.verify(old,old_pin)
    nm,_,_=portable.verify(new,new_pin)
    _,_,claim=prior.exclusive.location(root)
    files={}
    add_tree(files,"old",old,package_paths(om))
    add_tree(files,"new",new,package_paths(nm))
    add_tree(files,"reservation",claim,["START.json","COMPLETE.json",
        "native-attempt/START.json","native-attempt/COMPLETE.json","native-attempt/native-suffix.jsonl"])
    files["partial-suffix.jsonl"]=read(suffix)
    raw=files["new/handoff/native-prefix.jsonl"]+files["reservation/native-attempt/native-suffix.jsonl"]
    # The full audit has already checked the DAG; enumerate only verified events.
    records=[prior.forensic.audit.decode_json(line,"EVENT_JSON_INVALID") for line in raw.splitlines()]
    paths=[x["path"] for x in portable.native_inventory(root,records)]
    add_tree(files,"native",root,paths)
    need(len(files)<=MAX_FILES,"ARCHIVE_FILE_COUNT_EXCEEDED")
    need(prior.audit(**args)==report,"SOURCE_CHANGED_DURING_ARCHIVE")
    return files,report

def inventory(files):
    return [{"path":p,"bytes":len(b),"sha256":digest(b)} for p,b in sorted(files.items())]

def contents(root):
    native.real_directory(root,"ARCHIVE_DIRECTORY_INVALID")
    files={}
    def walk(directory,prefix="",depth=0):
        need(depth<=6,"ARCHIVE_DEPTH_EXCEEDED")
        for p in sorted(directory.iterdir()):
            rel=prefix+p.name
            if not prefix and p.name in ("ARCHIVE.json","ARCHIVE.sha256"):
                continue
            need(not p.is_symlink(),"ARCHIVE_SYMLINK_REFUSED")
            if p.is_dir():
                native.real_directory(p,"ARCHIVE_DIRECTORY_INVALID")
                need(any(p.iterdir()),"ARCHIVE_EMPTY_DIRECTORY")
                walk(p,rel+"/",depth+1)
            else:
                need(len(files)<MAX_FILES,"ARCHIVE_FILE_COUNT_EXCEEDED")
                files[rel]=read(p)
    walk(root)
    return files

def verify_payload(target,manifest):
    files=contents(target)
    need(inventory(files)==manifest["files"],"ARCHIVE_INVENTORY_MISMATCH")
    pins=manifest["pins"]
    om,_,oe=portable.verify(target/"old",pins["old_pin"])
    nm,_,ne=portable.verify(target/"new",pins["new_pin"])
    partial=prior.forensic.reconcile(target/"old",pins["old_pin"],target/"new"/"native",
                                     target/"partial-suffix.jsonl",pins["suffix_pin"])
    full=prior.forensic.reconcile(target/"new",pins["new_pin"],target/"native",
                  target/"reservation"/"native-attempt"/"native-suffix.jsonl",pins["final_suffix_pin"])
    need(partial["status"]=="CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT" and
         full["status"]=="COMPLETE_VERIFIED" and len(oe)<len(ne) and
         ne[:len(oe)]==oe and nm["prefix_sha256"]==partial["combined_events_sha256"],
         "ARCHIVED_NATIVE_CHAIN_INVALID")
    report=manifest["original_audit"]
    need(report["status"]=="COMPLETE_EVIDENCE_VERIFIED" and
         report["verified_events_total"]==full["verified_events_total"] and
         report["verified_native_jobs"]==full["verified_native_jobs"] and
         report["final_suffix_sha256"]==pins["final_suffix_pin"],"ARCHIVED_REPORT_MISMATCH")
    return full

def verify(target,expected_pin):
    need(type(expected_pin) is str and PIN.fullmatch(expected_pin) is not None,"EXTERNAL_ARCHIVE_PIN_REQUIRED")
    native.real_directory(target,"ARCHIVE_DIRECTORY_INVALID")
    raw=read(target/"ARCHIVE.json",MAX_MANIFEST)
    need(digest(raw)==expected_pin,"ARCHIVE_PIN_MISMATCH")
    need(read(target/"ARCHIVE.sha256",65)==(expected_pin+"\n").encode("ascii"),"ARCHIVE_MARKER_INVALID")
    m=prior.forensic.audit.decode_json(raw,"ARCHIVE_MANIFEST_INVALID")
    need(type(m) is dict and set(m)=={"schema","pins","original_audit","files"} and m["schema"]==SCHEMA,"ARCHIVE_SCHEMA_INVALID")
    need(set(m["pins"])=={"old_pin","new_pin","suffix_pin","engine_pin","source_pin","final_suffix_pin"} and
         all(type(p) is str and PIN.fullmatch(p) is not None for p in m["pins"].values()),"ARCHIVE_PINS_INVALID")
    full=verify_payload(target,m)
    return {"schema":SCHEMA,"status":"ARCHIVE_VERIFIED","archive_sha256":expected_pin,
            "verified_native_jobs":full["verified_native_jobs"],"execution_authorized":False,
            "reservation_release_authorized":False,"automatic_retry":False,
            "original_workspace_identity_reverified":False,"global_antirollback":"NOT_PROVEN",
            "crash_durability":"UNCLAIMED","production":"NOT_AUTHORIZED"}

def archive(target,**args):
    target=target.absolute()
    native.real_directory(target.parent,"ARCHIVE_PARENT_INVALID")
    canonical,_,claim=prior.exclusive.location(args["root"])
    parent=target.parent.resolve(strict=True)
    for path in (args["old"],args["new"],canonical,claim,args["suffix"]):
        p=path.resolve(strict=True)
        need(parent!=p and p not in parent.parents and target.resolve()!=p and
             target.resolve() not in p.parents,"ARCHIVE_OVERLAPS_EVIDENCE")
    need(not target.exists() and not target.is_symlink(),"ARCHIVE_DESTINATION_EXISTS")
    files,report=snapshot(**args)
    pins={k:v for k,v in args.items() if k.endswith("_pin")}
    manifest={"schema":SCHEMA,"pins":pins,"original_audit":report,"files":inventory(files)}
    raw=encoded(manifest)
    need(len(raw)<=MAX_MANIFEST,"ARCHIVE_MANIFEST_TOO_LARGE")
    pin=digest(raw)
    target.mkdir(mode=0o700)
    try:
        for rel,data in sorted(files.items()):
            p=target/rel
            p.parent.mkdir(parents=True,exist_ok=True)
            portable.safe_write(p,data)
        portable.safe_write(target/"ARCHIVE.json",raw)
        verify_payload(target,manifest)
        again,again_report=snapshot(**args)
        need(again==files and again_report==report,"SOURCE_CHANGED_BEFORE_PUBLICATION")
        portable.safe_write(target/"ARCHIVE.sha256",(pin+"\n").encode("ascii"))
        result=verify(target,pin)
        result["status"]="ARCHIVE_CREATED_VERIFIED"
        result["reservation_retained"]=True
        return result
    except Exception as exc:
        raise prior.forensic.Blocked("ARCHIVE_PUBLICATION_UNKNOWN_NO_REPLAY") from exc

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    a=sub.add_parser("archive")
    for name in ("target","old","root","suffix","new"):
        a.add_argument("--"+name,type=Path,required=True)
    for name in ("old-pin","suffix-pin","new-pin","engine-pin","source-pin","final-suffix-pin"):
        a.add_argument("--"+name,required=True)
    v=sub.add_parser("verify")
    v.add_argument("--target",type=Path,required=True)
    v.add_argument("--expected-pin",required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        result=archive(**args) if action=="archive" else verify(**args)
        rc=0
    except Exception as exc:
        result={"schema":SCHEMA,"status":"BLOCK","reason":getattr(exc,"code","ARCHIVE_NOT_VERIFIED"),
                "execution_authorized":False,"reservation_release_authorized":False,
                "automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(result,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
