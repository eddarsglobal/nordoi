"""CERT-015-R1: independent adversarial Bureau review of APP-015 at pinned C20.

Independent fixture-building directly invokes real NORDOI Rust via APP-012,
then real APP-013 seal and APP-014 portable pack. No import or mock of the
APP-015 unittest fixture. The pivotal falsification creates *two different
internally consistent* native packages using different genuine .noi rules:
both pass APP-014 consistency, but the receiver's trusted source pin must
separate the histories by reproducible native result.

One CI job per platform; no signers, no production or privileged capabilities.
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
ENGINE = REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
SOURCE = REPO / "examples/apps/app003_integrated_quality/src"
sys.path.insert(0, str(HERE.parent / "app015_independent_native_replay"))
import replay_app015 as verifier


def command(script: Path, *args, timeout: int = 180):
    p = subprocess.run([sys.executable, "-B", str(script), *map(str,args)],
                       cwd=REPO, capture_output=True, timeout=timeout)
    try:
        rows = [json.loads(line) for line in p.stdout.splitlines()]
    except (ValueError, UnicodeError):
        raise AssertionError(f"NON_JSON output={p.stdout!r} err={p.stderr!r}")
    return p, rows


def expected(p, rows, code: int, status: str):
    assert p.returncode == code, (rows, p.stderr)
    assert len(rows) == 1 and rows[0]["status"] == status, rows
    return rows[0]


class BureauCertificationR1(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="nordoi_cert015_r1_")
        cls.base = Path(cls.temp.name)
        cls.good_source = SOURCE
        cls.good_source_sha = verifier.pinned.source_tree_sha256(SOURCE)
        cls.engine_sha = verifier.pinned.engine_sha256(ENGINE)
        cls.alternate_source = cls.base/"genuine-other-native-source"
        shutil.copytree(SOURCE, cls.alternate_source)
        path = cls.alternate_source/"app/rules.noi"
        original = path.read_text(encoding="utf-8")
        assert "score >= 60" in original
        path.write_text(original.replace("score >= 60", "score >= 1000"),
                        encoding="utf-8")
        cls.alternate_source_sha = verifier.pinned.source_tree_sha256(
            cls.alternate_source)
        cls.packages = {}
        for label, source, pin in (
            ("original", SOURCE, cls.good_source_sha),
            ("alternate", cls.alternate_source, cls.alternate_source_sha),
        ):
            cls.packages[label] = cls.make_real_package(label, source, pin)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    @classmethod
    def make_real_package(cls, label, source, source_sha):
        area = cls.base / label
        area.mkdir()
        plan = area / "complete-dag.json"
        plan.write_text(json.dumps({
            "schema":"NORDOI_AGENT_DAG_V1",
            "nodes":[
                {"key_code":73,"depends_on":[],"gate":"always"},
                {"key_code":59,"depends_on":[],"gate":"always"},
                {"key_code":80,"depends_on":[1],"gate":"all_committed"},
            ]}, separators=(",",":"),ensure_ascii=True), encoding="utf-8")
        plan_sha = hashlib.sha256(plan.read_bytes()).hexdigest()
        root = area/"real-p27"
        root.mkdir()
        p, rows = command(HERE.parent/"app012_clean_checkpoint/run_app012.py",
            "--plan", plan, "--engine", ENGINE, "--source-root", source,
            "--grant-output-dir", root, "--expected-plan-sha256", plan_sha,
            "--expected-engine-sha256", cls.engine_sha,
            "--expected-source-sha256", source_sha,
            "--stop-after", "2")
        assert p.returncode == 0 and [r["status"] for r in rows] == ["COMMITTED"]*2, (rows,p.stderr)
        events = area/"real-prefix.jsonl"
        events.write_bytes(p.stdout)
        handoff = area/"app013-signed-never"
        p, rows = command(HERE.parent/"app013_verified_handoff/handoff_app013.py",
            "seal","--bundle-dir",handoff,"--native-root",root,
            "--plan",plan,"--prefix-events",events,
            "--expected-plan-sha256",plan_sha)
        inner_pin = expected(p, rows, 0, "SEALED")["manifest_sha256"]
        out = area/"real-app014-portable"
        p, rows = command(HERE.parent/"app014_portable_checkpoint/portable_app014.py",
            "pack","--portable-dir",out,"--handoff-dir",handoff,
            "--native-root",root,"--expected-handoff-sha256",inner_pin)
        pin = expected(p, rows, 0, "PACKED")["portable_sha256"]
        return {"portable":out,"sha":pin,"root":root,
                "plan_sha":plan_sha,"events":events,"source_sha":source_sha}

    def setUp(self):
        self.case = tempfile.TemporaryDirectory(
            prefix="nordoi_cert015_case_",dir=self.base)
        self.work = Path(self.case.name)
        self.portable = self.work/"fresh-received-portable"
        shutil.copytree(self.packages["original"]["portable"], self.portable)
        self.anchor = self.packages["original"]["sha"]
        self.restore_root = self.work/"empty-real-restore-root"
        self.restore_root.mkdir()

    def tearDown(self):
        self.case.cleanup()

    def install_alternate(self):
        shutil.rmtree(self.portable)
        shutil.copytree(self.packages["alternate"]["portable"],self.portable)
        self.anchor=self.packages["alternate"]["sha"]

    def replay(self, action="verify-replay", *,
               source=None, source_sha=None, expected_sha=None,
               package=None, root=None):
        args = ["--portable-dir", self.portable if package is None else package,
                "--expected-portable-sha256",
                self.anchor if expected_sha is None else expected_sha,
                "--engine",ENGINE,"--source-root",
                SOURCE if source is None else source,
                "--expected-engine-sha256",self.engine_sha,
                "--expected-source-sha256",
                self.good_source_sha if source_sha is None else source_sha]
        if action != "verify-replay":
            args += ["--restore-root",
                     self.restore_root if root is None else root]
        return command(HERE.parent/"app015_independent_native_replay/replay_app015.py",
                       action,*args)

    def assert_rejected(self, code, action="verify-replay", **kwargs):
        p,rows=self.replay(action,**kwargs)
        record=expected(p,rows,2,"REJECTED")
        self.assertEqual(record["error_code"],code)
        self.assertEqual(record["publication"],"NOT_STARTED")

    def native_snapshot(self,path):
        return {f.relative_to(path).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()
                for f in path.rglob("*") if f.is_file()}

    def test_01_unmodified_original_passes_exact_pinned_replay(self):
        p, rows=self.replay()
        report=expected(p,rows,0,"PASS")
        self.assertEqual(report["native_jobs_recomputed"],2)
        self.assertEqual(report["native_files_bytewise_equal"],4)
        self.assertTrue(report["native_receipts_exactly_equal"])

    def test_02_other_native_history_passes_app014_consistency(self):
        self.install_alternate()
        p,rows=command(HERE.parent/"app014_portable_checkpoint/portable_app014.py",
                       "verify","--portable-dir",self.portable,
                       "--expected-portable-sha256",self.anchor)
        expected(p,rows,0,"VERIFIED")

    def test_03_coherently_altered_native_history_rejected_by_trusted_replay(self):
        self.install_alternate()
        self.assert_rejected("INDEPENDENT_NATIVE_RECEIPT_MISMATCH")

    def test_04_coherently_altered_history_replays_with_other_trusted_source(self):
        self.install_alternate()
        p,rows=self.replay(source=self.alternate_source,
                           source_sha=self.alternate_source_sha)
        report=expected(p,rows,0,"PASS")
        self.assertEqual(report["native_jobs_recomputed"],2)
        # This exposes the actual boundary: pins need an external authority.
        self.assertEqual(report["external_pin_authentication"],"NOT_PERFORMED")

    def test_05_original_history_rejected_under_alternate_but_pinned_rule(self):
        self.assert_rejected("INDEPENDENT_NATIVE_RECEIPT_MISMATCH",
            source=self.alternate_source,source_sha=self.alternate_source_sha)

    def test_06_mismatched_source_sha_never_authorizes_replay(self):
        self.assert_rejected("SOURCE_PIN_MISMATCH",source_sha="0"*64)

    def test_07_wrong_package_sha_blocks_before_native_replay(self):
        self.assert_rejected("PORTABLE_EXTERNAL_PIN_MISMATCH",
                             expected_sha="0"*64)

    def test_08_portable_manifest_byte_change_blocks_before_replay(self):
        m=self.portable/"portable.json"
        m.write_bytes(m.read_bytes()+b"undisclosed-change")
        self.assert_rejected("PORTABLE_EXTERNAL_PIN_MISMATCH")

    def test_09_native_file_mutation_blocks_before_replay(self):
        f=self.portable/"native/job-0001/quality-decision/report.txt"
        f.write_bytes(f.read_bytes()+b"adversarial")
        self.assert_rejected("REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")

    def test_10_extra_job_directory_blocks_before_replay(self):
        (self.portable/"native/job-0003").mkdir()
        self.assert_rejected("ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_11_truncated_package_marker_blocks_before_replay(self):
        (self.portable/"PORTABLE.sha256").write_bytes(b"0\n")
        self.assert_rejected("PORTABLE_MARKER_MISMATCH")

    def test_12_alternate_consistent_history_blocked_before_restore(self):
        self.install_alternate()
        self.assert_rejected("INDEPENDENT_NATIVE_RECEIPT_MISMATCH",
                             action="restore-replay")
        self.assertEqual(list(self.restore_root.iterdir()),[])

    def test_13_original_history_restored_only_after_positive_replay(self):
        p,rows=self.replay("restore-replay")
        report=expected(p,rows,0,"RESTORED")
        self.assertEqual(report["replay_qualification"],
                         "PASS_EXACT_NATIVE_RUST_REPLAY")
        self.assertEqual({x.name for x in self.restore_root.iterdir()},
                         {"job-0001","job-0002"})

    def test_14_rehydrated_native_files_match_packaged_source_bytewise(self):
        p,rows=self.replay("restore-replay")
        expected(p,rows,0,"RESTORED")
        self.assertEqual(self.native_snapshot(self.restore_root),
                         self.native_snapshot(self.portable/"native"))

    def test_15_replay_qualifies_then_resumes_only_missing_rust_job(self):
        p,rows=self.replay("restore-replay")
        expected(p,rows,0,"RESTORED")
        before=self.native_snapshot(self.restore_root)
        p,rows=self.replay("resume-replay")
        self.assertEqual(p.returncode,0,(rows,p.stderr))
        self.assertEqual([x["step"] for x in rows],[3])
        self.assertEqual(rows[0]["status"],"COMMITTED")
        self.assertEqual({name:hashlib.sha256((self.restore_root/name).read_bytes()).hexdigest()
                          for name in before},before)
        complete=self.work/"full-real-dag.jsonl"
        complete.write_bytes((self.portable/"handoff/native-prefix.jsonl").read_bytes()+p.stdout)
        report,code=verifier.independent.audit(
            self.restore_root,self.portable/"handoff/plan.json",complete,
            self.packages["original"]["plan_sha"])
        self.assertEqual(code,0,report)
        self.assertEqual(report["status"],"PASS")

    def test_16_repeated_restore_is_denied_without_any_overwrite(self):
        p,rows=self.replay("restore-replay")
        expected(p,rows,0,"RESTORED")
        before=self.native_snapshot(self.restore_root)
        self.assert_rejected("RESTORE_ROOT_NOT_EMPTY","restore-replay")
        self.assertEqual(self.native_snapshot(self.restore_root),before)

    def test_17_duplicate_resume_after_native_success_is_denied(self):
        expected(*self.replay("restore-replay"),0,"RESTORED")
        p,rows=self.replay("resume-replay")
        self.assertEqual(p.returncode,0,(rows,p.stderr))
        self.assert_rejected("ROOT_UNEXPECTED_JOB_OR_FILE","resume-replay")

    def test_18_original_published_files_never_changed_by_replay(self):
        initial=self.native_snapshot(self.packages["original"]["root"])
        self.replay("verify-replay")
        self.assertEqual(initial,self.native_snapshot(self.packages["original"]["root"]))

    def test_19_received_package_unchanged_by_verification(self):
        original=self.native_snapshot(self.portable)
        expected(*self.replay("verify-replay"),0,"PASS")
        self.assertEqual(original,self.native_snapshot(self.portable))

    def test_20_verdict_explicitly_disclaims_authentication_and_sandbox(self):
        report=expected(*self.replay("verify-replay"),0,"PASS")
        self.assertEqual(report["external_pin_authentication"],"NOT_PERFORMED")
        self.assertEqual(report["native_engine_sandbox"],"NOT_PROVIDED")
        self.assertEqual(report["production_admission"],"NOT_AUTHORIZED")
        self.assertEqual(report["p27_assurance"],
            "EXACT_PINNED_NATIVE_REPRODUCTION_NOT_SEPARATE_CRYPTO")


if __name__ == "__main__":
    unittest.main()
