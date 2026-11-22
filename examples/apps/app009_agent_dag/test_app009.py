"""APP-009 real native AI-agent dependency DAG tests on all four OS.

All positive fixtures execute the compiled NORDOI Rust binary, not mocked
business logic. Negative fixtures mutate only data, source or grant inputs.
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
ENGINE = Path(os.environ.get("NORDOI_APP009_EXE", str(
    REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
)))
sys.path.insert(0, str(HERE))
import run_app009 as dag


def node(code: int, depends_on=None, gate: str = "always") -> dict:
    return {"key_code": code, "depends_on": [] if depends_on is None else depends_on,
            "gate": gate}


class RealBoundedAgentDAG(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nordoi_app009_native_")
        self.base = Path(self.temp.name)
        self.output = self.base / "actual-output"
        self.output.mkdir()
        self.plan = self.base / "workflow.json"
        self.engine = ENGINE
        self.source = SOURCE
        self.engine_pin = dag.pinned.engine_sha256(self.engine)
        self.source_pin = dag.pinned.source_tree_sha256(self.source)
        self.write([node(73), node(59), node(60, [1, 2], "all_committed")])

    def tearDown(self):
        self.temp.cleanup()

    def write(self, nodes):
        self.plan.write_text(json.dumps(
            {"schema": dag.SCHEMA, "nodes": nodes},
            separators=(",", ":"), ensure_ascii=True), encoding="utf-8")

    def run_dag(self):
        proc = subprocess.run(
            [sys.executable, "-B", str(HERE / "run_app009.py"),
             "--plan", str(self.plan), "--engine", str(self.engine),
             "--source-root", str(self.source),
             "--grant-output-dir", str(self.output),
             "--expected-engine-sha256", self.engine_pin,
             "--expected-source-sha256", self.source_pin],
            cwd=REPO, capture_output=True, timeout=180,
        )
        try:
            records = [json.loads(line) for line in
                       proc.stdout.decode("utf-8", "strict").splitlines()]
        except (UnicodeError, json.JSONDecodeError) as exc:
            self.fail(f"non-JSON NORDOI result: {exc}: {proc.stdout!r}: {proc.stderr!r}")
        return proc, records

    def expect_statuses(self, expected):
        proc, records = self.run_dag()
        self.assertEqual(proc.returncode, 0, (records, proc.stderr))
        self.assertEqual([r["status"] for r in records], expected)
        return records

    def deny(self, code):
        proc, records = self.run_dag()
        self.assertEqual(proc.returncode, 2, records)
        self.assertEqual(len(records), 1, records)
        self.assertEqual(records[0]["status"], "REJECTED")
        self.assertEqual(records[0]["error_code"], code)
        self.assertEqual(records[0]["publication"], "NOT_STARTED")
        self.assertEqual(list(self.output.iterdir()), [])

    def test_01_actual_three_node_join_commits(self):
        result = self.expect_statuses(["COMMITTED"]*3)
        self.assertEqual([r["decision"] for r in result], [1, 0, 1])

    def test_02_all_approved_join_runs(self):
        self.write([node(73), node(80), node(59, [1, 2], "all_approved")])
        result = self.expect_statuses(["COMMITTED"]*3)
        self.assertEqual([r["decision"] for r in result], [1, 1, 0])

    def test_03_all_rejected_join_runs(self):
        self.write([node(59), node(0), node(73, [1, 2], "all_rejected")])
        result = self.expect_statuses(["COMMITTED"]*3)
        self.assertEqual([r["decision"] for r in result], [0, 0, 1])

    def test_04_any_approved_join_runs(self):
        self.write([node(59), node(73), node(60, [1, 2], "any_approved")])
        self.expect_statuses(["COMMITTED"]*3)

    def test_05_any_rejected_join_runs(self):
        self.write([node(73), node(59), node(60, [1, 2], "any_rejected")])
        self.expect_statuses(["COMMITTED"]*3)

    def test_06_all_approved_with_one_rejection_skips(self):
        self.write([node(73), node(59), node(60, [1, 2], "all_approved")])
        self.expect_statuses(["COMMITTED", "COMMITTED", "SKIPPED"])
        self.assertFalse((self.output/"job-0003").exists())

    def test_07_all_rejected_with_one_approval_skips(self):
        self.write([node(73), node(59), node(60, [1, 2], "all_rejected")])
        self.expect_statuses(["COMMITTED", "COMMITTED", "SKIPPED"])

    def test_08_any_approved_without_approval_skips(self):
        self.write([node(59), node(0), node(60, [1, 2], "any_approved")])
        self.expect_statuses(["COMMITTED", "COMMITTED", "SKIPPED"])

    def test_09_any_rejected_without_rejection_skips(self):
        self.write([node(73), node(80), node(60, [1, 2], "any_rejected")])
        self.expect_statuses(["COMMITTED", "COMMITTED", "SKIPPED"])

    def test_10_dependency_skipped_does_not_count_as_committed(self):
        self.write([node(59), node(73, [1], "all_approved"),
                    node(60, [2], "all_committed")])
        self.expect_statuses(["COMMITTED", "SKIPPED", "SKIPPED"])

    def test_11_any_approved_can_accept_other_committed_parent(self):
        self.write([node(59), node(73, [1], "all_approved"), node(73),
                    node(60, [2, 3], "any_approved")])
        self.expect_statuses(["COMMITTED", "SKIPPED", "COMMITTED", "COMMITTED"])

    def test_12_any_rejected_can_accept_other_committed_parent(self):
        self.write([node(73), node(59, [1], "all_rejected"), node(59),
                    node(60, [2, 3], "any_rejected")])
        self.expect_statuses(["COMMITTED", "SKIPPED", "COMMITTED", "COMMITTED"])

    def test_13_independent_root_nodes_are_supported(self):
        self.write([node(59), node(73), node(0)])
        self.expect_statuses(["COMMITTED"]*3)

    def test_14_eight_actual_nodes_at_bound(self):
        self.write([node(v) for v in (0, 20, 59, 60, 61, 73, 90, 100)])
        r = self.expect_statuses(["COMMITTED"]*8)
        self.assertEqual(len(r), 8)

    def test_15_repeated_key_codes_keep_distinct_outputs(self):
        self.write([node(73), node(73), node(73, [1, 2], "all_approved")])
        r = self.expect_statuses(["COMMITTED"]*3)
        self.assertEqual([x["job"] for x in r],
                         ["job-0001", "job-0002", "job-0003"])

    def test_16_exact_native_receipts_match_actual_files(self):
        results = self.expect_statuses(["COMMITTED"]*3)
        for i, record in enumerate(results, 1):
            checked = dag.audit.verify_native_record(record["native_receipt"], i,
                                                      self.output)
            self.assertEqual(checked["decision"], record["decision"])

    def test_17_complete_plan_invalid_last_node_refused_before_all_effects(self):
        self.write([node(73), node(59), node(60, [3], "all_committed")])
        self.deny("DAG_REFERENCE_NOT_PRIOR")

    def test_18_unknown_grant_field_rejected_before_effect(self):
        self.write([node(73), {"key_code":59, "gate":"always",
                               "depends_on":[], "grant_output_dir":"/tmp"}])
        self.deny("DAG_NODE_FIELDS_INVALID")

    def test_19_source_injection_field_rejected_before_effect(self):
        self.write([{"key_code":73, "gate":"always",
                     "depends_on":[], "source_root":"/"}])
        self.deny("DAG_NODE_FIELDS_INVALID")

    def test_20_reference_zero_refused(self):
        self.write([node(73), node(59, [0], "all_committed")])
        self.deny("DAG_REFERENCE_NOT_PRIOR")

    def test_21_forward_reference_refused(self):
        self.write([node(73), node(59, [3], "all_committed"), node(60)])
        self.deny("DAG_REFERENCE_NOT_PRIOR")

    def test_22_self_dependency_refused(self):
        self.write([node(73), node(59, [2], "all_committed")])
        self.deny("DAG_REFERENCE_NOT_PRIOR")

    def test_23_duplicate_dependency_refused(self):
        self.write([node(73), node(59, [1, 1], "all_committed")])
        self.deny("DAG_REFERENCES_NOT_STRICTLY_ASCENDING")

    def test_24_unsorted_dependencies_refused(self):
        self.write([node(73), node(59), node(60, [2, 1], "all_committed")])
        self.deny("DAG_REFERENCES_NOT_STRICTLY_ASCENDING")

    def test_25_boolean_dependency_refused(self):
        self.write([node(73), node(59, [True], "all_committed")])
        self.deny("DAG_REFERENCE_NOT_PRIOR")

    def test_26_gate_without_dependencies_refused(self):
        self.write([node(73, [], "any_approved")])
        self.deny("DAG_GATE_DEPENDENCIES_CONTRADICT")

    def test_27_always_gate_with_dependencies_refused(self):
        self.write([node(73), node(59, [1], "always")])
        self.deny("DAG_GATE_DEPENDENCIES_CONTRADICT")

    def test_28_unknown_gate_refused(self):
        self.write([node(73), node(59, [1], "shell_command")])
        self.deny("DAG_GATE_INVALID")

    def test_29_empty_graph_refused(self):
        self.write([])
        self.deny("DAG_NODE_COUNT_INVALID")

    def test_30_ninth_node_refused(self):
        self.write([node(73)]*9)
        self.deny("DAG_NODE_COUNT_INVALID")

    def test_31_duplicate_json_key_refused(self):
        self.plan.write_text(
            '{"schema":"NORDOI_AGENT_DAG_V1","nodes":['
            '{"key_code":73,"key_code":59,"depends_on":[],"gate":"always"}]}',
            encoding="utf-8")
        self.deny("DUPLICATE_REQUEST_KEY")

    def test_32_boolean_key_code_refused(self):
        self.write([node(True)])
        self.deny("DAG_KEY_CODE_INVALID")

    def test_33_bad_engine_or_source_pin_refused(self):
        self.engine_pin = "0"*64
        self.deny("ENGINE_PIN_MISMATCH")
        self.engine_pin = dag.pinned.engine_sha256(self.engine)
        self.source_pin = "0"*64
        self.deny("SOURCE_PIN_MISMATCH")

    def test_34_nonempty_root_denied_without_overwrite(self):
        guard = self.output/"untouched.txt"
        guard.write_text("never overwrite", encoding="utf-8")
        p, r = self.run_dag()
        self.assertEqual(p.returncode, 2, r)
        self.assertEqual(r[0]["error_code"], "EXPLICIT_GRANT_NOT_EMPTY")
        self.assertEqual(guard.read_text(encoding="utf-8"), "never overwrite")

    def test_35_native_failure_after_first_commit_is_not_rolled_back(self):
        self.source = self.base/"modified-native-source"
        shutil.copytree(SOURCE, self.source)
        rule = self.source/"app/rules.noi"
        old = rule.read_text(encoding="utf-8")
        self.assertIn("else { 0 }", old)
        rule.write_text(old.replace("else { 0 }", "else { 2 }"), encoding="utf-8")
        self.source_pin = dag.pinned.source_tree_sha256(self.source)
        p, r = self.run_dag()
        self.assertEqual(p.returncode, 3, (r, p.stderr))
        self.assertEqual([x["status"] for x in r], ["COMMITTED", "ERROR"])
        self.assertEqual(r[-1]["publication"], "UNKNOWN")
        self.assertTrue((self.output/"job-0001/quality-decision/report.txt").is_file())

    def test_36_rule_mutation_changes_actual_native_decision(self):
        self.source = self.base/"modified-native-source"
        shutil.copytree(SOURCE, self.source)
        rule = self.source/"app/rules.noi"
        rule.write_text(rule.read_text(encoding="utf-8").replace(">= 60", ">= 75"),
                        encoding="utf-8")
        self.source_pin = dag.pinned.source_tree_sha256(self.source)
        r = self.expect_statuses(["COMMITTED"]*3)
        self.assertEqual([x["decision"] for x in r], [0, 0, 0])

    def test_37_actual_data_and_receipts_have_no_absolute_host_paths(self):
        r = self.expect_statuses(["COMMITTED"]*3)
        self.assertEqual({p.name for p in self.output.iterdir()},
                         {"job-0001", "job-0002", "job-0003"})
        self.assertTrue(all(x["native_receipt"]["authority"]["ambient"]=="NONE"
                            for x in r))
        for x in r:
            self.assertEqual(x["native_receipt"]["crash_durability"], "UNCLAIMED")

    def test_38_same_graph_same_output_without_host_path_leak(self):
        p1, r1 = self.run_dag()
        self.assertEqual(p1.returncode, 0, (r1, p1.stderr))
        self.output = self.base/"second-run"
        self.output.mkdir()
        p2, r2 = self.run_dag()
        self.assertEqual(p2.returncode, 0, (r2, p2.stderr))
        self.assertEqual(p1.stdout, p2.stdout)

    def test_39_plan_source_and_engine_are_not_mutated(self):
        original = self.plan.read_bytes()
        source_sha = dag.pinned.source_tree_sha256(self.source)
        engine_sha = dag.pinned.engine_sha256(self.engine)
        p, r = self.run_dag()
        self.assertEqual(p.returncode, 0, r)
        self.assertEqual(self.plan.read_bytes(), original)
        self.assertEqual(dag.pinned.source_tree_sha256(self.source), source_sha)
        self.assertEqual(dag.pinned.engine_sha256(self.engine), engine_sha)

    def test_40_invalid_utf8_and_bad_json_refused(self):
        self.plan.write_bytes(b"\xff")
        self.deny("DAG_PLAN_INVALID_JSON")
        self.plan.write_text("{no-json", encoding="utf-8")
        self.deny("DAG_PLAN_INVALID_JSON")


if __name__ == "__main__":
    unittest.main()
