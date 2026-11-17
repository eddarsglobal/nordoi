"""R6F/E12: 24 genuine machine tests for the twelve-gate intake boundary."""
import copy
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import subprocess
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e12_ai_only_native_evidence_intake as e12

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],
                                   cwd=ROOT,text=True).strip()

def encode(c):
    return e12.e11.e10.canonical(c)

class E12Portable(unittest.TestCase):
    def setUp(self):
        self.claim=e12.asserted_claim(head())
    def check(self,c=None,crypto=False):
        return e12.intake_untrusted_json(
            ROOT,head(),encode(self.claim if c is None else c),
            local_crypto_valid=crypto)
    def test_01_source_audit_block(self):
        self.assertEqual(e12.audit(ROOT,head())["result"],"BLOCK")
    def test_02_exact_parent_e11_green(self):
        self.assertEqual(e12.PARENT,"96e56f954721eb96f3526e529a27b2456bd2e8ce")
    def test_03_exact_twelve_native_gate_ids_and_kinds(self):
        self.assertEqual(e12.GATES,tuple(e12.guardian.GATE_IDS))
        self.assertEqual(len(e12.GATES),len(e12.KINDS))
        self.assertEqual(len(set(e12.GATES)),12)
    def test_04_external_trust_store_unenrolled(self):
        c=e12.e3.strictly_load(e12.e11.e10.source(ROOT,e12.CONTRACT))
        self.assertEqual(c["external_trust_roots_enrolled"],0)
        self.assertEqual(c["external_witness_receipts_authenticated"],0)
        self.assertEqual(c["real_independent_bureau_audits"],0)
    def test_05_naive_self_asserted_gate_would_open(self):
        self.assertTrue(e12.unsafe_naive_self_promotion(self.claim))
    def test_06_correct_gate_denies_all_twelve_self_claims(self):
        d=self.check()
        self.assertEqual(d["result"],"BLOCK")
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertTrue(d["naive_only_self_claim_gate_would_promote"])
        self.assertEqual(d["trusted_external_gate_admissions"],0)
    def test_07_partial_claim_still_blocked(self):
        d=self.check(e12.asserted_claim(head(),all_met=False))
        self.assertFalse(d["naive_only_self_claim_gate_would_promote"])
        self.assertEqual(d["result"],"BLOCK")
    def test_08_wrong_checkout_head_rejected(self):
        with self.assertRaises(Exception):e12.audit(ROOT,"0"*40)
    def test_09_duplicate_json_field_rejected(self):
        raw=encode(self.claim)
        bad=raw.replace(b'"project":',b'"project":"fake","project":',1)
        with self.assertRaises(Exception):
            e12.intake_untrusted_json(ROOT,head(),bad)
    def test_10_wrong_source_claim_rejected(self):
        c=copy.deepcopy(self.claim);c["source_head"]="f"*40
        with self.assertRaisesRegex(e12.Blocked,"WRONG_BINDING"):self.check(c)
    def test_11_missing_one_gate_rejected(self):
        c=copy.deepcopy(self.claim);c["claimed_native_gate_states"].pop()
        with self.assertRaisesRegex(e12.Blocked,"MISSING_DUPLICATE"):self.check(c)
    def test_12_duplicate_gate_id_rejected(self):
        c=copy.deepcopy(self.claim)
        c["claimed_native_gate_states"][1]["id"]=e12.GATES[0]
        with self.assertRaisesRegex(e12.Blocked,"MISSING_DUPLICATE"):self.check(c)
    def test_13_unknown_evidence_gate_rejected(self):
        c=copy.deepcopy(self.claim)
        c["claimed_native_gate_states"][0]["id"]="non_nordoi_gate"
        with self.assertRaisesRegex(e12.Blocked,"MISSING_DUPLICATE"):self.check(c)
    def test_14_nonboolean_authority_claim_rejected(self):
        c=copy.deepcopy(self.claim);c["claimed_quorum_verified"]=1
        with self.assertRaisesRegex(e12.Blocked,"MALFORMED_SELF_ASSERTED_BOOL"):
            self.check(c)
    def test_15_fake_receipt_digest_invalid(self):
        c=copy.deepcopy(self.claim);c["local_same_admin_receipt_sha256"]="short"
        with self.assertRaisesRegex(e12.Blocked,"RECEIPT_HASH_INVALID"):self.check(c)
    def test_16_even_machine_crypto_valid_bool_is_not_authority(self):
        c=e12.asserted_claim(head(),local_receipt_sha="a"*64)
        d=self.check(c,crypto=True)
        self.assertTrue(d["actual_local_sshsig_verified"])
        self.assertEqual(d["authenticated_external_witness_receipts"],0)
        self.assertEqual(d["native_admission_gates_unmet"],12)
    def test_17_caller_cannot_override_denial(self):
        d=e12.deny("LOCAL",result="ALLOW",production="AUTHORIZED",
                   human_testers=1,native_admission_gates_unmet=0)
        self.assertEqual(d["result"],"BLOCK")
        self.assertEqual(d["production"],"NOT_AUTHORIZED")
        self.assertEqual(d["human_testers"],0)
        self.assertEqual(d["native_admission_gates_unmet"],12)
    def test_18_ai_only_no_founder_manual_tests(self):
        c=e12.e3.strictly_load(e12.e11.e10.source(ROOT,e12.CONTRACT))
        self.assertFalse(c["human_testers_allowed"])
        self.assertEqual(c["founder_role"],"DIRECTOR_NO_MANUAL_TEST_DUTY")
        self.assertEqual(c["execution"],"AI_AND_AUTOMATION_ONLY")
        self.assertEqual(e12.audit(ROOT,head())["r6g"],"LOCKED_BY_SEQUENCE")

@unittest.skipUnless(e12.e6.r4.is_posix_ssh(),
                     "genuine POSIX OpenSSH Ed25519 required; no simulated signatures")
class E12GenuineOSSignature(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp=tempfile.TemporaryDirectory(prefix="nordoi-e12-true-ssh-")
        cls.home=Path(cls.temp.name)
        os.chmod(cls.home,0o700)
        cls.fixture=e12.e6.local_crypto_fixture(cls.home,head())
    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()
    def test_19_true_openssh_signature_files_present(self):
        d=self.fixture
        for k in ("key","allowed","public","receipt","signature"):
            self.assertTrue(d[k].is_file(),k)
        self.assertEqual(len(d["nonce"]),64)
    def test_20_true_second_process_checks_ed25519(self):
        d=e12.e6.check_second_process(self.fixture)
        self.assertTrue(d["cryptographic_signature_valid"])
        self.assertEqual(d["result"],"BLOCK")
    def test_21_genuine_local_signature_and_twelve_self_claims_denied(self):
        raw=e12.asserted_claim(head(),local_receipt_sha=hashlib.sha256(
            self.fixture["raw"]).hexdigest())
        d=e12.intake_untrusted_json(ROOT,head(),encode(raw),
                                    local_crypto_valid=True)
        self.assertTrue(d["naive_only_self_claim_gate_would_promote"])
        self.assertEqual(d["trusted_external_gate_admissions"],0)
    def test_22_replay_locally_valid_still_no_outside_domain(self):
        a=e12.e6.check_second_process(self.fixture)
        b=e12.e6.check_second_process(self.fixture)
        self.assertEqual(a["result"],"BLOCK")
        self.assertEqual(b["result"],"BLOCK")
        self.assertTrue(a["cryptographic_signature_valid"])
        self.assertTrue(b["cryptographic_signature_valid"])
    def test_23_real_wrong_namespace_signature_not_external_authority(self):
        sig=e12.e6.signer.sign_local(
            self.fixture["receipt"],self.fixture["key"],
            namespace="nordoi-e12-adversarial-wrong-namespace")
        checked=e12.e6.check_second_process(
            self.fixture,**{"--signature":str(sig)})
        self.assertEqual(checked["result"],"BLOCK")
        self.assertEqual(checked["authenticated_external_witness_receipts"],0)
    def test_24_full_true_posix_machine_evidence_intake(self):
        d=e12.exercise(ROOT,head())
        self.assertEqual(d["true_posix_local_ed25519_signatures_verified"],2)
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertFalse(d["human_testing_performed"])
        self.assertFalse(d["native_effect_performed"])
        self.assertEqual(d["result"],"BLOCK")

if __name__=="__main__":unittest.main()
