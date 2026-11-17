"""E6: 24 actual tests; 18 source/packet cases and six true POSIX SSHSIG cases.

The portable packets are explicitly UNTRUSTED test data, not fake external
signatures. POSIX cases use real local OpenSSH Ed25519 and OS processes.
No humans test NORDOI, no independently authorized custodian exists.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e6_signed_receipt_rebinding as e6
import g03_r5x_signed_external_provenance as signer

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,
                                   text=True).strip()

def specimen():
    return e6.make(head(),"a"*64,"b"*64,"c"*64,"d"*64)

class E6SourceAndCanonicalNegative(unittest.TestCase):
    def test_01_exact_source_bound_e5_pr_predecessor(self):
        d=e6.audit(ROOT,head())
        self.assertEqual(d["reason"],
                         "E6_LOCAL_RECEIPT_CRYPTO_PROVENANCE_CONTRACT_UNMET")
        self.assertEqual(d["signed_source_head"],head())
    def test_02_e5_specialized_and_general_all_green(self):
        p=e6.e3.strictly_load(e6.e3.real_file_bytes(ROOT/e6.CONTRACT))
        self.assertEqual(p["predecessor_e5_cis"]["specialized_success"],4)
        self.assertEqual(p["predecessor_e5_cis"]["general_success"],5)
    def test_03_thirteen_receipt_fields_are_exact(self):
        p=e6.parse(specimen(),head())
        self.assertEqual(p.keys(),e6.FIELDS)
    def test_04_real_source_head_and_protocol_canonical(self):
        d=e6.parse(specimen(),head())
        self.assertEqual(d["source_head"],head())
        self.assertEqual(d["protocol"],e6.PROTOCOL)
    def test_05_claims_cannot_become_external_witness_authority(self):
        p=e6.parse(specimen(),head())
        p["claimed_external_custody"]=True
        d=e6.parse(e6.canonical(p),head())
        self.assertTrue(d["claimed_external_custody"])
        self.assertEqual(e6.refuse("NO_AUTHORITY")["native_admission_gates_unmet"],12)
        self.assertEqual(e6.refuse("NO_AUTHORITY")["authenticated_external_witness_receipts"],0)
    def test_06_false_owner_oob_and_genuine_witness_still_missing(self):
        p=e6.e3.strictly_load(e6.e3.real_file_bytes(ROOT/e6.CONTRACT))
        self.assertEqual(p["owner_oob_root"],"NOT_AUTHENTICATED")
        self.assertFalse(p["real_external_signer_authenticated"])
    def test_07_no_human_and_no_operator_contact(self):
        p=e6.e3.strictly_load(e6.e3.real_file_bytes(ROOT/e6.CONTRACT))
        self.assertFalse(p["human_testers_allowed"])
        self.assertFalse(p["operator_contacted"])
    def test_08_local_time_window_not_too_long(self):
        p=e6.parse(specimen(),head());p["expires_at"]=1800
        with self.assertRaisesRegex(e6.Blocked,"TIMEBOX"):
            e6.parse(e6.canonical(p),head())
    def test_09_true_duplicate_json_keys_refused(self):
        bad=specimen().replace(b'"project":',b'"project":"wrong","project":',1)
        with self.assertRaisesRegex(e6.Blocked,"DUPLICATE"):
            e6.parse(bad,head())
    def test_10_bad_noncanonical_pretty_json_refused(self):
        pretty=json.dumps(e6.parse(specimen(),head()),indent=2).encode()
        with self.assertRaisesRegex(e6.Blocked,"NONCANONICAL"):
            e6.parse(pretty,head())
    def test_11_nonfinite_epoch_refused(self):
        bad=specimen().replace(b'"key_epoch":1',b'"key_epoch":NaN')
        with self.assertRaisesRegex(e6.Blocked,"NONFINITE"):
            e6.parse(bad,head())
    def test_12_epoch_bool_is_not_valid_integer(self):
        p=e6.parse(specimen(),head());p["key_epoch"]=True
        with self.assertRaisesRegex(e6.Blocked,"BAD_LOCAL_NUMBER"):
            e6.parse(e6.canonical(p),head())
    def test_13_extra_positive_native_grant_field_refused(self):
        p=e6.parse(specimen(),head());p["open_r6g"]=True
        with self.assertRaisesRegex(e6.Blocked,"WRONG_FIELDS"):
            e6.parse(e6.canonical(p),head())
    def test_14_short_sha256_fields_refused(self):
        p=e6.parse(specimen(),head());p["candidate_public_sha256"]="d"*63
        with self.assertRaisesRegex(e6.Blocked,"INVALID_SHA256"):
            e6.parse(e6.canonical(p),head())
    def test_15_replay_packet_other_source_head_refused(self):
        with self.assertRaisesRegex(e6.Blocked,"WRONG_SOURCE"):
            e6.parse(specimen(),"f"*40)
    def test_16_refusal_is_never_positive_external_admission(self):
        d=e6.refuse("RESEARCH")
        self.assertEqual(d["result"],"BLOCK")
        self.assertFalse(d["independently_authenticated_guardian"])
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(d["production"],"NOT_AUTHORIZED")
    def test_17_all_six_receipt_proof_boundaries_not_promoted(self):
        p=e6.e3.strictly_load(e6.e3.real_file_bytes(ROOT/e6.CONTRACT))
        self.assertEqual(len(p["proof_boundaries"]),6)
        self.assertTrue(all(v["status"]!="VERIFIED_EXTERNAL" for v in p["proof_boundaries"]))
    def test_18_even_signed_high_epoch_claim_is_untrusted(self):
        p=e6.parse(specimen(),head())
        p["key_epoch"]=2**63-1
        p["spent_nonce_floor"]=2**63-1
        self.assertEqual(e6.parse(e6.canonical(p),head())["key_epoch"],2**63-1)
        self.assertFalse(e6.refuse("LOCAL_DATA_ONLY")["independent_monotonic_history"])

@unittest.skipUnless(e6.r4.is_posix_ssh(),
                     "needs real POSIX OpenSSH Ed25519, no simulated signing")
class E6RealLocalEd25519(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(
            prefix="nordoi-r5l-r5q-r5s-r5t-e6-test-")
        self.root=Path(self.tmp.name);os.chmod(self.root,0o700)
        self.d=e6.local_crypto_fixture(self.root,head())
    def tearDown(self):self.tmp.cleanup()
    def verify(self,**changes):
        d=dict(self.d)
        d.update(changes)
        return e6.verify_local(d["raw"],d["signature"],d["allowed"],d["public"],
                        head(),d["manifest"],d["previous"],d["nonce"],1200)
    def test_19_real_local_ed25519_sshsig_crypto_valid_but_no_custody(self):
        d=self.verify()
        self.assertEqual(d["reason"],
                         "E6_REAL_LOCAL_SSHSIG_VALID_NO_INDEPENDENT_WITNESS")
        self.assertTrue(d["cryptographic_signature_valid"])
        self.assertFalse(d["independently_authenticated_guardian"])
    def test_20_signed_receipt_wrong_real_ssh_namespace_rejected(self):
        p=self.root/"wrong_context.json";p.write_bytes(self.d["raw"])
        sig=signer.sign_local(p,self.d["key"],
                 namespace="nordoi-r6f-e6-different-real-namespace")
        d=self.verify(signature=sig)
        self.assertEqual(d["reason"],
                         "E6_INVALID_OR_CONTEXT_MISMATCHED_RECEIPT_DENIED")
    def test_21_actual_signature_rejects_modified_epoch_bytes(self):
        raw=self.d["raw"].replace(b'"key_epoch":1',b'"key_epoch":2')
        d=self.verify(raw=raw)
        self.assertEqual(d["reason"],
                         "E6_INVALID_OR_CONTEXT_MISMATCHED_RECEIPT_DENIED")
    def test_22_same_local_key_resigns_different_actual_manifest(self):
        different=e6.digest(os.urandom(37))
        raw=e6.make(head(),different,self.d["previous"],self.d["nonce"],
                     e6.digest(self.d["public"].read_bytes()))
        p=self.root/"rebound.json";p.write_bytes(raw);os.chmod(p,0o600)
        sig=signer.sign_local(p,self.d["key"],namespace=e6.NAMESPACE)
        d=self.verify(raw=raw,signature=sig,manifest=different)
        self.assertTrue(d["cryptographic_signature_valid"])
        self.assertFalse(d["independently_authenticated_guardian"])
    def test_23_local_nonce_replay_set_refuses_but_new_process_without_it_accepts(self):
        old=e6.verify_local(self.d["raw"],self.d["signature"],
              self.d["allowed"],self.d["public"],head(),
              self.d["manifest"],self.d["previous"],self.d["nonce"],
              1200,spent_local=frozenset([self.d["nonce"]]))
        self.assertEqual(old["reason"],"E6_LOCAL_SPENT_NONCE_REPLAY_DETECTED")
        replay=e6.check_second_process(self.d)
        self.assertTrue(replay["cryptographic_signature_valid"])
        self.assertFalse(replay["independently_authenticated_guardian"])
    def test_24_real_second_os_process_plus_real_r5q_fails_external_promotion(self):
        d=e6.exercise(ROOT,head())
        self.assertEqual(d["reason"],
          "E6_TWO_REAL_LOCAL_REBOUND_RECEIPTS_AND_CROSS_PROCESS_REPLAY_DENIED")
        self.assertEqual(d["real_signed_local_receipts"],2)
        self.assertTrue(d["wrong_signature_namespace_refused"])
        self.assertTrue(d["original_r5q_three_file_rollback_reproduced"])
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["human_testers"],0)

if __name__=="__main__":unittest.main()
