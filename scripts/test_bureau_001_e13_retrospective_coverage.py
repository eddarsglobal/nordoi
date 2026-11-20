"""24 machine-only E13 tests: 99 real source blobs != 57 fresh executions."""
import copy
from pathlib import Path
import subprocess
import sys
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e13_retrospective_coverage as e13

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()

def canonical(v):
    return e13.e12.e11.e10.canonical(v)

class E13Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.sha=head()
        cls.groups,cls.pins=e13.verify_sources(ROOT,cls.sha)
        cls.claim=e13.optimistic_claim(cls.sha,cls.groups)
    def intake(self,c):
        return e13.strict_claim_intake(self.sha,canonical(c),self.groups)
    def test_01_e12_green_exact_source_parent(self):
        self.assertEqual(e13.PARENT,"4ce07de0ee0478ee05794303bf528d071ef07488")
    def test_02_source_audit_block_not_native_admission(self):
        d=e13.audit(ROOT,self.sha)
        self.assertEqual(d["result"],"BLOCK")
        self.assertEqual(d["source_pins_reported"],99)
    def test_03_real_git_tree_exists(self):
        self.assertGreater(len(e13.true_git_tree(ROOT,self.sha)),99)
    def test_04_99_pinned_real_git_tree_blobs(self):
        tree=e13.true_git_tree(ROOT,self.sha)
        self.assertEqual(len(self.pins),99)
        self.assertTrue(all(tree[p][2]==row["git_blob_sha1"]
                            for p,row in self.pins.items()))
    def test_05_real_24_native_rust_test_files(self):
        self.assertEqual(len(self.groups["rust"]),24)
        self.assertTrue(all(x.endswith(".rs") for x in self.groups["rust"]))
    def test_06_real_33_python_sources(self):
        self.assertEqual(len(self.groups["python_source"]),33)
    def test_07_real_33_python_test_pairs(self):
        self.assertEqual(len(self.groups["python_test"]),33)
        self.assertEqual({p.replace("research/g03_","research/test_g03_",1)
                          for p in self.groups["python_source"]},
                         set(self.groups["python_test"]))
    def test_08_exact_four_source_snapshots(self):
        self.assertEqual(set(self.groups["snapshots"]),set(e13.r1.SNAPSHOTS))
    def test_09_exact_five_governance_basis_files(self):
        self.assertEqual(set(self.groups["basis"]),set(e13.r1.BUREAU_OLD))
    def test_10_no_overlap_or_unmapped_files(self):
        paths=sum((self.groups[k] for k in
          ("rust","python_source","python_test","snapshots","basis")),[])
        self.assertEqual(len(paths),99)
        self.assertEqual(set(paths),set(self.pins))
    def test_11_naive_57_local_file_claims_would_pass_wrongly(self):
        self.assertTrue(e13.naive_accept_claim(self.claim))
    def test_12_strict_57_claims_still_no_runtime_attestation(self):
        d=self.intake(self.claim)
        self.assertEqual(d["untrusted_claimed_historical_test_file_executions"],57)
        self.assertEqual(d["legacy_historical_execution_attestations_verified_by_e13"],0)
        self.assertEqual(d["result"],"BLOCK")
    def test_13_untrusted_nine_independent_review_claim_not_accepted(self):
        self.assertEqual(self.claim["self_declared_independent_bureau_reviews"],9)
        d=self.intake(self.claim)
        self.assertEqual(d["independent_specialist_reviews"],0)
    def test_14_untrusted_external_custodian_claim_not_accepted(self):
        d=self.intake(self.claim)
        self.assertFalse(d["actual_external_trust_verified"])
        self.assertEqual(d["independent_external_witness_receipts"],0)
    def test_15_untrusted_native_admission_claim_not_accepted(self):
        d=self.intake(self.claim)
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(d["production"],"NOT_AUTHORIZED")
    def test_16_nine_handoff_roles_still_formally_unsubmitted(self):
        h=e13.e3.strictly_load(e13.r1.raw(ROOT,
                "governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json"))
        self.assertEqual(len(h["review_functions"]),9)
        self.assertTrue(all(x["review_status"]=="NOT_SUBMITTED" for x in h["review_functions"]))
    def test_17_wrong_source_head_rejected(self):
        with self.assertRaises(Exception):e13.audit(ROOT,"0"*40)
    def test_18_owner_ai_only_zero_humans(self):
        c=e13.e3.strictly_load(e13.r1.raw(ROOT,e13.CONTRACT))
        self.assertFalse(c["human_testers_allowed"])
        self.assertFalse(c["founder_manual_test_duty"])
        self.assertEqual(c["execution"],"AI_AND_AUTOMATION_ONLY")
    def test_19_missing_rust_file_claim_rejected(self):
        c=copy.deepcopy(self.claim);c["asserted_rust_executed_pass"].pop()
        with self.assertRaisesRegex(e13.Blocked,"INCOMPLETE_DUPLICATED"):
            self.intake(c)
    def test_20_duplicated_python_test_claim_rejected(self):
        c=copy.deepcopy(self.claim)
        c["asserted_python_executed_pass"][-1]=c["asserted_python_executed_pass"][0]
        with self.assertRaisesRegex(e13.Blocked,"INCOMPLETE_DUPLICATED"):
            self.intake(c)
    def test_21_injected_fake_file_claim_rejected(self):
        c=copy.deepcopy(self.claim)
        c["asserted_rust_executed_pass"][0]="tests/fake_native_admission.rs"
        with self.assertRaisesRegex(e13.Blocked,"INCOMPLETE_DUPLICATED"):
            self.intake(c)
    def test_22_invalid_boolean_as_independent_review_count_rejected(self):
        c=copy.deepcopy(self.claim)
        c["self_declared_independent_bureau_reviews"]=True
        with self.assertRaisesRegex(e13.Blocked,"INVALID_UNTRUSTED_AUTHORITY"):
            self.intake(c)
    def test_23_duplicate_json_field_rejected(self):
        raw=canonical(self.claim)
        forged=raw.replace(b'"project":',b'"project":"forged","project":',1)
        with self.assertRaises(e13.e3.Rejected):
            e13.strict_claim_intake(self.sha,forged,self.groups)
    def test_24_real_e13_git_and_coverage_attack_no_native(self):
        d=e13.exercise(ROOT,self.sha)
        self.assertEqual(d["actual_git_tree_blobs_compared_to_frozen_manifest"],99)
        self.assertEqual(d["actual_historical_rust_source_test_files"],24)
        self.assertEqual(d["actual_historical_python_source_test_pairs"],33)
        self.assertTrue(d["false_positive_naive_coverage_reproduced"])
        self.assertTrue(d["strict_coverage_guard_rejected_execution_laundering"])
        self.assertTrue(d["missing_rust_test_file_claim_rejected"])
        self.assertEqual(d["actual_history_tests_reexecuted_by_this_program"],0)
        self.assertTrue(d["no_claim_of_absence_of_prior_ci_tests"])
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["human_testers"],0)

if __name__=="__main__":unittest.main()
