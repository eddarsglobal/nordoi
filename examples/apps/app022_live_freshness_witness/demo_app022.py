#!/usr/bin/env python3
"""APP-022 live witness demonstration with real Rust and SSHSIG.

Proofs represent cross-process same-admin laboratory behavior only. They do
not establish third-party witness custody, restart-proof monotonicity or
global rollback prevention. All private keys are temporary CI fixtures.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import test_app022 as tests
import receiver_app022 as app


def run(proof:Path):
    proof.mkdir(parents=True,exist_ok=True)
    suite=tests.LiveWitness
    claims=[]
    suite.setUpClass()
    try:
        a=suite("test_01_live_signed_bootstrap_witness")
        a.setUp()
        try:
            status=a.good()[1][0]
            assert status["live_witness"]=="NEW_NONCE_SSHSIG_ED25519_VERIFIED"
            claims.append("APP022_NEW_CHALLENGE_REAL_ED25519_SSHSIG=PASS")
            assert status["witness_process"]=="LOOPBACK_SEPARATE_PROCESS"
            claims.append("APP022_DISTINCT_OS_WITNESS_PROCESS=PASS")
            assert status["witness_counter"]==1
            claims.append("APP022_PINNED_WITNESS_COUNTER=PASS")
            assert status["native_engine"]=="NOT_EXECUTED"
            claims.append("APP022_NONPRIVILEGED_STATUS=PASS")
            replay=a.good("verify")[1][0]
            assert replay["native_jobs_recomputed"]==2
            assert replay["native_files_bytewise_equal"]==4
            claims.append("APP022_REAL_NORDOI_RUST_NOI_REPLAY=PASS")
            restored=a.good("restore")[1][0]
            assert restored["status"]=="RESTORED"
            assert restored["fresh_witness_sshsig"]=="VERIFIED_TWICE_BEFORE_NATIVE_EFFECT"
            claims.append("APP022_SECOND_LIVE_ATTEST_BEFORE_RESTORE=PASS")
            res,rows=a.good("resume")
            assert [r["step"] for r in rows]==[3,4]
            claims.append("APP022_REAL_NATIVE_AGENT_RESUME=PASS")
            root=a.fx.fx.fx.fx.fx.dest
            joined=a.work/"APP022_full_native.jsonl"
            joined.write_bytes(
                (a.fx.fx.fx.fx.fx.portable/"handoff/native-prefix.jsonl").read_bytes()
                +res.stdout)
            auditor,code=app.portable.independent.audit(
                root,a.fx.fx.fx.fx.fx.portable/"handoff/plan.json",joined,
                a.fx.fx.fx.fx.fx.policy["plan_sha256"])
            assert code==0 and auditor["status"]=="PASS"
            claims.append("APP022_APP010_INDEPENDENT_FINAL_NATIVE_AUDIT=PASS")
        finally:
            a.tearDown()
        b=suite("test_01_live_signed_bootstrap_witness")
        b.setUp()
        try:
            b.rotate()
            b.denied("WITNESS_CURRENT_HEAD_OR_CHAIN_MISMATCH",
                     epoch=2,seq=2,gen=2)
            claims.append("APP022_STALE_WITNESS_CHAIN_REFUSED=PASS")
            b.update_state(counter=2)
            state=b.good(epoch=2,seq=2,gen=2,witness_counter=2)[1][0]
            assert state["authority_epoch"]==2 and state["witness_counter"]==2
            claims.append("APP022_ROTATED_AUTHORITY_NEW_WITNESS=PASS")
            native=b.good("verify",epoch=2,seq=2,gen=2,witness_counter=2,
                          source=b.fx.fx.fx.fx.fx.alt_source)[1][0]
            assert native["native_jobs_recomputed"]==2
            claims.append("APP022_ROTATED_REAL_RUST_REPLAY=PASS")
            b.fx.document.write_bytes(b.old_signed_document)
            b.fx.signature.write_bytes(b.old_signed_signature)
            b.denied("SSHSIG_SIGNATURE_VERIFICATION_FAILED",
                     epoch=2,seq=2,gen=2,witness_counter=2)
            claims.append("APP022_REVOKED_OLD_AUTHORITY_DENIED=PASS")
        finally:
            b.tearDown()
        c=suite("test_01_live_signed_bootstrap_witness")
        c.setUp()
        try:
            c.freeze()
            c.update_state(counter=2)
            assert c.good(epoch=2,seq=2,gen=2,witness_counter=2)[1][0]["status"]=="FROZEN"
            claims.append("APP022_SIGNED_FROZEN_WITNESS_STATUS=PASS")
            c.denied("WITNESS_FROZEN_NATIVE_ADMISSION_DENIED",
                     "restore",epoch=2,seq=2,gen=2,witness_counter=2)
            assert not any(c.fx.fx.fx.fx.fx.dest.iterdir())
            claims.append("APP022_FROZEN_NO_NATIVE_EFFECT=PASS")
        finally:
            c.tearDown()
    finally:
        suite.tearDownClass()
    assert len(claims)==14,len(claims)
    receipt={
      "schema":"NORDOI_APP022_LIVE_WITNESS_NATIVE_DEMONSTRATION_V1",
      "status":"PASS_WITH_CONTROLS_RESEARCH_ONLY",
      "assertions":claims,
      "live_distinct_process":"ACTUALLY_EXECUTED",
      "sshsig_ed25519":"ACTUALLY_SIGNED_AND_VERIFIED",
      "rust_engine":"ACTUALLY_EXECUTED",
      "independent_host_or_admin":"NOT_PROVEN",
      "external_key_custody":"NOT_PROVEN",
      "restart_protected_antirollback":"NOT_PROVEN",
      "formal_bureau_certification":"NOT_CLAIMED",
      "production":"NOT_AUTHORIZED"
    }
    (proof/"APP022_NATIVE_FRESHNESS_WITNESS.json").write_text(
        json.dumps(receipt,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    for claim in claims:
        print(claim)
    print("APP022_LIVE_WITNESS_REAL_NATIVE_DEMO_RESEARCH_ONLY=PASS")


if __name__=="__main__":
    if len(sys.argv)!=2:
        raise SystemExit("usage: demo_app022.py PROOF_DIR")
    run(Path(sys.argv[1]))
