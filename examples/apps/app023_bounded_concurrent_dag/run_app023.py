#!/usr/bin/env python3
"""APP-023: bounded concurrent AI-agent DAG with REAL NORDOI Rust execution.

The validated APP-009 DAG stays unchanged. Independent nodes are submitted
concurrently to a bounded ThreadPoolExecutor (1 or 2 workers), each running
an authentic Rust nordoi agent-request-json process. A dependent node NEVER
starts before all its parents' genuine native receipts have been verified.
Outputs are deterministic by graph index, not finish order.

No arbitrary code execution from agent JSON. Engine, trusted pins, sources
and output grant remain outside the hostile plan. Failures drain already
running jobs and stop future admissions; there is NO group rollback, global
atomicity, OS sandbox, fairness, durable restart or production authority.
"""
from __future__ import annotations
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app009_agent_dag"))
import run_app009 as dag

pinned=dag.pinned
audit=dag.audit
old=dag.previous
EVENT="NORDOI_AGENT_BOUNDED_CONCURRENT_DAG_EVENT_V1"


def record(index:int,status:str,node:dict|None,plan_sha256:str|None,**more):
    return {
      "schema":EVENT,
      "step":index,
      "status":status,
      "gate":node["gate"] if node is not None else None,
      "depends_on":node["depends_on"] if node is not None else [],
      "plan_sha256":plan_sha256,
      **more
    }


def native_job(index:int,node:dict,plan_hash:str,grant:Path,
               engine:Path,source:Path,engine_pin:str,source_pin:str):
    target=grant/f"job-{index:04d}"
    try:
        target.mkdir(mode=0o700)
    except OSError:
        return record(index,"ERROR",node,plan_hash,
                      error_code="JOB_DIRECTORY_CREATE_FAILED",
                      publication="UNKNOWN")
    request=json.dumps({
        "schema":"NORDOI_AGENT_REQUEST_V1","key_code":node["key_code"]
    },separators=(",",":")).encode("utf-8")+b"\n"
    try:
        process=subprocess.run(
            [str(engine.resolve(strict=True)),"agent-request-json",
             str(source.resolve(strict=True)),"app.main",
             "--grant-output-dir",str(target)],
            input=request,capture_output=True,
            cwd=HERE.parents[2],timeout=120)
    except (OSError,subprocess.TimeoutExpired):
        return record(index,"ERROR",node,plan_hash,
                      error_code="NATIVE_EXECUTION_INCOMPLETE",
                      publication="UNKNOWN")
    if process.returncode!=0:
        return record(index,"ERROR",node,plan_hash,
                      error_code="NATIVE_OPERATION_FAILED",
                      native_exit_code=process.returncode,
                      publication="UNKNOWN")
    try:
        native=old.native_json(process.stdout)
        audit.verify_native_record(native,index,grant)
        dag.require(native["key_code"]==node["key_code"],
                    "NATIVE_KEY_CODE_CONTRADICTION")
        dag.require(pinned.engine_sha256(engine)==engine_pin,
                    "ENGINE_CHANGED_DURING_CONCURRENT_WORKFLOW")
        dag.require(pinned.source_tree_sha256(source)==source_pin,
                    "SOURCE_CHANGED_DURING_CONCURRENT_WORKFLOW")
    except (pinned.Denied,audit.AuditRejected,KeyError,TypeError) as exc:
        return record(index,"ERROR",node,plan_hash,
                      error_code="NATIVE_RESULT_NOT_VERIFIED",
                      error_detail=getattr(exc,"code","NATIVE_RECORD_INVALID"),
                      publication="UNKNOWN")
    return record(index,"COMMITTED",node,plan_hash,
                  job=f"job-{index:04d}",decision=native["decision"],
                  native_receipt=native)


def run(plan:Path,engine:Path,source:Path,grant:Path,
        expected_engine:str,expected_source:str,limit:int)->int:
    # Validate entire hostile plan, pins and grant BEFORE any effect.
    dag.require(type(limit) is int and 1<=limit<=2,
                "CONCURRENT_IN_FLIGHT_LIMIT_INVALID")
    for digest in (expected_engine,expected_source):
        dag.require(type(digest) is str and
                    pinned.PIN.fullmatch(digest) is not None,
                    "INVALID_PIN_FORMAT")
    nodes,plan_hash=dag.load_plan(plan)
    dag.require(pinned.engine_sha256(engine)==expected_engine,
                "ENGINE_PIN_MISMATCH")
    dag.require(pinned.source_tree_sha256(source)==expected_source,
                "SOURCE_PIN_MISMATCH")
    old.validate_grant(grant,source)

    decisions=[None]*len(nodes)
    finished=set()
    events={}
    in_flight={}
    with ThreadPoolExecutor(max_workers=limit,
                            thread_name_prefix="nordoi-app023") as pool:
        def drain()->bool:
            # Drain every already admitted task, even after another fails.
            # Never emit fictitious global rollback of previously published files.
            nonlocal in_flight
            inflight=in_flight
            in_flight={}
            bad=False
            for job_id in sorted(inflight):
                try:
                    result=inflight[job_id].result()
                except (OSError,ValueError,TypeError,KeyError) as exc:
                    result=record(job_id,"ERROR",nodes[job_id-1],plan_hash,
                                  error_code="NATIVE_WORKER_UNKNOWN",
                                  publication="UNKNOWN")
                events[job_id]=result
                finished.add(job_id)
                if result["status"]=="COMMITTED":
                    decisions[job_id-1]=result["decision"]
                else:
                    bad=True
            return not bad

        for i,node in enumerate(nodes,1):
            # A task must never observe a parent that is still in flight.
            # A full batch drains before any additional worker admission.
            if in_flight and (len(in_flight)>=limit
                              or any(dep in in_flight
                                     for dep in node["depends_on"])):
                if not drain():
                    break
            dag.require(all(dep in finished for dep in node["depends_on"]),
                        "INTERNAL_UNRESOLVED_DAG_DEPENDENCY")
            if not dag.should_execute(node["gate"],node["depends_on"],decisions):
                events[i]=record(i,"SKIPPED",node,plan_hash,
                                 reason="DEPENDENCY_GATE_FALSE")
                finished.add(i)
                continue
            in_flight[i]=pool.submit(
                native_job,i,node,plan_hash,grant,engine,source,
                expected_engine,expected_source)
        else:
            if in_flight:
                drain()
        # No job is abandoned: ThreadPoolExecutor has joined all processes.
        # Emit canonical graph-index order, never nondeterministic completion order.
    for i in sorted(events):
        dag.emit(events[i])
    return 3 if any(x["status"]=="ERROR" for x in events.values()) else 0


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--plan",required=True)
    p.add_argument("--engine",required=True)
    p.add_argument("--source-root",required=True)
    p.add_argument("--grant-output-dir",required=True)
    p.add_argument("--expected-engine-sha256",required=True)
    p.add_argument("--expected-source-sha256",required=True)
    p.add_argument("--max-in-flight",type=int,choices=(1,2),required=True)
    args=p.parse_args(argv)
    try:
        return run(Path(args.plan),Path(args.engine),Path(args.source_root),
                   Path(args.grant_output_dir),
                   args.expected_engine_sha256,args.expected_source_sha256,
                   args.max_in_flight)
    except pinned.Denied as exc:
        dag.emit(record(0,"REJECTED",None,None,
                        error_code=exc.code,publication="NOT_STARTED"))
        return 2


if __name__=="__main__":
    raise SystemExit(main())
