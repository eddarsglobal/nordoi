"""APP-011: native-backed clean-prefix DAG resume, adversarial on four OS.

Every positive receipt and original historical file is produced by the real
compiled NORDOI Rust engine. Negative cases mutate those authentic outputs.
No simulated business-rule engine, external certification or human tester.
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
ENGINE = Path(os.environ.get("NORDOI_APP011_EXE", str(
    REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
)))
sys.path.insert(0, str(HERE))
import run_app011 as resume


def node(value: int, deps=None, gate: str = "always") -> dict:
    return {"key_code": value, "depends_on": [] if deps is None else deps,
            "gate": gate}


class VerifiedNativeDAGResume(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nordoi_app011_real_")
        self.base = Path(self.temp.name)
        self.root = self.base / "native-output"
        self.root.mkdir()
        self.plan = self.base / "original-plan.json"
        self.prefix = self.base / "native-prefix.jsonl"
        self.engine = ENGINE
        self.source = SOURCE
        self.engine_sha = resume.pinned.engine_sha256(self.engine)
        self.source_sha = resume.pinned.source_tree_sha256(self.source)
        self.generate([node(73), node(59), node(60, [1, 2], "all_committed"),
                       node(80, [3], "all_approved")], prefix_count=2)

    def tearDown(self):
        self.temp.cleanup()

    def generate(self, nodes, prefix_count=2):
        shutil.rmtree(self.root)
        self.root.mkdir()
        self.plan.write_text(json.dumps(
            {"schema": "NORDOI_AGENT_DAG_V1", "nodes": nodes},
            ensure_ascii=True, separators=(",", ":")), encoding="utf-8")
        self.plan_sha = hashlib.sha256(self.plan.read_bytes()).hexdigest()
        p = subprocess.run(
            [sys.executable, "-B",
             str(HERE.parent / "app009_agent_dag/run_app009.py"),
             "--plan", str(self.plan), "--engine", str(self.engine),
             "--source-root", str(self.source),
             "--grant-output-dir", str(self.root),
             "--expected-engine-sha256", self.engine_sha,
             "--expected-source-sha256", self.source_sha],
            cwd=REPO, capture_output=True, timeout=155,
        )
        self.assertEqual(p.returncode, 0, p.stderr.decode("utf-8", "replace"))
        original = p.stdout.splitlines(keepends=True)
        self.assertEqual(len(original), len(nodes))
        self.original_bytes = p.stdout
        self.original_rows = [json.loads(v) for v in p.stdout.splitlines()]
        self.prefix.write_bytes(b"".join(original[:prefix_count]))
        self.prefix_sha = hashlib.sha256(self.prefix.read_bytes()).hexdigest()
        for i in range(prefix_count + 1, len(nodes) + 1):
            f = self.root / f"job-{i:04d}"
            if f.exists():
                shutil.rmtree(f)

    def run_resume(self):
        p = subprocess.run(
            [sys.executable, "-B", str(HERE / "run_app011.py"),
             "--plan", str(self.plan),
             "--prefix-events", str(self.prefix),
             "--grant-output-dir", str(self.root),
             "--engine", str(self.engine), "--source-root", str(self.source),
             "--expected-plan-sha256", self.plan_sha,
             "--expected-prefix-sha256", self.prefix_sha,
             "--expected-engine-sha256", self.engine_sha,
             "--expected-source-sha256", self.source_sha],
            cwd=REPO, capture_output=True, timeout=155,
        )
        try:
            events = [json.loads(x) for x in p.stdout.splitlines()]
        except (UnicodeError, json.JSONDecodeError):
            self.fail(f"non-JSON native resume: {p.stdout!r}, {p.stderr!r}")
        return p, events

    def ok(self, statuses):
        p, rows = self.run_resume()
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        self.assertEqual([row["status"] for row in rows], statuses)
        return p, rows

    def deny(self, code):
        before = self.prefix.read_bytes()
        p, rows = self.run_resume()
        self.assertEqual(p.returncode, 2, (rows, p.stderr))
        self.assertEqual(len(rows), 1, rows)
        self.assertEqual(rows[0]["schema"], resume.RESUME_ERROR)
        self.assertEqual(rows[0]["status"], "REJECTED")
        self.assertEqual(rows[0]["error_code"], code)
        self.assertEqual(rows[0]["publication"], "NOT_STARTED")
        self.assertEqual(self.prefix.read_bytes(), before)

    def historical(self):
        return {p.relative_to(self.root).as_posix(): p.read_bytes()
                for p in self.root.rglob("*") if p.is_file()}

    def final_audit(self, suffix):
        merged = self.base/"merged-native-events.jsonl"
        merged.write_bytes(self.prefix.read_bytes() + suffix)
        report, code = resume.independent.audit(
            self.root, self.plan, merged, self.plan_sha)
        self.assertEqual(code, 0, report)
        self.assertEqual(report["status"], "PASS")
        return report

    def mutate_prefix(self, change):
        newline = b"\r\n" if self.prefix.read_bytes().endswith(b"\r\n") else b"\n"
        events = [json.loads(x) for x in self.prefix.read_bytes().splitlines()]
        change(events)
        self.prefix.write_bytes(b"".join(json.dumps(
            x, separators=(",", ":"), ensure_ascii=True).encode()+newline
            for x in events))
        self.prefix_sha = hashlib.sha256(self.prefix.read_bytes()).hexdigest()

    def test_01_two_real_tasks_preserved_two_native_tasks_continued(self):
        p, rows = self.ok(["COMMITTED", "COMMITTED"])
        self.assertEqual([r["step"] for r in rows], [3, 4])
        self.assertEqual([r["decision"] for r in rows], [1, 1])
        self.final_audit(p.stdout)

    def test_02_resume_from_one_historical_native_task(self):
        self.generate([node(73), node(59), node(60)], prefix_count=1)
        p, rows = self.ok(["COMMITTED", "COMMITTED"])
        self.assertEqual([r["step"] for r in rows], [2, 3])
        self.final_audit(p.stdout)

    def test_03_resume_from_penultimate_native_task(self):
        self.generate([node(73), node(59), node(60)], prefix_count=2)
        p, rows = self.ok(["COMMITTED"])
        self.assertEqual(rows[0]["step"], 3)
        self.final_audit(p.stdout)

    def test_04_historical_native_approval_is_used_to_authorize_join(self):
        self.generate([node(73), node(80), node(59, [1, 2], "all_approved")], 2)
        p, rows = self.ok(["COMMITTED"])
        self.assertEqual(rows[0]["decision"], 0)
        self.final_audit(p.stdout)

    def test_05_historical_native_rejection_is_used_to_authorize_join(self):
        self.generate([node(59), node(0), node(73, [1, 2], "all_rejected")], 2)
        p, rows = self.ok(["COMMITTED"])
        self.assertEqual(rows[0]["decision"], 1)
        self.final_audit(p.stdout)

    def test_06_false_all_approved_branch_stays_skipped(self):
        self.generate([node(73), node(59), node(60, [1, 2], "all_approved")], 2)
        p, rows = self.ok(["SKIPPED"])
        self.assertFalse((self.root/"job-0003").exists())
        self.final_audit(p.stdout)

    def test_07_true_any_approved_branch_runs_native_engine(self):
        self.generate([node(59), node(73), node(60, [1, 2], "any_approved")], 2)
        p, _ = self.ok(["COMMITTED"])
        self.final_audit(p.stdout)

    def test_08_true_any_rejected_branch_runs_native_engine(self):
        self.generate([node(73), node(59), node(60, [1, 2], "any_rejected")], 2)
        p, _ = self.ok(["COMMITTED"])
        self.final_audit(p.stdout)

    def test_09_false_any_rejected_branch_stays_skipped(self):
        self.generate([node(73), node(80), node(60, [1, 2], "any_rejected")], 2)
        p, _ = self.ok(["SKIPPED"])
        self.final_audit(p.stdout)

    def test_10_historical_skipped_parent_stays_uncommitted(self):
        self.generate([node(59), node(73, [1], "all_approved"),
                       node(60, [2], "all_committed")], prefix_count=2)
        p, _ = self.ok(["SKIPPED"])
        self.assertFalse((self.root/"job-0002").exists())
        self.assertFalse((self.root/"job-0003").exists())
        self.final_audit(p.stdout)

    def test_11_an_independent_new_root_runs_after_historical_skip(self):
        self.generate([node(59), node(73, [1], "all_approved"),
                       node(60)], prefix_count=2)
        p, _ = self.ok(["COMMITTED"])
        self.final_audit(p.stdout)

    def test_12_any_approved_ignores_skipped_parent_but_uses_real_approved_one(self):
        self.generate([node(59), node(73, [1], "all_approved"),
                       node(73), node(60, [2, 3], "any_approved")], prefix_count=3)
        p, _ = self.ok(["COMMITTED"])
        self.final_audit(p.stdout)

    def test_13_eight_node_real_native_graph_resumes_suffix(self):
        self.generate([node(x) for x in (0, 20, 59, 60, 61, 73, 90, 100)], 4)
        p, rows = self.ok(["COMMITTED"] * 4)
        self.assertEqual([r["step"] for r in rows], [5, 6, 7, 8])
        self.final_audit(p.stdout)

    def test_14_repeated_codes_are_not_collapsed(self):
        self.generate([node(73), node(73), node(73)], 1)
        p, rows = self.ok(["COMMITTED", "COMMITTED"])
        self.assertEqual([r["step"] for r in rows], [2, 3])
        self.final_audit(p.stdout)

    def test_15_historical_published_bytes_remain_unchanged(self):
        before = self.historical()
        p, _ = self.ok(["COMMITTED", "COMMITTED"])
        self.assertEqual({k: (self.root/k).read_bytes() for k in before}, before)
        self.final_audit(p.stdout)

    def test_16_original_prefix_and_plan_remain_unchanged(self):
        before = (self.prefix.read_bytes(), self.plan.read_bytes())
        self.ok(["COMMITTED", "COMMITTED"])
        self.assertEqual(before, (self.prefix.read_bytes(), self.plan.read_bytes()))

    def test_17_engine_and_source_sha256_are_unchanged(self):
        engine_before = resume.pinned.engine_sha256(self.engine)
        source_before = resume.pinned.source_tree_sha256(self.source)
        self.ok(["COMMITTED", "COMMITTED"])
        self.assertEqual(resume.pinned.engine_sha256(self.engine), engine_before)
        self.assertEqual(resume.pinned.source_tree_sha256(self.source), source_before)

    def test_18_incorrect_plan_pin_is_rejected_before_write(self):
        self.plan_sha = "0"*64
        self.deny("PLAN_SHA256_PIN_MISMATCH")

    def test_19_incorrect_prefix_pin_is_rejected_before_write(self):
        self.prefix_sha = "0"*64
        self.deny("PREFIX_SHA256_PIN_MISMATCH")

    def test_20_incorrect_engine_pin_is_rejected_before_write(self):
        self.engine_sha = "0"*64
        self.deny("ENGINE_PIN_MISMATCH")

    def test_21_incorrect_source_pin_is_rejected_before_write(self):
        self.source_sha = "0"*64
        self.deny("SOURCE_PIN_MISMATCH")

    def test_22_plan_mutation_without_trusted_pin_update_is_rejected(self):
        old = json.loads(self.plan.read_text())
        old["nodes"][2]["key_code"] = 77
        self.plan.write_text(json.dumps(old), encoding="utf-8")
        self.deny("PLAN_SHA256_PIN_MISMATCH")

    def test_23_mutated_historical_receipt_decision_rejected_even_with_new_prefix_pin(self):
        self.mutate_prefix(lambda r: r[0].__setitem__("decision", 0))
        self.deny("HISTORY_NATIVE_DECISION_CONTRADICTION")

    def test_24_changed_historical_native_file_rejected(self):
        p = self.root/"job-0001/quality-decision/report.txt"
        p.write_bytes(p.read_bytes() + b"altered")
        self.deny("HISTORY_REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")

    def test_25_missing_historical_native_file_rejected(self):
        (self.root/"job-0002/quality-decision/report.txt").unlink()
        self.deny("HISTORY_BUNDLE_FILE_SET_CHANGED")

    def test_26_extra_job_is_not_assumed_safe_to_resume(self):
        (self.root/"job-0003").mkdir()
        self.deny("HISTORY_ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_27_stale_in_flight_failed_job_is_never_reused(self):
        p = self.root/"job-0003"
        p.mkdir()
        (p/"unknown").write_bytes(b"may-have-committed")
        self.deny("HISTORY_ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_28_fully_completed_graph_does_not_resume(self):
        self.generate([node(73), node(59), node(60)], prefix_count=3)
        self.deny("DAG_ALREADY_FINISHED")

    def test_29_terminal_error_prefix_is_not_resumed(self):
        def append_error(rows):
            rows.append({"schema": "NORDOI_AGENT_DAG_EVENT_V1", "step":3,
                         "status":"ERROR","gate":"all_committed",
                         "depends_on":[1,2],"plan_sha256":self.plan_sha,
                         "error_code":"NATIVE_OPERATION_FAILED",
                         "native_exit_code":5,"publication":"UNKNOWN"})
        self.mutate_prefix(append_error)
        self.deny("UNCERTAIN_OR_FAILED_PREFIX_DENIED")

    def test_30_truncated_unterminated_prefix_rejected(self):
        self.prefix.write_bytes(self.prefix.read_bytes().rstrip(b"\n\r"))
        self.prefix_sha = hashlib.sha256(self.prefix.read_bytes()).hexdigest()
        self.deny("EVENTS_NOT_NEWLINE_TERMINATED")

    def test_31_mixed_lf_crlf_historical_events_rejected(self):
        lines = self.prefix.read_bytes().splitlines()
        self.prefix.write_bytes(lines[0]+b"\r\n"+lines[1]+b"\n")
        self.prefix_sha = hashlib.sha256(self.prefix.read_bytes()).hexdigest()
        self.deny("EVENT_LINE_INVALID")

    def test_32_duplicate_json_keys_in_historical_events_rejected(self):
        raw = self.prefix.read_bytes()
        self.prefix.write_bytes(raw.replace(b'"status":"COMMITTED"',
                     b'"status":"COMMITTED","status":"COMMITTED"',1))
        self.prefix_sha = hashlib.sha256(self.prefix.read_bytes()).hexdigest()
        self.deny("DUPLICATE_RECEIPT_FIELD")

    def test_33_symlinked_prefix_file_is_refused_when_supported(self):
        target = self.base/"linked-prefix.jsonl"
        try:
            target.symlink_to(self.prefix)
        except (OSError, NotImplementedError):
            self.skipTest("symlink not available")
        self.prefix = target
        self.deny("PREFIX_NOT_BOUNDED_REGULAR_FILE")

    def test_34_symlinked_grant_root_is_refused_when_supported(self):
        link = self.base/"linked-output"
        try:
            link.symlink_to(self.root, target_is_directory=True)
        except (OSError, NotImplementedError):
            self.skipTest("symlink not available")
        self.root = link
        self.deny("GRANT_NOT_REAL_DIRECTORY")

    def test_35_replaying_same_prefix_again_after_success_is_rejected(self):
        self.ok(["COMMITTED", "COMMITTED"])
        self.deny("HISTORY_ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_36_native_failure_after_checkpoint_preserves_previous_commits(self):
        self.generate([node(73), node(59)], prefix_count=1)
        before = self.historical()
        self.source = self.base/"modified-source"
        shutil.copytree(SOURCE, self.source)
        rule = self.source/"app/rules.noi"
        old = rule.read_text(encoding="utf-8")
        self.assertIn("else { 0 }", old)
        rule.write_text(old.replace("else { 0 }", "else { 2 }"), encoding="utf-8")
        self.source_sha = resume.pinned.source_tree_sha256(self.source)
        proc, rows = self.run_resume()
        self.assertEqual(proc.returncode, 3, rows)
        self.assertEqual([r["status"] for r in rows], ["ERROR"])
        self.assertEqual(rows[0]["error_code"], "NATIVE_OPERATION_FAILED")
        self.assertEqual(rows[0]["publication"], "UNKNOWN")
        self.assertEqual({k: (self.root/k).read_bytes() for k in before}, before)

    def test_37_malformed_plan_with_matching_digest_still_refused(self):
        plan = json.loads(self.plan.read_text(encoding="utf-8"))
        plan["nodes"][2]["depends_on"] = [3]
        self.plan.write_text(json.dumps(plan),encoding="utf-8")
        self.plan_sha = hashlib.sha256(self.plan.read_bytes()).hexdigest()
        self.deny("DAG_REFERENCE_NOT_PRIOR")

    def test_38_resume_output_only_contains_suffix_without_host_paths(self):
        p, rows = self.ok(["COMMITTED", "COMMITTED"])
        self.assertEqual(len(rows), 2)
        self.assertNotIn(str(self.base), p.stdout.decode("utf-8"))
        self.assertTrue(all(r["schema"]=="NORDOI_AGENT_DAG_EVENT_V1" for r in rows))
        self.assertTrue(all(r["native_receipt"]["authority"]["ambient"]=="NONE"
                            for r in rows))

    def test_39_explicit_lf_prefix_produces_lf_suffix_on_any_os(self):
        raw = self.prefix.read_bytes().replace(b"\r\n", b"\n")
        self.prefix.write_bytes(raw)
        self.prefix_sha = hashlib.sha256(raw).hexdigest()
        p, _ = self.ok(["COMMITTED", "COMMITTED"])
        self.assertNotIn(b"\r", p.stdout)
        self.final_audit(p.stdout)

    def test_40_explicit_crlf_prefix_produces_crlf_suffix_on_any_os(self):
        raw = self.prefix.read_bytes().replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
        self.prefix.write_bytes(raw)
        self.prefix_sha = hashlib.sha256(raw).hexdigest()
        p, _ = self.ok(["COMMITTED", "COMMITTED"])
        self.assertEqual(p.stdout.count(b"\r\n"), 2)
        self.final_audit(p.stdout)


if __name__ == "__main__":
    unittest.main()
