#!/usr/bin/env python3
"""APP-019 independent-from-main-test-runner native demonstration.

Uses the real APP-016/017/018 fixture and real Rust P2.7. Demonstrates BOTH
the pinned defense and the explicit failure when the attacker controls the
external pin. Neither constitutes a third-party trust certificate.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import test_app019 as suite
import receiver_app019 as app


def main(proof_dir: Path):
    proof_dir.mkdir(parents=True, exist_ok=True)
    cls = suite.ReceiverExternalHeadPin
    assertions = []
    cls.setUpClass()
    try:
        first = cls("test_01_capture_is_explicitly_untrusted")
        first.setUp()
        try:
            candidate = first.good("capture")[1][0]
            assert candidate["status"] == "UNAUTHENTICATED_CANDIDATE"
            assert candidate["external_pin_authentication"] == "NOT_PERFORMED"
            assertions.append("APP019_RECEIVER_EXTERNAL_HEAD_CANDIDATE_UNAUTHENTICATED=PASS")
            observed = first.good("inspect")[1][0]
            assert observed["native_engine"] == "NOT_EXECUTED"
            assertions.append("APP019_INSPECT_NO_NATIVE_ENGINE=PASS")
            native = first.good("verify")[1][0]
            assert native["native_jobs_recomputed"] == 2
            assert native["native_files_bytewise_equal"] == 4
            assertions.append("APP019_REAL_RUST_HISTORICAL_REPLAY=PASS")
            restored = first.good("restore")[1][0]
            assert restored["status"] == "RESTORED"
            baseline = {p.relative_to(first.fx.fx.fx.dest).as_posix():
                        hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in first.fx.fx.fx.dest.rglob("*") if p.is_file()}
            continuation, rows = first.good("resume")
            assert [r["step"] for r in rows] == [3, 4]
            for relative, original in baseline.items():
                assert hashlib.sha256(
                    (first.fx.fx.fx.dest / relative).read_bytes()
                ).hexdigest() == original
            assertions.append("APP019_PIN_GATED_REAL_NATIVE_RESTORE_RESUME=PASS")
            whole = first.fx.fx.fx.work / "APP019_native_complete.jsonl"
            whole.write_bytes(
                (first.fx.fx.fx.portable / "handoff/native-prefix.jsonl").read_bytes()
                + continuation.stdout)
            audit, code = app.portable.independent.audit(
                first.fx.fx.fx.dest,
                first.fx.fx.fx.portable / "handoff/plan.json",
                whole,
                first.fx.fx.fx.policy["plan_sha256"])
            assert code == 0 and audit["status"] == "PASS"
            assertions.append("APP019_APP010_FULL_NATIVE_DAG_AUDIT=PASS")
            first.refused("ROOT_UNEXPECTED_JOB_OR_FILE", "resume")
            assertions.append("APP019_DUPLICATE_NATIVE_RESUME_REFUSED=PASS")
        finally:
            first.tearDown()

        second = cls("test_01_capture_is_explicitly_untrusted")
        second.setUp()
        try:
            preserved = second.fx.fx.fx.work / "original-complete-anchor"
            shutil.copytree(second.root, preserved)
            old_pin = second.pin
            rotation = second.rotate()
            assert rotation["roster_generation"] == 2
            assert rotation["head_sha256"] != old_pin
            assertions.append("APP019_RECEIVER_PINNED_ROTATION=PASS")
            second.refused("HEAD_PIN_MISMATCH", "inspect", pin=old_pin)
            assertions.append("APP019_STALE_HEAD_PIN_REFUSED=PASS")
            new_source = second.fx.fx.fx.alt_source
            replay = second.good("verify", source=new_source)[1][0]
            assert replay["native_jobs_recomputed"] == 2
            assertions.append("APP019_TRUE_ALTERNATE_NOI_REPLAY=PASS")
            newest_pin = second.pin
            shutil.rmtree(second.root)
            shutil.copytree(preserved, second.root)
            second.refused("HEAD_PIN_MISMATCH", pin=newest_pin)
            assertions.append("APP019_COMPLETE_COHERENT_JOURNAL_ROLLBACK_DENIED_WITH_NEW_PIN=PASS")
            old_row = second.good(pin=old_pin)[1][0]
            assert old_row["roster_generation"] == 1
            assert old_row["external_pin_authentication"] == "NOT_PERFORMED"
            assertions.append("APP019_OLD_JOURNAL_AND_OLD_PIN_CAN_STILL_PASS_DOCUMENTED_LIMIT=PASS")
        finally:
            second.tearDown()

        third = cls("test_01_capture_is_explicitly_untrusted")
        third.setUp()
        try:
            third.freeze()
            third.refused("ROSTER_FROZEN_DENIES_ADMISSION", "inspect")
            third.refused("ROSTER_FROZEN_DENIES_ADMISSION", "restore")
            assert not any(third.fx.fx.fx.dest.iterdir())
            assertions.append("APP019_FROZEN_WITH_EXTERNAL_PIN_DENIES_NATIVE_EFFECT=PASS")
        finally:
            third.tearDown()
    finally:
        cls.tearDownClass()

    assert len(assertions) == 12, assertions
    proof = {
        "schema": "NORDOI_APP019_NATIVE_DEMONSTRATION_V1",
        "status": "INTERNAL_RESEARCH_DEMONSTRATION_PASS",
        "assertions": assertions,
        "native_rust": "EXECUTED",
        "independent_trust_pin_authentication": "NOT_PERFORMED",
        "external_security_certification": "NOT_CLAIMED",
        "host_adversary_protection": "NOT_PROVIDED",
        "global_anti_rollback": "NOT_PROVIDED",
        "production": "NOT_AUTHORIZED",
    }
    (proof_dir / "APP019_REAL_NATIVE_DEMONSTRATION.json").write_text(
        json.dumps(proof, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    for assertion in assertions:
        print(assertion)
    print("APP019_DEMO_RESEARCH_ONLY=PASS")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: demo_app019.py PROOF_DIR")
    main(Path(sys.argv[1]))
