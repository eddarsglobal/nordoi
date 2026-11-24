#!/usr/bin/env python3
"""APP-024: resume a REAL restored APP-014 checkpoint using concurrent Rust.

1. Full portable checkpoint verification against a host-supplied SHA256 pin.
2. Independent APP-010 audit of the RESTORED native P2.7 prefix.
3. Run only missing DAG tasks through actual APP-023 Rust native workers (1/2).
4. Publish only deterministic APP-009 JSONL suffix; independently audit
   concatenated original prefix+suffix on successful completion.

No rewind of UNKNOWN/in-flight operations, global multi-job atomicity,
sandbox against hostile local administrator, signed external identity,
durable recovery or production authority. Only genuine Rust is called;
Python does not implement the .noi business decision function.
"""
from __future__ import annotations
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app014_portable_checkpoint"))
import portable_app014 as portable
sys.path.insert(0,str(HERE.parent/"app023_bounded_concurrent_dag"))
import run_app023 as concurrent
sys.path.insert(0,str(HERE.parent/"app011_verified_resume"))
import run_app011 as previous

pinned=concurrent.pinned
auditor=portable.independent
native_audit=portable.audit
EVENT="NORDOI_AGENT_DAG_EVENT_V1"


def need(ok,reason):
    if not ok:
        raise portable.Refused(reason)


def json_line(obj,newline=b"\n"):
    return json.dumps(obj,sort_keys=True,separators=(",",":"),
                      ensure_ascii=True).encode("utf-8")+newline


def convert(result:dict)->dict:
    """APP-023 verified native result -> APP-009 independently audited event."""
    item={k:v for k,v in result.items() if k!="schema"}
    item["schema"]=EVENT
    return item


def error(index:int,node:dict,plan_sha:str,reason:str):
    return {
        "schema":EVENT,"step":index,"status":"ERROR",
        "gate":node["gate"],"depends_on":node["depends_on"],
        "plan_sha256":plan_sha,"error_code":reason,"publication":"UNKNOWN"
    }


def check_history(portable_dir:Path,sha:str,restored:Path,
                  manifest:dict,original:list[dict]):
    current,_,events=portable.verify(portable_dir,sha)
    need(current==manifest and events==original,
         "CHECKPOINT_CHANGED_DURING_RESUME")
    portable.restored_root_matches(restored,portable_dir,events,manifest)


def run(checkpoint:Path,expected_checkpoint:str,restored:Path,
        engine:Path,source:Path,engine_sha:str,source_sha:str,
        limit:int)->int:
    # Validation is fully read-only until the first real native task begins.
    need(type(limit) is int and limit in (1,2),"APP024_CONCURRENCY_LIMIT_INVALID")
    for x in (expected_checkpoint,engine_sha,source_sha):
        need(type(x) is str and pinned.PIN.fullmatch(x) is not None,
             "APP024_TRUSTED_SHA256_PIN_REQUIRED")
    manifest,report,historical=portable.verify(checkpoint,expected_checkpoint)
    plan=checkpoint/"handoff"/"plan.json"
    prefix=checkpoint/"handoff"/"native-prefix.jsonl"
    nodes,plan_sha=auditor.plan_nodes(plan,manifest["plan_sha256"])
    raw=native_audit.bounded_regular_file(prefix,auditor.MAX_EVENTS_BYTES,
                                         "CHECKPOINT_PREFIX_FILE_INVALID")
    need(hashlib.sha256(raw).hexdigest()==manifest["prefix_sha256"],
         "CHECKPOINT_PREFIX_PIN_CHANGED")
    need(0<len(historical)<len(nodes),"CHECKPOINT_NOT_CLEAN_INCOMPLETE")
    need(all(row["status"] in ("COMMITTED","SKIPPED") for row in historical),
         "CHECKPOINT_UNCERTAIN_JOB_REFUSED")
    # Must not nest restored jobs inside the package or the source tree.
    native_audit.real_directory(restored,"RESTORED_ROOT_INVALID")
    try:
        r=restored.resolve(strict=True)
        p=checkpoint.resolve(strict=True)
        s=source.resolve(strict=True)
        need(r!=p and r not in p.parents and p not in r.parents,
             "RESTORED_ROOT_OVERLAPS_CHECKPOINT")
        need(r!=s and r not in s.parents and s not in r.parents,
             "RESTORED_ROOT_OVERLAPS_SOURCE")
    except OSError:
        raise portable.Refused("RESTORED_ROOT_OR_SOURCE_UNRESOLVABLE") from None
    portable.restored_root_matches(restored,checkpoint,historical,manifest)
    need(pinned.engine_sha256(engine)==engine_sha,"ENGINE_PIN_MISMATCH")
    need(pinned.source_tree_sha256(source)==source_sha,"SOURCE_PIN_MISMATCH")
    newline=b"\r\n" if raw.endswith(b"\r\n") else b"\n"
    decisions=[x["native_receipt"]["decision"]
               if x["status"]=="COMMITTED" else None for x in historical]
    finished=set(range(1,len(historical)+1))
    new_events={}
    in_flight={}
    effect_started=False
    failed=False

    def make_worker_failure(i):
        return error(i,nodes[i-1],plan_sha,"NATIVE_WORKER_UNKNOWN")

    def drain():
        nonlocal in_flight,failed
        admitted=in_flight
        in_flight={}
        clean=True
        for index in sorted(admitted):
            try:
                item=convert(admitted[index].result())
                need(item["step"]==index and item["plan_sha256"]==plan_sha,
                     "WORKER_EVENT_BINDING_INVALID")
            except Exception:
                # This is already an attempted native effect, outcome unknown.
                item=make_worker_failure(index)
            new_events[index]=item
            finished.add(index)
            if item["status"]=="COMMITTED":
                decisions[index-1]=item["decision"]
            else:
                clean=False
        if not clean:
            failed=True
        return clean

    with ThreadPoolExecutor(max_workers=limit,
                            thread_name_prefix="nordoi-app024") as pool:
        for index in range(len(historical)+1,len(nodes)+1):
            node=nodes[index-1]
            if in_flight and (len(in_flight)>=limit or
                              any(x in in_flight for x in node["depends_on"])):
                if not drain():
                    break
            if failed:
                break
            # Verify historical source and P2.7 bytes before EVERY batch
            # admission, rather than relying on cached JSON.
            try:
                check_history(checkpoint,expected_checkpoint,restored,
                              manifest,historical)
                need(pinned.engine_sha256(engine)==engine_sha and
                     pinned.source_tree_sha256(source)==source_sha,
                     "TRUSTED_ENGINE_OR_SOURCE_CHANGED")
                need(all(dep in finished for dep in node["depends_on"]),
                     "UNRESOLVED_VERIFIED_DEPENDENCY")
            except (portable.Refused,portable.inner.Refused,
                    pinned.Denied,native_audit.AuditRejected,OSError,ValueError):
                if in_flight:
                    drain()
                # After any previous native effect cannot claim NOT_STARTED.
                if effect_started:
                    new_events[index]=error(index,node,plan_sha,
                                            "TRUSTED_HISTORY_CHANGED")
                    failed=True
                    break
                raise
            if not auditor.gate_true(node["gate"],node["depends_on"],decisions):
                new_events[index]=previous.producer.make_event(
                    index,"SKIPPED",node["gate"],node["depends_on"],plan_sha,
                    reason="DEPENDENCY_GATE_FALSE")
                finished.add(index)
                continue
            effect_started=True
            in_flight[index]=pool.submit(concurrent.native_job,index,node,
                 plan_sha,restored,engine,source,engine_sha,source_sha)
        else:
            if in_flight:
                drain()
    suffix=b"".join(json_line(new_events[k],newline)
                    for k in sorted(new_events))
    if not failed and len(historical)+len(new_events)==len(nodes):
        # An independent checker reopens every original and new native file,
        # validates graph semantics and catches extra/unexpected job folders.
        # Candidate suffix is NOT emitted until this full audit succeeds.
        try:
            check_history(checkpoint,expected_checkpoint,restored,
                          manifest,historical)
            with tempfile.TemporaryDirectory(prefix="nordoi_app024_full_audit_") as t:
                joined=Path(t)/"joined.jsonl"
                joined.write_bytes(raw+suffix)
                report,status=auditor.audit(restored,plan,joined,plan_sha)
                need(status==0 and report["status"]=="PASS"
                     and report["events_checked"]==len(nodes),
                     "COMPLETED_NATIVE_DAG_AUDIT_FAILED")
        except (portable.Refused,portable.inner.Refused,
                native_audit.AuditRejected,pinned.Denied,OSError,ValueError):
            if effect_started:
                failed=True
            else:
                raise
    else:
        failed=True
    if failed:
        # An errored concurrent wave may have completed a sibling after the
        # first error; such a partial history is NOT a resumable APP-011
        # checkpoint. File effects must be treated as UNKNOWN until audited.
        sys.stdout.buffer.write(suffix)
        sys.stdout.buffer.flush()
        return 3
    sys.stdout.buffer.write(suffix)
    sys.stdout.buffer.flush()
    return 0


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--portable-dir",required=True)
    p.add_argument("--expected-portable-sha256",required=True)
    p.add_argument("--restore-root",required=True)
    p.add_argument("--engine",required=True)
    p.add_argument("--source-root",required=True)
    p.add_argument("--expected-engine-sha256",required=True)
    p.add_argument("--expected-source-sha256",required=True)
    p.add_argument("--max-in-flight",type=int,choices=(1,2),required=True)
    a=p.parse_args(argv)
    try:
        return run(Path(a.portable_dir),a.expected_portable_sha256,
                   Path(a.restore_root),Path(a.engine),Path(a.source_root),
                   a.expected_engine_sha256,a.expected_source_sha256,
                   a.max_in_flight)
    except (portable.Refused,portable.inner.Refused,
            pinned.Denied,native_audit.AuditRejected,OSError,ValueError) as exc:
        portable.emit({"schema":"NORDOI_APP024_CONCURRENT_RESUME_RESULT_V1",
              "status":"REJECTED","error_code":getattr(exc,"code",
              "APP024_INPUT_NOT_AUTHORIZED"),"publication":"NOT_STARTED"})
        return 2


if __name__=="__main__":
    raise SystemExit(main())
