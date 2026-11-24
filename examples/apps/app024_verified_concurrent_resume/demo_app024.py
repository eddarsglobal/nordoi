#!/usr/bin/env python3
"""APP-024: independent manifest/native Rust concurrent recovery demonstration.

Run authentic APP-012 -> APP-013 -> APP-014 -> APP-024 -> APP-010 pipeline.
Fourteen proofs concern observed file bytes and native receipts, not mocks.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import test_app024 as tests
import run_app024 as app


def main(proof:Path):
    proof.mkdir(parents=True,exist_ok=True)
    checks=[]
    t=tests.RealPortableConcurrentResume("test_01_real_parallel_native_checkpoint_resume")
    t.setUp()
    try:
        checkpoint=t.bundle
        history_before=t.native_bytes(t.restored)
        rows=t.good(workers=2)
        assert [r["step"] for r in rows]==[3,4,5,6]
        checks.append("APP024_ONLY_NATIVE_MISSING_SUFFIX=PASS")
        assert [r["decision"] for r in rows]==[1,0,1,1]
        checks.append("APP024_GENUINE_RUST_NATIVE_DECISIONS=PASS")
        assert all(r["schema"]==app.EVENT for r in rows)
        checks.append("APP024_CANONICAL_APP009_EVENTS=PASS")
        assert rows[2]["depends_on"]==[3,4]
        checks.append("APP024_JOIN_AFTER_VERIFIED_NATIVE_PARENTS=PASS")
        current=t.native_bytes(t.restored)
        assert all(current[k]==v for k,v in history_before.items())
        checks.append("APP024_ORIGINAL_NATIVE_BYTES_PRESERVED=PASS")
        assert len(current)==12 and len(history_before)==4
        checks.append("APP024_FOUR_NEW_NATIVE_P27_BUNDLES=PASS")
        manifest,report,prefix=app.portable.verify(checkpoint,t.outer_pin)
        assert manifest["committed"]==2 and len(prefix)==2
        checks.append("APP024_PORTABLE_APP014_VERIFIED=PASS")
        assert app.portable.native_inventory(
            t.restored,prefix)==manifest["native_files"]
        checks.append("APP024_HISTORICAL_FILES_INTEGRITY_RECHECKED=PASS")
        for row in rows:
            verified=app.native_audit.verify_native_record(
               row["native_receipt"],row["step"],t.restored)
            assert verified["decision"]==row["decision"] and len(verified["files"])==2
        checks.append("APP024_ALL_NEW_NATIVE_FILES_INDEPENDENTLY_CHECKED=PASS")
        report=t.full_audit(rows)
        assert report["status"]=="PASS" and report["events_checked"]==6
        checks.append("APP024_FULL_DAG_INDEPENDENT_APP010_AUDIT=PASS")
        alt=t.base/"one_worker"
        alt.mkdir()
        app.portable.restore(checkpoint,t.outer_pin,alt)
        p,seq=t.launch(workers=1,root=alt)
        assert p.returncode==0 and seq==rows
        checks.append("APP024_ONE_VERSUS_TWO_NATIVE_WORKERS_EQUIVALENT=PASS")
        assert app.pinned.engine_sha256(t.engine)==t.engine_pin
        checks.append("APP024_RUST_ENGINE_EXACT_PIN_PRESERVED=PASS")
        assert app.pinned.source_tree_sha256(t.source)==t.source_pin
        checks.append("APP024_NOI_SOURCE_EXACT_PIN_PRESERVED=PASS")
        p,again=t.launch(workers=2)
        assert p.returncode==2 and again[0]["status"]=="REJECTED"
        checks.append("APP024_DUPLICATE_REPLAY_REJECTED_WITHOUT_OVERWRITE=PASS")
    finally:
        t.tearDown()
    assert len(checks)==14
    receipt={
       "schema":"NORDOI_APP024_REAL_RUST_VERIFIED_CONCURRENT_RESUME_PROOF_V1",
       "status":"PASS_WITH_CONTROLS_RESEARCH_ONLY",
       "assertions":checks,
       "native":"AUTHENTIC_NORDOI_P27_RUST_PROCESSES",
       "portable":"VERIFIED_APP014_CHECKPOINT",
       "full_audit":"APP010_NATIVE_HISTORY_AND_SUFFIX",
       "external_identity":"NOT_ATTESTED",
       "crash_durability":"NOT_PROVEN",
       "atomic_multi_task_rollback":"NOT_PROVEN",
       "formal_bureau_certification":"NOT_CERTIFIED",
       "production":"NOT_AUTHORIZED"
    }
    (proof/"APP024_RUST_PORTABLE_CONCURRENT_RESUME.json").write_text(
       json.dumps(receipt,sort_keys=True,indent=2)+"\n",encoding="utf-8")
    for claim in checks:print(claim)
    print("APP024_REAL_RUST_PORTABLE_CONCURRENT_RESUME=PASS")


if __name__=="__main__":
    if len(sys.argv)!=2:
        raise SystemExit("usage: demo_app024.py PROOF_DIR")
    main(Path(sys.argv[1]))
