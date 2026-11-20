"""24 actual E16 automated controls; genuine E14+E15 archives mandatory.

All negative dossier mutations target copies of real nine-process reports.
The nine processes check actual E14 logs, not fixtures or fake signers.
"""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e16_nine_machine_review_dossier as e16

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],
                                   cwd=ROOT,text=True).strip()

class E16MachineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.head=head()
        cls.art=os.environ.get("E16_E14_ARTIFACT_DIR")
        cls.anchor=os.environ.get("E16_E15_ANCHOR_FILE")
        if not cls.art or not cls.anchor:
            raise RuntimeError("E16_REQUIRES_REAL_DOWNLOADED_E14_AND_E15_ARTIFACTS")
        cls.art=Path(cls.art).resolve(strict=True)
        cls.anchor=Path(cls.anchor).resolve(strict=True)
        cls.c,cls.handoff,cls.groups,cls.pins=e16.audit(ROOT,cls.head)
        cls.temp=tempfile.TemporaryDirectory(prefix="nordoi-e16-actual-nine-")
        cls.folder=Path(cls.temp.name)/"dossier"
        cls.actual=e16.exercise(ROOT,cls.head,cls.art,cls.anchor,cls.folder)
        cls.manifest=e16.verify_dossier(ROOT,cls.head,cls.art,cls.anchor,cls.folder)
    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()
    def newcopy(self,td):
        x=Path(td)/"copy"
        shutil.copytree(self.folder,x)
        return x
    def verify(self,folder):
        return e16.verify_dossier(ROOT,self.head,self.art,self.anchor,folder)
    def test_01_parent_exact_E15_9_job_green(self):
        self.assertEqual(e16.PARENT,"8af265158bfbeb9c1406d2c310d505281dc98180")
    def test_02_actual_C20_run_ids(self):
        self.assertEqual(self.c["prior_e15"],{
            "specialized_run_id":44716501372,"specialized_success":4,
            "rust_run_id":44716501419,"rust_success":5,
            "e8_run_id":44716501426,"e8_success":4})
    def test_03_nine_exact_handoff_function_names(self):
        self.assertEqual(e16.ROLES,(
            "ATLAS","FERRUM","BOUNDARY","CIPHER","LEDGER",
            "CAPSTONE","ADVERSARY","VERITY","GUARDIAN"))
        self.assertEqual(tuple(x["role"] for x in self.handoff["review_functions"]),e16.ROLES)
    def test_04_no_real_formal_independent_review_claimed(self):
        self.assertTrue(all(x["review_status"]=="NOT_SUBMITTED"
             and x["formal_verdict"] is None for x in self.handoff["review_functions"]))
        self.assertEqual(self.manifest["formal_independent_specialist_reviews"],0)
    def test_05_actual_E15_anchor_bytes_SHA_pinned(self):
        self.assertEqual(e16.sha(self.anchor.read_bytes()),
            self.c["e15_anchor"]["sha256"])
    def test_06_actual_four_E14_platforms_rechecked(self):
        a=e16.actual_anchor(ROOT,self.head,self.anchor,self.art)[-1]
        self.assertEqual(len(a["platforms"]),4)
        self.assertEqual(a["genuine_prior_raw_logs_verified"],228)
    def test_07_99_frozen_git_sources_not_changed(self):
        self.assertEqual(len(self.pins),99)
        self.assertEqual(len(self.groups["rust"]),24)
        self.assertEqual(len(self.groups["python_test"]),33)
    def test_08_actual_nine_OS_processes_distinct_PID(self):
        self.assertEqual(self.actual["real_bureau_machine_child_processes"],9)
        self.assertEqual(len({x["pid"] for x in self.manifest["reports"]}),9)
    def test_09_nine_report_files_and_distinct_hashes(self):
        self.assertEqual(len(self.manifest["reports"]),9)
        self.assertEqual(len({x["sha256"] for x in self.manifest["reports"]}),9)
    def test_10_machine_reports_are_canonical_JSON(self):
        for x in self.manifest["reports"]:
            raw=(self.folder/x["filename"]).read_bytes()
            self.assertEqual(raw,e16.canonical(e16.e3.strictly_load(raw)))
    def test_11_real_role_sources_have_true_git_sha(self):
        for r in self.manifest["reports"]:
            obj=e16.e3.strictly_load((self.folder/r["filename"]).read_bytes())
            self.assertGreaterEqual(len(obj["source_evidence"]),2)
            for file in obj["source_evidence"]:
                self.assertEqual(file["source_git_blob_sha1"],
                    self.pins[file["path"]]["git_blob_sha1"])
                self.assertEqual(file["source_sha256"],
                    e16.sha((ROOT/file["path"]).read_bytes()))
    def test_12_each_role_traces_all_four_historical_OS_logs(self):
        for r in self.manifest["reports"]:
            o=e16.e3.strictly_load((self.folder/r["filename"]).read_bytes())
            for f in o["source_evidence"]:
                self.assertEqual(len(f["true_E14_test_records_by_platform"]),4)
    def test_13_no_role_is_an_external_AI_specialist_signer(self):
        for r in self.manifest["reports"]:
            o=e16.e3.strictly_load((self.folder/r["filename"]).read_bytes())
            self.assertEqual(o["formal_review_status"],"NOT_SUBMITTED")
            self.assertIsNone(o["formal_independent_specialist_verdict"])
            self.assertFalse(o["external_administrative_independence_proven"])
    def test_14_r4m_true_mac_six_not_windows_native_claim(self):
        o=e16.e3.strictly_load((self.folder/"FERRUM.json").read_bytes())
        r=next(x for x in o["source_evidence"] if "r4m_pinned" in x["path"])
        p={x["platform"]:x for x in r["true_E14_test_records_by_platform"]}
        self.assertGreaterEqual(p["macos-15-intel"]["passed"],6)
        self.assertGreaterEqual(p["macos-15"]["passed"],6)
        self.assertLess(p["windows-latest"]["passed"],6)
    def test_15_cross_OS_skips_not_converted_to_PASS(self):
        self.assertEqual(self.manifest["contradictions"][0]["id"],
                         "CROSS_OS_SKIPS_NOT_NATIVE_EQUIVALENCE")
        self.assertEqual(self.manifest["contradictions"][0]["status"],
                         "OPEN_PORTABILITY_BOUNDARY")
    def test_16_seven_contradictions_with_explicit_limit_status(self):
        self.assertEqual(len(self.manifest["contradictions"]),7)
        self.assertEqual(tuple(x["id"] for x in self.manifest["contradictions"]),e16.ISSUES)
        self.assertTrue(all(x["status"].startswith(("OPEN","RESOLVED_TECHNICAL"))
             for x in self.manifest["contradictions"]))
    def test_17_twelve_native_conditions_still_UNMET(self):
        self.assertEqual(self.actual["native_admission_gates_unmet"],12)
        self.assertEqual(self.actual["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(self.actual["production"],"NOT_AUTHORIZED")
    def test_18_zero_human_testers_and_no_founder_manual_duty(self):
        self.assertEqual(self.manifest["human_testers"],0)
        self.assertFalse(self.c["human_testers_allowed"])
        self.assertFalse(self.c["founder_manual_test_duty"])
    def test_19_wrong_exact_head_rejected(self):
        with self.assertRaises(Exception):
            e16.audit(ROOT,"f"*40)
    def test_20_real_report_byte_tampering_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e16-tamper-") as td:
            out=self.newcopy(td)
            with (out/"LEDGER.json").open("ab") as f:f.write(b"tamper")
            with self.assertRaisesRegex(e16.Blocked,"REPORT_HASH_TAMPERED"):
                self.verify(out)
    def test_21_coherently_rehashed_fake_authority_still_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e16-rehash-") as td:
            out=self.newcopy(td)
            p=out/"CIPHER.json"
            r=e16.e3.strictly_load(p.read_bytes())
            r["formal_review_status"]="SIGNED_BY_INDEPENDENT_SPECIALIST"
            raw=e16.canonical(r)
            p.write_bytes(raw)
            mp=out/"MANIFEST.json"
            m=e16.e3.strictly_load(mp.read_bytes())
            next(x for x in m["reports"] if x["role"]=="CIPHER")["sha256"]=e16.sha(raw)
            mp.write_bytes(e16.canonical(m))
            with self.assertRaisesRegex(e16.Blocked,"SAME_ADMIN_ROLE_REWRITE"):
                self.verify(out)
    def test_22_missing_legacy_role_file_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e16-missing-") as td:
            out=self.newcopy(td)
            (out/"GUARDIAN.json").unlink()
            with self.assertRaisesRegex(e16.Blocked,"MISSING_EXTRA"):
                self.verify(out)
    def test_23_contradiction_matrix_change_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e16-matrix-") as td:
            out=self.newcopy(td)
            p=out/"MANIFEST.json"
            m=e16.e3.strictly_load(p.read_bytes())
            m["contradictions"][0]["status"]="RESOLVED_GLOBALLY"
            p.write_bytes(e16.canonical(m))
            with self.assertRaisesRegex(e16.Blocked,"MATRIX_REWRITTEN"):
                self.verify(out)
    def test_24_real_nine_reports_not_formal_certificate(self):
        d=self.verify(self.folder)
        self.assertEqual(len(d["reports"]),9)
        self.assertEqual(d["external_witness_receipts"],0)
        self.assertEqual(d["formal_independent_specialist_reviews"],0)
        self.assertEqual(d["native_admission_gates_unmet"],12)

if __name__=="__main__":unittest.main()
