"""BUREAU-001 E5: 24 strict witness-contract tests without human testers.

20 portable source/parser tests + 4 genuinely executed POSIX local
OpenSSH/R5Q cases. No simulated external guardian, key, signature,
independent custodian or certificate. Local example packets are explicitly
untrusted input for failure handling, not external evidence.
"""
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
sys.path.insert(0,str(ROOT/"research"))
import bureau_001_e5_separate_witness_contract as e5
import g03_r6f_r6_external_custody_feasibility as r6

def head():
    return subprocess.check_output(
        ["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()

class PortableContractAndPolicy(unittest.TestCase):
    def test_01_original_e4_real_forgery_status_audited(self):
        d=e5.audit(ROOT,head())
        self.assertEqual(d["reason"],
            "E5_WITNESS_PROOF_OBLIGATIONS_CLASSIFIED_AND_UNMET")
        self.assertTrue(d["prior_real_coherent_log_manifest_forgery_reproduced"])
    def test_02_exact_parent_e4_frozen(self):
        self.assertEqual(e5.PARENT,
            "0c4a4f5b1b349edb445871c4980daba57e936a59")
        p=e5.e3.strictly_load(e5.e3.real_file_bytes(ROOT/e5.CONTRACT))
        self.assertEqual(p["parent_e4_exact_head"],e5.PARENT)
    def test_03_six_independent_obligations_all_unmet(self):
        p=e5.e3.strictly_load(e5.e3.real_file_bytes(ROOT/e5.CONTRACT))
        self.assertEqual(tuple(x["id"] for x in p["proof_obligations"]),
                         e5.EXPECTED_OBLIGATIONS)
        self.assertEqual(tuple(x["status"] for x in p["proof_obligations"]),
                         e5.EXPECTED_STATES)
        self.assertTrue(all(x["real_evidence_sha256"] is None for x in
                            p["proof_obligations"]))
    def test_04_trust_domains_not_mislabelled_independent(self):
        p=e5.e3.strictly_load(e5.e3.real_file_bytes(ROOT/e5.CONTRACT))
        d=p["trust_domains"]
        self.assertEqual(len(d),5)
        self.assertEqual([x["independent_from_local"] for x in d],
                         [False,False,None,None,None])
    def test_05_twelve_native_gates_remain_exact_and_unmet(self):
        p=e5.e3.strictly_load(e5.e3.real_file_bytes(ROOT/e5.CONTRACT))
        self.assertEqual(tuple(x["id"] for x in
                         p["exact_native_admission_gates"]),e5.guardian.GATE_IDS)
        self.assertTrue(all(not x["promoted"] for x in
                            p["exact_native_admission_gates"]))
    def test_06_no_humans_no_external_network_no_contact(self):
        p=e5.e3.strictly_load(e5.e3.real_file_bytes(ROOT/e5.CONTRACT))
        self.assertTrue(p["no_human_testers"])
        self.assertFalse(p["external_network_called"])
        self.assertFalse(p["operator_contacted"])
        self.assertFalse(p["owner_authorization_to_enroll"])
    def test_07_local_untrusted_packet_canonical_parses(self):
        raw=e5.local_claim(head())
        self.assertEqual(e5.parse(raw,head())["source_head"],head())
    def test_08_empty_local_claim_still_denied(self):
        d=e5.assess(e5.local_claim(head()),head())
        self.assertEqual(d["reason"],"E5_NO_INDEPENDENT_EXTERNAL_WITNESS_EVIDENCE")
        self.assertEqual(d["authorized_native_gates"],0)
    def test_09_all_false_independent_flags_denied(self):
        x=dict(locally_claimed_separate_admin=True,
               locally_claimed_oob_verified=True,
               locally_claimed_external_quorum=True,
               locally_claimed_authorization=True)
        d=e5.assess(e5.local_claim(head(),**x),head())
        self.assertEqual(d["reason"],"E5_SELF_DECLARED_EXTERNAL_WITNESS_NOT_PROVEN")
        self.assertEqual(d["local_asserted_authority_count"],4)
        self.assertEqual(d["native_admission_gates_unmet"],12)
    def test_10_unknown_privileged_extra_flag_rejected(self):
        raw=e5.local_claim(head())
        p=e5.parse(raw,head());p["enable_production"]=True
        d=e5.assess(e5.canonical(p),head())
        self.assertEqual(d["reason"],"E5_INVALID_WITNESS_PROPOSAL_QUARANTINED")
    def test_11_missing_sha256_field_rejected(self):
        p=e5.parse(e5.local_claim(head()),head())
        p["producer_manifest_sha256"]="f"*63
        with self.assertRaisesRegex(e5.Blocked,"INVALID_SHA256"):
            e5.parse(e5.canonical(p),head())
    def test_12_replayed_wrong_source_head_rejected(self):
        d=e5.assess(e5.local_claim(head()),"f"*40)
        self.assertEqual(d["reason"],"E5_INVALID_WITNESS_PROPOSAL_QUARANTINED")
    def test_13_key_signature_field_cannot_create_authority(self):
        p=e5.parse(e5.local_claim(head()),head())
        p["independent_custodian_signed"]=True
        d=e5.assess(e5.canonical(p),head())
        self.assertEqual(d["authorized_native_gates"],0)
        self.assertEqual(d["production"],"NOT_AUTHORIZED")
    def test_14_duplicate_json_claim_rejected(self):
        raw=e5.local_claim(head())
        raw=raw.replace(b'"project":',
                        b'"project":"faker","project":',1)
        with self.assertRaisesRegex(e5.Blocked,"DUPLICATE"):
            e5.parse(raw,head())
    def test_15_nan_external_claim_rejected(self):
        raw=e5.local_claim(head()).replace(
             b'"declared_key_epoch":1',b'"declared_key_epoch":NaN')
        with self.assertRaisesRegex(e5.Blocked,"NONFINITE"):
            e5.parse(raw,head())
    def test_16_whitespace_noncanonical_packet_rejected(self):
        raw=json.dumps(e5.parse(e5.local_claim(head()),head()),indent=2).encode()
        with self.assertRaisesRegex(e5.Blocked,"NONCANONICAL"):
            e5.parse(raw,head())
    def test_17_string_true_external_flag_rejected(self):
        p=e5.parse(e5.local_claim(head()),head())
        p["locally_claimed_oob_verified"]="true"
        with self.assertRaisesRegex(e5.Blocked,"FALSE_BOOLEAN"):
            e5.parse(e5.canonical(p),head())
    def test_18_boolean_monotonic_floor_rejected(self):
        p=e5.parse(e5.local_claim(head()),head())
        p["declared_spent_nonce_floor"]=True
        with self.assertRaisesRegex(e5.Blocked,"INVALID_UNTRUSTED_MONOTONIC"):
            e5.parse(e5.canonical(p),head())
    def test_19_negative_local_revocation_floor_rejected(self):
        p=e5.parse(e5.local_claim(head()),head())
        p["declared_revocation_floor"]=-1
        with self.assertRaisesRegex(e5.Blocked,"INVALID_UNTRUSTED_MONOTONIC"):
            e5.parse(e5.canonical(p),head())
    def test_20_high_unverifiable_local_epoch_cannot_authorize(self):
        d=e5.assess(e5.local_claim(head(),
            declared_key_epoch=2**63-1,declared_revocation_floor=2**63-1,
            declared_spent_nonce_floor=2**63-1,
            locally_claimed_authorization=True),head())
        self.assertEqual(d["reason"],"E5_SELF_DECLARED_EXTERNAL_WITNESS_NOT_PROVEN")
        self.assertFalse(d["verified_external_monotonic_ledger"])

@unittest.skipUnless(r6.r4.is_posix_ssh(),
      "requires genuine POSIX OpenSSH Ed25519 and real R5Q filesystem process")
class RealNativeLocalCryptoAndHistory(unittest.TestCase):
    def test_21_true_local_two_key_signed_packet_no_independent_proof(self):
        home,data=r6.local_lab()
        try:
            d=r6.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(d["true_local_ed25519_signatures_verified"],2)
            self.assertEqual(d["reason"],
                "TWO_REAL_LOCAL_SSHSIGS_NO_INDEPENDENT_CUSTODY_EVIDENCE")
        finally:home.cleanup()
    def test_22_true_two_key_signed_fake_owner_oob_still_denied(self):
        home,data=r6.local_lab({
            "claimed_owner_oob_fingerprint_confirmed":True,
            "claimed_separate_administrative_domain":True,
            "claimed_external_operator_id":"same-local-admin-cannot-prove-separation",
            "claimed_independent_quorum":2})
        try:
            d=r6.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(d["true_local_ed25519_signatures_verified"],2)
            self.assertEqual(d["reason"],
                "TWO_REAL_LOCAL_SSHSIGS_SIGNED_UNTRUSTED_EXTERNAL_CLAIMS")
            self.assertFalse(d["externally_confirmed_owner_oob_root"])
        finally:home.cleanup()
    def test_23_actual_r5q_three_file_rollback_defeats_local_history(self):
        d=r6.inspect_real_rollback()
        self.assertTrue(d["same_admin_all_three_restored"])
        self.assertTrue(d["fresh_os_process_observed_old_local_history"])
        self.assertFalse(d["protected_independent_high_water"])
    def test_24_true_e5_live_negatives_return_no_go(self):
        d=e5.exercise(ROOT,head())
        self.assertEqual(d["reason"],
          "E5_REAL_LOCAL_SSHSIG_FALSE_WITNESS_AND_R5Q_ROLLBACK_STILL_NO_EXTERNAL_AUTHORITY")
        self.assertEqual(d["observed_local_signatures_verified"],2)
        self.assertTrue(d["six_proof_obligations_unmet"])
        self.assertEqual(d["authorized_native_gates"],0)
        self.assertEqual(d["human_testers"],0)

if __name__=="__main__":
    unittest.main()
