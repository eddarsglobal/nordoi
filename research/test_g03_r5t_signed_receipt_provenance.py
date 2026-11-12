"""G0.3 R5T: 24 genuine Ed25519 receipt/key provenance/anti-replay controls.

16 portable tests on actual R5S SQLite/JSON handoff files and strict contract.
8 genuine POSIX/OpenSSH tests; Windows intentionally SKIPS the native eight.
All results remain BLOCK/NOT_AUTHORIZED, even authentic local key possession.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5l_crash_restart_denial_journal as denial
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5q_coherent_local_rollback as history
import g03_r5s_challenge_bound_handoff as handoff
import g03_r5t_signed_receipt_provenance as lab

class PortableBoundReceipt(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-portable-")
        self.root=Path(self.tmp.name)
        if os.name!="nt":os.chmod(self.root,0o700)
        self.a,self.b,self.j=history.fixture(self.root)
        self.bundle=self.root/"handoff.zip"
        handoff.prepare(self.a,self.b,self.j,"a"*64,1000,1200,self.bundle)
        self.payload=lab.prepare_receipt(self.bundle,"a"*64,1100)
        self.receipt=self.root/"receipt.json"
        self.receipt.write_bytes(self.payload)
        if os.name!="nt":os.chmod(self.receipt,0o600)
    def tearDown(self):self.tmp.cleanup()
    def updated(self,**kwargs):
        x=lab.parse(self.payload);x.update(kwargs);return lab.canonical(x)
    def reject(self,payload):
        with self.assertRaises(lab.Refused):lab.parse(payload)
    def test_01_real_r5s_sqlite_json_transfer_bound_to_local_receipt(self):
        obj=lab.parse(self.payload)
        m,zip_digest,state=handoff.decode_package(self.bundle.read_bytes(),"a"*64,1100)
        self.assertEqual(obj["archive_sha256"],zip_digest)
        self.assertEqual(obj["manifest_sha256"],lab.sha(handoff.canonical(m)))
        self.assertEqual(obj["projection_sha256"],m["projection_sha256"])
        self.assertEqual(obj["local_primary_height"],state["primary_height"])
    def test_02_exact_frozen_r5s_parent_and_deny_only_operation(self):
        obj=lab.parse(self.payload)
        self.assertEqual(obj["parent"],lab.PARENT)
        self.assertEqual(obj["operation"],"SIGNED_LOCAL_RECEIPT_BLOCK_ONLY")
        self.assertEqual(lab.canonical(obj),self.payload)
    def test_03_real_handoff_challenge_mismatch_disqualifies_receipt(self):
        with self.assertRaises(handoff.Refused):
            lab.prepare_receipt(self.bundle,"b"*64,1100)
    def test_04_real_handoff_expired_local_timebox_disqualifies_receipt(self):
        with self.assertRaises(handoff.Refused):
            lab.prepare_receipt(self.bundle,"a"*64,1300)
    def test_05_duplicate_json_receipt_key_is_refused(self):
        self.reject(self.payload.replace(b'"project":',b'"project":"fake","project":'))
    def test_06_noncanonical_whitespace_or_unknown_receipt_key_refused(self):
        self.reject(json.dumps(lab.parse(self.payload),indent=2).encode())
        self.reject(self.updated(invented_authority=True))
    def test_07_grant_or_enroll_operation_and_wrong_frozen_parent_refused(self):
        for value in ("GRANT","ENROLL","APPROVE_NATIVE_ACCESS"):
            self.reject(self.updated(operation=value))
        self.reject(self.updated(parent="0"*40))
    def test_08_false_signer_identity_or_namespace_refused(self):
        for key,value in (("signer","foreign-guardian"),
                          ("namespace","unrelated-namespace"),
                          ("protocol","other")):
            self.reject(self.updated(**{key:value}))
    def test_09_invalid_receipt_sha256_challenge_or_projection_refused(self):
        for field in ("challenge","archive_sha256",
                      "projection_sha256","manifest_sha256"):
            self.reject(self.updated(**{field:"not-sha"}))
    def test_10_negative_boolean_or_noninteger_time_and_height_refused(self):
        for field,value in (("received_at",True),("expires_at",-1),
                            ("local_primary_height",0),
                            ("local_primary_height","1")):
            self.reject(self.updated(**{field:value}))
    def test_11_receipt_local_time_window_must_not_go_backwards(self):
        self.reject(self.updated(expires_at=1000))
    def test_12_truncated_invalid_or_oversized_receipt_refused(self):
        for value in (b"{}",b"\xff",b"x"*(lab.MAX_RECEIPT_BYTES+1)):
            self.reject(value)
    def test_13_original_r5s_zip_stays_self_consistent_after_new_real_history(self):
        history.advance_world(self.a,self.b,self.j)
        old=lab.prepare_receipt(self.bundle,"a"*64,1100)
        self.assertEqual(old,self.payload)
        self.assertEqual(history.projection(self.a,self.b,self.j)["primary_height"],2)
        self.assertEqual(lab.parse(old)["local_primary_height"],1)
    def test_14_actual_bundle_bytes_corrupted_cannot_make_receipt(self):
        self.bundle.write_bytes(b"not ZIP")
        with self.assertRaises(handoff.Refused):
            lab.prepare_receipt(self.bundle,"a"*64,1100)
    def test_15_missing_signature_or_external_key_does_not_authorize(self):
        r=lab.inspect(self.receipt,self.root/"missing.sig",
                      self.root/"missing.allow",self.bundle,"a"*64,1100)
        self.assertEqual(r["reason"],"SIGNED_RECEIPT_QUARANTINED")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
    def test_16_real_separate_python_cli_does_not_fake_signature_success(self):
        args=[sys.executable,"-B",str(Path(lab.__file__)),"--inspect",
              "--receipt",str(self.receipt),
              "--signature",str(self.root/"missing.sig"),
              "--allowed-signers",str(self.root/"missing.allow"),
              "--bundle",str(self.bundle),"--challenge","a"*64,"--now","1100"]
        p=subprocess.run(args,capture_output=True,text=True,timeout=45)
        self.assertEqual(p.returncode,2,p.stderr)
        r=json.loads(p.stdout)
        self.assertEqual(r["reason"],"SIGNED_RECEIPT_QUARANTINED")
        self.assertEqual(r["independent_custodian"],"NOT_ENROLLED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "genuine POSIX OpenSSH Ed25519 receipt signing and rekey/rollback lab")
class NativeSignedReceipt(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-native-")
        self.root=Path(self.tmp.name);os.chmod(self.root,0o700)
        self.a,self.b,self.j=history.fixture(self.root)
        self.bundle=self.root/"handoff.zip"
        handoff.prepare(self.a,self.b,self.j,"a"*64,1000,1200,self.bundle)
        self.receipt=self.root/"receipt.json"
        self.receipt.write_bytes(lab.prepare_receipt(self.bundle,"a"*64,1100))
        os.chmod(self.receipt,0o600)
        self.key,self.allowed=lab.create_same_admin_key(self.root)
        self.sig=lab.sign_local(self.receipt,self.key)
    def tearDown(self):self.tmp.cleanup()
    def check(self,**kwargs):
        r=lab.inspect(self.receipt,self.sig,self.allowed,self.bundle,
                      "a"*64,1100,**kwargs)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        return r
    def test_17_real_open_ssh_ed25519_sign_verify_cannot_enroll_guardian(self):
        r=self.check()
        self.assertEqual(r["reason"],"REAL_LOCAL_ED25519_SIGNATURE_VALID_NO_EXTERNAL_AUTHORITY")
        self.assertTrue(r["cryptographic_signature_verified"])
        self.assertFalse(r["independently_authenticated_signer"])
        self.assertEqual(r["independent_custodian"],"NOT_ENROLLED")
    def test_18_tampered_genuine_signed_receipt_fails_signature(self):
        self.receipt.write_bytes(self.receipt.read_bytes().replace(
            b'"local_primary_height":1',b'"local_primary_height":2'))
        r=self.check()
        self.assertEqual(r["reason"],"SIGNED_RECEIPT_QUARANTINED")
        self.assertIn("RECEIPT_SIGNED_CONTEXT_NOT_BOUND_TO_TRANSFER",r["detail"])
    def test_19_real_wrong_local_key_is_not_signature_authority(self):
        _,other_allowed=lab.create_same_admin_key(self.root,"other_signer")
        r=lab.inspect(self.receipt,self.sig,other_allowed,self.bundle,"a"*64,1100)
        self.assertIn("REAL_SSHSIG_SIGNATURE_INVALID",r["detail"])
        self.assertEqual(r["reason"],"SIGNED_RECEIPT_QUARANTINED")
    def test_20_actual_sqlite_receipt_ledger_blocks_same_nonce_replay(self):
        ledger=self.root/"receipt_denials";ledger.mkdir(mode=0o700)
        with denial.DenialJournal(ledger) as db:self.assertEqual(db.audit()["local_height"],0)
        first=self.check(ledger_dir=ledger)
        self.assertEqual(first["local_denial_height"],1)
        again=self.check(ledger_dir=ledger)
        self.assertIn("ALREADY_RECORDED_DENIAL_NONCE",again["detail"])
    def test_21_real_second_process_restore_of_nonce_ledger_reallows_local_denial(self):
        ledger=self.root/"receipt_denials";ledger.mkdir(mode=0o700)
        with denial.DenialJournal(ledger) as db:self.assertEqual(db.audit()["local_height"],0)
        saved=self.root/"saved_genesis.sqlite3"
        shutil.copy2(ledger/denial.NAME,saved)
        first=lab.second_process_check(self.receipt,self.sig,self.allowed,
                                        self.bundle,"a"*64,1100,ledger)
        self.assertEqual(first["local_denial_height"],1)
        fdgate.real_atomic_replace(saved,ledger/denial.NAME)
        second=lab.second_process_check(self.receipt,self.sig,self.allowed,
                                         self.bundle,"a"*64,1100,ledger)
        self.assertEqual(second["local_denial_height"],1)
        self.assertFalse(second["externally_protected_nonce"])
    def test_22_old_signed_handoff_after_new_world_still_valid_locally(self):
        history.advance_world(self.a,self.b,self.j)
        old=lab.second_process_check(self.receipt,self.sig,self.allowed,
                                      self.bundle,"a"*64,1100)
        self.assertTrue(old["cryptographic_signature_verified"])
        self.assertEqual(old["reason"],"REAL_LOCAL_ED25519_SIGNATURE_VALID_NO_EXTERNAL_AUTHORITY")
        self.assertEqual(history.projection(self.a,self.b,self.j)["primary_height"],2)
        self.assertEqual(handoff.decode_package(self.bundle.read_bytes(),"a"*64,1100)[2]["primary_height"],1)
    def test_23_local_key_rotation_can_sign_again_not_create_independence(self):
        key,allowed=lab.create_same_admin_key(self.root,"rotated_key")
        other=self.root/"second_receipt.json";other.write_bytes(self.receipt.read_bytes())
        os.chmod(other,0o600)
        sig=lab.sign_local(other,key)
        r=lab.second_process_check(other,sig,allowed,self.bundle,"a"*64,1100)
        self.assertTrue(r["cryptographic_signature_verified"])
        self.assertFalse(r["independently_authenticated_signer"])
        self.assertEqual(r["independent_custodian"],"NOT_ENROLLED")
    def test_24_real_second_process_atomic_swap_and_full_native_cli_exit_two(self):
        duplicate=self.root/"byte_identical.json"
        shutil.copy2(self.receipt,duplicate);os.chmod(duplicate,0o600)
        r=self.check(after_first_observation=lambda:
                     fdgate.real_atomic_replace(duplicate,self.receipt))
        self.assertIn("OPEN_FD_AND_CURRENT_PATH_DIVERGED",r["detail"])
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--exercise"]
        p=subprocess.run(cmd,capture_output=True,text=True,timeout=180)
        self.assertEqual(p.returncode,2,p.stderr)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],"REAL_SIGNED_RECEIPT_LOCAL_PROVENANCE_LIMITS_QUALIFIED")
        self.assertEqual(x["real_open_ssh_ed25519_sign_and_verify"],
                         "PASS_LOCAL_KEY_POSSESSION_ONLY")
        self.assertEqual(x["spent_signed_receipt_replay_with_intact_ledger"],"QUARANTINED")
        self.assertEqual(x["spent_signed_receipt_after_same_admin_ledger_restore"],
                         "LOCAL_DENIAL_AGAIN")
        self.assertEqual(x["real_receipt_inode_swap"],"QUARANTINED")
        self.assertEqual(x["real_external_signed_guardian_receipt"],"NONE")
        self.assertFalse(x["native_authority_granted"])

if __name__=="__main__":unittest.main()
