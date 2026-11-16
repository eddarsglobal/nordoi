"""G0.3 R6F BUREAU-001 R9 — 24 genuine evidence and no-go tests.

16 portable contradictory source / authority fraud checks. Eight POSIX tests
use real OpenSSH Ed25519 keys/signatures and actual OS process file changes
and historical SQLite/JSON three-file coherent rollback (no simulated actor).
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
import bureau_001_r9_final_evidence_decision as r9
import bureau_001_r8_closure_gate as r8
import bureau_001_r7_contradiction_guard as r7
import g03_r6f_r6_external_custody_feasibility as r6
import g03_r6f_r5_inplace_aba as r5
import g03_r6f_r4_fd_race as r4
import g03_r5q_coherent_local_rollback as r5q

def parts():
    return (
       r9.load(ROOT,r7.PLAN),r9.load(ROOT,r8.DOSSIER),
       r9.load(ROOT,r7.FROZEN),r9.load(ROOT,r7.R2),
       r9.load(ROOT,r9.DOSSIER)
    )

class PortableSourceAndDecision(unittest.TestCase):
    def test_01_final_source_bound_audit_declares_no_go(self):
        x=r9.audit(ROOT)
        self.assertEqual(x["reason"],
          "R9_FINAL_EVIDENCE_DEPENDENCIES_RECONCILED_FORMAL_CLOSURE_HELD")
        self.assertEqual(x["technical_evidence_categories"],2)
        self.assertEqual(x["external_evidence_classes_missing"],5)
        self.assertEqual(x["native_admission_gates_unmet"],12)
        self.assertFalse(x["formal_r6f_closure_granted"])

    def test_02_exact_unmodified_r8_parent_identity(self):
        self.assertEqual(r9.PARENT,"cad5140206725ed45e84a445176360be13ae8a36")
        self.assertEqual(parts()[-1]["parent_r8_exact_head"],r9.PARENT)

    def test_03_r8_snapshot_still_pending_later_owner_run_separately_logged(self):
        p=parts()
        self.assertEqual(p[1]["owner_mac_intel_r8"],"PENDING")
        o=p[-1]["post_r8_source_observation"]
        self.assertEqual(o["owner_total_regular_tests"],1255)
        self.assertFalse(o["owner_transcript_is_independent_attestation"])
        self.assertFalse(o["github_results_are_independent_custodian_attestation"])

    def test_04_prior_r8_seven_closure_criteria_exact(self):
        p=parts()
        self.assertEqual(tuple(x["id"] for x in p[-1]["closure_criteria"]),
                         r8.EXPECTED_CLOSURE_IDS)
        self.assertEqual(sum(x["status"]=="UNMET" for x in p[-1]["closure_criteria"]),5)

    def test_05_real_missing_external_dependencies_are_acyclic(self):
        b=parts()[-1]["missing_external_evidence_dependency_graph"]
        self.assertEqual(tuple(x["id"] for x in b),r9.GAP_IDS)
        self.assertEqual(tuple(tuple(x["depends_on"]) for x in b),r9.DEPENDENCIES)
        self.assertTrue(all(v in [x["id"] for x in b[:i]]
                            for i,item in enumerate(b)
                            for v in item["depends_on"]))

    def test_06_eight_named_non_simulated_adversarial_scenarios(self):
        s=parts()[-1]["red_team_scenarios"]
        self.assertEqual(tuple(x["id"] for x in s),r9.ATTACK_IDS)
        self.assertTrue(all(x["required_action"]=="DENY" for x in s))

    def test_07_nine_roles_unsubmitted_and_no_forged_signatures(self):
        roles=parts()[-1]["nine_roles_not_independently_reviewed"]
        self.assertEqual(tuple(x["role"] for x in roles),r9.guardian.ROLE_IDS)
        self.assertTrue(all(x["independent_signature"] is None for x in roles))
        self.assertEqual(parts()[3]["submissions"],[])

    def test_08_twelve_external_native_gates_not_promoted(self):
        gates=parts()[-1]["twelve_native_gates"]
        self.assertEqual(tuple(x["id"] for x in gates),r9.guardian.GATE_IDS)
        self.assertTrue(all(not x["admitted"] for x in gates))

    def test_09_green_ci_and_owner_log_cannot_grant_closure(self):
        x=r9.assess_candidate(r9.canonical(r9.candidate_template()))
        self.assertEqual(x["reason"],"R9_NO_NEW_INDEPENDENT_EVIDENCE_HOLD")
        self.assertEqual(x["independent_native_gates_promoted"],0)
        self.assertFalse(x["formal_r6f_closure_granted"])

    def test_10_all_six_forged_external_bureau_checks_still_denied(self):
        x=r9.candidate_template()
        for k in x:
            if k.startswith("claims_"): x[k]=True
        d=r9.assess_candidate(r9.canonical(x))
        self.assertEqual(d["reason"],
                         "R9_SELF_ATTESTED_EXTERNAL_BUREAU_APPROVAL_REJECTED")
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(d["native_admission_gates_unmet"],12)

    def test_11_wrong_parent_source_head_denied(self):
        x=r9.candidate_template()
        x["parent"]="0"*40
        self.assertEqual(r9.assess_candidate(r9.canonical(x))["reason"],
                         "R9_INVALID_OR_PRIVILEGED_DECISION_PACKET_QUARANTINED")

    def test_12_privileged_prod_operation_denied(self):
        x=r9.candidate_template()
        x["requested_operation"]="PROMOTE_R6G_AUTHORIZE_PRODUCTION"
        d=r9.assess_candidate(r9.canonical(x))
        self.assertEqual(d["production"],"NOT_AUTHORIZED")
        self.assertEqual(d["reason"],
                         "R9_INVALID_OR_PRIVILEGED_DECISION_PACKET_QUARANTINED")

    def test_13_json_duplicate_claim_key_denied(self):
        raw=r9.canonical(r9.candidate_template()).replace(
           b'"protocol":',b'"protocol":"other","protocol":',1)
        self.assertIn("DUPLICATE",r9.assess_candidate(raw)["detail"])

    def test_14_mutating_external_dependency_into_signed_proof_denied(self):
        p=list(parts())
        p[-1]=deepcopy(p[-1])
        p[-1]["missing_external_evidence_dependency_graph"][0][
            "verified_by_independent_operator"]=True
        with self.assertRaises(r9.Blocked):
            r9.validate_sources(*p)

    def test_15_marking_one_native_gate_as_admitted_denied(self):
        p=list(parts())
        p[-1]=deepcopy(p[-1])
        p[-1]["twelve_native_gates"][0]["admitted"]=True
        with self.assertRaises(r9.Blocked):
            r9.validate_sources(*p)

    def test_16_claiming_nine_signed_external_reviews_denied(self):
        p=list(parts())
        p[-1]=deepcopy(p[-1])
        p[-1]["nine_roles_not_independently_reviewed"][0]["completed"]=True
        p[-1]["nine_roles_not_independently_reviewed"][0][
            "independent_signature"]="self-signed-local"
        with self.assertRaises(r9.Blocked):
            r9.validate_sources(*p)

@unittest.skipUnless(r4.is_posix_ssh(),
                     "requires actual POSIX OS processes, OpenSSH and R5Q SQLite")
class ActualNegativeEvidence(unittest.TestCase):
    def test_17_two_genuine_same_admin_ed25519_keys_are_not_external(self):
        home,data=r6.local_lab()
        try:
            d=r6.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(d["true_local_ed25519_signatures_verified"],2)
            self.assertEqual(d["reason"],
                "TWO_REAL_LOCAL_SSHSIGS_NO_INDEPENDENT_CUSTODY_EVIDENCE")
        finally:home.cleanup()

    def test_18_real_signed_false_operator_claim_is_denied(self):
        home,data=r6.local_lab({
          "claimed_external_operator_id":"fictitious-other-administrator",
          "claimed_owner_oob_fingerprint_confirmed":True
        },filename="r9_forged_operator.json")
        try:
            d=r6.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(d["reason"],
              "TWO_REAL_LOCAL_SSHSIGS_SIGNED_UNTRUSTED_EXTERNAL_CLAIMS")
            self.assertFalse(d["externally_confirmed_owner_oob_root"])
        finally:home.cleanup()

    def test_19_actual_r5q_three_file_cross_epoch_rollback_still_undetectable(self):
        d=r5q.native_exercise()
        self.assertTrue(d["same_admin_all_three_restored"])
        self.assertTrue(d["fresh_os_process_observed_old_local_history"])
        self.assertFalse(d["protected_independent_high_water"])

    def test_20_actual_r4_second_os_process_allowlist_swap_is_quarantined(self):
        d,exit_code=r4.real_exercise()
        self.assertEqual(exit_code,2)
        self.assertEqual(d["reason"],
          "R4_REAL_SECOND_PROCESS_SSHSIG_PATH_RACE_REPRODUCED_AND_QUARANTINED")

    def test_21_actual_r5_local_two_file_mutation_and_restore_refused(self):
        home,data=r5.r6e.create_lab()
        try:
            d,history=r5.run_guarded_interleaving(data,"paired_inplace")
            self.assertEqual(d["reason"],r5.EXPECTED_QUARANTINE)
            self.assertTrue(history["all_original_bytes_restored"])
        finally:home.cleanup()

    def test_22_original_r8_local_signed_sources_refuse_formal_close(self):
        d,code=r8.exercise(ROOT)
        self.assertEqual(code,2)
        self.assertFalse(d["formal_R6F_closure_granted"])
        self.assertTrue(d["r5q_real_three_file_coherent_rollback"])

    def test_23_original_r9_real_signed_claims_never_promote(self):
        d,code=r9.exercise(ROOT)
        self.assertEqual(code,2)
        self.assertEqual(d["reason"],
           "R9_REAL_LOCAL_SIGNATURES_ROLLBACK_AND_FAKE_FINAL_DECISION_REFUSED")
        self.assertTrue(d["counterfeit_nine_reviews_and_twelve_native_gates_refused"])
        self.assertFalse(d["final_bureau_closure_granted"])

    def test_24_r9_cli_completes_intentional_fail_closed_exit_two(self):
        p=subprocess.run([sys.executable,"-B",
            str(ROOT/"scripts"/"bureau_001_r9_final_evidence_decision.py"),
            "--exercise"],cwd=ROOT,capture_output=True,text=True,timeout=480)
        self.assertEqual(p.returncode,2,p.stderr+"\n"+p.stdout)
        d=json.loads(p.stdout)
        self.assertEqual(d["reason"],
           "R9_REAL_LOCAL_SIGNATURES_ROLLBACK_AND_FAKE_FINAL_DECISION_REFUSED")
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["production"],"NOT_AUTHORIZED")

if __name__=="__main__":
    unittest.main()
