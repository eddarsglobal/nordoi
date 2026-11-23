#!/usr/bin/env python3
"""APP-020 independent script demonstrating real OpenSSH SSHSIG and Rust.

The locally generated ephemeral signing key is LAB-ONLY, not a separate
custodian or independently verified administrative authority.
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
import test_app020 as suite
import receiver_app020 as signed


def main(proof: Path):
    proof.mkdir(parents=True, exist_ok=True)
    observations = []
    cls = suite.SignedProvenance
    cls.setUpClass()
    try:
        first = cls("test_01_propose_is_exact_canonical_unsigned_candidate")
        first.setUp()
        try:
            status = first.good()[1][0]
            assert status["crypto_signature"] == "SSHSIG_ED25519_VERIFIED"
            observations.append("APP020_GENUINE_OPENSSH_ED25519_SIGN_AND_VERIFY=PASS")
            assert status["independent_key_custody"] == "NOT_PROVEN"
            observations.append("APP020_NO_FICTITIOUS_INDEPENDENT_KEY_CUSTODY=PASS")
            query = first.good("inspect")[1][0]
            assert query["native_engine"] == "NOT_EXECUTED"
            observations.append("APP020_SIGNED_INSPECT_WITHOUT_RUST=PASS")
            proofrow = first.good("verify")[1][0]
            assert proofrow["native_jobs_recomputed"] == 2
            assert proofrow["native_files_bytewise_equal"] == 4
            observations.append("APP020_SIGNED_REPLAY_REAL_RUST=PASS")
            restored = first.good("restore")[1][0]
            assert restored["status"] == "RESTORED"
            original = {p.relative_to(first.fx.fx.fx.fx.dest).as_posix():
                        hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in first.fx.fx.fx.fx.dest.rglob("*") if p.is_file()}
            run, rows = first.good("resume")
            assert [r["step"] for r in rows] == [3, 4]
            for relative, sha in original.items():
                assert hashlib.sha256(
                    (first.fx.fx.fx.fx.dest / relative).read_bytes()
                ).hexdigest() == sha
            observations.append("APP020_SIGNED_REAL_NATIVE_RESTORE_RESUME=PASS")
            entire = first.work / "app020-authentic-native-completed.jsonl"
            entire.write_bytes(
                (first.fx.fx.fx.fx.portable / "handoff/native-prefix.jsonl").read_bytes()
                + run.stdout)
            report,code = signed.portable.independent.audit(
                first.fx.fx.fx.fx.dest,
                first.fx.fx.fx.fx.portable / "handoff/plan.json",
                entire,
                first.fx.fx.fx.fx.policy["plan_sha256"])
            assert code == 0 and report["status"] == "PASS"
            observations.append("APP020_APP010_INDEPENDENT_FINAL_NATIVE_DAG_AUDIT=PASS")
            first.refused("ROOT_UNEXPECTED_JOB_OR_FILE", "resume")
            observations.append("APP020_SIGNED_DUPLICATE_RESUME_DENIED=PASS")
        finally:
            first.tearDown()

        second = cls("test_01_propose_is_exact_canonical_unsigned_candidate")
        second.setUp()
        try:
            original_anchor = second.work / "app020-preserved-first-journal"
            shutil.copytree(second.fx.root, original_anchor)
            signature = second.signature.read_bytes()
            statement = second.document.read_bytes()
            second.rotated_signed()
            current = second.good(minimum_sequence=2)[1][0]
            assert current["roster_generation"] == 2
            observations.append("APP020_SIGNED_NEW_GENERATION_AND_RECEIVER_FLOOR=PASS")
            latest_signature = second.signature.read_bytes()
            latest_statement = second.document.read_bytes()
            second.document.write_bytes(statement)
            second.signature.write_bytes(signature)
            second.refused("SSHSIG_SIGNED_STATE_NOT_CURRENT")
            second.document.write_bytes(latest_statement)
            second.signature.write_bytes(latest_signature)
            observations.append("APP020_PREVIOUS_SIGNED_HEAD_REJECTED_AFTER_ROTATION=PASS")
            replay = second.good(
                "verify", minimum_sequence=2,
                source=second.fx.fx.fx.fx.alt_source)[1][0]
            assert replay["native_jobs_recomputed"] == 2
            observations.append("APP020_ALTERNATE_NOI_REAL_RUST_PINNED=PASS")
            shutil.rmtree(second.fx.root)
            shutil.copytree(original_anchor, second.fx.root)
            second.document.write_bytes(statement)
            second.signature.write_bytes(signature)
            second.refused("SSHSIG_BELOW_RECEIVER_FLOOR", minimum_sequence=2)
            observations.append("APP020_COHERENT_OLD_SIGNED_JOURNAL_BLOCKED_BY_FLOOR=PASS")
            old = second.good()[1][0]
            assert old["roster_generation"] == 1
            assert old["global_anti_rollback"] == "NOT_PROVIDED"
            observations.append("APP020_OLD_VALID_SIGNATURE_WITH_ROLLED_BACK_FLOOR_REMAINS_POSSIBLE=PASS")
        finally:
            second.tearDown()

        third = cls("test_01_propose_is_exact_canonical_unsigned_candidate")
        third.setUp()
        try:
            third.frozen_signed()
            third.refused("ROSTER_FROZEN_DENIES_ADMISSION",
                          "restore", minimum_sequence=2)
            assert not any(third.fx.fx.fx.fx.dest.iterdir())
            observations.append("APP020_SIGNED_FROZEN_DENIES_ALL_NATIVE_EFFECTS=PASS")
        finally:
            third.tearDown()
    finally:
        cls.tearDownClass()

    assert len(observations) == 13, observations
    report = {
        "schema": "NORDOI_APP020_REAL_SSHSIG_NATIVE_DEMONSTRATION_V1",
        "status": "INTERNAL_RESEARCH_LAB_PASS",
        "assertions": observations,
        "native_rust": "EXECUTED",
        "signature": "SSHSIG_ED25519_VERIFIED",
        "private_key_fixture": "EPHEMERAL_LAB_ONLY_NOT_ARCHIVED",
        "independent_key_custody": "NOT_PROVEN",
        "global_anti_rollback": "NOT_PROVIDED",
        "third_party_security_certification": "NOT_CLAIMED",
        "production": "NOT_AUTHORIZED",
    }
    (proof / "APP020_REAL_NATIVE_SSHSIG_DEMONSTRATION.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    for line in observations:
        print(line)
    print("APP020_NATIVE_SSHSIG_DEMO_RESEARCH_ONLY=PASS")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: demo_app020.py PROOF_DIR")
    main(Path(sys.argv[1]))
