"""G0.3 R5U: 24 negative real SQLite/OpenSSH key custody tests.

16 portable real SQLite+canonical cryptographic-intent tests; eight POSIX
genuine OpenSSH Ed25519 two-key signatures, revocation and second-process
rollback. All successful local evidence remains BLOCK, never admission.
"""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import json
import sqlite3

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5l_crash_restart_denial_journal as denial
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5q_coherent_local_rollback as history
import g03_r5s_challenge_bound_handoff as handoff
import g03_r5t_signed_receipt_provenance as signed
import g03_r5u_key_rotation_custody as lab

class PortableLocalCustody(unittest.TestCase):
    def setUp(self):
        self.t=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-r5u-portable-")
        self.root=Path(self.t.name)
        if os.name!="nt":os.chmod(self.root,0o700)
        self.initial="a"*64
        self.state=lab.bootstrap(self.root,self.initial)
        self.payload=lab.intent(lab.ROTATE,1,self.initial,"b"*64,"c"*64,"d"*64)
    def tearDown(self):self.t.cleanup()
    def refuse(self,payload):
        with self.assertRaises(lab.Refused):lab.parse_intent(payload)
    def test_01_actual_sqlite_local_genesis_real_and_deny_only(self):
        self.assertTrue((self.root/lab.LEDGER_NAME).read_bytes().startswith(b"SQLite format 3\x00"))
        self.assertEqual(lab.audit(self.root)["height"],0)
        self.assertEqual(lab.audit(self.root)["active"],self.initial)
        self.assertFalse(lab.audit(self.root)["independent_history_authority"])
    def test_02_real_genesis_is_not_external_qualified_custody(self):
        self.assertEqual(self.state["head"],lab.GENESIS)
        self.assertEqual(self.state["spent_nonces"],[])
        self.assertFalse(self.state["revoked"])
    def test_03_pinned_canonical_rotation_payload_fields(self):
        d=lab.parse_intent(self.payload)
        self.assertEqual(d["parent"],lab.PARENT)
        self.assertEqual(d["operation"],lab.ROTATE)
        self.assertEqual(set(d),set(lab.INTENT_FIELDS))
        self.assertEqual(self.payload,lab.canonical(d))
    def test_04_revocation_intent_must_retain_exact_key_sha(self):
        raw=lab.intent(lab.REVOKE,2,"b"*64,"b"*64,"c"*64,"e"*64)
        self.assertEqual(lab.parse_intent(raw)["next_key_sha256"],"b"*64)
        self.refuse(lab.canonical({**lab.parse_intent(raw),"next_key_sha256":"f"*64}))
    def test_05_reject_grant_native_or_enroll_opcode(self):
        for op in ("GRANT","ENROLL","ROTATE_PRODUCTION_KEY"):
            self.refuse(lab.canonical({**lab.parse_intent(self.payload),"operation":op}))
    def test_06_dual_signature_transition_must_change_key_identity(self):
        self.refuse(lab.canonical({**lab.parse_intent(self.payload),
                                 "next_key_sha256":self.initial}))
    def test_07_nonce_receipt_and_key_pins_must_be_sha256_lowerhex(self):
        for field in ("prior_key_sha256","next_key_sha256","receipt_sha256","nonce"):
            self.refuse(lab.canonical({**lab.parse_intent(self.payload),field:"BAD"}))
    def test_08_invalid_bool_negative_or_zero_epoch_quarantined(self):
        for value in (True,False,-1,0,"1"):
            self.refuse(lab.canonical({**lab.parse_intent(self.payload),"epoch":value}))
    def test_09_mismatched_project_protocol_or_sha_lineage_refused(self):
        for field,value in (("project","bad"),("protocol","bad"),
                            ("parent","0"*40)):
            self.refuse(lab.canonical({**lab.parse_intent(self.payload),field:value}))
    def test_10_unknown_and_missing_fields_disqualify_proof(self):
        self.refuse(lab.canonical({**lab.parse_intent(self.payload),"external_grant":True}))
        x=lab.parse_intent(self.payload);del x["nonce"]
        self.refuse(lab.canonical(x))
    def test_11_duplicate_noncanonical_or_invalid_utf8_disallowed(self):
        self.refuse(self.payload.replace(b'"project":',b'"project":"fake","project":'))
        self.refuse(json.dumps(lab.parse_intent(self.payload),indent=2).encode())
        self.refuse(b"\xff")
    def test_12_missing_sqlite_ledger_refused_without_regenesis(self):
        p=self.root/lab.LEDGER_NAME
        p.unlink()
        with self.assertRaises(lab.Refused):lab.audit(self.root)
        self.assertFalse(p.exists())
    def test_13_corrupted_sqlite_genesis_is_quarantined(self):
        (self.root/lab.LEDGER_NAME).write_bytes(b"bad SQLite")
        with self.assertRaises(lab.Refused):lab.audit(self.root)
    def test_14_genesis_invalid_hash_or_truncated_event_chain_refused(self):
        # This changes real SQLite, not a fabricated status JSON.
        # sqlite3.Connection.__exit__ commits, but does not close the
        # handle on Windows; the real negative test must release it before
        # TemporaryDirectory cleanup without weakening the audit assertion.
        from contextlib import closing
        with closing(sqlite3.connect(str(self.root/lab.LEDGER_NAME))) as db:
            db.execute("UPDATE meta SET initial='NOT_A_SHA256'")
            db.commit()
        with self.assertRaises(lab.Refused):lab.audit(self.root)
    def test_15_no_unverified_key_signature_can_write_a_rotation(self):
        r=lab.append(self.root,self.payload,self.root/"missing.sig",self.root/"missing.allowed")
        self.assertEqual(r["reason"],"LOCAL_KEY_CHANGE_QUARANTINED")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        self.assertEqual(lab.audit(self.root)["height"],0)
    def test_16_real_new_python_process_audits_local_sqlite_only(self):
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--audit",
             "--ledger-dir",str(self.root)]
        p=subprocess.run(cmd,text=True,capture_output=True,timeout=35)
        self.assertEqual(p.returncode,2,p.stderr)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],"LOCAL_KEY_LEDGER_AUDITED_NOT_EXTERNAL")
        self.assertFalse(x["native_authority_granted"])
        self.assertEqual(x["independent_custodian"],"NOT_ENROLLED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "genuine POSIX OpenSSH local two-key transition and live SQLite rollback")
class NativeRealKeyCustody(unittest.TestCase):
    def setUp(self):
        self.t=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-r5u-native-")
        self.root=Path(self.t.name);os.chmod(self.root,0o700)
        a,b,j=history.fixture(self.root)
        self.zip=self.root/"handoff.zip"
        handoff.prepare(a,b,j,"a"*64,1000,1200,self.zip)
        self.receipt=self.root/"receipt.json"
        self.receipt.write_bytes(signed.prepare_receipt(self.zip,"a"*64,1100))
        os.chmod(self.receipt,0o600)
        self.old,self.old_allowed=signed.create_same_admin_key(self.root,"old_key")
        self.new,self.new_allowed=signed.create_same_admin_key(self.root,"new_key")
        self.old_pub=self.root/"old_key.pub";self.new_pub=self.root/"new_key.pub"
        self.old_pin=lab.public_sha(self.old_pub)
        self.new_pin=lab.public_sha(self.new_pub)
        self.old_sig=signed.sign_local(self.receipt,self.old)
        lab.bootstrap(self.root,self.old_pin)
        self.old_snapshot=self.root/"genesis.sqlite3"
        shutil.copy2(lab.ledger_path(self.root),self.old_snapshot)
        self.packet=self.root/"rotate.json"
        self.payload=lab.intent(lab.ROTATE,1,self.old_pin,self.new_pin,
                                lab.sha(self.receipt.read_bytes()),"1"*64)
        self.packet.write_bytes(self.payload)
        self.old_transition_sig=lab.sign_intent(self.packet,self.old)
        other=self.root/"second_approval.json";other.write_bytes(self.payload)
        self.next_transition_sig=lab.sign_intent(other,self.new)
    def tearDown(self):self.t.cleanup()
    def rotate(self):
        r=lab.append(self.root,self.payload,self.old_transition_sig,self.old_allowed,
                     self.next_transition_sig,self.new_allowed)
        self.assertEqual(r["reason"],"REAL_LOCAL_DUAL_SSHSIG_ROTATION_DENIED",r)
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        return r
    def next_receipt(self):
        p=self.root/"next_receipt.json"
        p.write_bytes(self.receipt.read_bytes());os.chmod(p,0o600)
        return p,signed.sign_local(p,self.new)
    def revoke(self):
        packet=self.root/"revoke.json"
        data=lab.intent(lab.REVOKE,2,self.new_pin,self.new_pin,
                        lab.sha(self.receipt.read_bytes()),"2"*64)
        packet.write_bytes(data)
        sig=lab.sign_intent(packet,self.new)
        r=lab.append(self.root,data,sig,self.new_allowed)
        self.assertEqual(r["reason"],"REAL_LOCAL_SIGNED_REVOCATION_DENIED",r)
        self.assertTrue(r["locally_revoked"])
        return r
    def test_17_actual_distinct_ed25519_prior_and_successor_both_sign(self):
        self.assertNotEqual(self.old_pin,self.new_pin)
        self.assertTrue(lab.verify_local(self.payload,self.old_transition_sig,self.old_allowed))
        self.assertTrue(lab.verify_local(self.payload,self.next_transition_sig,self.new_allowed))
        self.assertEqual(self.rotate()["local_height"],1)
    def test_18_only_old_signature_cannot_rotate(self):
        r=lab.append(self.root,self.payload,self.old_transition_sig,self.old_allowed)
        self.assertIn("TWO_GENUINE_SIGNATURES_REQUIRED",r["detail"])
        self.assertEqual(lab.audit(self.root)["height"],0)
    def test_19_wrong_signer_signature_or_allowlist_disqualifies(self):
        r=lab.append(self.root,self.payload,self.old_transition_sig,self.new_allowed,
                     self.next_transition_sig,self.new_allowed)
        self.assertIn("REAL_LOCAL_SSHSIG_INVALID_OR_WRONG_PIN",r["detail"])
        self.assertEqual(lab.audit(self.root)["height"],0)
    def test_20_old_key_not_active_after_real_signed_rotation(self):
        self.rotate()
        r=lab.active_key_check(self.root,self.old_pub,self.receipt,
                               self.old_sig,self.old_allowed)
        self.assertIn("LOCAL_SIGNER_NOT_ACTIVE",r["detail"])
        new_receipt,new_sig=self.next_receipt()
        r2=lab.active_key_check(self.root,self.new_pub,new_receipt,new_sig,
                                self.new_allowed)
        self.assertEqual(r2["reason"],"ACTIVE_LOCAL_KEY_VERIFIED_NOT_INDEPENDENT")
    def test_21_signed_successor_revocation_quarantines_its_local_use(self):
        self.rotate()
        r=self.revoke()
        self.assertEqual(r["local_height"],2)
        self.assertTrue(lab.audit(self.root)["revoked"])
        next_receipt,next_sig=self.next_receipt()
        check=lab.active_key_check(self.root,self.new_pub,next_receipt,next_sig,
                                   self.new_allowed)
        self.assertIn("LOCAL_SIGNER_REVOKED",check["detail"])
    def test_22_actual_second_process_rolls_back_signed_revocation(self):
        self.rotate()
        stable=self.root/"rotated_before_revoke.sqlite3"
        shutil.copy2(lab.ledger_path(self.root),stable)
        self.revoke()
        fdgate.real_atomic_replace(stable,lab.ledger_path(self.root))
        self.assertFalse(lab.audit(self.root)["revoked"])
        next_receipt,next_sig=self.next_receipt()
        x=lab.active_key_check(self.root,self.new_pub,next_receipt,next_sig,
                               self.new_allowed)
        self.assertEqual(x["reason"],"ACTIVE_LOCAL_KEY_VERIFIED_NOT_INDEPENDENT")
    def test_23_full_old_sqlite_key_lineage_restore_accepts_old_nonce_only_as_denial(self):
        self.rotate();self.revoke()
        fdgate.real_atomic_replace(self.old_snapshot,lab.ledger_path(self.root))
        self.assertEqual(lab.audit(self.root)["height"],0)
        result=lab.active_key_check(self.root,self.old_pub,self.receipt,
                                    self.old_sig,self.old_allowed)
        self.assertEqual(result["reason"],"ACTIVE_LOCAL_KEY_VERIFIED_NOT_INDEPENDENT")
        repeat=self.rotate()
        self.assertEqual(repeat["local_height"],1)
        self.assertEqual(repeat["admission"],"NOT_AUTHORIZED")
    def test_24_full_real_native_two_key_revoke_and_rollback_cli_exit_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),"--exercise"],
                         capture_output=True,text=True,timeout=180)
        self.assertEqual(p.returncode,2,p.stderr)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],"REAL_KEY_ROTATION_REVOCATION_CUSTODY_LIMITS_QUALIFIED")
        self.assertEqual(x["genuine_distinct_old_new_ed25519_signatures"],
                         "PASS_KEY_POSSESSION_ONLY")
        self.assertEqual(x["one_signature_rotation"],"QUARANTINED")
        self.assertEqual(x["rotated_out_old_key"],"LOCAL_QUARANTINED")
        self.assertEqual(x["signed_successor_revocation"],"LOCAL_QUARANTINED")
        self.assertEqual(x["revocation_after_real_sqlite_rollback"],"LOST_LOCALLY")
        self.assertEqual(x["old_key_after_full_sqlite_rollback"],
                         "LOCALLY_VALID_AGAIN_DENIAL_ONLY")
        self.assertEqual(x["old_rotation_nonce_after_full_rollback"],"LOCAL_DENIAL_AGAIN")
        self.assertEqual(x["independent_admin_domain"],"NOT_PROVEN")
        self.assertEqual(x["native_admission"],"BLOCK")
        self.assertFalse(x["native_authority_granted"])

if __name__=="__main__":unittest.main()
