"""G0.3 R5H: 24 cryptographic revocation fixture and fail-closed controls.

Native key tests use fresh real OpenSSH Ed25519 keys solely inside a temporary
lab. Any successful fixture signature is NOT real guardian independence.
"""
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5h_signed_revocation_lab as gate

WIRE=b"\x00\x00\x00\x0bssh-ed25519\x00\x00\x00\x20" + b"1"*32
DUMMY_PUB="ssh-ed25519 "+base64.b64encode(WIRE).decode("ascii")
FAKE_SIG=b"-----BEGIN SSH SIGNATURE-----\nfake\n-----END SSH SIGNATURE-----\n"

def payload(**changes):
    x={"protocol":gate.PROTOCOL,"project":gate.PROJECT,"parent_head":gate.PARENT,
       "operation":"REVOKE","sequence":1,"epoch":4,"nonce":"a"*64,
       "prev":gate.GENESIS,"resource_sha256":"b"*64,
       "guardian_pubkey_sha256":hashlib.sha256(DUMMY_PUB.encode("ascii")).hexdigest(),
       "issued_unix":1800000000,"expires_unix":1800000120}
    x.update(changes)
    return x

def packet(pl=None, **changes):
    x={"payload":payload() if pl is None else pl,"guardian_pubkey":DUMMY_PUB,
       "signature_b64":base64.b64encode(FAKE_SIG).decode("ascii")}
    x.update(changes)
    return gate.canonical(x)

class PortableDenialTests(unittest.TestCase):
    def denied(self,raw):
        with self.assertRaises(gate.Denied):gate.decode(raw)
    def test_01_empty_bundle_denied(self):self.denied(b"")
    def test_02_utf8_invalid_denied(self):self.denied(b"\xff")
    def test_03_duplicate_json_key_denied(self):
        self.denied(packet().replace(b'"project":',b'"project":"fake","project":'))
    def test_04_whitespace_noncanonical_denied(self):
        self.denied(json.dumps(json.loads(packet()),indent=2).encode("ascii"))
    def test_05_nonfinite_and_unknown_fields_denied(self):
        self.denied(packet().replace(b'"sequence":1',b'"sequence":NaN'))
        self.denied(gate.canonical({**json.loads(packet()),"extra":"fake"}))
    def test_06_oversize_and_nonbytes_denied(self):
        self.denied(b"x"*(gate.MAX_BYTES+1))
        self.denied("not bytes")
    def test_07_wrong_project_and_protocol_denied(self):
        self.denied(packet(payload(project="other/project")))
        self.denied(packet(payload(protocol="FAKE")))
    def test_08_wrong_parent_sha_denied(self):
        self.denied(packet(payload(parent_head="0"*40)))
    def test_09_unrevoke_and_grant_are_never_authorized(self):
        self.denied(packet(payload(operation="UNREVOKE")))
        self.denied(packet(payload(operation="GRANT")))
    def test_10_bad_epoch_and_bool_denied(self):
        self.denied(packet(payload(epoch=3)))
        self.denied(packet(payload(epoch=True)))
    def test_11_bad_sequence_and_bool_denied(self):
        self.denied(packet(payload(sequence=0)))
        self.denied(packet(payload(sequence=True)))
    def test_12_bad_digest_rejected(self):
        self.denied(packet(payload(nonce="0"*64)))
        self.denied(packet(payload(resource_sha256="INVALID")))
    def test_13_invalid_times_rejected(self):
        self.denied(packet(payload(issued_unix=0)))
        self.denied(packet(payload(expires_unix=False)))
    def test_14_long_negative_or_zero_ttl_rejected(self):
        self.denied(packet(payload(expires_unix=1800000900)))
        self.denied(packet(payload(expires_unix=1800000000)))
    def test_15_wrong_key_and_pin_rejected(self):
        self.denied(packet(guardian_pubkey="ssh-rsa AAAA"))
        self.denied(packet(payload(guardian_pubkey_sha256="c"*64)))
    def test_16_fake_signature_never_authenticates_external_guardian(self):
        r=gate.evaluate(packet(),DUMMY_PUB,1800000060)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertNotIn("signature_verified",r)

@unittest.skipUnless(Path(gate.SSH).is_file() and os.name!="nt", "native OpenSSH SSHSIG fixture only")
class NativeSSHSIGRevocations(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5h-tests-")
        cls.root=Path(cls.temp.name)
        os.chmod(cls.root,0o700)
        cls.private,cls.pub=gate.keygen(cls.root)
        cls.valid=gate.make_packet(cls.private,cls.pub,cls.root)
    @classmethod
    def tearDownClass(cls):cls.temp.cleanup()
    def setUp(self):
        self.tempdb=tempfile.TemporaryDirectory(prefix="nordoi-r5h-ledger-")
        self.dbroot=Path(self.tempdb.name)
        os.chmod(self.dbroot,0o700)
    def tearDown(self):self.tempdb.cleanup()
    def test_17_native_ed25519_sshsig_validates_key_possession_not_identity(self):
        r=gate.evaluate(self.valid,self.pub,1800000060)
        self.assertTrue(r["signature_verified"])
        self.assertEqual(r["guardian_identity"],"NOT_AUTHENTICATED")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
    def test_18_native_signed_revoke_is_real_local_sqlite_commit_but_not_admission(self):
        with gate.RevocationLedger(self.dbroot) as lab:
            r=lab.revoke(self.valid,self.pub,1800000060)
        self.assertEqual(r["status"],"LAB_LOCAL_REVOKE_COMMIT")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
    def test_19_native_local_replay_rejected_and_state_persisted(self):
        with gate.RevocationLedger(self.dbroot) as lab:
            lab.revoke(self.valid,self.pub,1800000060)
        with gate.RevocationLedger(self.dbroot) as lab:
            self.assertTrue(lab.audit()["local_revoked"])
            r=lab.revoke(self.valid,self.pub,1800000060)
        self.assertEqual(r["status"],"BLOCK")
        self.assertIn("ALREADY_REVOKED",r["detail"])
    def test_20_substituting_untrusted_real_public_key_does_not_pass_pin(self):
        with tempfile.TemporaryDirectory(prefix="r5h-substitute-") as d:
            _,other=gate.keygen(Path(d))
            r=gate.evaluate(self.valid,other,1800000060)
        self.assertEqual(r["reason"],"INVALID_OR_UNVERIFIED_REVOCATION")
        self.assertIn("UNTRUSTED_SIGNER_KEY_SUBSTITUTION",r["detail"])
    def test_21_signed_message_tampering_rejected(self):
        x=json.loads(self.valid)
        x["payload"]["resource_sha256"]="c"*64
        r=gate.evaluate(gate.canonical(x),self.pub,1800000060)
        self.assertEqual(r["status"],"BLOCK")
        self.assertIn("SSHSIG_NOT_VERIFIED",r["detail"])
    def test_22_valid_signature_wrong_freshness_rejected(self):
        r=gate.evaluate(self.valid,self.pub,1800000500)
        self.assertIn("EXPIRED_OR_FUTURE",r["detail"])
    def test_23_real_older_sqlite_snapshot_restoration_erases_signed_revocation(self):
        with gate.RevocationLedger(self.dbroot) as lab:
            self.assertFalse(lab.audit()["local_revoked"])
        saved=self.dbroot/"saved-old-db.sqlite3"
        shutil.copy2(self.dbroot/gate.DB_NAME,saved)
        with gate.RevocationLedger(self.dbroot) as lab:
            r=lab.revoke(self.valid,self.pub,1800000060)
            self.assertEqual(r["status"],"LAB_LOCAL_REVOKE_COMMIT")
            self.assertTrue(lab.audit()["local_revoked"])
        shutil.copy2(saved,self.dbroot/gate.DB_NAME)
        with gate.RevocationLedger(self.dbroot) as lab:
            old=lab.audit()
        self.assertFalse(old["local_revoked"])
        self.assertEqual(old["local_height"],0)
        self.assertEqual(old["global_antirollback"],"NOT_IMPLEMENTED")
    def test_24_native_cli_returns_exit_two_and_revocation_rollback_denial(self):
        result=subprocess.run([sys.executable,"-B",str(Path(gate.__file__)),"--exercise"],
                              capture_output=True,text=True,timeout=45)
        self.assertEqual(result.returncode,2)
        obj=json.loads(result.stdout)
        self.assertEqual(obj["reason"],"SIGNED_LOCAL_REVOCATION_LOST_AFTER_SAME_ADMIN_SNAPSHOT_RESTORE")
        self.assertTrue(obj["native_real_ed25519_sshsig"])
        self.assertFalse(obj["local_revoked_after_rollback"])
        self.assertEqual(obj["admission"],"NOT_AUTHORIZED")

if __name__=="__main__":unittest.main()
