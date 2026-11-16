"""BUREAU-001 R7: 24 adversarial contradiction / external-guardian controls.

16 portable tests inspect real frozen Git sources, source-review records,
temporal owner evidence, R3-R6 additive negative findings and all twelve
unmet native gates. Eight genuine POSIX tests execute existing Ed25519 SSHSIG,
second-OS-process file attack and R5Q real local historical rollback.
No independent custodian, reviewers, external signing, simulated approvals.
"""
from copy import deepcopy
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
sys.path.insert(0,str(ROOT/"research"))
import bureau_001_r7_contradiction_guard as r7
import g03_r6f_r6_external_custody_feasibility as r6
import g03_r6f_r4_fd_race as r4
import g03_r6f_r3_signer_binding as r3
import g03_r5q_coherent_local_rollback as r5q

def evidence():
    return (
        r7.load(ROOT,r7.r1.MANIFEST),
        r7.load(ROOT,r7.r1.CHRONOLOGY),
        r7.load(ROOT,r7.R2),
        r7.load(ROOT,r7.R3),
        r7.load(ROOT,r7.R4),
        r7.load(ROOT,r7.R5),
        r7.load(ROOT,r7.R6),
        r7.load(ROOT,r7.FROZEN),
        r7.load(ROOT,r7.PLAN),
    )

class PortableEvidenceAgainstFalseAuthority(unittest.TestCase):
    def test_01_full_source_audit_reports_seven_contradictions_not_admission(self):
        d=r7.audit(ROOT)
        self.assertEqual(d["reason"],
                         "R7_SOURCE_CONTRADICTIONS_RECONCILED_NO_EXTERNAL_CUSTODY")
        self.assertEqual(d["cross_audit_findings_reconciled"],7)
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["status"],"BLOCK")
    def test_02_original_99_git_objects_not_changed(self):
        manifest=evidence()[0]
        self.assertEqual(manifest["total_pinned_files"],99)
        self.assertEqual(len(manifest["pinned_files"]),99)
        self.assertEqual(r7.r1.audit(ROOT)["FROZEN_SOURCE_BLOBS"],"99/99_PASS")
    def test_03_exact_twelve_frozen_admission_gate_ids(self):
        parts=evidence()
        self.assertEqual(tuple(x["id"] for x in parts[7]["required_gates"]),
                         r7.guardian.GATE_IDS)
        self.assertEqual(tuple(x["id"] for x in parts[8]["external_gate_plan"]),
                         r7.guardian.GATE_IDS)
    def test_04_nine_versus_zero_independently_executed_reviews(self):
        reg=evidence()[2]
        self.assertEqual([x["role"] for x in reg["roles"]],
                         list(r7.guardian.ROLE_IDS))
        self.assertEqual(reg["independent_specialist_agents_executed"],0)
        self.assertEqual(reg["submissions"],[])
        self.assertTrue(all(row["specialist_review_status"]=="NOT_SUBMITTED"
                            for row in reg["roles"]))
    def test_05_old_pending_snapshots_remain_distinct_from_later_owner_logs(self):
        chr=evidence()[1]
        self.assertEqual(chr["source_snapshots_status"],"PENDING_AT_COMMIT_TIME")
        self.assertEqual(len(chr["stages"]),4)
        self.assertTrue(all(x["source_owner_status_expected"]=="PENDING"
                            and x["observed_owner_terminal"]==
                            "PASS_WITH_CONTROLS_RESEARCH_ONLY"
                            and x["source_correction_authorized"] is False
                            for x in chr["stages"]))
    def test_06_six_stage_source_heads_and_historical_reports(self):
        plan=evidence()[8]
        self.assertEqual(tuple((x["stage"],x["head"],
                                x["owner_reported_regular_tests"])
                               for x in plan["historical_reconciliation"]),
                         r7.EXPECTED_HEADS)
        self.assertEqual(plan["parent_r6_exact_head"],r7.PARENT)
    def test_07_blank_future_claim_always_denied_not_enrolled(self):
        x=r7.assess_candidate(r7.canonical(r7.candidate_template()))
        self.assertEqual(x["reason"],"R7_NO_EXTERNAL_AUTHORITY_EVIDENCE_SUBMITTED")
        self.assertEqual(x["external_gates_promoted"],0)
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")
    def test_08_same_admin_claims_external_owner_oob_but_cannot_promote(self):
        packet=r7.candidate_template()
        packet["asserted_operator_id"]="fake-independent-person"
        packet["asserted_owner_oob_root"]="unverified-oob-fingerprint"
        x=r7.assess_candidate(r7.canonical(packet))
        self.assertEqual(x["reason"],
                         "R7_SELF_ASSERTED_EXTERNAL_AUTHORITY_STAGED_UNAUTHENTICATED")
        self.assertEqual(x["native_admission_gates_unmet"],12)
    def test_09_signed_quorum_claim_and_fake_monotonic_floor_still_denied(self):
        packet=r7.candidate_template()
        packet["asserted_independent_quorum"]="two-locally-operated-keys"
        packet["asserted_external_monotonic_floor"]="locally-replayed-history"
        x=r7.assess_candidate(r7.canonical(packet))
        self.assertEqual(x["external_gates_promoted"],0)
        self.assertFalse(x["signed_external_authority_authenticated"])
    def test_10_duplicate_candidate_json_fields_are_quarantined(self):
        raw=r7.canonical(r7.candidate_template()).replace(
            b'"protocol":',b'"protocol":"other","protocol":',1)
        d=r7.assess_candidate(raw)
        self.assertEqual(d["reason"],"R7_EXTERNAL_CLAIM_QUARANTINED")
        self.assertIn("DUPLICATE_JSON_FIELD",d["detail"])
    def test_11_privileged_native_requested_operation_is_quarantined(self):
        packet=r7.candidate_template()
        packet["requested_operation"]="NATIVE_ADMISSION_GRANT"
        x=r7.assess_candidate(r7.canonical(packet))
        self.assertEqual(x["reason"],"R7_EXTERNAL_CLAIM_QUARANTINED")
    def test_12_reordered_or_duplicated_external_gate_set_is_quarantined(self):
        packet=r7.candidate_template()
        packet["asserted_native_gate_ids"]=packet["asserted_native_gate_ids"][::-1]
        x=r7.assess_candidate(r7.canonical(packet))
        self.assertIn("REORDERED",x["detail"])
        self.assertEqual(x["native_admission_gates_unmet"],12)
    def test_13_unknown_self_approved_native_gate_is_quarantined(self):
        packet=r7.candidate_template()
        packet["asserted_native_gate_ids"].append("fake_global_certified")
        d=r7.assess_candidate(r7.canonical(packet))
        self.assertEqual(d["reason"],"R7_EXTERNAL_CLAIM_QUARANTINED")
    def test_14_forged_future_external_gate_is_not_read_as_evidence(self):
        parts=list(evidence())
        parts[-1]=deepcopy(parts[-1])
        parts[-1]["external_gate_plan"][0]["admitted"]=True
        with self.assertRaises(r7.Blocked):
            r7.check_cross_sources(*parts)
    def test_15_forged_nine_independent_specialist_attestations_blocked(self):
        parts=list(evidence())
        parts[2]=deepcopy(parts[2])
        parts[2]["independent_specialist_agents_executed"]=9
        with self.assertRaises(r7.Blocked):
            r7.check_cross_sources(*parts)
    def test_16_forged_external_custodian_in_original_r6_dossier_blocked(self):
        parts=list(evidence())
        parts[6]=deepcopy(parts[6])
        parts[6]["independent_custodian_enrolled"]=True
        with self.assertRaises(r7.Blocked):
            r7.check_cross_sources(*parts)

@unittest.skipUnless(r4.is_posix_ssh(),
                      "actual POSIX OpenSSH keys, real OS file operations and SQLite required")
class ActualIndependentCustodyNegativeResearch(unittest.TestCase):
    def test_17_two_actual_local_ed25519_signatures_prove_only_possession(self):
        home,data=r6.local_lab()
        try:
            d=r6.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(d["true_local_ed25519_signatures_verified"],2)
            self.assertEqual(d["reason"],
                             "TWO_REAL_LOCAL_SSHSIGS_NO_INDEPENDENT_CUSTODY_EVIDENCE")
        finally:home.cleanup()
    def test_18_real_two_signatures_of_untrue_oob_claim_refused(self):
        home,data=r6.local_lab(
            {"claimed_external_operator_id":"false-independent-operator",
             "claimed_owner_oob_fingerprint_confirmed":True},
            filename="r7_forgeries.json"
        )
        try:
            d=r6.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(d["reason"],
                             "TWO_REAL_LOCAL_SSHSIGS_SIGNED_UNTRUSTED_EXTERNAL_CLAIMS")
            self.assertFalse(d["externally_confirmed_owner_oob_root"])
        finally:home.cleanup()
    def test_19_original_r3_real_rogue_signer_attribution_refused(self):
        d,rc=r3.exercise()
        self.assertEqual(rc,2)
        self.assertEqual(d["reason"],
                         "R3_GENUINE_ROGUE_SSHSIG_ATTRIBUTION_GAP_REPRODUCED_AND_BLOCKED")
    def test_20_original_r4_real_second_os_process_path_swap_refused(self):
        d,rc=r4.real_exercise()
        self.assertEqual(rc,2)
        self.assertEqual(d["reason"],
                         "R4_REAL_SECOND_PROCESS_SSHSIG_PATH_RACE_REPRODUCED_AND_QUARANTINED")
    def test_21_actual_three_file_cross_epoch_restore_has_no_global_floor(self):
        d=r5q.native_exercise()
        self.assertEqual(d["reason"],
                         "REAL_TWO_HISTORIES_SAME_LOCAL_EVIDENCE_DENIED")
        self.assertTrue(d["fresh_os_process_observed_old_local_history"])
        self.assertFalse(d["protected_independent_high_water"])
    def test_22_original_r6_full_true_signed_negative_and_rollback(self):
        d,rc=r6.native_exercise()
        self.assertEqual(rc,2)
        self.assertEqual(d["reason"],
                         "R6_REAL_LOCAL_TWO_KEY_OOB_CLAIMS_AND_CROSS_EPOCH_ROLLBACK_NO_EXTERNAL_AUTHORITY")
        self.assertEqual(d["true_local_ssh_signatures_verified"],2)
    def test_23_genuine_local_signed_packet_tamper_is_quarantined(self):
        home,data=r6.local_lab()
        try:
            obj=r6.parse(data["r6_claim_raw"])
            obj["locally_expires_at"]+=1
            d=r6.assess(r6.canonical(obj),data["r6e_body"],1200,data)
            self.assertEqual(d["reason"],
                             "R6_LOCAL_CUSTODY_FEASIBILITY_PROOF_QUARANTINED")
        finally:home.cleanup()
    def test_24_r7_cli_true_ssh_and_rollback_returns_intentional_denial_two(self):
        p=subprocess.run([sys.executable,"-B",str(ROOT/"scripts"/
                          "bureau_001_r7_contradiction_guard.py"),
                          "--exercise"],capture_output=True,text=True,
                         timeout=350,check=False)
        self.assertEqual(p.returncode,2,p.stderr+"\n"+p.stdout)
        d=json.loads(p.stdout)
        self.assertEqual(d["reason"],
                         "R7_REAL_R6_LOCAL_SIGNATURES_R5Q_ROLLBACK_AND_FALSE_CUSTODY_REFUSED")
        self.assertTrue(d["real_original_r5q_three_file_cross_epoch_rollback"])
        self.assertEqual(d["independent_external_gates_promoted"],0)
        self.assertEqual(d["native_admission_gates_unmet"],12)

if __name__=="__main__":
    unittest.main()
