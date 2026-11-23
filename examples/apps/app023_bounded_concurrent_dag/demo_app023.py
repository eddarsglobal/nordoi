#!/usr/bin/env python3
"""APP-023 native proof: two real Rust jobs admitted in one bounded wave.

Business decisions are computed by compiled nordoi, not by Python. The
scheduler uses a ThreadPoolExecutor(max_workers=2). No measured throughput
improvement, cross-host scheduling or real administrative sandbox is claimed.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import test_app023 as tests
import run_app023 as app


def run(proof:Path):
    proof.mkdir(parents=True,exist_ok=True)
    claims=[]
    a=tests.NativeConcurrentDAG("test_01_three_node_parallel_roots_and_native_join")
    a.setUp()
    try:
        result=a.good(["COMMITTED"]*3,limit=2)
        assert [x["decision"] for x in result]==[1,0,1]
        claims.append("APP023_CONCURRENT_ROOTS_AND_JOIN_REAL_RUST=PASS")
        assert [x["step"] for x in result]==[1,2,3]
        claims.append("APP023_CANONICAL_INDEX_ORDER=PASS")
        assert result[2]["depends_on"]==[1,2]
        claims.append("APP023_JOIN_AFTER_BOTH_NATIVE_PARENTS=PASS")
        for row in result:
            evidence=app.audit.verify_native_record(
                row["native_receipt"],row["step"],a.grant)
            assert evidence["decision"]==row["decision"]
            assert len(evidence["files"])==2
        claims.append("APP023_ALL_NATIVE_P27_FILES_REAUDITED=PASS")
        assert sorted(x.name for x in a.grant.iterdir())==[
            "job-0001","job-0002","job-0003"]
        claims.append("APP023_DISTINCT_EXCLUSIVE_NATIVE_JOB_DIRECTORIES=PASS")
        assert all(x["native_receipt"]["publication"]=="DIRECTORY_RENAME"
                   for x in result)
        claims.append("APP023_GENUINE_P27_PUBLICATION=PASS")
        assert all(x["plan_sha256"]==result[0]["plan_sha256"] for x in result)
        claims.append("APP023_PLAN_PIN_SHARED_ACROSS_WAVE=PASS")
        assert str(a.root) not in json.dumps(result)
        claims.append("APP023_NO_HOST_ABSOLUTE_PATH_IN_RECEIPTS=PASS")
        other=a.root/"sequential_output"
        other.mkdir()
        p, sequential=a.run_cli(limit=1,grant=other)
        assert p.returncode==0 and sequential==result
        claims.append("APP023_SINGLE_VERSUS_TWO_WORKERS_EQUIVALENT=PASS")
        assert app.pinned.engine_sha256(a.engine)==a.engine_pin
        claims.append("APP023_RUST_ENGINE_PIN_UNCHANGED=PASS")
        assert app.pinned.source_tree_sha256(a.source)==a.source_pin
        claims.append("APP023_NOI_SOURCE_PIN_UNCHANGED=PASS")
    finally:
        a.tearDown()
    b=tests.NativeConcurrentDAG("test_01_three_node_parallel_roots_and_native_join")
    b.setUp()
    try:
        b.write([tests.node(73),tests.node(59),
                 tests.node(60,[1,2],"all_approved")])
        rows=b.good(["COMMITTED","COMMITTED","SKIPPED"])
        assert not (b.grant/"job-0003").exists()
        claims.append("APP023_SKIPPED_GATE_CREATES_NO_OUTPUT=PASS")
        assert rows[-1]["reason"]=="DEPENDENCY_GATE_FALSE"
        claims.append("APP023_ONLY_VERIFIED_PARENT_DECISIONS_USED=PASS")
    finally:
        b.tearDown()
    c=tests.NativeConcurrentDAG("test_01_three_node_parallel_roots_and_native_join")
    c.setUp()
    try:
        c.write([tests.node(73,[2],"all_committed"),tests.node(59)])
        c.deny("DAG_REFERENCE_NOT_PRIOR")
        claims.append("APP023_INVALID_DAG_REFUSED_BEFORE_ANY_FILEWRITE=PASS")
    finally:
        c.tearDown()
    assert len(claims)==14
    receipt={
        "schema":"NORDOI_APP023_REAL_NATIVE_BOUNDED_CONCURRENCY_PROOF_V1",
        "status":"PASS_WITH_CONTROLS_RESEARCH_ONLY",
        "assertions":claims,
        "rust_engine":"GENUINE_NATIVE_PROCESS_PER_ADMITTED_JOB",
        "scheduler":"PYTHON_BOUNDED_THREADPOOL_1_OR_2_REAL_NATIVE_PROCESSES",
        "dependency_join":"VERIFIED_NATIVE_RECEIPTS_BEFORE_CHILD",
        "parallel_speed_gain":"NOT_MEASURED",
        "global_atomicity":"NOT_PROVIDED",
        "os_sandbox":"NOT_PROVEN",
        "production":"NOT_AUTHORIZED",
        "formal_bureau_certification":"NOT_CERTIFIED"
    }
    (proof/"APP023_REAL_NATIVE_CONCURRENT_DAG.json").write_text(
       json.dumps(receipt,sort_keys=True,indent=2)+"\n",encoding="utf-8")
    for claim in claims:
        print(claim)
    print("APP023_GENUINE_NORDOI_RUST_CONCURRENT_DAG_DEMO=PASS")


if __name__=="__main__":
    if len(sys.argv)!=2:
        raise SystemExit("usage: demo_app023.py PROOF_DIR")
    run(Path(sys.argv[1]))
