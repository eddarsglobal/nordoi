#!/usr/bin/env python3
"""APP-021 native evidence: real dual-SSHSIG rotation, revoke, Rust replay.

No externally controlled root key or independent storage exists in this lab.
The tests sign only with temporary local fixture keys, deliberately disclose
old-chain + old-journal + old receiver-floor coherent rollback limitations.
"""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import shutil
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import test_app021 as tests
import receiver_app021 as app


def run(proof: Path):
    proof.mkdir(parents=True, exist_ok=True)
    suite = tests.AuthorityRotation
    assertions = []
    suite.setUpClass()
    try:
        a = suite("test_01_initial_chain_genuine_root_signature")
        a.setUp()
        try:
            row = a.good()[1][0]
            assert row["authority_epoch"] == 1
            assert row["active_key_sha256"] == a.root_sha
            assertions.append("APP021_ROOT_PINNED_BOOTSTRAP_SSHSIG=PASS")
            inspected = a.good("inspect")[1][0]
            assert inspected["native_engine"] == "NOT_EXECUTED"
            assertions.append("APP021_BOOTSTRAP_INSPECT_NO_NATIVE_ENGINE=PASS")
            replay = a.good("verify")[1][0]
            assert replay["native_jobs_recomputed"] == 2
            assert replay["native_files_bytewise_equal"] == 4
            assertions.append("APP021_BOOTSTRAP_REAL_RUST_REPLAY=PASS")
            restored = a.good("restore")[1][0]
            assert restored["status"] == "RESTORED"
            native_root = a.fx.fx.fx.fx.fx.dest
            baseline = {p.relative_to(native_root).as_posix():
                        hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in native_root.rglob("*") if p.is_file()}
            result, rows = a.good("resume")
            assert [x["step"] for x in rows] == [3,4]
            for p,sha in baseline.items():
                assert hashlib.sha256((native_root / p).read_bytes()).hexdigest() == sha
            assertions.append("APP021_BOOTSTRAP_REAL_NATIVE_RESTORE_RESUME=PASS")
            whole = a.work/"APP021_full_native.jsonl"
            whole.write_bytes(
                (a.fx.fx.fx.fx.fx.portable/"handoff/native-prefix.jsonl").read_bytes()
                + result.stdout)
            audited,code = app.portable.independent.audit(
                native_root,
                a.fx.fx.fx.fx.fx.portable/"handoff/plan.json",
                whole,
                a.fx.fx.fx.fx.fx.policy["plan_sha256"])
            assert code==0 and audited["status"]=="PASS"
            assertions.append("APP021_APP010_INDEPENDENT_FINAL_AUDIT=PASS")
            a.denied("ROOT_UNEXPECTED_JOB_OR_FILE","resume")
            assertions.append("APP021_DUPLICATE_NATIVE_RESUME_DENIED=PASS")
        finally:
            a.tearDown()

        b = suite("test_01_initial_chain_genuine_root_signature")
        b.setUp()
        try:
            prior = b.work/"untouched-original-anchor"
            shutil.copytree(b.root, prior)
            old_records = copy.deepcopy(b.records)
            old_doc = b.fx.document.read_bytes()
            old_sig = b.fx.signature.read_bytes()
            old_pin = b.chain_sha
            b.rotate()
            rotated = b.good(epoch=2, seq=2, gen=2)[1][0]
            assert rotated["authority_epoch"]==2
            assert rotated["active_key_sha256"]==b.successor_sha
            assert rotated["retired_authorities"]==1
            assertions.append("APP021_DUAL_OPENSSH_SSHSIG_ROTATION=PASS")
            b.denied("AUTHORITY_CHAIN_EXTERNAL_PIN_MISMATCH",
                     pin=old_pin, epoch=2, seq=2, gen=2)
            assertions.append("APP021_OLD_CHAIN_PIN_DENIED=PASS")
            new_source = b.fx.fx.fx.fx.fx.alt_source
            native = b.good("verify", epoch=2, seq=2, gen=2,
                            source=new_source)[1][0]
            assert native["native_jobs_recomputed"]==2
            assertions.append("APP021_ROTATED_NEW_KEY_REAL_RUST_NOI_REPLAY=PASS")
            b.fx.document.write_bytes(old_doc)
            b.fx.signature.write_bytes(old_sig)
            b.denied("SSHSIG_SIGNATURE_VERIFICATION_FAILED",
                     epoch=2, seq=2, gen=2)
            assertions.append("APP021_REVOKED_OLD_SIGNER_DENIED=PASS")
            shutil.rmtree(b.root)
            shutil.copytree(prior, b.root)
            b.records=old_records
            b.write_chain()
            b.denied("AUTHORITY_EPOCH_BELOW_RECEIVER_FLOOR",
                     epoch=2, seq=2, gen=2)
            assertions.append("APP021_COMPLETE_OLD_CHAIN_BLOCKED_BY_RECEIVER_FLOOR=PASS")
            old=b.good(epoch=1, seq=1, gen=1)[1][0]
            assert old["authority_epoch"]==1
            assert old["global_anti_rollback"]=="NOT_PROVIDED"
            assertions.append("APP021_ALL_OLD_TRUST_REFERENCES_STILL_ROLL_BACK_LIMIT=PASS")
        finally:
            b.tearDown()

        c = suite("test_01_initial_chain_genuine_root_signature")
        c.setUp()
        try:
            c.freeze()
            status = c.good(epoch=2, seq=2, gen=2)[1][0]
            assert status["status"]=="FROZEN"
            assertions.append("APP021_SIGNED_FREEZE_STATUS_VISIBLE=PASS")
            c.denied("AUTHORITY_FROZEN_DENIES_NATIVE_ADMISSION",
                     "restore", epoch=2, seq=2, gen=2)
            assert not any(c.fx.fx.fx.fx.fx.dest.iterdir())
            assertions.append("APP021_REVOKED_FROZEN_DENIES_NATIVE_EFFECT=PASS")
        finally:
            c.tearDown()
    finally:
        suite.tearDownClass()
    assert len(assertions)==14,assertions
    receipt = {
        "schema":"NORDOI_APP021_NATIVE_AUTHORITY_ROTATION_DEMONSTRATION_V1",
        "status":"INTERNAL_RESEARCH_ONLY_PASS",
        "assertions":assertions,
        "native_rust":"ACTUALLY_EXECUTED",
        "signature":"OPENSSH_SSHSIG_ED25519_ACTUALLY_VERIFIED",
        "independent_key_custody":"NOT_PROVEN",
        "independent_freshness_floor":"NOT_PROVEN",
        "global_anti_rollback":"NOT_PROVIDED",
        "production":"NOT_AUTHORIZED",
    }
    (proof/"APP021_REAL_NATIVE_AUTHORITY_ROTATION.json").write_text(
        json.dumps(receipt,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    for v in assertions:
        print(v)
    print("APP021_NATIVE_AUTHORITY_ROTATION_DEMO_RESEARCH_ONLY=PASS")


if __name__=="__main__":
    if len(sys.argv)!=2:
        raise SystemExit("usage: demo_app021.py PROOF_DIR")
    run(Path(sys.argv[1]))
