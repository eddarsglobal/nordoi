"""NORDOI G0.3 R6F / E10: 24 real automated controls, zero humans."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e10_ai_only_machine_dossier as e10

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],
                                   cwd=ROOT,text=True).strip()

class E10MachineTests(unittest.TestCase):
    def test_01_exact_parent_green_commit(self):
        self.assertEqual(e10.PARENT,"c789f02c637d9d8215772a67ca2e85cef35e64af")
    def test_02_real_source_audit_is_block_not_admission(self):
        d=e10.audit(ROOT,head())
        self.assertEqual(d["result"],"BLOCK")
        self.assertEqual(d["actual_machine_checks_executed"],0)
    def test_03_exact_nine_function_names(self):
        self.assertEqual(e10.ROLES,(
            "ATLAS","FERRUM","BOUNDARY","CIPHER","LEDGER",
            "CAPSTONE","ADVERSARY","VERITY","GUARDIAN"))
    def test_13_wrong_git_head_rejected(self):
        with self.assertRaises(Exception):
            e10.contract(ROOT,"f"*40)
    def test_14_owner_no_human_and_no_manual_test_duty(self):
        c=e10.contract(ROOT,head())
        self.assertFalse(c["human_testers_allowed"])
        self.assertFalse(c["owner_manual_test_duty"])
        self.assertEqual(c["real_independent_ai_agents_launched"],0)
    def test_15_zero_independent_specialists_quorum_and_receipts(self):
        c=e10.contract(ROOT,head())
        self.assertEqual(c["actual_external_witness_receipts"],0)
        self.assertEqual(c["independent_specialist_reviews"],0)
        self.assertEqual(c["external_quorum"],"NOT_PROVEN")
    def test_16_actual_five_source_anchored_contradictions(self):
        c=e10.contract(ROOT,head())
        for row in c["contradictions"]:
            self.assertTrue((ROOT/row["source"]).is_file())
            self.assertNotEqual(row["status"],"RESOLVED_GLOBALLY")
        self.assertEqual(len(c["contradictions"]),5)
    def test_17_external_intake_must_reject_self_asserted_independence(self):
        a=e10.contract(ROOT,head())["external_evidence_intake"]
        self.assertTrue(a["self_certified_or_same_admin_artifacts_must_be_rejected"])
        self.assertEqual(a["current_acceptance"],"NONE")
        self.assertFalse(a["fabrication_allowed"])
    def test_18_deny_cannot_be_overridden(self):
        d=e10.deny("LAB",production="AUTHORIZED",r6g="OPEN",
                   native_admission_gates_unmet=0,human_testers=1)
        self.assertEqual(d["result"],"BLOCK")
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["human_testers"],0)
        self.assertEqual(d["production"],"NOT_AUTHORIZED")
    def test_19_canonical_report_roundtrip(self):
        d=e10.role_check(ROOT,head(),"ATLAS")
        raw=e10.canonical(d)
        self.assertEqual(e10.sealed_report(raw)["role"],"ATLAS")
    def test_20_false_independent_role_report_is_rejected(self):
        d=e10.role_check(ROOT,head(),"FERRUM")
        d["formal_independent_review"]="CERTIFIED"
        with self.assertRaisesRegex(e10.Blocked,"PROMOTION"):
            e10.sealed_report(e10.canonical(d))
    def test_21_noncanonical_evidence_is_rejected(self):
        d=e10.role_check(ROOT,head(),"CIPHER")
        raw=json.dumps(d,indent=2).encode()
        with self.assertRaisesRegex(e10.Blocked,"NONCANONICAL"):
            e10.sealed_report(raw)
    def test_22_invalid_function_is_rejected(self):
        with self.assertRaisesRegex(e10.Blocked,"INVALID_BUREAU_FUNCTION"):
            e10.role_check(ROOT,head(),"TEN_TENTH_SIGNER")
    def test_23_real_report_substitution_is_detected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e10-test-") as p:
            out=Path(p)/"output"
            d=e10.exercise(ROOT,head(),out)
            self.assertEqual(d["generated_report_files"],9)
            source=out/"LEDGER.json"
            source.write_bytes(source.read_bytes()+b" ")
            with self.assertRaises((e10.Blocked,ValueError)):
                e10.validate_dossier(out,head())
    def test_24_real_nine_child_processes_and_manifest(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-e10-test-") as p:
            out=Path(p)/"dossier"
            d=e10.exercise(ROOT,head(),out)
            self.assertEqual(d["actual_distinct_child_processes"],9)
            self.assertEqual(d["human_testers"],0)
            self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
            manifest=e10.validate_dossier(out,head())
            self.assertEqual(manifest["machine_function_count"],9)
            self.assertEqual(manifest["formal_independent_reviews"],0)
            self.assertEqual(manifest["external_witness_receipts"],0)

def make_role_test(role):
    def check(self):
        d=e10.role_check(ROOT,head(),role)
        self.assertEqual(d["role"],role)
        self.assertEqual(d["verdict"],"MACHINE_CHECK_PASS")
        self.assertEqual(d["formal_independent_review"],"NOT_SUBMITTED")
        self.assertFalse(d["native_admission"])
        self.assertGreaterEqual(len(d["source_sha256"]),1)
    return check

for index,role in enumerate(e10.ROLES,start=4):
    setattr(E10MachineTests,
            "test_%02d_actual_machine_function_%s"%(index,role.lower()),
            make_role_test(role))

if __name__=="__main__":unittest.main()
