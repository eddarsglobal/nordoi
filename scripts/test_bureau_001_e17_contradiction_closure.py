"""E17: 24 machine-only tests using four genuinely archived E16 CI bundles.

Copies used for tampering are marked negative test material. Actual E16 reports
are verified by pinned source SHA256; no outside custodians are invented.
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
import bureau_001_e17_contradiction_closure as e17

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],
         cwd=ROOT,text=True).strip()

class E17RealArchiveTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.h=head()
        val=os.environ.get("E17_E16_ARTIFACT_DIR")
        if not val:raise RuntimeError("E17_REQUIRES_REAL_GITHUB_E16_BUNDLES")
        cls.folder=Path(val).resolve(strict=True)
        cls.c,cls.e16c,cls.pins=e17.audit(ROOT,cls.h)
        cls.actual=e17.verify_bundle(ROOT,cls.h,cls.folder)
    def copy(self,parent):
        destination=Path(parent)/"copied_original_E16_archives"
        shutil.copytree(self.folder,destination)
        return destination
    def test_01_exact_parent_source_E16(self):
        self.assertEqual(e17.PARENT,"dc6f9d54872a1aad899800280bb74e64eb3fe9fd")
    def test_02_exact_9_job_E16_C20_green(self):
        self.assertEqual(self.c["prior_e16"],{
          "specialized_run_id":44720657493,"specialized_success":4,
          "rust_run_id":44720657509,"rust_success":5,
          "e8_run_id":44720657394,"e8_success":4})
    def test_03_all_four_REAL_artifact_ids_pinned(self):
        self.assertEqual(len(e17.ARTS),4)
        self.assertEqual(tuple((a["id"],a["github_artifact_id"],
            a["manifest_sha256"]) for a in self.c["e16_artifacts"]),e17.ARTS)
    def test_04_original_E16_artifacts_have_exact_filenames(self):
        self.assertEqual({x.name for x in self.folder.iterdir()},
                         {x[0] for x in e17.ARTS})
    def test_05_four_genuine_manifest_sha_matches(self):
        for p,_,dig in e17.ARTS:
            self.assertEqual(e17.sha((self.folder/p/"MANIFEST.json").read_bytes()),dig)
    def test_06_E17_audit_does_not_claim_to_run_historical_tests(self):
        self.assertEqual(self.c["expected_machine_role_reports"],36)
        self.assertEqual(self.c["expected_contradiction_records"],28)
    def test_07_all_36_original_E16_machine_reports_rechecked(self):
        self.assertEqual(self.actual["real_archived_e16_machine_reports_revalidated"],36)
    def test_08_all_28_real_archived_contradiction_rows_rechecked(self):
        self.assertEqual(self.actual["real_archived_e16_contradictions_revalidated"],28)
    def test_09_every_platform_has_nine_reports(self):
        self.assertEqual([p["machine_reports_revalidated"] for p in
            self.actual["four_ci_artifact_platforms"]],[9,9,9,9])
    def test_10_six_distinct_open_issues_not_closed(self):
        self.assertEqual(self.actual["open_blockers"],6)
        self.assertEqual(set(self.actual["distinct_open_contradictions"]),set(e17.OPEN))
    def test_11_one_limited_technical_provenance_issue_resolved(self):
        self.assertEqual(self.actual["technical_provenance_limit_resolved"],1)
        self.assertEqual(self.actual["distinct_technical_provenance_closures"],
            ["HISTORIC_GIT_PINS_NOT_RUNTIME_PROOF"])
    def test_12_all_nine_role_names_exact(self):
        for osname,_,_ in e17.ARTS:
            m=e17.e3.strictly_load((self.folder/osname/"MANIFEST.json").read_bytes())
            self.assertEqual(tuple(x["role"] for x in m["reports"]),e17.e16.ROLES)
    def test_13_role_report_sha256_and_actual_source_bytes(self):
        osname="macos-15-intel"
        m=e17.e3.strictly_load((self.folder/osname/"MANIFEST.json").read_bytes())
        for row in m["reports"]:
            r=e17.e3.strictly_load((self.folder/osname/row["filename"]).read_bytes())
            for source in r["source_evidence"]:
                self.assertEqual(source["source_git_blob_sha1"],
                    self.pins[source["path"]]["git_blob_sha1"])
                self.assertEqual(source["source_sha256"],
                    e17.sha(subprocess.check_output(["git","show",
                        self.h+":"+source["path"]],cwd=ROOT)))
    def test_14_real_reports_confirm_formal_review_NOT_SUBMITTED(self):
        for osname,_,_ in e17.ARTS:
            r=e17.e3.strictly_load((self.folder/osname/"CIPHER.json").read_bytes())
            self.assertEqual(r["formal_review_status"],"NOT_SUBMITTED")
            self.assertIsNone(r["formal_independent_specialist_verdict"])
            self.assertFalse(r["external_administrative_independence_proven"])
    def test_15_source_frozen_pins_99_not_57_new_executions(self):
        self.assertEqual(len(self.pins),99)
        self.assertTrue(self.actual["e17_does_not_reexecute_e14_or_e16"])
    def test_16_owner_not_forced_to_test_and_no_human_testers(self):
        self.assertFalse(self.c["human_testers_allowed"])
        self.assertFalse(self.c["founder_manual_test_duty"])
        self.assertEqual(self.actual["human_testers"],0)
    def test_17_external_authorities_still_absent(self):
        self.assertEqual(self.actual["formal_independent_reviews"],0)
        self.assertEqual(self.actual["outside_signed_receipts"],0)
        self.assertEqual(self.actual["owner_oob_root"],"NOT_AUTHENTICATED")
        self.assertEqual(self.actual["external_custodian"],"NOT_ENROLLED")
    def test_18_all_12_native_admission_gates_unmet(self):
        self.assertEqual(self.actual["native_admission_gates_unmet"],12)
        self.assertEqual(self.actual["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(self.actual["production"],"NOT_AUTHORIZED")
        self.assertEqual(self.actual["r6f"],"IN_PROGRESS_UNCERTIFIED")
    def test_19_naive_machine_report_count_would_falsely_certify(self):
        self.assertTrue(e17.unsafe_naive_certification(36,9))
        self.assertEqual(self.actual["formal_independent_reviews"],0)
    def test_20_missing_one_real_role_report_is_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e17-missing-") as t:
            f=self.copy(t)
            (f/"macos-15-intel"/"GUARDIAN.json").unlink()
            with self.assertRaisesRegex(e17.Blocked,"MISSING_OR_INJECTED"):
                e17.verify_bundle(ROOT,self.h,f)
    def test_21_cross_swapped_mac_and_linux_is_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e17-swap-") as t:
            f=self.copy(t)
            a,b=f/"macos-15-intel",f/"ubuntu-latest"
            a.rename(f/"swap_temp");b.rename(a);(f/"swap_temp").rename(b)
            with self.assertRaisesRegex(e17.Blocked,"MANIFEST_SHA_WRONG"):
                e17.verify_bundle(ROOT,self.h,f)
    def test_22_one_actual_report_byte_tamper_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e17-tamper-") as t:
            f=self.copy(t)
            with (f/"windows-latest"/"LEDGER.json").open("ab") as h:
                h.write(b"adversarial alteration")
            with self.assertRaisesRegex(e17.Blocked,"CHANGED_ROLE_SHA"):
                e17.verify_bundle(ROOT,self.h,f)
    def test_23_forge_full_formal_certification_and_rehash_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e17-fake-") as t:
            f=self.copy(t)
            folder=f/"macos-15"
            path=folder/"CIPHER.json"
            obj=e17.e3.strictly_load(path.read_bytes())
            obj["formal_review_status"]="CERTIFIED"
            path.write_bytes(e17.canonical(obj))
            mp=folder/"MANIFEST.json"
            m=e17.e3.strictly_load(mp.read_bytes())
            next(x for x in m["reports"] if x["role"]=="CIPHER")["sha256"] = e17.sha(path.read_bytes())
            mp.write_bytes(e17.canonical(m))
            with self.assertRaisesRegex(e17.Blocked,"MANIFEST_SHA_WRONG"):
                e17.verify_bundle(ROOT,self.h,f)
    def test_24_wrong_exact_source_commit_rejected(self):
        with self.assertRaises(Exception):
            e17.audit(ROOT,"f"*40)

if __name__=="__main__":unittest.main()
