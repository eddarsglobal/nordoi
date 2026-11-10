"""G0.3 R5G: 24 controls, genuine separate-process same-UID takeover only on POSIX.

No invented external organization, signatures or owner OOB attestation.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5g_effective_control_lab as gate

def claim(**updates):
    data={"protocol":gate.PROTOCOL,"project":gate.PROJECT,"source_commit":gate.R5F_HEAD,
          "publisher_controller":"eddarsglobal","guardian_controller_claim":"other-admin",
          "witness_controller_claim":"other-witness","recovery_controller_claim":"other-recovery",
          "owner_oob_root_sha256_claim":"a"*64,
          "guardian_receipt_sha256_claim":"b"*64,
          "witness_highwater_sha256_claim":"c"*64,
          "durable_nonce_ledger_sha256_claim":"d"*64,
          "provenance_channel_claim":"CLAIMED_OFFLINE"}
    data.update(updates)
    return gate.canonical(data)

class UntrustedAdministrativeClaims(unittest.TestCase):
    def blocked(self,raw):
        result=gate.assess_claim(raw)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        return result
    def rejected(self,raw):
        self.assertEqual(self.blocked(raw)["reason"],"INVALID_UNTRUSTED_CONTROL_CLAIMS")
    def test_01_all_distinct_claimed_operators_remain_untrusted(self):
        r=self.blocked(claim());self.assertEqual(r["reason"],"NAMES_AND_DIGESTS_CANNOT_AUTHENTICATE_EXTERNAL_CONTROL")
    def test_02_accepted_claims_cannot_enroll_owner_oob_root(self):
        self.assertEqual(self.blocked(claim())["actual_oob_channel"],"NOT_ATTESTED")
    def test_03_accepted_claims_cannot_enroll_external_highwater(self):
        self.assertEqual(self.blocked(claim())["external_protected_highwater"],"NOT_ENROLLED")
    def test_04_same_publisher_guardian_claim_disqualified(self):
        r=self.blocked(claim(guardian_controller_claim="eddarsglobal"));self.assertEqual(r["reason"],"SHARED_CONTROLLER_CLAIM_DISQUALIFIES")
    def test_05_same_guardian_and_witness_claim_disqualified(self):
        r=self.blocked(claim(witness_controller_claim="other-admin"));self.assertEqual(r["reason"],"SHARED_CONTROLLER_CLAIM_DISQUALIFIES")
    def test_06_shared_recovery_controller_claim_disqualified(self):
        r=self.blocked(claim(recovery_controller_claim="other-admin"));self.assertEqual(r["reason"],"SHARED_CONTROLLER_CLAIM_DISQUALIFIES")
    def test_07_fake_offline_channel_does_not_authenticate_root(self):
        self.assertEqual(self.blocked(claim(provenance_channel_claim="CLAIMED_OFFLINE"))["cryptographic_identity"],"NOT_VERIFIED")
    def test_08_publisher_label_does_not_authenticate_root(self):
        self.assertEqual(self.blocked(claim(provenance_channel_claim="DECLARED_BY_PUBLISHER"))["administrator_independence"],"NOT_VERIFIED")
    def test_09_wrong_protocol_denied(self):
        self.rejected(claim(protocol="NORDOI-FAKE"))
    def test_10_wrong_project_denied(self):
        self.rejected(claim(project="unrelated/project"))
    def test_11_wrong_frozen_source_denied(self):
        self.rejected(claim(source_commit="0"*40))
    def test_12_zero_and_invalid_digest_claims_denied(self):
        self.rejected(claim(owner_oob_root_sha256_claim="0"*64))
        self.rejected(claim(durable_nonce_ledger_sha256_claim="bad"))
    def test_13_boolean_or_malformed_controller_denied(self):
        self.rejected(claim(guardian_controller_claim=True))
        self.rejected(claim(recovery_controller_claim="A"*200))
    def test_14_duplicate_keys_refused(self):
        x=claim().replace(b'"project":',b'"project":"duplicate","project":')
        self.rejected(x)
    def test_15_noncanonical_json_invalid_utf8_and_oversized_denied(self):
        self.rejected(json.dumps(json.loads(claim()),indent=2).encode())
        self.rejected(b"\xff")
        self.rejected(b"x"*(gate.MAX_BYTES+1))
    def test_16_no_user_path_accepted_by_public_run_cli(self):
        with tempfile.TemporaryDirectory() as temp:
            cmd=[sys.executable,"-B",str(Path(gate.__file__)),"--run","--root",temp]
            r=subprocess.run(cmd,capture_output=True,text=True,timeout=20)
            self.assertEqual(r.returncode,2)
            self.assertEqual(json.loads(r.stdout)["reason"],"LOCAL_CONTROL_EXPERIMENT_REFUSED")

@unittest.skipIf(os.name=="nt","POSIX mode and actual same-UID chmod control required")
class RealSeparateProcessSameUID(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.result=gate.exercise()
    def test_17_two_actual_sqlite_databases_not_dummy_data(self):
        self.assertTrue(self.result["two_actual_sqlite_databases"])
    def test_18_real_second_process_same_uid_rewrite_confirmed(self):
        self.assertTrue(self.result["separate_real_actor_process"])
        self.assertEqual(self.result["same_uid_control_weakness"],"CONFIRMED_LOCAL_ONLY")
    def test_19_owner_read_only_witness_mode_was_enforced_before_actor(self):
        self.assertEqual(self.result["witness_mode_before_actor"],0o400)
    def test_20_owner_restored_own_write_bit_in_distinct_process(self):
        self.assertEqual(self.result["witness_mode_after_actor"],0o600)
    def test_21_actual_two_store_rollback_previously_height_two(self):
        self.assertEqual(self.result["witness_before_restore_height"],2)
    def test_22_joint_snapshot_restoration_hides_history_even_with_two_processes(self):
        self.assertEqual(self.result["witness_after_restore_height"],1)
        self.assertEqual(self.result["both_store_rollback_detections"],"NO")
    def test_23_no_external_identity_or_owner_oob_root_created(self):
        self.assertEqual(self.result["owner_oob_root"],"NOT_ENROLLED")
        self.assertEqual(self.result["independent_external_operator"],"NOT_ENROLLED")
    def test_24_real_cli_always_exits_two_and_denies_admission(self):
        p=subprocess.run([sys.executable,"-B",str(Path(gate.__file__)),"--run"],capture_output=True,text=True,timeout=35)
        self.assertEqual(p.returncode,2)
        result=json.loads(p.stdout)
        self.assertEqual(result["reason"],"SAME_UID_CAN_RESTORE_READONLY_LOCAL_WITNESS")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertEqual(result["global_monotonic_antirollback"],"NOT_IMPLEMENTED")

if __name__=="__main__": unittest.main()
