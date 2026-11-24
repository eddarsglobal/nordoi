#!/usr/bin/env python3
"""APP-025 genuine Rust recovery classification demonstration.

Observed files were produced by real NORDOI Rust APP-012/APP-024, sealed by
APP-013 and restored by APP-014. This process audits them read-only.
"""
from __future__ import annotations
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import test_app025 as native_tests
import audit_app025 as app


def execute(proof:Path):
    proof.mkdir(parents=True,exist_ok=True)
    claims=[]
    t=native_tests.NativeInterruptedForensicTests(
        "test_01_original_checkpoint_reusable_if_zero_new_effects")
    t.setUp()
    try:
        r=t.good("ORIGINAL_CHECKPOINT_REUSABLE")
        assert r["new_verified_events"]==0
        claims.append("APP025_AUTHENTIC_ORIGINAL_CHECKPOINT_REUSABLE=PASS")
        assert r["verified_native_jobs"]==2
        claims.append("APP025_ORIGINAL_TWO_RUST_JOBS_VERIFIED=PASS")
        original=t.fixture.native_bytes(t.root)
        rows=t.capture_success()
        assert len(rows)==4
        claims.append("APP025_GENUINE_APP024_FOUR_NEW_NATIVE_JOBS=PASS")
        r=t.good("COMPLETE_VERIFIED")
        assert r["verified_events_total"]==6
        claims.append("APP025_COMPLETE_SIX_STEP_DAG_AUDIT=PASS")
        assert r["verified_native_jobs"]==6
        claims.append("APP025_SIX_REAL_NATIVE_P27_BUNDLES_AUDITED=PASS")
        assert all(t.fixture.native_bytes(t.root)[k]==v for k,v in original.items())
        claims.append("APP025_ORIGINAL_P27_BYTES_UNCHANGED=PASS")
        assert r["next_action"]=="DO_NOT_REPLAY_COMPLETED_DAG"
        claims.append("APP025_COMPLETE_REPLAY_FORBIDDEN=PASS")
        assert r["native_rust_processes_started_by_auditor"]==0
        claims.append("APP025_AUDITOR_STARTS_NO_NATIVE_JOBS=PASS")
        assert r["actual_user_data_files_written"]==0
        claims.append("APP025_AUDITOR_CHANGES_NO_USER_FILES=PASS")
        assert r["global_antirollback"]=="NOT_PROVEN"
        claims.append("APP025_NO_FICTITIOUS_GLOBAL_ANTIROLLBACK=PASS")
    finally:
        t.tearDown()
    q=native_tests.NativeInterruptedForensicTests(
        "test_03_single_verified_new_job_requires_new_checkpoint")
    q.setUp()
    try:
        q.fresh_partial(1)
        r=q.good("CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT")
        assert r["new_verified_events"]==1
        claims.append("APP025_ONE_NEW_GENUINE_RUST_JOB_CLEAN_PREFIX=PASS")
        assert r["next_action"]=="SEAL_NEW_CHECKPOINT_BEFORE_ANY_FURTHER_RESUME"
        claims.append("APP025_PARTIAL_PREFIX_NEEDS_NEW_CHECKPOINT=PASS")
    finally:
        q.tearDown()
    b=native_tests.NativeInterruptedForensicTests(
        "test_08_concurrent_orphan_without_suffix_is_blocked")
    b.setUp()
    try:
        b.capture_success()
        b.write_suffix(b"")
        report=b.blocked("ROOT_UNEXPECTED_JOB_OR_FILE")
        assert report["next_action"]=="STOP_NO_AUTOMATIC_NATIVE_REPLAY"
        claims.append("APP025_UNLOGGED_RUST_EFFECT_BLOCKS_REPLAY=PASS")
        assert report["status"]=="UNCERTAIN_BLOCKED"
        claims.append("APP025_UNKNOWN_PARTIAL_WAVE_NEVER_PROMOTED=PASS")
    finally:
        b.tearDown()
    assert len(claims)==14
    receipt={
      "schema":"NORDOI_APP025_REAL_RUST_INTERRUPTED_DAG_RECOVERY_PROOF_V1",
      "status":"PASS_WITH_CONTROLS_RESEARCH_ONLY",
      "assertions":claims,
      "native_data_source":"REAL_RUST_APP012_APP024_P27",
      "envelope":"APP013_SEALED_AND_APP014_PORTABLE",
      "audit":"APP010_INDEPENDENT_NATIVE_DAG_AND_FILES",
      "native_execution_during_forensic_audit":0,
      "external_trust_anchor":"REQUIRES_INDEPENDENT_HUMAN_OR_AGENT_TRUSTED_PIN_PROVENANCE",
      "restart_safe_antirollback":"NOT_PROVEN",
      "formal_bureau_certification":"NOT_CERTIFIED",
      "production":"NOT_AUTHORIZED"
    }
    (proof/"APP025_REAL_NATIVE_FORENSICS.json").write_text(
        json.dumps(receipt,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    for item in claims:
        print(item)
    print("APP025_REAL_RUST_INTERRUPTED_DAG_FORENSICS=PASS")


if __name__=="__main__":
    if len(sys.argv)!=2:
        raise SystemExit("usage: demo_app025.py PROOF_DIR")
    execute(Path(sys.argv[1]))
