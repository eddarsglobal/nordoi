"""APP-007 machine-only tests: original data is produced by real Rust NORDOI.

No simulation of the engine, no external signatures, no production activation.
Tampered files/receipts are derived from actual NORDOI results.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
SOURCE = REPO / "examples/apps/app003_integrated_quality/src"
EXE = Path(os.environ.get("NORDOI_APP007_EXE", str(
    REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
)))
sys.path.insert(0, str(HERE))
import verify_app007 as verify


def request(value: int) -> bytes:
    return json.dumps({"schema": "NORDOI_AGENT_REQUEST_V1", "key_code": value},
                      separators=(",", ":")).encode("utf-8") + b"\n"


class PinnedNativeAgentReplay(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nordoi_app007_actual_")
        self.base = Path(self.temp.name)
        self.root = self.base / "original"
        self.root.mkdir()
        self.receipts = self.base / "original-receipts.jsonl"
        self.requests = self.base / "original-requests.jsonl"
        self.engine = EXE
        self.source = SOURCE
        self.engine_pin = verify.engine_sha256(self.engine)
        self.source_pin = verify.source_tree_sha256(self.source)
        self.requests.write_bytes(request(73) + request(59))
        self.requests_pin = hashlib.sha256(self.requests.read_bytes()).hexdigest()
        self.native()

    def tearDown(self):
        self.temp.cleanup()

    def native(self):
        proc = subprocess.run(
            [str(self.engine.resolve()), "agent-batch-jsonl", str(self.source),
             "app.main", "--grant-output-dir", str(self.root)],
            input=self.requests.read_bytes(), cwd=REPO,
            capture_output=True, timeout=120,
        )
        self.receipts.write_bytes(proc.stdout)
        self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8", "replace"))

    def check(self):
        p = subprocess.run(
            [sys.executable, "-B", str(HERE / "verify_app007.py"),
             "--batch-root", str(self.root), "--receipts", str(self.receipts),
             "--requests", str(self.requests), "--engine", str(self.engine),
             "--source-root", str(self.source),
             "--expected-engine-sha256", self.engine_pin,
             "--expected-source-sha256", self.source_pin,
             "--expected-requests-sha256", self.requests_pin],
            capture_output=True, cwd=REPO, timeout=140,
        )
        try:
            report = json.loads(p.stdout.decode("utf-8"))
        except Exception:
            self.fail(f"non-JSON verifier result: {p.stdout!r}, stderr={p.stderr!r}")
        return p, report

    def assert_denied(self, reason: str):
        p, r = self.check()
        self.assertEqual(p.returncode, 2, r)
        self.assertEqual(r["status"], "REJECTED")
        self.assertEqual(r["error_code"], reason)
        self.assertEqual(r["external_authentication"], "NOT_PERFORMED")

    def replace_receipt(self, transform):
        records = [json.loads(line) for line in self.receipts.read_text(encoding="utf-8").splitlines()]
        transform(records)
        self.receipts.write_text(
            "".join(json.dumps(rec, separators=(",", ":"), ensure_ascii=True) + "\n"
                    for rec in records), encoding="utf-8",
        )

    def test_01_real_two_jobs_native_proof_replay_passes(self):
        p, r = self.check()
        self.assertEqual(p.returncode, 0, r)
        self.assertEqual(r["status"], "PASS")
        self.assertEqual(r["jobs_replayed"], 2)
        self.assertEqual(r["files_replayed"], 4)

    def test_02_one_native_job_replays(self):
        shutil.rmtree(self.root)
        self.root.mkdir()
        self.requests.write_bytes(request(73))
        self.requests_pin = hashlib.sha256(self.requests.read_bytes()).hexdigest()
        self.native()
        p, r = self.check()
        self.assertEqual(p.returncode, 0, r)
        self.assertEqual(r["jobs_replayed"], 1)

    def test_03_eight_real_jobs_replay_at_bound(self):
        shutil.rmtree(self.root)
        self.root.mkdir()
        self.requests.write_bytes(b"".join(map(request, (0, 20, 59, 60, 61, 73, 90, 100))))
        self.requests_pin = hashlib.sha256(self.requests.read_bytes()).hexdigest()
        self.native()
        p, r = self.check()
        self.assertEqual(p.returncode, 0, r)
        self.assertEqual(r["jobs_replayed"], 8)
        self.assertEqual(r["files_replayed"], 16)

    def test_04_repeated_inputs_are_replayed_not_collapsed(self):
        shutil.rmtree(self.root)
        self.root.mkdir()
        self.requests.write_bytes(request(73) * 3)
        self.requests_pin = hashlib.sha256(self.requests.read_bytes()).hexdigest()
        self.native()
        p, r = self.check()
        self.assertEqual(p.returncode, 0, r)
        self.assertEqual(r["jobs_replayed"], 3)

    def test_05_replay_output_is_deterministic(self):
        p1, r1 = self.check()
        p2, r2 = self.check()
        self.assertEqual((p1.returncode, r1), (p2.returncode, r2))
        self.assertEqual(r1["original_receipts_sha256"], r1["replay_receipts_sha256"])

    def test_06_original_native_bytes_are_unchanged(self):
        tracked = [self.requests, self.receipts,
                   self.root/"job-0001/quality-decision/decision.txt",
                   self.root/"job-0002/quality-decision/report.txt"]
        before = [p.read_bytes() for p in tracked]
        proc, report = self.check()
        self.assertEqual(proc.returncode, 0, report)
        self.assertEqual(before, [p.read_bytes() for p in tracked])

    def test_07_a_single_modified_file_is_rejected_by_prior_audit(self):
        p = self.root / "job-0001/quality-decision/decision.txt"
        p.write_bytes(p.read_bytes() + b"forged")
        self.assert_denied("APP006_REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")

    def test_08_coordinated_file_and_receipt_mutation_fails_replay(self):
        name = "decision.txt"
        p = self.root / "job-0001/quality-decision" / name
        forged = b"approved=true, score=73, next=999\n"
        p.write_bytes(forged)
        def update(records):
            file = next(f for f in records[0]["files"] if f["name"] == name)
            file["content"] = forged.decode("utf-8")
            file["bytes"] = len(forged)
        self.replace_receipt(update)
        self.assert_denied("NATIVE_REPLAY_RECEIPT_MISMATCH")

    def test_09_forged_p27_bundle_digest_fails_replay(self):
        self.replace_receipt(lambda rows: rows[0].__setitem__("bundle_receipt_sha256", "a"*64))
        self.assert_denied("NATIVE_REPLAY_RECEIPT_MISMATCH")

    def test_10_forged_v13_rule_receipt_fails_replay(self):
        self.replace_receipt(lambda rows: rows[0].__setitem__(
            "rule_receipt_hex", "aa" + rows[0]["rule_receipt_hex"][2:]))
        self.assert_denied("NATIVE_REPLAY_RECEIPT_MISMATCH")

    def test_11_forged_file_proof_sha256_fails_replay(self):
        self.replace_receipt(lambda rows: rows[0]["files"][0].__setitem__(
            "proof_sha256", "b"*64))
        self.assert_denied("NATIVE_REPLAY_RECEIPT_MISMATCH")

    def test_12_wrong_engine_sha256_refuses_before_replay(self):
        self.engine_pin = "0"*64
        self.assert_denied("ENGINE_PIN_MISMATCH")

    def test_13_wrong_source_sha256_refuses_before_replay(self):
        self.source_pin = "0"*64
        self.assert_denied("SOURCE_PIN_MISMATCH")

    def test_14_wrong_request_sha256_refuses_before_replay(self):
        self.requests_pin = "0"*64
        self.assert_denied("REQUEST_PIN_MISMATCH")

    def test_15_modified_original_input_refuses_anchored_pin(self):
        self.requests.write_bytes(request(73) + request(60))
        self.assert_denied("REQUEST_PIN_MISMATCH")

    def test_16_request_forgery_with_updated_pin_fails_receipt_binding(self):
        self.requests.write_bytes(request(73) + request(60))
        self.requests_pin = hashlib.sha256(self.requests.read_bytes()).hexdigest()
        self.assert_denied("RECEIPT_REQUEST_CODES_MISMATCH")

    def test_17_mutated_source_with_updated_pin_changes_replay(self):
        self.source = self.base / "mutated-src"
        shutil.copytree(SOURCE, self.source)
        rule = self.source / "app/rules.noi"
        rule.write_text(rule.read_text().replace(">= 60", ">= 75"), encoding="utf-8")
        self.source_pin = verify.source_tree_sha256(self.source)
        self.assert_denied("NATIVE_REPLAY_RECEIPT_MISMATCH")

    def test_18_source_symlink_fails_when_available(self):
        target = self.base / "linked-src"
        try:
            target.symlink_to(SOURCE, target_is_directory=True)
        except (OSError, NotImplementedError):
            self.skipTest("platform forbids symlink creation")
        self.source = target
        self.assert_denied("SOURCE_ROOT_INVALID")

    def test_19_partial_original_is_inconclusive_never_pass(self):
        self.replace_receipt(lambda rows: rows.append({
            "schema": "NORDOI_AGENT_BUNDLE_RESULT_V1", "status": "ERROR",
            "error_code": "NATIVE_OPERATION_FAILED", "exit_code": 5,
            "publication": "UNKNOWN",
        }))
        p, r = self.check()
        self.assertEqual(p.returncode, 3, r)
        self.assertEqual(r["status"], "INCONCLUSIVE")
        self.assertEqual(r["error_code"], "ORIGINAL_BATCH_NOT_FULLY_COMMITTED")

    def test_20_unexpected_output_file_denied_before_replay(self):
        (self.root/"job-0001/quality-decision/extra.txt").write_text("untrusted")
        self.assert_denied("APP006_BUNDLE_FILE_SET_CHANGED")

    def test_21_missing_original_receipt_refused(self):
        self.receipts.unlink()
        self.assert_denied("APP006_RECEIPTS_NOT_BOUNDED_REGULAR_FILE")

    def test_22_pin_format_not_accepted(self):
        self.engine_pin = "not-a-sha"
        self.assert_denied("INVALID_PIN_FORMAT")

    def test_23_real_receipt_sha256_is_exact_original_jsonl(self):
        p, report = self.check()
        self.assertEqual(p.returncode, 0, report)
        self.assertEqual(report["original_receipts_sha256"],
                         hashlib.sha256(self.receipts.read_bytes()).hexdigest())

    def test_24_replay_does_not_claim_external_signatures(self):
        p, report = self.check()
        self.assertEqual(p.returncode, 0, report)
        self.assertEqual(report["external_authentication"], "NOT_PERFORMED")
        self.assertEqual(report["native_production_admission"], "NOT_AUTHORIZED")
        self.assertEqual(report["crash_durability"], "UNCLAIMED")
        self.assertEqual(report["p27_proof_validation"],
                         "EXACT_NATIVE_REPRODUCTION_NOT_STANDALONE_CRYPTO_RECOMPUTATION")


if __name__ == "__main__":
    unittest.main()
