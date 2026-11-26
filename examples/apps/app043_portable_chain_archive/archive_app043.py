#!/usr/bin/env python3
"""Bounded canonical portable evidence archive, replayed with APP-042.

Only allowlisted evidence bytes are collected. Reconstruction is isolated in
a temporary directory. External archive, original plan and anchor pins remain
required. No automatic collection on every transition or global freshness.
"""
from __future__ import annotations
import argparse
import base64
import json
from pathlib import Path
import re
import sys
import tempfile
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app042_verified_seal_chain"))
import chain_app042 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
seal=prior.prior.prior.prior
SCHEMA="NORDOI_APP043_PORTABLE_CHAIN_ARCHIVE_V1"
MAX_ARCHIVE=32*1024*1024
MAX_BYTES=20*1024*1024
MAX_FILES=16384
MAX_FILE=prior.MAX_PLAN
HEX=r"[0-9a-f]{64}"
SAFE=re.compile(r"(?:successors/"+HEX+r"/(?:START|TRANSITION|COMPLETE)\.json|steps/[0-9]{2}/(?:old-seal|new-seal|old-contexts|new-contexts)\.json|steps/[0-9]{2}/registry/"+HEX+r"/(?:EXPECTED\.json|accepted/(?:START\.json|COMPLETE\.json|ACK\.json|ACK\.sig|RECEIVER\.pub))|steps/[0-9]{2}/journal/"+HEX+r"/"+HEX+r"/(?:START\.json|COMPLETE\.json|(?:original|conflict)-(?:ACK\.json|ACK\.sig|RECEIVER\.pub)))\Z")

def collect(successors,evidence_root,steps):
    root=registry.root(evidence_root)
    files={}
    total=0
    def add(name,path):
        nonlocal total
        need(SAFE.fullmatch(name) is not None and name not in files,"ARCHIVE_MEMBER_INVALID")
        raw=registry.read(path,MAX_FILE)
        total+=len(raw)
        need(total<=MAX_BYTES and len(files)<MAX_FILES,"ARCHIVE_BUDGET_EXCEEDED")
        files[name]=base64.b64encode(raw).decode("ascii")
    normalized=[]
    for i,step in enumerate(steps):
        prefix=f"steps/{i:02d}"
        resolved={k:prior.evidence_path(root,step[k]) for k in prior.PATHS}
        target={k:step[k] for k in prior.PINS}
        for k in ("old_seal","new_seal","old_contexts","new_contexts"):
            name=prefix+"/"+k.replace("_","-")+".json"
            add(name,resolved[k])
            target[k]=name
        target.update(registry_path=prefix+"/registry",journal=prefix+"/journal")
        contexts=seal.load_contexts(resolved["new_contexts"],step["new_contexts_pin"])
        doc,_,_=prior.prior.prior.load_seal(resolved["new_seal"],step["new_pin"],
                                             step["new_contexts_pin"],contexts)
        for nonce in sorted(contexts):
            reg=resolved["registry_path"]/nonce
            add(prefix+"/registry/"+nonce+"/EXPECTED.json",reg/"EXPECTED.json")
            for n in ("START.json","COMPLETE.json","ACK.json","ACK.sig","RECEIVER.pub"):
                add(prefix+"/registry/"+nonce+"/accepted/"+n,reg/"accepted"/n)
        for row in doc["entries"]:
            nonce,pin=row["nonce"],row["conflict_sha256"]
            path=resolved["journal"]/nonce/pin
            for n in sorted(seal.NAMES):
                add(prefix+"/journal/"+nonce+"/"+pin+"/"+n,path/n)
        for n in ("START.json","TRANSITION.json","COMPLETE.json"):
            add("successors/"+step["old_pin"]+"/"+n,successors/step["old_pin"]/n)
        normalized.append(target)
    return files,normalized

def replay(doc,source_plan_pin,anchor_pin):
    need(type(doc) is dict and set(doc)=={"schema","source_plan","source_plan_sha256","anchor_sha256",
         "head_sha256","decisions_sha256","transition_count","files"},"ARCHIVE_SCHEMA_INVALID")
    need(doc["schema"]==SCHEMA and doc["source_plan_sha256"]==source_plan_pin
         and doc["anchor_sha256"]==anchor_pin,"ARCHIVE_TRUST_MISMATCH")
    source=doc["source_plan"]
    need(transfer.sha(portable.serialize(source))==source_plan_pin,"SOURCE_PLAN_PIN_MISMATCH")
    files=doc["files"]
    need(type(files) is dict and 0<len(files)<=MAX_FILES,"ARCHIVE_MEMBER_COUNT_INVALID")
    with tempfile.TemporaryDirectory(prefix="nordoi_app043_") as tmp:
        root=Path(tmp).resolve()
        original=root/"source-plan.json"
        portable.safe_write(original,portable.serialize(source))
        steps,_=prior.plan(original,source_plan_pin)
        normalized=[]
        for i,step in enumerate(steps):
            prefix=f"steps/{i:02d}"
            target={k:step[k] for k in prior.PINS}
            target.update({k:prefix+"/"+k.replace("_","-")+".json" for k in
                          ("old_seal","new_seal","old_contexts","new_contexts")})
            target.update(registry_path=prefix+"/registry",journal=prefix+"/journal")
            normalized.append(target)
        total=0
        for name,value in sorted(files.items()):
            need(SAFE.fullmatch(name) is not None and type(value) is str
                 and len(value)<=4*((MAX_FILE+2)//3),"ARCHIVE_MEMBER_INVALID")
            data=base64.b64decode(value,validate=True)
            need(len(data)<=MAX_FILE and base64.b64encode(data).decode("ascii")==value,"ARCHIVE_BASE64_INVALID")
            total+=len(data)
            need(total<=MAX_BYTES,"ARCHIVE_BUDGET_EXCEEDED")
            path=root.joinpath(*name.split("/"))
            path.parent.mkdir(parents=True,exist_ok=True)
            portable.safe_write(path,data)
        normalized_path=root/"normalized-plan.json"
        raw=portable.serialize({"schema":prior.PLAN_SCHEMA,"steps":normalized})
        portable.safe_write(normalized_path,raw)
        audited=prior.audit(root/"successors",root,normalized_path,transfer.sha(raw),anchor_pin)
        captured,rebuilt=collect(root/"successors",root,normalized)
        need(captured==files and rebuilt==normalized,"ARCHIVE_EXTRA_OR_CHANGED_MEMBER")
        need(audited["head_sha256"]==doc["head_sha256"]
             and audited["decisions_sha256"]==doc["decisions_sha256"]
             and audited["transition_count"]==doc["transition_count"],"ARCHIVE_REPLAY_MISMATCH")
        return audited

def outcome(status,doc,pin):
    return {"schema":SCHEMA,"status":status,"archive_sha256":pin,
            "source_plan_sha256":doc["source_plan_sha256"],"anchor_sha256":doc["anchor_sha256"],
            "head_sha256":doc["head_sha256"],"decisions_sha256":doc["decisions_sha256"],
            "transition_count":doc["transition_count"],"member_count":len(doc["files"]),
            "original_locations_required":False,"global_antirollback":"NOT_PROVEN",
            "distributed_consensus":"NOT_PROVEN","freshness":"NOT_PROVEN",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}

def archive(successors,evidence_root,plan_file,plan_pin,anchor_pin,output):
    root=registry.root(successors)
    prior.prior.native.real_directory(output.parent,"ARCHIVE_PARENT_INVALID")
    dest=output.parent.resolve(strict=True)/output.name
    need(root!=dest and root not in dest.parents and not registry.present(dest),"ARCHIVE_OUTPUT_INVALID")
    steps,raw_plan=prior.plan(plan_file,plan_pin)
    for step in steps:
        for key in ("registry_path","journal"):
            path=prior.evidence_path(registry.root(evidence_root),step[key])
            need(path!=dest and path not in dest.parents,"ARCHIVE_OUTPUT_OVERLAP")
    audited=prior.audit(root,evidence_root,plan_file,plan_pin,anchor_pin)
    files,_=collect(root,evidence_root,steps)
    doc={"schema":SCHEMA,"source_plan":json.loads(raw_plan),"source_plan_sha256":plan_pin,
         "anchor_sha256":anchor_pin,"head_sha256":audited["head_sha256"],
         "decisions_sha256":audited["decisions_sha256"],"transition_count":audited["transition_count"],
         "files":files}
    replay(doc,plan_pin,anchor_pin)
    need(collect(root,evidence_root,steps)[0]==files
         and prior.audit(root,evidence_root,plan_file,plan_pin,anchor_pin)==audited,
         "SOURCE_CHAIN_CHANGED")
    raw=portable.serialize(doc)
    need(len(raw)<=MAX_ARCHIVE,"ARCHIVE_TOO_LARGE")
    portable.safe_write(dest,raw)
    need(registry.read(dest,MAX_ARCHIVE)==raw,"ARCHIVE_PUBLICATION_MISMATCH")
    return outcome("CHAIN_ARCHIVE_CREATED",doc,transfer.sha(raw))

def verify(output,archive_pin,source_plan_pin,anchor_pin):
    for pin in (archive_pin,source_plan_pin,anchor_pin): transfer.pin(pin)
    raw=registry.read(output,MAX_ARCHIVE)
    need(transfer.sha(raw)==archive_pin,"ARCHIVE_PIN_MISMATCH")
    doc=json.loads(raw)
    need(raw==portable.serialize(doc),"ARCHIVE_NOT_CANONICAL")
    replay(doc,source_plan_pin,anchor_pin)
    need(registry.read(output,MAX_ARCHIVE)==raw,"ARCHIVE_CHANGED_DURING_REPLAY")
    return outcome("CHAIN_ARCHIVE_VERIFIED",doc,archive_pin)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    a=sub.add_parser("archive")
    for n in ("successors","evidence-root","plan-file","output"): a.add_argument("--"+n,type=Path,required=True)
    for n in ("plan-pin","anchor-pin"): a.add_argument("--"+n,required=True)
    v=sub.add_parser("verify")
    v.add_argument("--output",type=Path,required=True)
    for n in ("archive-pin","source-plan-pin","anchor-pin"): v.add_argument("--"+n,required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"archive":archive,"verify":verify}[action](**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED","reason":getattr(exc,"code","ARCHIVE_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
