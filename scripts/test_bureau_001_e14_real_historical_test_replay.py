"""E14 machine-only contract and receipt-parser controls (24 tests).

Parser examples are unit fixtures, never called real historical executions.
The E14 CI exercise separately launches 57 actual historical test subprocesses.
"""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e14_real_historical_test_replay as e14

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,
                                   text=True).strip()

class E14Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.head=head()
        cls.groups,cls.pins=e14.audit(ROOT,cls.head)
        cls.tasks=e14.commands(cls.groups)
    def test_01_parent_exact_green_E13(self):
        self.assertEqual(e14.PARENT,"4da31db4363931606530f130b4d5330510a88184")
    def test_02_real_source_audit(self):
        self.assertEqual(e14.audit(ROOT,self.head)[0],self.groups)
    def test_03_source_is_exact_99_pin_git_tree(self):
        self.assertEqual(len(self.pins),99)
        self.assertEqual(set(self.pins),set(sum(
            (self.groups[k] for k in
             ("rust","python_source","python_test","snapshots","basis")),[])))
    def test_04_exact_57_distinct_executable_targets(self):
        self.assertEqual(len(self.tasks),57)
        self.assertEqual(len({x[0] for x in self.tasks}),57)
    def test_05_exact_24_rust_and_33_python(self):
        self.assertEqual(len(self.groups["rust"]),24)
        self.assertEqual(len(self.groups["python_test"]),33)
    def test_06_python_command_is_real_unittest_discovery(self):
        p,c=next((p,c) for p,c in self.tasks if p.endswith(".py"))
        self.assertEqual(c,[sys.executable,"-B","-m","unittest","discover",
                            "-s","research","-p",Path(p).name,"-v"])
    def test_07_rust_command_is_real_offline_native_test(self):
        p,c=next((p,c) for p,c in self.tasks if p.endswith(".rs"))
        self.assertEqual(c,["cargo","test","--offline","--release",
                            "--test",Path(p).stem,"--",
                            "--nocapture","--test-threads=1"])
    def test_08_native_mac_R4M_historical_target_is_in_set(self):
        self.assertIn("tests/g03_r4m_pinned_code_identity_lab.rs",
                      self.groups["rust"])
    def test_09_R6E_historical_true_POSIX_test_in_set(self):
        self.assertIn("research/test_g03_r6e_oob_origin_intake.py",
                      self.groups["python_test"])
    def test_10_real_python_unit_summary_parser(self):
        d=e14.parse_count("research/test_x.py",
             b"Ran 24 tests in 1.123s\n\nOK\n",0)
        self.assertEqual((d["ran"],d["passed"],d["skipped"]),(24,24,0))
        self.assertTrue(d["success"])
    def test_11_skipped_python_are_not_passed(self):
        d=e14.parse_count("research/test_x.py",
             b"Ran 24 tests in 1.123s\n\nOK (skipped=8)\n",0)
        self.assertEqual((d["ran"],d["passed"],d["skipped"]),(24,16,8))
    def test_12_real_rust_summary_parser(self):
        d=e14.parse_count("tests/x.rs",
            b"test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n",0)
        self.assertEqual((d["passed"],d["ran"],d["skipped"]),(6,6,0))
        self.assertTrue(d["success"])
    def test_13_skipped_rust_are_not_passed(self):
        d=e14.parse_count("tests/x.rs",
            b"test result: ok. 2 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out\n",0)
        self.assertEqual((d["passed"],d["ran"],d["skipped"]),(2,8,6))
    def test_14_zero_rust_tests_not_pass(self):
        d=e14.parse_count("tests/x.rs",
            b"test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n",0)
        self.assertFalse(d["success"])
    def test_15_nonzero_return_code_even_with_OK_is_failure(self):
        a=e14.parse_count("research/test_x.py",
            b"Ran 1 test in 0.1s\n\nOK\n",1)
        b=e14.parse_count("tests/x.rs",
            b"test result: ok. 1 passed; 0 failed; 0 ignored\n",1)
        self.assertFalse(a["success"])
        self.assertFalse(b["success"])
    def test_16_missing_unittest_count_fails_closed(self):
        with self.assertRaisesRegex(e14.Blocked,"COUNT_ABSENT"):
            e14.parse_count("research/test_x.py",b"OK\n",0)
    def test_17_ambiguous_unittest_count_fails_closed(self):
        with self.assertRaisesRegex(e14.Blocked,"COUNT_ABSENT"):
            e14.parse_count("research/test_x.py",
                b"Ran 1 test in 0.1s\nRan 2 tests in 0.1s\nOK\n",0)
    def test_18_missing_rust_test_result_fails_closed(self):
        with self.assertRaisesRegex(e14.Blocked,"COUNT_ABSENT"):
            e14.parse_count("tests/x.rs",b"compile only\n",0)
    def test_19_wrong_checkout_head_denied(self):
        with self.assertRaises(Exception):
            e14.audit(ROOT,"f"*40)
    def test_20_no_human_testers_and_no_independent_quorum(self):
        c=e14.e3.strictly_load(e14.e13.r1.raw(ROOT,e14.CONTRACT))
        self.assertFalse(c["human_testers_allowed"])
        self.assertFalse(c["founder_manual_test_duty"])
        self.assertEqual(c["independent_bureau_reviews_completed"],0)
        self.assertEqual(c["native_admission_gates_unmet"],12)
    def test_21_untrusted_native_promotion_cannot_override_denial(self):
        d=e14.denied("CHECK",result="ALLOW",r6g="OPEN",
                      human_testers=7,native_admission_gates_unmet=0)
        self.assertEqual(d["result"],"BLOCK")
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(d["human_testers"],0)
    def test_22_real_python_one_historical_target_with_raw_log(self):
        target="research/test_g03_r4y_external_custody_intake.py"
        cmd=next(c for p,c in self.tasks if p==target)
        with tempfile.TemporaryDirectory(prefix="nordoi-e14-unittest-") as folder:
            out=Path(folder)
            row=e14.run_target(ROOT,target,cmd,out,self.pins[target])
            self.assertTrue(row["success"],row)
            self.assertGreater(row["passed_tests"],0)
            self.assertEqual(row["return_code"],0)
            self.assertEqual(row["raw_output_sha256"],
                             e14.digest((out/row["log_filename"]).read_bytes()))
    def test_23_one_real_receipt_rejects_manifest_log_corruption(self):
        target="research/test_g03_r4y_external_custody_intake.py"
        cmd=next(c for p,c in self.tasks if p==target)
        with tempfile.TemporaryDirectory(prefix="nordoi-e14-evidence-") as folder:
            out=Path(folder)
            row=e14.run_target(ROOT,target,cmd,out,self.pins[target])
            m={"schema":e14.EVIDENCE_SCHEMA,"exact_source_head":self.head,
               "expected_target_count":1,"human_testers":0,
               "native_admission_gates_unmet":12,"independent_bureau_reviews":0,
               "r6g":"LOCKED_BY_SEQUENCE","production":"NOT_AUTHORIZED",
               "records":[row]}
            (out/"MANIFEST.json").write_bytes(e14.canonical(m))
            self.assertEqual(len(e14.validate_manifest(out,self.head,1)["records"]),1)
            with (out/row["log_filename"]).open("ab") as f:f.write(b"tamper")
            with self.assertRaisesRegex(e14.Blocked,"DIGEST_CHANGED"):
                e14.validate_manifest(out,self.head,1)
    def test_24_57_real_subprocesses_required_for_final_qualification(self):
        # This test verifies the contract, not 57 falsely claimed executions:
        # CI --exercise performs the actual 57 target processes separately.
        a=e14.audit(ROOT,self.head)
        self.assertEqual((len(a[0]["rust"]),len(a[0]["python_test"])),(24,33))
        self.assertEqual(e14.denied("LAB")["production"],"NOT_AUTHORIZED")

if __name__=="__main__":unittest.main()
