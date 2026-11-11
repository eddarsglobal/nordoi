"""R5K: 24 actual deny-only concurrent recovery/TOCTOU controls.

Eight native controls use two real disposable Ed25519 OpenSSH signing keys
and real on-disk R5H revocation/R5I checkpoint databases.
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
import g03_r5j_signed_recovery_quarantine as rec
import g03_r5k_concurrent_recovery_quarantine as gate

WIRE=b"\x00\x00\x00\x0bssh-ed25519\x00\x00\x00\x20"+b"1"*32
PUB="ssh-ed25519 "+base64.b64encode(WIRE).decode("ascii")
SIG=b"-----BEGIN SSH SIGNATURE-----\nfake\n-----END SSH SIGNATURE-----\n"

def fake_packet(**changes):
    pl={"protocol":rec.PROTOCOL,"project":rec.PROJECT,"parent_commit":rec.PARENT,
        "operation":"RECOVERY_INTENT_DENY_ONLY","floor_height":1,"floor_head":"a"*64,
        "primary_head":"b"*64,"floor_epoch":4,"nonce":"c"*64,
        "issued_unix":1800000000,"expires_unix":1800000120,
        "fixture_key_sha256":hashlib.sha256(PUB.encode("ascii")).hexdigest()}
    pl.update(changes)
    return rec.canon({"payload":pl,"fixture_public_key":PUB,
                      "signature_b64":base64.b64encode(SIG).decode("ascii")})

class PortableBatchRefusals(unittest.TestCase):
    def setUp(self):
        self.p=fake_packet()
        self.good=gate.envelope(self.p,self.p)
    def refused(self,raw):
        with self.assertRaises(gate.Refused):
            gate.parse(raw)
    def test_01_missing_batch_rejected(self):
        self.refused(b"")
    def test_02_invalid_utf8_rejected(self):
        self.refused(b"\xff")
    def test_03_duplicate_json_field_rejected(self):
        self.refused(self.good.replace(b'"project":',b'"project":"fake","project":'))
    def test_04_whitespace_noncanonical_rejected(self):
        self.refused(json.dumps(json.loads(self.good),indent=2).encode("ascii"))
    def test_05_nonfinite_json_rejected(self):
        self.refused(self.good.replace(b'"intents":',b'"fake":NaN,"intents":'))
    def test_06_oversized_and_nonbytes_rejected(self):
        self.refused(b"x"*(gate.MAX_BYTES+1))
        self.refused("not bytes")
    def test_07_wrong_source_protocol_refused(self):
        x=json.loads(self.good);x["protocol"]="FAKE"
        self.refused(gate.canonical(x))
    def test_08_wrong_project_refused(self):
        x=json.loads(self.good);x["project"]="other/project"
        self.refused(gate.canonical(x))
    def test_09_wrong_frozen_r5j_parent_refused(self):
        x=json.loads(self.good);x["parent_commit"]="0"*40
        self.refused(gate.canonical(x))
    def test_10_reordered_or_untrusted_candidate_roles_refused(self):
        x=json.loads(self.good);x["intents"][0]["role"]="candidate_b"
        self.refused(gate.canonical(x))
    def test_11_wrong_pair_cardinality_refused(self):
        x=json.loads(self.good)
        x["intents"].pop()
        self.refused(gate.canonical(x))
        x["intents"]=json.loads(self.good)["intents"]*2
        self.refused(gate.canonical(x))
    def test_12_extra_unrecognized_field_refused(self):
        x=json.loads(self.good);x["intents"][0]["extra"]="grant"
        self.refused(gate.canonical(x))
    def test_13_invalid_signed_intent_base64_refused(self):
        x=json.loads(self.good);x["intents"][0]["intent_b64"]="x!??"
        self.refused(gate.canonical(x))
    def test_14_signed_intent_digest_mismatch_refused(self):
        x=json.loads(self.good);x["intents"][0]["intent_sha256"]="e"*64
        self.refused(gate.canonical(x))
    def test_15_malformed_embedded_signed_recovery_intent_refused(self):
        self.refused(gate.envelope(b'{"fake":true}\n',self.p))
    def test_16_fake_sshsig_cannot_grant_recovery(self):
        x=gate.decide(self.good,{"candidate_a":PUB,"candidate_b":PUB},
                      "/missing/r5k-primary","/missing/r5k-floor",1800000060)
        self.assertEqual(x["status"],"BLOCK")
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")
        self.assertEqual(x["reason"],"CONCURRENT_RECOVERY_EVIDENCE_QUARANTINED")
        self.assertIn("REAL_SSHSIG_VERIFICATION_FAILED",x["detail"])

@unittest.skipUnless(os.name!="nt" and Path(rev.SSH).is_file(),
                      "genuine native Unix/OpenSSH Ed25519 and SQLite required")
class NativeConcurrentRealSQLite(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.keys=tempfile.TemporaryDirectory(prefix="nordoi-r5k-real-keys-")
        root=Path(cls.keys.name);os.chmod(root,0o700)
        d1=root/"first";d2=root/"second";d1.mkdir(mode=0o700);d2.mkdir(mode=0o700)
        os.chmod(d1,0o700);os.chmod(d2,0o700)
        cls.priv_a,cls.pub_a=rev.keygen(d1)
        cls.priv_b,cls.pub_b=rev.keygen(d2)
    @classmethod
    def tearDownClass(cls):
        cls.keys.cleanup()
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5k-actual-sqlite-")
        root=Path(self.tmp.name);os.chmod(root,0o700)
        self.a=root/"primary";self.b=root/"floor"
        self.a.mkdir(mode=0o700);self.b.mkdir(mode=0o700)
        os.chmod(self.a,0o700);os.chmod(self.b,0o700)
        with rev.RevocationLedger(self.a) as local:
            self.assertEqual(local.audit()["local_height"],0)
        self.assertEqual(floor.execute(self.a,self.b,"audit")["floor_height"],0)
        self.old_a=root/"oldprimary.sqlite3";self.old_b=root/"oldfloor.sqlite3"
        shutil.copy2(self.a/rev.DB_NAME,self.old_a)
        shutil.copy2(self.b/floor.NAME,self.old_b)
        with tempfile.TemporaryDirectory(prefix="nordoi-r5k-signed-revoke-") as d:
            revocation=rev.make_packet(self.priv_a,self.pub_a,Path(d))
        with rev.RevocationLedger(self.a) as local:
            self.assertEqual(local.revoke(revocation,self.pub_a,1800000060)["status"],
                             "LAB_LOCAL_REVOKE_COMMIT")
        self.assertEqual(floor.execute(self.a,self.b,"checkpoint")["status"],
                         "LAB_LOCAL_CHECKPOINT_COMMIT")
        self.state=rec.observed(self.a,self.b)
        self.req_a=rec.intent(self.priv_a,self.pub_a,self.state,nonce="a"*64)
        self.req_b=rec.intent(self.priv_b,self.pub_b,self.state,nonce="b"*64)
        self.keys_map={"candidate_a":self.pub_a,"candidate_b":self.pub_b}
    def tearDown(self):
        self.tmp.cleanup()
    def decision(self,p=None,keys=None,after_first_read=None):
        r=gate.decide(p if p is not None else gate.envelope(self.req_a,self.req_b),
                      keys if keys is not None else self.keys_map,
                      self.a,self.b,1800000060,after_first_read=after_first_read)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertEqual(r["independent_guardian"],"NOT_ENROLLED")
        return r
    def test_17_two_real_distinct_ed25519_signatures_even_matching_history_denied(self):
        r=self.decision()
        self.assertEqual(r["reason"],"CONCURRENT_VALID_SIGNED_INTENTS_NO_EXTERNAL_AUTHORITY")
        self.assertEqual(r["real_native_signatures"],2)
    def test_18_duplicate_same_real_signed_intent_replay_denied(self):
        r=self.decision(gate.envelope(self.req_a,self.req_a),
                        {"candidate_a":self.pub_a,"candidate_b":self.pub_a})
        self.assertEqual(r["reason"],"REPLAYED_IDENTICAL_SIGNED_INTENT_REFUSED")
    def test_19_same_signer_two_distinct_signed_nonces_denied(self):
        another=rec.intent(self.priv_a,self.pub_a,self.state,nonce="d"*64)
        r=self.decision(gate.envelope(self.req_a,another),
                        {"candidate_a":self.pub_a,"candidate_b":self.pub_a})
        self.assertEqual(r["reason"],"SAME_FIXTURE_SIGNER_COLLISION_REFUSED")
    def test_20_two_real_keys_same_signed_nonce_collision_denied(self):
        another=rec.intent(self.priv_b,self.pub_b,self.state,nonce="a"*64)
        r=self.decision(gate.envelope(self.req_a,another))
        self.assertEqual(r["reason"],"CONCURRENT_SIGNED_NONCE_COLLISION_REFUSED")
    def test_21_signed_payload_tampering_and_signer_swap_quarantined(self):
        raw=json.loads(self.req_a)
        raw["payload"]["nonce"]="d"*64
        altered=rec.canon(raw)
        r=self.decision(gate.envelope(altered,self.req_b))
        self.assertIn("REAL_SSHSIG_VERIFICATION_FAILED",r["detail"])
        r=self.decision(keys={"candidate_a":self.pub_b,"candidate_b":self.pub_a})
        self.assertIn("SIGNING_KEY_SUBSTITUTION",r["detail"])
    def test_22_real_second_process_primary_rollback_between_checks_quarantined(self):
        r=self.decision(after_first_read=lambda:gate._child_restore([
            (self.old_a,self.a/rev.DB_NAME)]))
        self.assertIn("STALE_DIVERGENT",r["detail"])
    def test_23_real_second_process_joint_rollback_between_checks_quarantined(self):
        r=self.decision(after_first_read=lambda:gate._child_restore([
            (self.old_a,self.a/rev.DB_NAME),(self.old_b,self.b/floor.NAME)]))
        self.assertIn("CHECKPOINT_CHANGED_DURING_VALIDATION",r["detail"])
        self.assertEqual(r["protected_global_recovery_ledger"],"NOT_IMPLEMENTED")
    def test_24_native_cli_full_real_concurrent_recovery_replay_exits_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(gate.__file__)),"--exercise"],
                         capture_output=True,text=True,timeout=60)
        self.assertEqual(p.returncode,2,p.stderr)
        report=json.loads(p.stdout)
        self.assertEqual(report["reason"],"CONCURRENT_REAL_SIGNED_RECOVERY_ALWAYS_DENIED")
        self.assertTrue(report["two_valid_disposable_signer_keys"])
        self.assertEqual(report["actual_separate_process_primary_restore"],"BLOCK")
        self.assertEqual(report["actual_separate_process_both_store_restore"],"BLOCK")
        self.assertFalse(report["native_access_or_recovery_granted"])

if __name__=="__main__":
    unittest.main()
