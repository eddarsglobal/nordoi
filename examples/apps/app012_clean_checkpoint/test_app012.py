"""APP-012: 40 authentic native DAG clean-checkpoint and adversarial tests.

Positive histories are produced directly by real Rust NORDOI in APP-012 with
NO full-DAG execution and NO deleting simulated future job directories.
Every valid clean prefix is checked by independent APP-010; end-to-end
continuation invokes APP-011 and recombines real native events and files.
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
ENGINE = Path(os.environ.get("NORDOI_APP012_EXE", str(
    REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
)))
sys.path.insert(0, str(HERE))
import run_app012 as bounded
sys.path.insert(0, str(HERE.parent / "app011_verified_resume"))
import run_app011 as resume


def node(value: int, deps=None, gate: str = "always") -> dict:
    return {"key_code": value, "depends_on": [] if deps is None else deps,
            "gate": gate}


class NativeCleanCheckpoint(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="nordoi_app012_native_")
        self.base = Path(self.tmp.name)
        self.root = self.base / "actual-native-files"
        self.root.mkdir()
        self.plan = self.base / "original-dag.json"
        self.events = self.base / "true-native-prefix.jsonl"
        self.source = SOURCE
        self.engine = ENGINE
        self.stop = 2
        self.source_sha = bounded.pinned.source_tree_sha256(self.source)
        self.engine_sha = bounded.pinned.engine_sha256(self.engine)
        self.write([node(73), node(59), node(60, [1, 2], "all_committed"),
                    node(80, [3], "all_approved")])

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, nodes):
        self.plan.write_text(json.dumps({"schema": "NORDOI_AGENT_DAG_V1",
            "nodes": nodes}, ensure_ascii=True,
            separators=(",", ":")), encoding="utf-8")
        self.plan_sha = hashlib.sha256(self.plan.read_bytes()).hexdigest()

    def invoke(self):
        cmd = [sys.executable, "-B", str(HERE / "run_app012.py"),
               "--plan", str(self.plan), "--engine", str(self.engine),
               "--source-root", str(self.source),
               "--grant-output-dir", str(self.root),
               "--stop-after", str(self.stop),
               "--expected-plan-sha256", self.plan_sha,
               "--expected-engine-sha256", self.engine_sha,
               "--expected-source-sha256", self.source_sha]
        p = subprocess.run(cmd, cwd=REPO, capture_output=True, timeout=150)
        try:
            records = [json.loads(x) for x in p.stdout.splitlines()]
        except (ValueError, UnicodeError) as error:
            self.fail(f"non-json checkpoint: {p.stdout!r}, {p.stderr!r}, {error!r}")
        return p, records

    def accepted(self, statuses=None):
        p, records = self.invoke()
        self.assertEqual(p.returncode, 0, (records, p.stderr))
        self.assertEqual(len(records), self.stop)
        if statuses is not None:
            self.assertEqual([r["status"] for r in records], statuses)
        self.assertEqual(p.stdout.count(b"\n"), self.stop)
        self.assertNotIn(b"\r", p.stdout)
        self.events.write_bytes(p.stdout)
        report, code = bounded.independent.audit(
            self.root, self.plan, self.events, self.plan_sha)
        self.assertEqual(code, 3, report)
        self.assertEqual(report["status"], "INCONCLUSIVE")
        self.assertEqual(report["events_checked"], self.stop)
        return p, records, report

    def denied(self, expected):
        p, rows = self.invoke()
        self.assertEqual(p.returncode, 2, (rows, p.stderr))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["schema"], bounded.SCHEMA)
        self.assertEqual(rows[0]["status"], "REJECTED")
        self.assertEqual(rows[0]["error_code"], expected)
        self.assertEqual(rows[0]["publication"], "NOT_STARTED")
        self.assertEqual(list(self.root.iterdir()), [])

    def continue_real(self, original: bytes):
        prefix_sha = hashlib.sha256(original).hexdigest()
        cmd = [sys.executable, "-B",
               str(HERE.parent / "app011_verified_resume/run_app011.py"),
               "--plan", str(self.plan), "--prefix-events", str(self.events),
               "--grant-output-dir", str(self.root),
               "--engine", str(self.engine), "--source-root", str(self.source),
               "--expected-plan-sha256", self.plan_sha,
               "--expected-prefix-sha256", prefix_sha,
               "--expected-engine-sha256", self.engine_sha,
               "--expected-source-sha256", self.source_sha]
        p = subprocess.run(cmd, cwd=REPO, capture_output=True, timeout=150)
        self.assertEqual(p.returncode, 0, (p.stdout, p.stderr))
        merged = self.base / "final-verifiable-native-dag.jsonl"
        merged.write_bytes(original + p.stdout)
        report, code = bounded.independent.audit(
            self.root, self.plan, merged, self.plan_sha)
        self.assertEqual(code, 0, report)
        self.assertEqual(report["status"], "PASS")
        self.assertEqual(report["events_checked"], len(
            json.loads(self.plan.read_text())["nodes"]))
        return p, report

    def test_01_genuine_two_node_clean_native_checkpoint(self):
        p, rows, report = self.accepted(["COMMITTED", "COMMITTED"])
        self.assertEqual([x["decision"] for x in rows], [1, 0])
        self.assertEqual(report["committed_jobs_verified"], 2)
        self.assertEqual({x.name for x in self.root.iterdir()},
                         {"job-0001", "job-0002"})

    def test_02_one_node_real_checkpoint(self):
        self.stop = 1
        p, rows, report = self.accepted(["COMMITTED"])
        self.assertEqual(report["committed_jobs_verified"], 1)
        self.assertFalse((self.root/"job-0002").exists())

    def test_03_three_of_four_native_nodes_checkpoint(self):
        self.stop = 3
        p, rows, report = self.accepted(["COMMITTED"]*3)
        self.assertEqual([r["decision"] for r in rows], [1, 0, 1])
        self.assertFalse((self.root/"job-0004").exists())

    def test_04_seven_of_eight_nodes_checkpoint(self):
        self.write([node(v) for v in (0, 20, 59, 60, 61, 73, 90, 100)])
        self.stop = 7
        p, rows, report = self.accepted(["COMMITTED"]*7)
        self.assertEqual(report["committed_jobs_verified"], 7)
        self.assertFalse((self.root/"job-0008").exists())

    def test_05_identical_genuine_native_receipts_audited(self):
        _, rows, _ = self.accepted()
        for i, record in enumerate(rows, 1):
            result = bounded.native_audit.verify_native_record(
                record["native_receipt"], i, self.root)
            self.assertEqual(result["decision"], record["decision"])

    def test_06_no_full_dag_execution_before_checkpoint(self):
        _, _, _ = self.accepted()
        self.assertFalse((self.root/"job-0003").exists())
        self.assertFalse((self.root/"job-0004").exists())

    def test_07_full_end_to_end_resume_from_real_checkpoint(self):
        p, _, _ = self.accepted(["COMMITTED", "COMMITTED"])
        suffix, report = self.continue_real(p.stdout)
        rows = [json.loads(x) for x in suffix.stdout.splitlines()]
        self.assertEqual([x["step"] for x in rows], [3, 4])
        self.assertEqual(report["committed_jobs_verified"], 4)

    def test_08_real_one_node_checkpoint_then_resume_all_remaining(self):
        self.stop = 1
        p, _, _ = self.accepted(["COMMITTED"])
        suffix, report = self.continue_real(p.stdout)
        self.assertEqual(len(suffix.stdout.splitlines()), 3)
        self.assertEqual(report["committed_jobs_verified"], 4)

    def test_09_native_checkpoint_then_resume_last_node(self):
        self.stop = 3
        p, _, _ = self.accepted(["COMMITTED"]*3)
        suffix, report = self.continue_real(p.stdout)
        self.assertEqual(len(suffix.stdout.splitlines()), 1)
        self.assertEqual(report["committed_jobs_verified"], 4)

    def test_10_historical_jobs_preserved_after_real_resume(self):
        p, _, _ = self.accepted(["COMMITTED", "COMMITTED"])
        files = {f.relative_to(self.root).as_posix(): f.read_bytes()
                 for f in self.root.rglob("*") if f.is_file()}
        self.continue_real(p.stdout)
        self.assertEqual({x: (self.root/x).read_bytes() for x in files}, files)

    def test_11_prefix_bytes_preserved_after_native_resume(self):
        p, _, _ = self.accepted()
        before = self.events.read_bytes()
        self.continue_real(p.stdout)
        self.assertEqual(self.events.read_bytes(), before)

    def test_12_true_all_approved_join_is_committed(self):
        self.write([node(73), node(80), node(59, [1,2], "all_approved"),
                    node(73, [3], "all_rejected")])
        self.stop = 3
        p, rows, report = self.accepted(["COMMITTED"]*3)
        self.assertEqual([r["decision"] for r in rows], [1, 1, 0])
        self.continue_real(p.stdout)

    def test_13_false_all_approved_join_is_skipped(self):
        self.write([node(73), node(59), node(60, [1,2], "all_approved"),
                    node(80)])
        self.stop = 3
        p, rows, report = self.accepted(["COMMITTED","COMMITTED","SKIPPED"])
        self.assertFalse((self.root/"job-0003").exists())
        self.assertEqual(report["skipped_nodes_verified"], 1)
        self.continue_real(p.stdout)

    def test_14_true_any_approved_join_committed(self):
        self.write([node(59), node(73), node(60, [1,2], "any_approved"), node(80)])
        self.stop = 3
        p, rows, _ = self.accepted(["COMMITTED"]*3)
        self.continue_real(p.stdout)

    def test_15_true_any_rejected_join_committed(self):
        self.write([node(73), node(59), node(60, [1,2], "any_rejected"), node(80)])
        self.stop = 3
        p, rows, _ = self.accepted(["COMMITTED"]*3)
        self.continue_real(p.stdout)

    def test_16_skipped_dependency_is_not_approved_or_committed(self):
        self.write([node(59), node(73, [1], "all_approved"),
                    node(60, [2], "all_committed"), node(73)])
        self.stop = 3
        p, rows, report = self.accepted(["COMMITTED","SKIPPED","SKIPPED"])
        self.assertEqual(report["committed_jobs_verified"], 1)
        self.assertEqual(report["skipped_nodes_verified"], 2)
        self.continue_real(p.stdout)

    def test_17_independent_root_after_skip_runs_real_native(self):
        self.write([node(59), node(73, [1], "all_approved"), node(73), node(60)])
        self.stop = 3
        p, rows, _ = self.accepted(["COMMITTED","SKIPPED","COMMITTED"])
        self.continue_real(p.stdout)

    def test_18_repeated_native_codes_are_independent_job_directories(self):
        self.write([node(73),node(73),node(73),node(73)])
        self.stop = 3
        p, rows, _ = self.accepted(["COMMITTED"]*3)
        self.assertEqual([r["job"] for r in rows],["job-0001","job-0002","job-0003"])

    def test_19_stdout_is_exact_original_jsonl_digest(self):
        p, _, report = self.accepted()
        self.assertEqual(report["original_events_sha256"],
                         hashlib.sha256(p.stdout).hexdigest())
        self.assertTrue(p.stdout.endswith(b"\n"))

    def test_20_plan_sha256_bound_to_each_real_event(self):
        _, rows, _ = self.accepted()
        self.assertTrue(all(r["plan_sha256"] == self.plan_sha for r in rows))

    def test_21_stop_after_zero_denied_before_effect(self):
        self.stop = 0
        self.denied("CHECKPOINT_MUST_BE_NONEMPTY_PROPER_PREFIX")

    def test_22_stop_after_negative_denied_before_effect(self):
        self.stop = -1
        self.denied("CHECKPOINT_MUST_BE_NONEMPTY_PROPER_PREFIX")

    def test_23_stop_at_all_nodes_denied_not_fake_partial(self):
        self.stop = 4
        self.denied("CHECKPOINT_MUST_BE_NONEMPTY_PROPER_PREFIX")

    def test_24_stop_beyond_all_nodes_denied(self):
        self.stop = 8
        self.denied("CHECKPOINT_MUST_BE_NONEMPTY_PROPER_PREFIX")

    def test_25_empty_plan_denied(self):
        self.write([])
        self.denied("DAG_NODE_COUNT_INVALID")

    def test_26_cycle_or_future_dep_denied_before_effect(self):
        self.write([node(73),node(59,[3],"all_committed"),node(60)])
        self.denied("DAG_REFERENCE_NOT_PRIOR")

    def test_27_self_dependency_denied(self):
        self.write([node(73),node(59,[2],"all_committed"),node(60),node(80)])
        self.denied("DAG_REFERENCE_NOT_PRIOR")

    def test_28_permission_injected_into_agent_data_denied(self):
        data=json.loads(self.plan.read_text())
        data["nodes"][3]["grant_output_dir"]="/tmp"
        self.write(data["nodes"])
        self.denied("DAG_NODE_FIELDS_INVALID")

    def test_29_duplicated_refs_denied_before_effect(self):
        self.write([node(73),node(59),node(60,[1,1],"all_committed"),node(80)])
        self.denied("DAG_REFERENCES_NOT_ASCENDING")

    def test_30_untrusted_plan_mutation_denied_by_pin(self):
        data=json.loads(self.plan.read_text())
        data["nodes"][1]["key_code"]=80
        self.plan.write_text(json.dumps(data),encoding="utf-8")
        self.denied("PLAN_SHA256_PIN_MISMATCH")

    def test_31_wrong_engine_sha_pin_denied(self):
        self.engine_sha="0"*64
        self.denied("ENGINE_PIN_MISMATCH")

    def test_32_wrong_source_sha_pin_denied(self):
        self.source_sha="0"*64
        self.denied("SOURCE_PIN_MISMATCH")

    def test_33_wrong_plan_sha_pin_denied(self):
        self.plan_sha="0"*64
        self.denied("PLAN_SHA256_PIN_MISMATCH")

    def test_34_nonempty_filewrite_grant_denied_no_overwrite(self):
        marker=self.root/"original.user.file"
        marker.write_bytes(b"preserve")
        p,rows=self.invoke()
        self.assertEqual(p.returncode,2,rows)
        self.assertEqual(rows[0]["error_code"],"EXPLICIT_GRANT_NOT_EMPTY")
        self.assertEqual(marker.read_bytes(),b"preserve")
        self.assertEqual({x.name for x in self.root.iterdir()},{"original.user.file"})

    def test_35_reexecute_same_prefix_is_denied_not_overwritten(self):
        self.accepted()
        before={f.relative_to(self.root).as_posix():f.read_bytes()
                for f in self.root.rglob("*") if f.is_file()}
        p,rows=self.invoke()
        self.assertEqual(p.returncode,2,rows)
        self.assertEqual(rows[0]["error_code"],"EXPLICIT_GRANT_NOT_EMPTY")
        self.assertEqual({k:(self.root/k).read_bytes() for k in before},before)

    def test_36_symlinked_grant_denied_when_supported(self):
        link=self.base/"linked-grant"
        try:
            link.symlink_to(self.root,target_is_directory=True)
        except (OSError,NotImplementedError):
            self.skipTest("symlink unavailable")
        self.root=link
        self.denied("EXPLICIT_GRANT_NOT_REAL_DIRECTORY")

    def test_37_unknown_plan_json_field_rejected(self):
        obj=json.loads(self.plan.read_text())
        obj["commands"]="never_execute"
        self.plan.write_text(json.dumps(obj),encoding="utf-8")
        self.plan_sha=hashlib.sha256(self.plan.read_bytes()).hexdigest()
        self.denied("DAG_SCHEMA_INVALID")

    def test_38_duplicate_json_plan_key_rejected(self):
        raw=self.plan.read_bytes()
        raw=raw.replace(b'"schema":"NORDOI_AGENT_DAG_V1"',
                        b'"schema":"NORDOI_AGENT_DAG_V1","schema":"NORDOI_AGENT_DAG_V1"',1)
        self.plan.write_bytes(raw)
        self.plan_sha=hashlib.sha256(raw).hexdigest()
        self.denied("DUPLICATE_RECEIPT_FIELD")

    def test_39_actual_native_failure_is_terminal_not_clean_checkpoint(self):
        self.source=self.base/"mutated-native-source"
        shutil.copytree(SOURCE,self.source)
        rule=self.source/"app/rules.noi"
        raw=rule.read_text(encoding="utf-8")
        self.assertIn("else { 0 }",raw)
        rule.write_text(raw.replace("else { 0 }","else { 2 }"),
                        encoding="utf-8")
        self.source_sha=bounded.pinned.source_tree_sha256(self.source)
        p,rows=self.invoke()
        self.assertEqual(p.returncode,3,(rows,p.stderr))
        self.assertEqual([r["status"] for r in rows],["COMMITTED","ERROR"])
        self.assertEqual(rows[-1]["publication"],"UNKNOWN")
        self.events.write_bytes(p.stdout)
        report,code=bounded.independent.audit(
            self.root,self.plan,self.events,self.plan_sha)
        self.assertEqual(code,3,report)
        self.assertEqual(report["status"],"INCONCLUSIVE")
        # APP-011 must refuse a suffix that might have been committed.
        cmd=[sys.executable,"-B",
             str(HERE.parent/"app011_verified_resume/run_app011.py"),
             "--plan",str(self.plan),"--prefix-events",str(self.events),
             "--grant-output-dir",str(self.root),
             "--engine",str(self.engine),"--source-root",str(self.source),
             "--expected-plan-sha256",self.plan_sha,
             "--expected-prefix-sha256",hashlib.sha256(p.stdout).hexdigest(),
             "--expected-engine-sha256",self.engine_sha,
             "--expected-source-sha256",self.source_sha]
        check=subprocess.run(cmd,cwd=REPO,capture_output=True,timeout=30)
        self.assertEqual(check.returncode,2,check.stdout)

    def test_40_fresh_checkpoint_does_not_claim_external_certification(self):
        p,rows,report=self.accepted()
        self.assertEqual(report["attestation"],
                         "READ_ONLY_DAG_AND_REAL_FILE_CONSISTENCY_NOT_SIGNED")
        self.assertEqual(report["p27_native_proof"],
                         "SHAPE_ONLY_NOT_CRYPTOGRAPHICALLY_RECOMPUTED")
        self.assertEqual(report["external_pin_authentication"],
                         "NOT_PERFORMED")
        self.assertEqual(report["crash_durability"],"UNCLAIMED")


if __name__ == "__main__":
    unittest.main()
