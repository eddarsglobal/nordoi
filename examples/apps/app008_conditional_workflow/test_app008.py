"""APP-008: automated real Rust workflow tests, four OS; no business-rule simulation."""
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
EXE = Path(os.environ.get("NORDOI_APP008_EXE", str(
    REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
)))
sys.path.insert(0, str(HERE))
import run_app008 as workflow


def step(code: int, when: str = "always") -> dict:
    return {"key_code": code, "when": when}


class ConditionalNativeWorkflow(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nordoi_app008_actual_")
        self.base = Path(self.temp.name)
        self.out = self.base / "outputs"
        self.out.mkdir()
        self.plan = self.base / "plan.json"
        self.source = SOURCE
        self.engine = EXE
        self.sha_engine = workflow.pinned.engine_sha256(self.engine)
        self.sha_source = workflow.pinned.source_tree_sha256(self.source)
        self.write([step(73), step(59)])

    def tearDown(self):
        self.temp.cleanup()

    def write(self, steps):
        self.plan.write_text(json.dumps(
            {"schema": workflow.SCHEMA, "steps": steps},
            ensure_ascii=True, separators=(",", ":")), encoding="utf-8")

    def check(self):
        proc = subprocess.run(
            [sys.executable, "-B", str(HERE / "run_app008.py"),
             "--plan", str(self.plan), "--engine", str(self.engine),
             "--source-root", str(self.source),
             "--grant-output-dir", str(self.out),
             "--expected-engine-sha256", self.sha_engine,
             "--expected-source-sha256", self.sha_source],
            cwd=REPO, capture_output=True, timeout=140,
        )
        try:
            records = [json.loads(line) for line in
                       proc.stdout.decode("utf-8", errors="strict").splitlines()]
        except Exception as err:
            self.fail(f"non-json output: {err!r}: stdout={proc.stdout!r}, stderr={proc.stderr!r}")
        return proc, records

    def denied(self, code: str):
        proc, rows = self.check()
        self.assertEqual(proc.returncode, 2, rows)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["status"], "REJECTED")
        self.assertEqual(rows[0]["error_code"], code)
        self.assertEqual(rows[0]["publication"], "NOT_STARTED")
        self.assertEqual(list(self.out.iterdir()), [])

    def test_01_real_two_always_jobs_commit(self):
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual([r["status"] for r in rows], ["COMMITTED"]*2)
        self.assertEqual([r["decision"] for r in rows], [1, 0])

    def test_02_previous_approved_runs_real_second(self):
        self.write([step(73), step(59, "if_previous_approved")])
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual([r["status"] for r in rows], ["COMMITTED", "COMMITTED"])

    def test_03_previous_rejected_runs_real_second(self):
        self.write([step(59), step(73, "if_previous_rejected")])
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual([r["status"] for r in rows], ["COMMITTED", "COMMITTED"])

    def test_04_unmet_approved_condition_skips_without_file(self):
        self.write([step(59), step(73, "if_previous_approved")])
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual([r["status"] for r in rows], ["COMMITTED", "SKIPPED"])
        self.assertFalse((self.out/"job-0002").exists())

    def test_05_unmet_rejected_condition_skips_without_file(self):
        self.write([step(73), step(59, "if_previous_rejected")])
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual([r["status"] for r in rows], ["COMMITTED", "SKIPPED"])
        self.assertFalse((self.out/"job-0002").exists())

    def test_06_skipped_step_cannot_authorize_third(self):
        self.write([step(59), step(73, "if_previous_approved"),
                    step(73, "if_previous_rejected")])
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual([r["status"] for r in rows],
                         ["COMMITTED", "SKIPPED", "SKIPPED"])
        self.assertFalse((self.out/"job-0003").exists())

    def test_07_always_resets_after_skipped(self):
        self.write([step(59), step(73, "if_previous_approved"),
                    step(73), step(59, "if_previous_approved")])
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual([r["status"] for r in rows],
                         ["COMMITTED", "SKIPPED", "COMMITTED", "COMMITTED"])
        self.assertFalse((self.out/"job-0002").exists())

    def test_08_eight_real_jobs_up_to_bound(self):
        self.write([step(x) for x in (0, 20, 59, 60, 61, 73, 90, 100)])
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual(len(rows), 8)
        self.assertTrue(all(x["status"] == "COMMITTED" for x in rows))

    def test_09_repeated_inputs_are_independent_real_jobs(self):
        self.write([step(73)]*3)
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual(len(rows), 3)
        self.assertEqual([r["job"] for r in rows],
                         ["job-0001", "job-0002", "job-0003"])

    def test_10_last_malformed_step_rejects_before_first_effect(self):
        self.write([step(73), {"key_code":59, "when":"always", "grant":"untrusted"}])
        self.denied("INVALID_WORKFLOW_STEP_FIELDS")

    def test_11_json_permission_injection_rejected(self):
        self.write([{"key_code":73, "when":"always", "grant_output_dir":"/tmp"}])
        self.denied("INVALID_WORKFLOW_STEP_FIELDS")

    def test_12_source_path_injection_rejected(self):
        self.write([{"key_code":73, "when":"always", "source_root":"/"}])
        self.denied("INVALID_WORKFLOW_STEP_FIELDS")

    def test_13_boolean_key_code_rejected(self):
        self.write([{"key_code":True, "when":"always"}])
        self.denied("INVALID_WORKFLOW_KEY_CODE")

    def test_14_negative_key_code_rejected(self):
        self.write([step(-1)])
        self.denied("INVALID_WORKFLOW_KEY_CODE")

    def test_15_overflow_key_code_rejected(self):
        self.write([step(2**32)])
        self.denied("INVALID_WORKFLOW_KEY_CODE")

    def test_16_duplicate_raw_step_key_rejected(self):
        self.plan.write_text(
            '{"schema":"NORDOI_AGENT_WORKFLOW_V1","steps":['
            '{"key_code":73,"key_code":59,"when":"always"}]}',
            encoding="utf-8")
        self.denied("DUPLICATE_REQUEST_KEY")

    def test_17_first_step_must_run(self):
        self.write([step(73, "if_previous_approved")])
        self.denied("FIRST_WORKFLOW_STEP_MUST_RUN")

    def test_18_unknown_condition_rejected(self):
        self.write([step(73), step(59, "if_network_available")])
        self.denied("INVALID_WORKFLOW_CONDITION")

    def test_19_empty_workflow_rejected(self):
        self.write([])
        self.denied("INVALID_WORKFLOW_STEP_COUNT")

    def test_20_ninth_step_rejected(self):
        self.write([step(73)]*9)
        self.denied("INVALID_WORKFLOW_STEP_COUNT")

    def test_21_nonempty_grant_denied_without_any_new_effect(self):
        (self.out/"existing.marker").write_text("keep", encoding="utf-8")
        proc, rows = self.check()
        self.assertEqual(proc.returncode, 2)
        self.assertEqual(rows[0]["error_code"], "EXPLICIT_GRANT_NOT_EMPTY")
        self.assertEqual(list(self.out.iterdir())[0].read_text(), "keep")

    def test_22_output_symlink_denied_when_supported(self):
        root = self.base / "linked-output"
        try:
            root.symlink_to(self.out, target_is_directory=True)
        except (OSError, NotImplementedError):
            self.skipTest("symlink not available")
        self.out = root
        proc, rows = self.check()
        self.assertEqual(proc.returncode, 2)
        self.assertEqual(rows[0]["error_code"],
                         "EXPLICIT_GRANT_NOT_REAL_DIRECTORY")

    def test_23_engine_pin_mismatch_denied_before_effect(self):
        self.sha_engine = "0"*64
        self.denied("ENGINE_PIN_MISMATCH")

    def test_24_source_pin_mismatch_denied_before_effect(self):
        self.sha_source = "0"*64
        self.denied("SOURCE_PIN_MISMATCH")

    def test_25_real_engine_failure_preserves_prior_commit(self):
        self.source = self.base / "mutated-source"
        shutil.copytree(SOURCE, self.source)
        rule = self.source / "app/rules.noi"
        content = rule.read_text(encoding="utf-8")
        self.assertIn("else { 0 }", content)
        rule.write_text(content.replace("else { 0 }", "else { 2 }"), encoding="utf-8")
        self.sha_source = workflow.pinned.source_tree_sha256(self.source)
        p, rows = self.check()
        self.assertEqual(p.returncode, 3, p.stderr)
        self.assertEqual([r["status"] for r in rows], ["COMMITTED", "ERROR"])
        self.assertEqual(rows[-1]["publication"], "UNKNOWN")
        self.assertTrue((self.out/"job-0001/quality-decision/report.txt").is_file())

    def test_26_real_rule_change_changes_native_branch(self):
        self.source = self.base / "mutated-source"
        shutil.copytree(SOURCE, self.source)
        rule = self.source / "app/rules.noi"
        rule.write_text(rule.read_text(encoding="utf-8").replace(">= 60", ">= 75"),
                        encoding="utf-8")
        self.sha_source = workflow.pinned.source_tree_sha256(self.source)
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual([r["decision"] for r in rows], [0, 0])

    def test_27_real_native_receipts_match_actual_files(self):
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        for i, record in enumerate(rows, 1):
            checked = workflow.audit.verify_native_record(
                record["native_receipt"], i, self.out)
            self.assertEqual(checked["decision"], record["decision"])

    def test_28_output_contains_only_native_permitted_paths(self):
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual({p.name for p in self.out.iterdir()},
                         {"job-0001", "job-0002"})
        self.assertTrue(all((self.out / r["job"] / "quality-decision" /
                             "decision.txt").is_file() for r in rows))

    def test_29_repeated_replay_is_deterministic_ignoring_new_root(self):
        first, rows = self.check()
        self.assertEqual(first.returncode, 0, first.stderr)
        first_bytes = first.stdout
        self.out = self.base / "another-output"
        self.out.mkdir()
        second, again = self.check()
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(first_bytes, second.stdout)
        self.assertEqual(rows, again)

    def test_30_original_plan_not_modified_by_workflow(self):
        before = self.plan.read_bytes()
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual(self.plan.read_bytes(), before)

    def test_31_invalid_utf8_plan_rejected(self):
        self.plan.write_bytes(b"\xff\xfe")
        self.denied("INVALID_WORKFLOW_JSON")

    def test_32_bad_json_plan_rejected(self):
        self.plan.write_text("{not-json", encoding="utf-8")
        self.denied("INVALID_WORKFLOW_JSON")

    def test_33_real_receipts_have_no_host_paths_and_no_fake_signatures(self):
        p, rows = self.check()
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertNotIn(str(self.base), p.stdout.decode("utf-8"))
        for r in rows:
            self.assertEqual(r["schema"], workflow.EVENT)
            self.assertEqual(r["native_receipt"]["authority"]["ambient"], "NONE")
            self.assertEqual(r["native_receipt"]["crash_durability"], "UNCLAIMED")


if __name__ == "__main__":
    unittest.main()
