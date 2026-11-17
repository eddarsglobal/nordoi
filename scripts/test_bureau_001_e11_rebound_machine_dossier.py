"""E11: 24 machine-only controls, including actual coherent on-disk tampering."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e11_rebound_machine_dossier as e11
def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],
                                   cwd=ROOT,text=True).strip()

class E11SourceAndMachinePolicy(unittest.TestCase):
    def test_01_exact_green_parent_sha(self):
        self.assertEqual(e11.PARENT,"ae84f34795c680a6929453f0d4723bec47189a4e")
    def test_02_real_contract_audit_refuses(self):
        d=e11.audit(ROOT,head())
        self.assertEqual(d["result"],"BLOCK")
        self.assertFalse(d["e11_lab_executed"])
    def test_03_wrong_head_rejected(self):
        with self.assertRaises(Exception):e11.audit(ROOT,"f"*40)
    def test_13_nine_roles_exact(self):
        self.assertEqual(e11.e10.ROLES,(
            "ATLAS","FERRUM","BOUNDARY","CIPHER","LEDGER",
            "CAPSTONE","ADVERSARY","VERITY","GUARDIAN"))
    def test_14_no_human_and_founder_not_manual_tester(self):
        c=e11.e3.strictly_load(e11.e10.source(ROOT,e11.CONTRACT))
        self.assertFalse(c["human_testers_allowed"])
        self.assertFalse(c["founder_manual_test_duty"])
        self.assertEqual(c["execution"],"AI_AND_AUTOMATED_MACHINES_ONLY")
    def test_15_false_nine_independent_reviews_rejected(self):
        c=e11.e3.strictly_load(e11.e10.source(ROOT,e11.CONTRACT))
        self.assertEqual(c["formal_independent_specialist_reviews"],0)
        self.assertFalse(c["same_admin_report_signature_is_independent"])
    def test_16_exact_c20_prior_ci_pinned(self):
        c=e11.e3.strictly_load(e11.e10.source(ROOT,e11.CONTRACT))
        self.assertEqual(c["prior_e10"],{
            "specialized_run_id":44067965884,"specialized_success":4,
            "rust_run_id":44067966070,"rust_success":5})
    def test_17_external_security_controls_unmet(self):
        d=e11.audit(ROOT,head())
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(d["production"],"NOT_AUTHORIZED")
        self.assertEqual(d["human_testers"],0)
    def test_18_attack_protocol_exact(self):
        c=e11.e3.strictly_load(e11.e10.source(ROOT,e11.CONTRACT))
        self.assertEqual(tuple(c["attack_steps"]),e11.ATTACK_STEPS)
        self.assertEqual(len(e11.ATTACK_STEPS),7)

def one_role_check(role):
    def test(self):
        d=e11.e10.role_check(ROOT,head(),role)
        self.assertEqual(d["role"],role)
        self.assertEqual(d["verdict"],"MACHINE_CHECK_PASS")
        self.assertEqual(d["source_sha256"],
            {p:e11.e10.digest(e11.e10.source(ROOT,p))
             for p in e11.e10.TARGETS[role]})
    return test

for i,role in enumerate(e11.e10.ROLES,start=4):
    setattr(E11SourceAndMachinePolicy,
        "test_%02d_true_machine_source_%s"%(i,role.lower()),
        one_role_check(role))

class E11ActualDossierTamper(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp=tempfile.TemporaryDirectory(prefix="nordoi-e11-actual-tests-")
        cls.folder=Path(cls.temp.name)/"e10"
        cls.replay=e11.e10.exercise(ROOT,head(),cls.folder)
    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()
    def test_19_actual_nine_process_bundle_source_bound(self):
        self.assertEqual(self.replay["actual_distinct_child_processes"],9)
        d=e11.rebind_actual_repo_sources(ROOT,self.folder,head())
        self.assertEqual(d["actual_source_rebound_roles"],9)
    def test_20_coherent_manifest_and_role_rewrite_old_accepts(self):
        report=self.folder/"ATLAS.json"
        manifest=self.folder/"MANIFEST.json"
        a,b=report.read_bytes(),manifest.read_bytes()
        try:
            changed=e11.rewrite_source_and_rehash_manifest(self.folder)
            self.assertNotEqual(changed["old_sha"],changed["forged_sha"])
            # E10's old validator checks only self-contained local report digests.
            self.assertEqual(
                e11.e10.validate_dossier(self.folder,head())["machine_function_count"],9)
        finally:
            report.write_bytes(a);manifest.write_bytes(b)
    def test_21_new_guard_rejects_consistently_rehashed_atlas(self):
        report=self.folder/"ATLAS.json"
        manifest=self.folder/"MANIFEST.json"
        a,b=report.read_bytes(),manifest.read_bytes()
        try:
            e11.rewrite_source_and_rehash_manifest(self.folder)
            with self.assertRaisesRegex(e11.Blocked,"SOURCE_HASH_NOT_BOUND"):
                e11.rebind_actual_repo_sources(ROOT,self.folder,head())
        finally:
            report.write_bytes(a);manifest.write_bytes(b)
    def test_22_new_guard_rejects_other_forged_role(self):
        report=self.folder/"LEDGER.json"
        manifest=self.folder/"MANIFEST.json"
        a,b=report.read_bytes(),manifest.read_bytes()
        try:
            e11.rewrite_source_and_rehash_manifest(self.folder,role="LEDGER")
            self.assertEqual(e11.e10.validate_dossier(
                self.folder,head())["machine_function_count"],9)
            with self.assertRaisesRegex(e11.Blocked,"SOURCE_HASH_NOT_BOUND"):
                e11.rebind_actual_repo_sources(ROOT,self.folder,head())
        finally:
            report.write_bytes(a);manifest.write_bytes(b)
    def test_23_restored_reports_and_dossier_hashes_pass(self):
        d=e11.rebind_actual_repo_sources(ROOT,self.folder,head())
        self.assertEqual(d["result"],"BLOCK")
        self.assertFalse(d["external_independent_authentication"])
    def test_24_full_real_machine_e11_counterexample(self):
        d=e11.exercise(ROOT,head())
        self.assertTrue(d["old_e10_validator_accepted_coherently_rewritten_report"])
        self.assertTrue(d["new_source_rebound_guard_rejected_coherent_rewrite"])
        self.assertTrue(d["restored_original_source_reports_passed"])
        self.assertFalse(d["external_custody_proven"])
        self.assertEqual(d["native_admission_gates_unmet"],12)

if __name__=="__main__":unittest.main()
