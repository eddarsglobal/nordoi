"""G0.3 R5J: 24 adversarial signed recovery-intent and actual R5H/R5I history tests.

Native tests use real disposable OpenSSH Ed25519 fixture keys and real
on-disk signed R5H revocation plus R5I checkpoint under one administrator.
They NEVER enroll a real guardian or authorize recovery.
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
import g03_r5h_signed_revocation_lab as rev
import g03_r5i_revocation_floor as floor
import g03_r5j_signed_recovery_quarantine as gate

WIRE=b"\x00\x00\x00\x0bssh-ed25519\x00\x00\x00\x20"+b"1"*32
PUB="ssh-ed25519 "+base64.b64encode(WIRE).decode("ascii")
FAKE=b"-----BEGIN SSH SIGNATURE-----\nfake\n-----END SSH SIGNATURE-----\n"

def payload(**updates):
    data={"protocol":gate.PROTOCOL,"project":gate.PROJECT,"parent_commit":gate.PARENT,
          "operation":"RECOVERY_INTENT_DENY_ONLY","floor_height":1,
          "floor_head":"a"*64,"primary_head":"b"*64,"floor_epoch":4,"nonce":"c"*64,
          "issued_unix":1800000000,"expires_unix":1800000120,
          "fixture_key_sha256":hashlib.sha256(PUB.encode("ascii")).hexdigest()}
    data.update(updates)
    return data

def pack(pl=None,**updates):
    value={"payload":payload() if pl is None else pl,
           "fixture_public_key":PUB,"signature_b64":base64.b64encode(FAKE).decode("ascii")}
    value.update(updates)
    return gate.canon(value)

class PortableQuarantineTests(unittest.TestCase):
    def reject(self,raw):
        with self.assertRaises(gate.Invalid):gate.decode(raw)
    def test_01_missing_intent_rejected(self):self.reject(b"")
    def test_02_utf8_invalid_rejected(self):self.reject(b"\xff")
    def test_03_duplicate_field_rejected(self):
        self.reject(pack().replace(b'"project":',b'"project":"fake","project":'))
    def test_04_noncanonical_whitespace_rejected(self):
        self.reject(json.dumps(json.loads(pack()),indent=2).encode("ascii"))
    def test_05_nonfinite_and_extra_fields_rejected(self):
        self.reject(pack().replace(b'"floor_epoch":4',b'"floor_epoch":NaN'))
        self.reject(gate.canon({**json.loads(pack()),"extra":"untrusted"}))
    def test_06_oversize_and_nonbytes_rejected(self):
        self.reject(b"x"*(gate.MAX_BYTES+1))
        self.reject("not bytes")
    def test_07_wrong_protocol_project_and_parent_rejected(self):
        self.reject(pack(payload(protocol="FAKE")))
        self.reject(pack(payload(project="other/repo")))
        self.reject(pack(payload(parent_commit="0"*40)))
    def test_08_grant_unrevoke_and_auto_recover_forbidden(self):
        for op in ("GRANT","UNREVOKE","RECOVER","APPROVE"):
            self.reject(pack(payload(operation=op)))
    def test_09_genesis_floor_and_bool_height_refused(self):
        self.reject(pack(payload(floor_height=0)))
        self.reject(pack(payload(floor_height=True)))
    def test_10_invalid_epoch_and_bool_refused(self):
        self.reject(pack(payload(floor_epoch=3)))
        self.reject(pack(payload(floor_epoch=False)))
    def test_11_invalid_nonce_head_and_fixture_hash_refused(self):
        for key in ("floor_head","primary_head","nonce","fixture_key_sha256"):
            self.reject(pack(payload(**{key:"0"*64})))
    def test_12_invalid_timestamp_and_boolean_expiry_refused(self):
        self.reject(pack(payload(issued_unix=0)))
        self.reject(pack(payload(expires_unix=True)))
    def test_13_intent_ttl_expired_or_unbounded_rejected(self):
        self.reject(pack(payload(expires_unix=1800000900)))
        self.reject(pack(payload(expires_unix=1800000000)))
    def test_14_ed25519_wire_and_fingerprint_qualification(self):
        self.reject(pack(fixture_public_key="ssh-rsa AAAA"))
        self.reject(pack(payload(fixture_key_sha256="d"*64)))
    def test_15_fake_signature_cannot_authenticate_custody(self):
        result=gate.assessment(pack(),PUB,"/missing/r5j-primary","/missing/r5j-floor",1800000060)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertNotIn("fixture_sshsig_verified",result)
    def test_16_cli_does_not_accept_caller_specified_database_paths(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-r5j-protect-cli-") as root:
            p=subprocess.run([sys.executable,"-B",str(Path(gate.__file__)),
                              "--exercise","--primary-directory",root],
                             capture_output=True,text=True,timeout=20)
            self.assertEqual(p.returncode,2)
            self.assertIn("unrecognized arguments",p.stderr)

@unittest.skipUnless(os.name!="nt" and Path(gate.SSH).is_file(),
                      "genuine native Unix/OpenSSH Ed25519 fixture required")
class NativeRecoveryQuarantine(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.keys=tempfile.TemporaryDirectory(prefix="nordoi-r5j-native-keys-")
        cls.keydir=Path(cls.keys.name)
        os.chmod(cls.keydir,0o700)
        cls.private,cls.pub=rev.keygen(cls.keydir)
    @classmethod
    def tearDownClass(cls):cls.keys.cleanup()
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5j-native-store-")
        self.root=Path(self.temp.name)
        self.primary=self.root/"publisher"
        self.secondary=self.root/"checkpoint"
        self.primary.mkdir(mode=0o700)
        self.secondary.mkdir(mode=0o700)
        os.chmod(self.primary,0o700)
        os.chmod(self.secondary,0o700)
        with rev.RevocationLedger(self.primary) as journal:
            self.assertEqual(journal.audit()["local_height"],0)
        self.assertEqual(floor.execute(self.primary,self.secondary,"audit")["floor_height"],0)
        self.old_primary=self.root/"pre-revoke.sqlite3"
        self.old_floor=self.root/"pre-floor.sqlite3"
        shutil.copy2(self.primary/rev.DB_NAME,self.old_primary)
        shutil.copy2(self.secondary/floor.NAME,self.old_floor)
        with tempfile.TemporaryDirectory(prefix="nordoi-r5j-revoke-packet-") as temp:
            signed=rev.make_packet(self.private,self.pub,Path(temp))
        with rev.RevocationLedger(self.primary) as journal:
            record=journal.revoke(signed,self.pub,1800000060)
        self.assertEqual(record["status"],"LAB_LOCAL_REVOKE_COMMIT")
        checkpoint=floor.execute(self.primary,self.secondary,"checkpoint")
        self.assertEqual(checkpoint["status"],"LAB_LOCAL_CHECKPOINT_COMMIT")
        self.state=gate.observed(self.primary,self.secondary)
        self.request=gate.intent(self.private,self.pub,self.state)
    def tearDown(self):self.temp.cleanup()
    def check(self,request=None,public_key=None,now=1800000060):
        result=gate.assessment(self.request if request is None else request,
                               self.pub if public_key is None else public_key,
                               self.primary,self.secondary,now)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        return result
    def test_17_valid_native_ed25519_fixture_still_denies_recovery(self):
        result=self.check()
        self.assertTrue(result["fixture_sshsig_verified"])
        self.assertEqual(result["reason"],"VALID_LOCAL_FIXTURE_RECOVERY_INTENT_NO_EXTERNAL_AUTHORITY")
        self.assertEqual(result["real_external_guardian"],"NOT_ENROLLED")
    def test_18_real_alternative_public_key_substitution_refused(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-r5j-alt-key-") as folder:
            _,different=rev.keygen(Path(folder))
            result=self.check(public_key=different)
        self.assertIn("UNTRUSTED_SIGNER_SUBSTITUTION",result["detail"])
    def test_19_signed_payload_tamper_detected(self):
        payload=json.loads(self.request)
        payload["payload"]["nonce"]="d"*64
        result=self.check(request=gate.canon(payload))
        self.assertIn("NATIVE_SSHSIG_NOT_VERIFIED",result["detail"])
    def test_20_signed_recovery_intent_expiry_refused(self):
        result=self.check(now=1800000500)
        self.assertIn("STALE_OR_NOT_YET_VALID_INTENT",result["detail"])
    def test_21_primary_rollback_with_preserved_floor_quarantined(self):
        shutil.copy2(self.old_primary,self.primary/rev.DB_NAME)
        result=self.check()
        self.assertIn("STALE_DIVERGENT_OR_UNVERIFIED",result["detail"])
    def test_22_real_same_height_alternative_signed_fork_quarantined(self):
        shutil.copy2(self.old_primary,self.primary/rev.DB_NAME)
        with tempfile.TemporaryDirectory(prefix="nordoi-r5j-divergent-signature-") as folder:
            alt=rev.make_packet(self.private,self.pub,Path(folder),nonce="d"*64)
        with rev.RevocationLedger(self.primary) as journal:
            out=journal.revoke(alt,self.pub,1800000060)
        self.assertEqual(out["status"],"LAB_LOCAL_REVOKE_COMMIT")
        result=self.check()
        self.assertIn("STALE_DIVERGENT_OR_UNVERIFIED",result["detail"])
    def test_23_joint_genesis_rollback_with_signed_intent_still_refused(self):
        shutil.copy2(self.old_primary,self.primary/rev.DB_NAME)
        shutil.copy2(self.old_floor,self.secondary/floor.NAME)
        result=self.check()
        self.assertIn("PRE_REVOCATION_OR_UNALIGNED_HISTORY",result["detail"])
        self.assertEqual(result["protected_remote_recovery"],"NOT_IMPLEMENTED")
    def test_24_real_native_cli_quarantines_every_recovery_and_exits_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(gate.__file__)),"--exercise"],
                         capture_output=True,text=True,timeout=60)
        self.assertEqual(p.returncode,2,p.stderr)
        result=json.loads(p.stdout)
        self.assertEqual(result["reason"],"SIGNED_LOCAL_RECOVERY_INTENT_NEVER_REENABLES_AUTHORITY")
        self.assertTrue(result["real_fixture_ed25519_sshsig"])
        self.assertEqual(result["both_old_snapshots_without_external_anchor"],
                         "NOT_GLOBALLY_DETECTABLE")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")

if __name__=="__main__":unittest.main()
