"""NORDOI G0.3 R6F BUREAU-001 R8 — 24 closure anti-self-certification tests.

16 portable source-anchored R1-R7 closure requirements and bogus approval
negative cases; eight real POSIX Ed25519 OpenSSH / three-file coherent rollback
checks. No independent human reviewer is run by this suite.
"""
from copy import deepcopy
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
sys.path.insert(0,str(ROOT/"research"))
import bureau_001_r8_closure_gate as r8
import bureau_001_r7_contradiction_guard as r7
import g03_r6f_r6_external_custody_feasibility as r6
import g03_r6f_r4_fd_race as r4
import g03_r5q_coherent_local_rollback as r5q

def sources():
    return (
        r8.load(ROOT,r7.PLAN),
        r8.load(ROOT,r7.R2),
        r8.load(ROOT,r7.FROZEN),
        r8.load(ROOT,r7.R6),
        r8.load(ROOT,r8.DOSSIER),
    )

class PortableR6FClosureDenial(unittest.TestCase):
    def test_01_real_seven_stage_closure_audit_blocks_formal_r6f(self):
        x=r8.audit(ROOT)
        self.assertEqual(x["reason"],
                         "R8_RESEARCH_DOSSIER_RECONCILED_FORMAL_R6F_CLOSURE_BLOCKED")
        self.assertEqual(x["research_technical_evidence_classified"],2)
        self.assertEqual(x["external_closure_criteria_unmet"],5)
        self.assertEqual(x["native_admission_gates_unmet"],12)
        self.assertFalse(x["bureau_research_dossier_formal_closure_granted"])

    def test_02_all_seven_owner_stage_source_pins_reconciled(self):
        s=sources()[-1]["evidence_timeline"]
        self.assertEqual(tuple(x["stage"] for x in s),r8.STAGES)
        self.assertEqual(tuple(x["owner_reported_regular_tests"] for x in s),
                         r8.EXPECTED_COUNTS)
        self.assertEqual(s[-1]["head"],r8.PARENT)

    def test_03_original_native_twelve_gates_are_exact_and_unmet(self):
        s=sources()
        self.assertEqual(tuple(x["id"] for x in s[2]["required_gates"]),
                         r8.guardian.GATE_IDS)
        self.assertEqual(tuple(x["id"] for x in s[-1]["twelve_native_gates"]),
                         r8.guardian.GATE_IDS)
        self.assertTrue(all(not x["admitted"] for x in s[-1]["twelve_native_gates"]))

    def test_04_only_two_technical_criteria_not_formal_certification(self):
        rows=sources()[-1]["closure_criteria"]
        self.assertEqual(len(rows),7)
        self.assertEqual(sum(x["status"]=="UNMET" for x in rows),5)
        self.assertEqual(rows[0]["category"],"LOCAL_TECHNICAL_SUPPORTED_OWNER_LOG")

    def test_05_nine_role_cards_have_no_independent_signed_submissions(self):
        d=sources()[-1]
        self.assertEqual(len(d["nine_bureau_roles"]),9)
        self.assertTrue(all(x["independent_signature"] is None and
                            not x["review_submitted"] for x in d["nine_bureau_roles"]))
        self.assertEqual(d["independently_executed_specialist_reviews"],0)

    def test_06_real_r7_source_audit_does_not_become_certification(self):
        x=r7.audit(ROOT)
        self.assertEqual(x["cross_audit_findings_reconciled"],7)
        self.assertEqual(x["verified_r1_frozen_git_objects"],99)
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")

    def test_07_owner_terminal_success_is_not_independent_identity(self):
        for x in sources()[-1]["evidence_timeline"]:
            self.assertEqual(x["owner_evidence_class"],
                            "OWNER_SUPPLIED_TERMINAL_NOT_INDEPENDENT_HARDWARE_ATTESTATION")

    def test_08_ordinary_technical_green_claim_cannot_close(self):
        x=r8.assess_candidate(r8.canonical(r8.candidate_template()))
        self.assertEqual(x["reason"],"R8_LOCAL_RESEARCH_TESTS_CANNOT_CLOSE_R6F")
        self.assertFalse(x["formal_closure_granted"])

    def test_09_nine_locally_asserted_specialists_are_not_independent(self):
        p=r8.candidate_template()
        p["claimed_independent_reviewer_count"]=9
        x=r8.assess_candidate(r8.canonical(p))
        self.assertEqual(x["reason"],
                         "R8_SELF_ASSERTED_INDEPENDENT_CLOSURE_UNAUTHENTICATED")
        self.assertEqual(x["independent_specialist_reviews"],0)

    def test_10_twelve_falsely_green_native_gates_do_not_promote(self):
        p=r8.candidate_template()
        p["claimed_native_gate_status"]=["VERIFIED"]*12
        p["claimed_external_custodian"]="locally-invented-third-party"
        x=r8.assess_candidate(r8.canonical(p))
        self.assertEqual(x["native_gates_promoted"],0)
        self.assertEqual(x["native_admission_gates_unmet"],12)

    def test_11_duplicate_json_is_quarantined(self):
        raw=r8.canonical(r8.candidate_template())
        raw=raw.replace(b'"protocol":',b'"protocol":"other","protocol":',1)
        x=r8.assess_candidate(raw)
        self.assertEqual(x["reason"],
                         "R8_MALFORMED_OR_PRIVILEGED_CLOSURE_CLAIM_QUARANTINED")
        self.assertIn("DUPLICATE_JSON_FIELD",x["detail"])

    def test_12_mismatched_parent_r7_commit_is_quarantined(self):
        p=r8.candidate_template()
        p["parent"]="0"*40
        x=r8.assess_candidate(r8.canonical(p))
        self.assertEqual(x["reason"],
                         "R8_MALFORMED_OR_PRIVILEGED_CLOSURE_CLAIM_QUARANTINED")

    def test_13_requested_production_permission_is_quarantined(self):
        p=r8.candidate_template()
        p["requested_operation"]="AUTHORIZE_NATIVE_ADMISSION_AND_R6G"
        x=r8.assess_candidate(r8.canonical(p))
        self.assertEqual(x["native_authority_granted"],False)
        self.assertEqual(x["reason"],
                         "R8_MALFORMED_OR_PRIVILEGED_CLOSURE_CLAIM_QUARANTINED")

    def test_14_dossier_attempt_to_admit_an_external_gate_is_rejected(self):
        parts=list(sources())
        parts[-1]=deepcopy(parts[-1])
        parts[-1]["twelve_native_gates"][0]["admitted"]=True
        with self.assertRaises(r8.Blocked):
            r8.check_sources(*parts)

    def test_15_forged_hardware_test_count_is_rejected(self):
        parts=list(sources())
        parts[-1]=deepcopy(parts[-1])
        parts[-1]["evidence_timeline"][-1]["owner_reported_regular_tests"]=7777
        with self.assertRaises(r8.Blocked):
            r8.check_sources(*parts)

    def test_16_fake_external_signed_specialist_review_is_rejected(self):
        parts=list(sources())
        parts[-1]=deepcopy(parts[-1])
        parts[-1]["nine_bureau_roles"][0]["review_submitted"]=True
        parts[-1]["nine_bureau_roles"][0]["independent_signature"]="signed-local"
        with self.assertRaises(r8.Blocked):
            r8.check_sources(*parts)

@unittest.skipUnless(r4.is_posix_ssh(),
                      "real POSIX OpenSSH, genuine Ed25519 and SQLite file rollback required")
class ActualSignedLocalEvidenceNotCustody(unittest.TestCase):
    def test_17_actual_two_local_ed25519_signatures_only_prove_possession(self):
        home,data=r6.local_lab()
        try:
            x=r6.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(x["true_local_ed25519_signatures_verified"],2)
            self.assertFalse(x["independent_operator_attested"])
        finally:
            home.cleanup()

    def test_18_true_signatures_over_false_oob_operator_still_denied(self):
        h,d=r6.local_lab({
            "claimed_owner_oob_fingerprint_confirmed":True,
            "claimed_external_operator_id":"fictional-custodian",
            "claimed_independent_quorum":2
        },filename="r8_fake_independent_provenance.json")
        try:
            r=r6.assess(d["r6_claim_raw"],d["r6e_body"],1200,d)
            self.assertEqual(r["reason"],
                             "TWO_REAL_LOCAL_SSHSIGS_SIGNED_UNTRUSTED_EXTERNAL_CLAIMS")
            self.assertFalse(r["externally_confirmed_owner_oob_root"])
        finally:
            h.cleanup()

    def test_19_actual_local_three_file_rollback_is_not_global_ledger(self):
        d=r5q.native_exercise()
        self.assertEqual(d["reason"],
                         "REAL_TWO_HISTORIES_SAME_LOCAL_EVIDENCE_DENIED")
        self.assertTrue(d["fresh_os_process_observed_old_local_history"])
        self.assertFalse(d["protected_independent_high_water"])

    def test_20_r7_real_two_signed_local_false_approval_is_still_blocked(self):
        d,code=r7.exercise(ROOT)
        self.assertEqual(code,2)
        self.assertTrue(d["real_original_r5q_three_file_cross_epoch_rollback"])
        self.assertEqual(d["independent_external_gates_promoted"],0)

    def test_21_r8_real_source_check_still_blocks_formal_closure(self):
        d=r8.audit(ROOT)
        self.assertEqual(d["external_closure_criteria_unmet"],5)
        self.assertFalse(d["bureau_research_dossier_formal_closure_granted"])

    def test_22_real_open_ssh_custody_signature_tampering_quarantined(self):
        home,data=r6.local_lab()
        try:
            obj=r6.parse(data["r6_claim_raw"])
            obj["locally_expires_at"]+=1
            d=r6.assess(r6.canonical(obj),data["r6e_body"],1200,data)
            self.assertEqual(d["reason"],
                             "R6_LOCAL_CUSTODY_FEASIBILITY_PROOF_QUARANTINED")
        finally:
            home.cleanup()

    def test_23_real_signed_claims_and_local_full_rollback_do_not_close(self):
        d,code=r8.exercise(ROOT)
        self.assertEqual(code,2)
        self.assertEqual(
            d["reason"],
            "R8_REAL_SIGNED_LOCAL_PROOFS_CANNOT_CLOSE_BUREAU_WITHOUT_EXTERNAL_AUTHORITY")
        self.assertFalse(d["formal_R6F_closure_granted"])
        self.assertEqual(d["external_admission_gates_promoted"],0)

    def test_24_actual_cli_research_deny_only_exit_2(self):
        p=subprocess.run([
            sys.executable,"-B",str(ROOT/"scripts"/"bureau_001_r8_closure_gate.py"),
            "--exercise"],cwd=ROOT,capture_output=True,text=True,timeout=400)
        self.assertEqual(p.returncode,2,p.stderr+"\n"+p.stdout)
        d=json.loads(p.stdout)
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertFalse(d["formal_R6F_closure_granted"])

if __name__=="__main__":
    unittest.main()
