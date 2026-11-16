"""G0.3 R6F BUREAU-001 R6 — 24 adversarial real local vs OOB custody tests.

16 strictly portable packet/protocol/deny-only tests; eight real POSIX
OpenSSH Ed25519 signature and R5Q coherent rollback controls.
No simulation of an external custodian, signatures or network.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r6f_r6_external_custody_feasibility as lab

def fixture(**extra):
    p=dict(protocol=lab.PROTOCOL,project=lab.PROJECT,
           parent=lab.PARENT,operation=lab.OPERATION,
           r6e_receipt_sha256="a"*64,r6e_challenge="b"*64,
           owner_key_sha256="c"*64,guardian_key_sha256="d"*64,
           locally_issued_at=1200,locally_expires_at=1320)
    p.update(dict.fromkeys(lab.CLAIMS,None))
    p.update(extra)
    return p

class PortableTrustBoundaries(unittest.TestCase):
    def test_01_exact_parent_is_owner_qualified_r5(self):
        self.assertEqual(lab.PARENT,"4d3bd3cbff8c67cef98481750270a2c6c0ce7b37")
    def test_02_canonical_local_packet_parses(self):
        self.assertEqual(lab.parse(lab.canonical(fixture())),fixture())
    def test_03_protocol_scope_never_native_grant(self):
        self.assertIn("NO-ADMISSION",lab.PROTOCOL)
        x=lab.deny("NOT_EXTERNAL")
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")
        self.assertEqual(x["native_admission_gates_unmet"],12)
        self.assertEqual(x["production"],"NOT_AUTHORIZED")
    def test_04_one_missing_custody_field_refused(self):
        p=fixture();p.pop("owner_key_sha256")
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_05_unknown_external_authority_field_refused(self):
        p=fixture(authorized_production=True)
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_06_duplicate_json_field_refused(self):
        x=lab.canonical(fixture())
        x=x.replace(b'"parent":',b'"parent":"bogus","parent":',1)
        with self.assertRaises(lab.Refused):lab.parse(x)
    def test_07_noncanonical_spacing_refused(self):
        raw=json.dumps(fixture(),indent=2).encode()+b"\n"
        with self.assertRaises(lab.Refused):lab.parse(raw)
    def test_08_nonfinite_claim_refused(self):
        raw=lab.canonical(fixture()).replace(
            b'"claimed_independent_quorum":null',b'"claimed_independent_quorum":NaN')
        with self.assertRaises(lab.Refused):lab.parse(raw)
    def test_09_one_key_posing_as_two_distinct_keys_refused(self):
        p=fixture(guardian_key_sha256="c"*64)
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_10_invalid_hash_algorithm_or_width_refused(self):
        p=fixture(r6e_challenge="a"*63)
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_11_privileged_operation_refused(self):
        p=fixture(operation="ENROLL_PRODUCTION_GUARDIAN")
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_12_other_parent_or_history_refused(self):
        p=fixture(parent="0"*40)
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_13_local_timebox_too_long_refused(self):
        p=fixture(locally_expires_at=1501)
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_14_boolean_local_timebox_refused(self):
        p=fixture(locally_issued_at=True)
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_15_unbounded_external_identity_claim_refused(self):
        p=fixture(claimed_external_operator_id="X"*2048)
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))
    def test_16_fake_external_proof_data_structure_refused(self):
        p=fixture(claimed_remote_monotonic_head={"host":"external","value":"abc"})
        with self.assertRaises(lab.Refused):lab.parse(lab.canonical(p))

@unittest.skipUnless(lab.r4.is_posix_ssh(),
    "requires real POSIX OpenSSH and actual signed same-admin ED25519 keys")
class RealNegativeExternalAuthority(unittest.TestCase):
    def setUp(self):
        self.home,self.data=lab.local_lab()
    def tearDown(self):
        self.home.cleanup()
    def test_17_real_R4_preflight_six_signatures_are_still_local(self):
        r=lab.r4.assess_lab(self.data)
        self.assertEqual(r["reason"],
            "R4_SIX_LOCAL_SSHSIGS_VERIFIED_WITH_HELD_ALLOWED_FDS_NO_EXTERNAL_CUSTODY")
        self.assertFalse(r["native_authority_granted"])
    def test_18_two_real_local_ed25519_sshsigs_cannot_authenticate_operator(self):
        r=lab.assess(self.data["r6_claim_raw"],self.data["r6e_body"],1200,self.data)
        self.assertEqual(r["reason"],
            "TWO_REAL_LOCAL_SSHSIGS_NO_INDEPENDENT_CUSTODY_EVIDENCE")
        self.assertEqual(r["true_local_ed25519_signatures_verified"],2)
        self.assertEqual(r["native_admission_gates_unmet"],12)
    def test_19_true_local_two_key_signed_oob_claim_is_refused(self):
        claims={"claimed_owner_oob_fingerprint_confirmed":True,
                "claimed_external_operator_id":"fake-independent-operator",
                "claimed_independent_quorum":2}
        home,data=lab.local_lab(claims,filename="r6_fake_1.json")
        try:
            r=lab.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(r["reason"],
                "TWO_REAL_LOCAL_SSHSIGS_SIGNED_UNTRUSTED_EXTERNAL_CLAIMS")
            self.assertEqual(r["true_local_ed25519_signatures_verified"],2)
            self.assertFalse(r["externally_confirmed_owner_oob_root"])
        finally:home.cleanup()
    def test_20_true_local_signed_false_global_head_is_refused(self):
        claims={"claimed_remote_monotonic_head":"f"*64,
                "claimed_external_nonce_floor":1000,
                "claimed_external_revocation_floor":500}
        home,data=lab.local_lab(claims,filename="r6_fake_history.json")
        try:
            r=lab.assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
            self.assertEqual(r["reason"],
                "TWO_REAL_LOCAL_SSHSIGS_SIGNED_UNTRUSTED_EXTERNAL_CLAIMS")
            self.assertTrue(r["external_monotonic_custody_not_verified"])
        finally:home.cleanup()
    def test_21_modified_signed_custody_packet_is_quarantined(self):
        obj=lab.parse(self.data["r6_claim_raw"])
        obj["locally_expires_at"]+=1
        tampered=lab.canonical(obj)
        r=lab.assess(tampered,self.data["r6e_body"],1200,self.data)
        self.assertEqual(r["reason"],"R6_LOCAL_CUSTODY_FEASIBILITY_PROOF_QUARANTINED")
        self.assertIn("ACTUAL_ED25519_SSHSIG_INVALID",r["detail"])
    def test_22_missing_real_owner_signer_is_quarantined(self):
        r=lab.assess(self.data["r6_claim_raw"],self.data["r6e_body"],
                     1200,self.data,
                     owner_signature=self.data["root"]/"missing_r6_owner.sig")
        self.assertEqual(r["reason"],"R6_LOCAL_CUSTODY_FEASIBILITY_PROOF_QUARANTINED")
    def test_23_real_three_file_coherent_history_rollback_remains_unresolved(self):
        r=lab.inspect_real_rollback()
        self.assertTrue(r["same_admin_all_three_restored"])
        self.assertTrue(r["fresh_os_process_observed_old_local_history"])
        self.assertFalse(r["protected_independent_high_water"])
    def test_24_cli_real_two_key_claims_and_rollback_denied(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),
                          "--exercise"],capture_output=True,text=True,timeout=300)
        self.assertEqual(p.returncode,2,p.stderr+"\n"+p.stdout)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],
          "R6_REAL_LOCAL_TWO_KEY_OOB_CLAIMS_AND_CROSS_EPOCH_ROLLBACK_NO_EXTERNAL_AUTHORITY")
        self.assertTrue(x["malicious_same_admin_oob_claim_genuinely_signed"])
        self.assertTrue(x["real_r5q_three_file_coherent_rollback_reproduced"])
        self.assertEqual(x["native_admission_gates_unmet"],12)
        self.assertEqual(x["production"],"NOT_AUTHORIZED")

if __name__=="__main__":
    unittest.main()
