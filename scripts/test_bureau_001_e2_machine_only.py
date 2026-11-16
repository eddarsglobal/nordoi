"""BUREAU-001 E2: 24 portable tests of zero-human execution policy and evidence.

These tests check the source-bound machine test contract and parser. No
fabricated specialist report, human test, or external administrative signature.
Actual nine role suites are executed separately by the E2 CI macOS runner.
"""
from copy import deepcopy
from pathlib import Path
import json
import sys
from unittest import TestCase, main
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e2_machine_only as e2

class RealSourcePolicyChecks(TestCase):
    def test_01_real_frozen_sources_and_nine_roles(self):
        d=e2.audit(ROOT)
        self.assertEqual(d["role_source_pins"],"9/9_PASS")
        self.assertEqual(d["frozen_r1_git_blobs"],"99/99_PASS")
    def test_02_zero_humans_required_by_owner(self):
        a=e2.load(ROOT,e2.POLICY)
        self.assertFalse(a["human_testers_allowed"])
        self.assertEqual(a["human_test_execution_count"],0)
    def test_03_human_testing_outreach_permanently_disabled(self):
        a=e2.load(ROOT,e2.POLICY)
        self.assertFalse(a["human_reviewer_outreach_allowed"])
        self.assertFalse(a["human_review_packet_distribution_allowed"])
    def test_04_legacy_human_invitation_cancelled_not_sent(self):
        a=e2.load(ROOT,e2.POLICY)
        self.assertEqual(len(a["superseded_e1_actions"]),3)
        b=e2.load(ROOT,e2.E1)
        self.assertEqual(b["status"]["requests_sent"],0)
        self.assertFalse(b["status"]["operator_contacted"])
    def test_05_original_nine_git_blob_sha1_anchors_recomputed(self):
        a=e2.load(ROOT,e2.E1)["independent_review_requests"]
        self.assertEqual(len(a),9)
        for x in a:
            self.assertEqual(
                e2.source_hash(ROOT,x["baseline_research_anchor"]["path"]),
                x["baseline_research_anchor"]["git_blob_sha1"])
    def test_06_nine_roles_are_not_independent_ai_agents(self):
        a=e2.load(ROOT,e2.POLICY)
        self.assertTrue(all(
          x["execution_type"]=="MACHINE_TEST_SUITE_NOT_INDEPENDENT_SPECIALIST_ATTESTATION"
          and x["status"]=="SCHEDULED_ONLY_NOT_EXECUTED"
          and x["independent_administrative_authority"] is False
          for x in a["nine_bureau_machine_test_functions"]))
    def test_07_already_received_external_specialist_reports_still_zero(self):
        r=e2.load(ROOT,e2.R2)
        self.assertEqual(r["submissions"],[])
        self.assertEqual(r["independent_specialist_agents_executed"],0)
    def test_08_the_twelve_native_gates_remain_unmet(self):
        self.assertEqual(e2.load(ROOT,e2.POLICY)["native_admission_gates_unmet"],12)
        self.assertEqual(e2.audit(ROOT)["r6g"],"LOCKED_BY_SEQUENCE")
    def test_09_external_guardian_not_enrolled_no_oob_forgery(self):
        a=e2.load(ROOT,e2.POLICY)
        self.assertEqual(a["real_external_custodian"],"NOT_ENROLLED")
        self.assertEqual(a["owner_oob_root"],"NOT_AUTHENTICATED")
    def test_10_no_global_monotonic_rollback_certificate(self):
        a=e2.load(ROOT,e2.POLICY)
        self.assertEqual(a["global_monotonic_antirollback"],"NOT_VERIFIED")
        self.assertEqual(a["formal_r6f_closure"],"HOLD_MISSING_INDEPENDENT_EVIDENCE")
    def test_11_owner_governance_authorization_is_not_manual_testing(self):
        a=e2.load(ROOT,e2.POLICY)
        self.assertEqual(
           a["manual_owner_privileged_authorization"],
           "STILL_REQUIRED_BY_FROZEN_GOVERNANCE_AND_IS_NOT_A_TEST")
    def test_12_exact_nine_machine_role_suites_have_unique_names(self):
        specs=e2.specs()
        self.assertEqual(tuple(x[0] for x in specs),e2.EXPECTED_ROLES)
        self.assertEqual(len(set(x[0] for x in specs)),9)
    def test_13_machine_commands_are_actual_cargo_or_unittest(self):
        s=e2.specs()
        self.assertTrue(all(x[1][0] in ("cargo",sys.executable) for x in s))
        self.assertFalse(any("curl" in x[1] or "ssh" in x[1] for x in s))
    def test_14_native_r4m_is_real_macos_serial_six_no_waiver(self):
        role=[x for x in e2.specs() if x[0]=="FERRUM"][0]
        self.assertIn("g03_r4m_pinned_code_identity_lab",role[1])
        self.assertIn("--test-threads=1",role[1])
        self.assertEqual(role[2],"RUST_PASS_6")
    def test_15_security_boundary_runs_true_second_os_process_tests(self):
        role=[x for x in e2.specs() if x[0]=="BOUNDARY"][0]
        self.assertIn("test_g03_r6f_r4_fd_race.py",role[1])
        self.assertTrue((ROOT/"research/test_g03_r6f_r4_fd_race.py").is_file())
    def test_16_cipher_runs_true_ssh_signature_negative_cases(self):
        role=[x for x in e2.specs() if x[0]=="CIPHER"][0]
        self.assertIn("test_g03_r6f_r6_external_custody_feasibility.py",role[1])
    def test_17_ledger_runs_actual_r5_multifile_aba_and_rollback(self):
        role=[x for x in e2.specs() if x[0]=="LEDGER"][0]
        self.assertIn("test_g03_r6f_r5_inplace_aba.py",role[1])
    def test_18_capstone_and_adversary_both_verify_no_native_grants(self):
        s={role:cmd for role,cmd,_ in e2.specs()}
        self.assertIn("test_bureau_001_r8_closure_gate.py",s["CAPSTONE"])
        self.assertIn("test_bureau_001_r9_final_evidence_decision.py",
                      s["ADVERSARY"])
    def test_19_verity_checks_actual_rustfmt_no_generated_waiver(self):
        s={role:cmd for role,cmd,_ in e2.specs()}
        self.assertEqual(s["VERITY"],["cargo","fmt","--all","--","--check"])
    def test_20_guardian_runs_real_24_original_tests(self):
        role=[x for x in e2.specs() if x[0]=="GUARDIAN"][0]
        self.assertIn("test_roadmap_guardian.py",role[1])
        self.assertEqual(role[2],"PYTHON_PASS_24")
    def test_21_parser_rejects_python_case_count_mismatch(self):
        with self.assertRaises(e2.EvidenceError):
            e2.verify_result(b"Ran 23 tests in 0.1s\n\nOK\n",
                             "PYTHON_PASS_24",0)
    def test_22_parser_rejects_rust_failure_even_with_test_output(self):
        with self.assertRaises(e2.EvidenceError):
            e2.verify_result(b"test result: FAILED. 5 passed; 1 failed",
                             "RUST_PASS_6",1)
    def test_23_parser_checks_exact_native_six_no_skip(self):
        self.assertTrue(e2.verify_result(
            b"test result: ok. 6 passed; 0 failed; 0 ignored;",
            "RUST_PASS_6",0))
        with self.assertRaises(e2.EvidenceError):
            e2.verify_result(
                b"test result: ok. 5 passed; 0 failed; 1 ignored;",
                "RUST_PASS_6",0)
    def test_24_policy_rejects_fake_human_testing_approval(self):
        original=e2.load(ROOT,e2.POLICY)
        assert original["human_testers_allowed"] is False
        def fake_load(root,path):
            if path==e2.POLICY:
                cloned=deepcopy(original)
                cloned["human_testers_allowed"]=True
                return cloned
            return e2.load_original(root,path)
        e2.load_original=e2.load
        try:
            with patch.object(e2,"load",side_effect=fake_load):
                with self.assertRaisesRegex(e2.EvidenceError,
                    "ZERO_HUMAN_POLICY_NOT_ENFORCED"):
                    e2.audit(ROOT)
        finally:
            del e2.load_original

if __name__=="__main__": main()
