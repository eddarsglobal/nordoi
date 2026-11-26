#!/usr/bin/env python3
"""One append-only transition between two externally pinned APP-039 seals.

The old seal is historical trusted input; only the new visible journal is
re-audited. No global head, fork prevention, timestamp or rollback guarantee.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app039_conflict_journal_seal"))
import seal_app039 as prior
registry=prior.registry
transfer=prior.transfer
portable=prior.portable
need=prior.need
SCHEMA="NORDOI_APP040_JOURNAL_SEAL_CONTINUITY_V1"
MAX_TRANSITION_BYTES=65536

def load_seal(path,pin,contexts_pin,contexts):
    transfer.pin(pin)
    raw=registry.read(path,prior.MAX_SEAL_BYTES)
    need(transfer.sha(raw)==pin,"HISTORICAL_SEAL_PIN_MISMATCH")
    obj=json.loads(raw)
    need(type(obj) is dict and set(obj)=={"schema","contexts_sha256","entry_count","entries",
        "inventory_sha256","scope","execution_authorized","production"},"SEAL_SCHEMA_INVALID")
    need(obj["schema"]==prior.SCHEMA and obj["contexts_sha256"]==contexts_pin
         and obj["scope"]=="VISIBLE_JOURNAL_SNAPSHOT" and obj["execution_authorized"] is False
         and obj["production"]=="NOT_AUTHORIZED","SEAL_CONTEXT_INVALID")
    rows=obj["entries"]
    need(type(rows) is list and type(obj["entry_count"]) is int
         and len(rows)==obj["entry_count"] and len(rows)<=prior.MAX_ENTRIES,"SEAL_COUNT_INVALID")
    indexed={}
    for row in rows:
        need(type(row) is dict and set(row)=={"nonce","conflict_sha256","record_sha256","files"},
             "SEAL_ROW_INVALID")
        for k in ("nonce","conflict_sha256","record_sha256"): transfer.pin(row[k])
        need(type(row["files"]) is dict and set(row["files"])==prior.NAMES,"SEAL_FILES_INVALID")
        for digest in row["files"].values(): transfer.pin(digest)
        key=(row["nonce"],row["conflict_sha256"])
        need(key not in indexed,"SEAL_DUPLICATE_ROW")
        indexed[key]=row
    need(list(indexed)==sorted(indexed),"SEAL_ORDER_INVALID")
    need({k[0] for k in indexed}==set(contexts),"SEAL_CONTEXT_SET_MISMATCH")
    need(obj["inventory_sha256"]==transfer.sha(portable.serialize(rows))
         and raw==portable.serialize(obj),"SEAL_CANONICAL_MISMATCH")
    return obj,indexed,raw

def transition(registry_path,journal,old_seal,old_pin,old_contexts,old_contexts_pin,
               new_seal,new_pin,new_contexts,new_contexts_pin):
    old_ctx=prior.load_contexts(old_contexts,old_contexts_pin)
    new_ctx=prior.load_contexts(new_contexts,new_contexts_pin)
    need(all(new_ctx.get(n)==v for n,v in old_ctx.items()),"HISTORICAL_CONTEXT_CHANGED")
    old,old_rows,old_raw=load_seal(old_seal,old_pin,old_contexts_pin,old_ctx)
    new,new_rows,new_raw=load_seal(new_seal,new_pin,new_contexts_pin,new_ctx)
    need(all(new_rows.get(k)==v for k,v in old_rows.items()),"HISTORICAL_EVIDENCE_REMOVED_OR_CHANGED")
    added=sorted(set(new_rows)-set(old_rows))
    need(bool(added),"NO_NEW_CONTRADICTION")
    verified=prior.verify(registry_path,journal,new_contexts,new_contexts_pin,new_seal,new_pin)
    need(verified["status"]=="JOURNAL_SEAL_VERIFIED","NEW_SEAL_NOT_VERIFIED")
    need(registry.read(old_seal,prior.MAX_SEAL_BYTES)==old_raw
         and registry.read(new_seal,prior.MAX_SEAL_BYTES)==new_raw
         and prior.load_contexts(old_contexts,old_contexts_pin)==old_ctx
         and prior.load_contexts(new_contexts,new_contexts_pin)==new_ctx,"TRANSITION_INPUT_CHANGED")
    doc={"schema":SCHEMA,"status":"APPEND_ONLY_TRANSITION",
         "previous_seal_sha256":old_pin,"successor_seal_sha256":new_pin,
         "previous_contexts_sha256":old_contexts_pin,"successor_contexts_sha256":new_contexts_pin,
         "previous_inventory_sha256":old["inventory_sha256"],
         "successor_inventory_sha256":new["inventory_sha256"],
         "preserved_entry_count":len(old_rows),"new_entry_count":len(added),
         "added_entries":[{"nonce":n,"conflict_sha256":p} for n,p in added],
         "execution_authorized":False,"production":"NOT_AUTHORIZED"}
    raw=portable.serialize(doc)
    need(len(raw)<=MAX_TRANSITION_BYTES,"TRANSITION_TOO_LARGE")
    return doc,raw

def outcome(status,doc,pin):
    return {"schema":SCHEMA,"status":status,"transition_sha256":pin,
            "previous_seal_sha256":doc["previous_seal_sha256"],
            "successor_seal_sha256":doc["successor_seal_sha256"],
            "preserved_entry_count":doc["preserved_entry_count"],"new_entry_count":doc["new_entry_count"],
            "fork_prevention":"NOT_PROVEN","global_antirollback":"NOT_PROVEN",
            "historical_seal_trust":"EXTERNAL_PIN_REQUIRED","journal_completeness":"NOT_PROVEN",
            "execution_authorized":False,"reservation_release_authorized":False,
            "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}

def publish(output,**args):
    dest=prior.outside(args["registry_path"],args["journal"],output)
    need(not registry.present(dest),"TRANSITION_ALREADY_EXISTS")
    doc,raw=transition(**args)
    portable.safe_write(dest,raw)
    need(registry.read(dest,MAX_TRANSITION_BYTES)==raw,"TRANSITION_PUBLICATION_MISMATCH")
    return outcome("SEAL_TRANSITION_CREATED",doc,transfer.sha(raw))

def verify(output,transition_pin,**args):
    transfer.pin(transition_pin)
    dest=prior.outside(args["registry_path"],args["journal"],output)
    raw=registry.read(dest,MAX_TRANSITION_BYTES)
    need(transfer.sha(raw)==transition_pin,"TRANSITION_PIN_MISMATCH")
    doc,expected=transition(**args)
    need(raw==expected,"TRANSITION_CONTENT_MISMATCH")
    need(registry.read(dest,MAX_TRANSITION_BYTES)==raw,"TRANSITION_CHANGED_DURING_VERIFY")
    return outcome("SEAL_TRANSITION_VERIFIED",doc,transition_pin)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    for action in ("publish","verify"):
        a=sub.add_parser(action)
        for n in ("registry-path","journal","old-seal","new-seal","old-contexts","new-contexts","output"):
            a.add_argument("--"+n,type=Path,required=True)
        for n in ("old-pin","new-pin","old-contexts-pin","new-contexts-pin"):
            a.add_argument("--"+n,required=True)
        if action=="verify": a.add_argument("--transition-pin",required=True)
    args=vars(p.parse_args(argv))
    action=args.pop("action")
    try:
        out={"publish":publish,"verify":verify}[action](**args)
        rc=0
    except Exception as exc:
        out={"schema":SCHEMA,"status":"UNCERTAIN_BLOCKED","reason":getattr(exc,"code","TRANSITION_NOT_VERIFIED"),
             "execution_authorized":False,"reservation_release_authorized":False,
             "nonce_reset_authorized":False,"automatic_retry":False,"production":"NOT_AUTHORIZED"}
        rc=3
    print(json.dumps(out,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
