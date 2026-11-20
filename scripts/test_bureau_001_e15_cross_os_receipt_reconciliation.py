"""R6F/E15: 24 tests using REAL downloaded predecessor E14 CI artifacts.

Fixtures used for tamper tests are COPIES of real artifacts, not purported
fresh historical test executions or independent outside attestations.
"""
import copy
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e15_cross_os_receipt_reconciliation as e15

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()

class E15RealArtifactTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.sha=head()
        cls.c,cls.groups,cls.pins=e15.audit(ROOT,cls.sha)
        value=os.environ.get("E15_ARTIFACT_DIR")
        if not value:
            raise RuntimeError("E15_REQUIRES_REAL_GITHUB_ARTIFACT_DIR_NOT_FAKE_DATA")
        cls.input=Path(value).resolve(strict=True)
        if not cls.input.is_dir():
            raise RuntimeError("E15_REAL_GITHUB_ARTIFACTS_NOT_DOWNLOADED")
        cls.targets=dict(e15.e14.commands(cls.groups))
    def duplicate(self,folder):
        out=Path(folder)/"real_artifact_copies"
        shutil.copytree(self.input,out)
        return out
    def test_01_exact_green_e14_parent(self):
        self.assertEqual(e15.PARENT,"deb34302acdbe18563ab06eb10d852865e3b3c6a")
    def test_02_exact_prior_C20_rust_and_four_os_runs(self):
        self.assertEqual(self.c["prior_e14"],{
          "specialized_run_id":44713831052,"specialized_success":4,
          "rust_run_id":44713831212,"rust_success":5,
          "e8_run_id":44713830975,"e8_success":4})
    def test_03_exact_artifact_ids_and_manifest_sha(self):
        self.assertEqual(len(self.c["artifacts"]),4)
        self.assertEqual(
          [(x["id"],x["github_artifact_id"],x["sha256_manifest"],
            x["recorded_os"],x["recorded_arch"],
            x["expected_passed_cases"],x["expected_skipped_cases"])
             for x in self.c["artifacts"]],e15.EXPECTED)
    def test_04_source_git_tree_really_verified_99_pins(self):
        self.assertEqual(len(self.pins),99)
        self.assertEqual(self.c["parent_e14_exact_head"],e15.PARENT)
    def test_05_target_paths_are_complete_57(self):
        self.assertEqual(len(self.targets),57)
        self.assertEqual(len(set(self.targets)),57)
    def test_06_target_kinds_are_24_rust_33_python(self):
        self.assertEqual(len(self.groups["rust"]),24)
        self.assertEqual(len(self.groups["python_test"]),33)
    def test_07_actual_228_raw_logs_required(self):
        self.assertEqual(self.c["expected_total_raw_logs"],228)
        self.assertEqual(self.c["expected_target_files_per_platform"],57)
    def test_08_actual_pass_skip_counts_across_four_platforms(self):
        self.assertEqual(sum(x[5] for x in e15.EXPECTED),3629)
        self.assertEqual(sum(x[6] for x in e15.EXPECTED),251)
    def test_09_frozen_native_gates_not_satisfied(self):
        self.assertEqual(self.c["native_admission_gates_unmet"],12)
        self.assertEqual(self.c["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(self.c["production"],"NOT_AUTHORIZED")
    def test_10_no_humans_or_fake_outside_witnesses(self):
        self.assertFalse(self.c["human_testers_allowed"])
        self.assertFalse(self.c["founder_manual_test_duty"])
        self.assertEqual(self.c["independent_specialist_reviews"],0)
        self.assertEqual(self.c["independent_external_witness_receipts"],0)
    def test_11_real_four_downloaded_artifacts_are_present(self):
        self.assertEqual({p.name for p in self.input.iterdir()},
            {x["id"] for x in self.c["artifacts"]})
        self.assertTrue(all((self.input/x["id"]/"MANIFEST.json").is_file()
            for x in self.c["artifacts"]))
    def test_12_real_four_pinned_manifest_hashes_match_bytes(self):
        for x in self.c["artifacts"]:
            self.assertEqual(e15.e14.digest(
                (self.input/x["id"]/"MANIFEST.json").read_bytes()),
                x["sha256_manifest"])
    def test_13_real_full_228_log_reconciliation(self):
        d=e15.reconcile(ROOT,self.sha,self.input)
        self.assertEqual(d["genuine_prior_raw_logs_verified"],228)
        self.assertEqual(d["real_prior_ci_artifacts_rechecked"],4)
        self.assertEqual(d["native_admission_gates_unmet"],12)
    def test_14_all_four_platforms_have_57_complete_receipts(self):
        d=e15.reconcile(ROOT,self.sha,self.input)
        self.assertEqual([x["verified_log_files"] for x in d["platforms"]],
                         [57,57,57,57])
        self.assertEqual([x["rust_targets"] for x in d["platforms"]],[24]*4)
        self.assertEqual([x["python_targets"] for x in d["platforms"]],[33]*4)
    def test_15_real_os_and_arch_reports_distinguished(self):
        d=e15.reconcile(ROOT,self.sha,self.input)
        self.assertEqual(
           [(x["host_os_recorded_by_E14"],x["host_arch_recorded_by_E14"])
            for x in d["platforms"]],
           [(x[3],x[4]) for x in e15.EXPECTED])
    def test_16_real_skips_not_counted_as_passed(self):
        d=e15.reconcile(ROOT,self.sha,self.input)
        self.assertEqual([x["cases_skipped"] for x in d["platforms"]],
                         [3,3,19,226])
        self.assertEqual([x["cases_passed"] for x in d["platforms"]],
                         [1039,1039,881,670])
    def test_17_six_native_r4m_cases_proven_only_for_macos(self):
        for name in ("macos-15-intel","macos-15"):
            m=e15.e3.strictly_load(
                (self.input/name/"MANIFEST.json").read_bytes())
            row=next(x for x in m["records"]
                if x["target_path"]==
                "tests/g03_r4m_pinned_code_identity_lab.rs")
            self.assertGreaterEqual(row["passed_tests"],6)
        win=e15.e3.strictly_load(
            (self.input/"windows-latest"/"MANIFEST.json").read_bytes())
        target=next(x for x in win["records"]
            if x["target_path"]==
            "tests/g03_r4m_pinned_code_identity_lab.rs")
        self.assertLess(target["passed_tests"],6)
    def test_18_wrong_git_head_rejected(self):
        with self.assertRaises(Exception):
            e15.audit(ROOT,"f"*40)
    def test_19_swapped_real_platform_bundles_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e15-swap-") as p:
            folder=self.duplicate(p)
            a,b=folder/"macos-15-intel",folder/"ubuntu-latest"
            a.rename(folder/"swap_temp")
            b.rename(a)
            (folder/"swap_temp").rename(b)
            with self.assertRaisesRegex(e15.Blocked,"CHANGED_OR_CROSS_SWAPPED"):
                e15.reconcile(ROOT,self.sha,folder)
    def test_20_tampered_raw_log_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e15-tamper-log-") as p:
            folder=self.duplicate(p)
            m=e15.e3.strictly_load((folder/"macos-15"/"MANIFEST.json").read_bytes())
            fp=folder/"macos-15"/m["records"][0]["log_filename"]
            with fp.open("ab") as f:f.write(b"altered-by-adversarial-test")
            with self.assertRaisesRegex(e15.e14.Blocked,"DIGEST_CHANGED"):
                e15.reconcile(ROOT,self.sha,folder)
    def test_21_rehashed_local_manifest_without_pinned_root_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e15-rehash-") as p:
            folder=self.duplicate(p)
            fp=folder/"macos-15"/"MANIFEST.json"
            m=e15.e3.strictly_load(fp.read_bytes())
            m["test_cases_skipped"]+=1
            fp.write_bytes(e15.e14.canonical(m))
            with self.assertRaisesRegex(e15.Blocked,"CHANGED_OR_CROSS_SWAPPED"):
                e15.reconcile(ROOT,self.sha,folder)
    def test_22_missing_one_historical_raw_log_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e15-missing-") as p:
            folder=self.duplicate(p)
            m=e15.e3.strictly_load(
                (folder/"windows-latest"/"MANIFEST.json").read_bytes())
            (folder/"windows-latest"/m["records"][0]["log_filename"]).unlink()
            with self.assertRaises(e15.e14.Blocked):
                e15.reconcile(ROOT,self.sha,folder)
    def test_23_injected_fifth_artifact_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e15-fifth-") as p:
            folder=self.duplicate(p)
            (folder/"untrusted-fifth-os").mkdir()
            with self.assertRaisesRegex(e15.Blocked,"NOT_EXACTLY_FOUR"):
                e15.reconcile(ROOT,self.sha,folder)
    def test_24_genuine_dossier_does_not_create_external_custodian(self):
        d=e15.reconcile(ROOT,self.sha,self.input)
        self.assertEqual(d["external_witness_receipts"],0)
        self.assertEqual(d["formal_external_reviews"],0)
        self.assertEqual(d["trust_boundary"],
            "GITHUB_ARTIFACT_RECHECK_NOT_INDEPENDENT_EXTERNAL_CUSTODY")
        self.assertFalse(any(x["formal_external_witness"] for x in d["platforms"]))

if __name__=="__main__":unittest.main()
