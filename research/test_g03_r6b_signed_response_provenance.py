"""NORDOI G0.3 R6B — 24 fail-closed signed response provenance controls.

16 portable source/challenge/schema/replay controls; 8 native genuine
same-admin OpenSSH SSHSIG controls. No actual external custodian enrolled.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r6b_signed_response_provenance as lab
import g03_r6a_oob_witness_challenge as r6a
import g03_r5z_preserved_witness_rollback as r5z
import g03_r5y_signed_nonce_rollback as r5y
import g03_r5x_signed_external_provenance as r5x
import g03_r5w_external_admin_intake as r5w
import g03_r5v_admin_separation_evidence as r5v
import g03_r5t_signed_receipt_provenance as ssh

class PortableResponseProvenance(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix=lab.LAB_PREFIX)
        self.root = Path(self.temp.name)
        if os.name != "nt":
            os.chmod(self.root, 0o700)
        a = r5v.observe("owner_proxy", "lab-owner-domain", "a"*64,
                         0, "b"*64, "c"*64)
        b = dict(a, role="custodian_proxy", pid=a["pid"]+1,
                 claimed_domain="lab-candidate-domain", key_sha256="d"*64)
        dossier = r5w.canonical(r5w.candidate(r5v.canonical(a),r5v.canonical(b)))
        source = r5x.canonical(r5x.make_candidate(dossier,"d"*64,1000))
        r5y.bootstrap(self.root)
        self.assertEqual(r5y.commit_local_denial(self.root,source,1100,1)["reason"],
                         "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.witness = r5z.canonical(r5z.candidate(self.root,source,1100))
        self.request = r6a.canonical(r6a.create_request(
            self.witness,"d"*64,1100,challenge="e"*64))
        self.x = lab.make_response(self.request,1100)
        self.raw = lab.canonical(self.x)

    def tearDown(self):
        self.temp.cleanup()

    def verified(self, d=None, request=None, witness_raw=None, now=1200, **kw):
        r = lab.assess(lab.canonical(self.x if d is None else d),
                       self.request if request is None else request,
                       self.witness if witness_raw is None else witness_raw,
                       now, **kw)
        self.assertEqual(r["status"], "BLOCK")
        self.assertEqual(r["admission"], "NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        self.assertEqual(r["native_admission_gates_unmet"], 12)
        return r

    def refused(self, raw):
        with self.assertRaises(lab.Refused):
            lab.parse(raw)

    def test_01_frozen_protocol_project_parent_and_operation(self):
        self.assertEqual(self.x["protocol"],lab.PROTOCOL)
        self.assertEqual(self.x["project"],lab.PROJECT)
        self.assertEqual(self.x["parent"],lab.PARENT)
        self.assertEqual(self.x["operation"],lab.OPERATION)
        self.assertEqual(set(self.x),lab.FIELDS)

    def test_02_exact_request_and_witness_binding(self):
        self.assertEqual(self.x["request_sha256"],r5y.sha(self.request))
        self.assertEqual(self.x["challenge"],"e"*64)
        self.assertEqual(self.x["witness_sha256"],r5y.sha(self.witness))
        self.assertEqual(self.x["witness_height"],1)
        self.assertEqual(self.x["nonce_set_sha256"],r5z.parse(self.witness)["nonce_set_sha256"])

    def test_03_unsigned_candidate_never_authenticates_custodian(self):
        self.assertEqual(self.verified()["reason"],
                         "MISSING_VERIFIABLE_LOCAL_SIGNER_NOT_EXTERNAL_AUTHORITY")

    def test_04_duplicate_pretty_and_missing_json_fields_refused(self):
        self.refused(self.raw.replace(b'"parent":',b'"parent":"fake","parent":'))
        self.refused(json.dumps(self.x,indent=2).encode())
        d=dict(self.x)
        del d["challenge"]
        self.refused(lab.canonical(d))
        self.refused(self.raw.rstrip(b"\n"))

    def test_05_extra_oversized_and_malformed_utf8_refused(self):
        self.refused(lab.canonical(dict(self.x,native_grant=True)))
        self.refused(b"\xff")
        self.refused(b"x"*(lab.MAX_BYTES+1))

    def test_06_spoofed_source_project_and_grant_verb_refused(self):
        for k,v in (("parent","0"*40),("project","fake/repo"),
                    ("protocol","FAKE"),("operation","AUTHORIZE_NATIVE")):
            self.refused(lab.canonical(dict(self.x,**{k:v})))

    def test_07_invalid_hashes_and_untrusted_external_ids_refused(self):
        for k,v in (("request_sha256","short"),("challenge","Z"*64),
                    ("candidate_key_sha256","not-a-hash"),
                    ("claimed_external_operator_id","/root/guardian"),
                    ("claimed_admin_domain","production"):
            self.refused(lab.canonical(dict(self.x,**{k:v})))

    def test_08_invalid_numbers_nan_bool_negative_or_float(self):
        for k,v in (("witness_height",True),("witness_height",0),
                    ("local_revocation_floor",-1),("issued_at_local",True),
                    ("expires_at_local",None),
                    ("claimed_external_spent_nonce_floor",0.5)):
            self.refused(lab.canonical(dict(self.x,**{k:v})))
        self.refused(self.raw.replace(b'"issued_at_local":1100',
                      b'"issued_at_local":NaN'))

    def test_09_local_timebox_invalid_and_excessive_ttl(self):
        for ttl in (0,lab.MAX_WINDOW+1,True,-1):
            with self.assertRaises(lab.Refused):
                lab.make_response(self.request,1100,ttl)
        self.refused(lab.canonical(dict(self.x,expires_at_local=1099)))

    def test_10_old_or_future_response_denied(self):
        for now in (1099,1281):
            self.assertEqual(self.verified(now=now)["reason"],
                             "LOCAL_TIMEBOX_STALE_OR_FUTURE_NOT_EXTERNAL_FRESHNESS")

    def test_11_changed_request_nonce_or_payload_hash_rejected(self):
        r = r6a.canonical(r6a.create_request(
            self.witness,"d"*64,1100,challenge="f"*64))
        self.assertEqual(self.verified(request=r)["reason"],
                         "SIGNED_RESPONSE_PROVENANCE_QUARANTINED")
        self.assertEqual(self.verified(dict(self.x,challenge="f"*64))["reason"],
                         "SIGNED_RESPONSE_PROVENANCE_QUARANTINED")

    def test_12_changed_witness_height_history_and_signer_refused(self):
        for k,v in (("witness_sha256","a"*64),("witness_height",2),
                    ("nonce_set_sha256","f"*64),
                    ("local_revocation_floor",2),
                    ("candidate_key_sha256","e"*64)):
            self.assertEqual(self.verified(dict(self.x,**{k:v}))["reason"],
                             "SIGNED_RESPONSE_PROVENANCE_QUARANTINED")

    def test_13_declared_external_provenance_never_accepted(self):
        for k,v in (("claimed_external_operator_id","lab-fake-operator"),
                    ("claimed_admin_domain","lab-claimed-domain"),
                    ("claimed_owner_oob_root_sha256","f"*64),
                    ("claimed_external_history_sha256","b"*64),
                    ("claimed_external_revocation_floor",100),
                    ("claimed_external_spent_nonce_floor",100)):
            a=self.verified(dict(self.x,**{k:v}))
            self.assertEqual(a["reason"],
                             "SELF_DECLARED_SIGNED_EXTERNAL_PROVENANCE_UNTRUSTED")
            self.assertTrue(a["claims_not_independently_authenticated"])

    def test_14_local_spent_nonce_refused_not_global(self):
        a=self.verified(local_spent=frozenset(["e"*64]))
        self.assertEqual(a["reason"],"LOCALLY_REPLAYED_CHALLENGE_REFUSED")
        self.assertFalse(a["independent_nonce_floor"])

    def test_15_local_key_revocation_refused_not_global(self):
        a=self.verified(locally_revoked_keys=frozenset(["d"*64]))
        self.assertEqual(a["reason"],"LOCALLY_REVOKED_CANDIDATE_REFUSED")
        self.assertFalse(a["independent_revocation_floor"])

    def test_16_invalid_spent_revoked_inputs_quarantined(self):
        for kw in ({"local_spent":["e"*64]},
                   {"locally_revoked_keys":frozenset(["bad"])},
                   {"local_spent":frozenset([4])}):
            self.assertEqual(self.verified(**kw)["reason"],
                             "SIGNED_RESPONSE_PROVENANCE_QUARANTINED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "true POSIX OpenSSH local identity and signatures required")
class NativeResponseProvenance(unittest.TestCase):
    def setUp(self):
        self.temp,self.data=lab.make_lab()

    def tearDown(self):
        self.temp.cleanup()

    def check(self, response=None, request=None, signature=None, allowed=None,
              public=None, now=1200, **kwargs):
        d=self.data
        return lab.assess(d["response"] if response is None else response,
                          d["request"] if request is None else request,
                          d["witness"],now,
                          d["signature"] if signature is None else signature,
                          d["allowed"] if allowed is None else allowed,
                          d["public"] if public is None else public,
                          **kwargs)

    def test_17_actual_signed_candidate_same_uid_not_external(self):
        d=self.data
        self.assertNotEqual(d["owner_pid"],d["candidate_pid"])
        self.assertEqual(d["owner_uid"],d["candidate_uid"])
        self.assertNotEqual(d["owner_pin"],d["candidate_pin"])
        r=self.check()
        self.assertEqual(r["reason"],
                         "VALID_SSHSIG_LOCAL_SIGNER_NOT_INDEPENDENT_CUSTODIAN")
        self.assertTrue(r["cryptographic_signature_valid"])
        self.assertFalse(r["signer_independence_proven"])

    def test_18_tampered_signed_body_real_crypto_refused(self):
        r=self.check(response=lab.canonical(dict(
            lab.parse(self.data["response"]),issued_at_local=1101)))
        self.assertEqual(r["reason"],"SIGNED_RESPONSE_PROVENANCE_QUARANTINED")
        self.assertIn("REAL_ED25519_SSHSIG_INVALID",r["detail"])

    def test_19_wrong_allowed_signers_refused(self):
        r=self.check(allowed=self.data["owner_allowed"])
        self.assertEqual(r["reason"],"SIGNED_RESPONSE_PROVENANCE_QUARANTINED")
        self.assertIn("REAL_ED25519_SSHSIG_INVALID",r["detail"])

    def test_20_wrong_signature_namespace_refused(self):
        d=self.data
        p=d["root"]/"r6b_wrong_namespace.json"
        p.write_bytes(d["response"])
        os.chmod(p,0o600)
        sig=lab.sign_local(p,d["candidate_key"],"nordoi-r6b-wrong")
        self.assertEqual(self.check(signature=sig)["reason"],
                         "SIGNED_RESPONSE_PROVENANCE_QUARANTINED")

    def test_21_genuinely_signed_false_external_claim_still_denied(self):
        d=self.data
        fake=dict(lab.parse(d["response"]),
                  claimed_external_operator_id="lab-fake-custodian")
        raw=lab.canonical(fake)
        p=d["root"]/"r6b_fake_external_claim.json"
        p.write_bytes(raw)
        os.chmod(p,0o600)
        sig=lab.sign_local(p,d["candidate_key"])
        self.assertEqual(ssh.run_ssh(["-Y","verify","-f",str(d["allowed"]),
                                      "-I",ssh.IDENTITY,"-n",lab.NAMESPACE,
                                      "-s",str(sig)],raw),0)
        r=self.check(response=raw,signature=sig)
        self.assertEqual(r["reason"],
                         "SELF_DECLARED_SIGNED_EXTERNAL_PROVENANCE_UNTRUSTED")

    def test_22_real_signed_cross_request_replay_denied(self):
        d=self.data
        alternate=r6a.canonical(r6a.create_request(
            d["witness"],d["candidate_pin"],1100))
        self.assertNotEqual(r6a.parse(alternate)["challenge"],
                            lab.parse(d["response"])["challenge"])
        r=self.check(request=alternate)
        self.assertEqual(r["reason"],"SIGNED_RESPONSE_PROVENANCE_QUARANTINED")

    def test_23_real_valid_key_but_locally_revoked_denied(self):
        d=self.data
        r=self.check(locally_revoked_keys=frozenset([d["candidate_pin"]]))
        self.assertEqual(r["reason"],"LOCALLY_REVOKED_CANDIDATE_REFUSED")
        self.assertFalse(r["independent_revocation_floor"])

    def test_24_actual_full_crypto_exercise_exit_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),"--exercise"],
                         text=True,capture_output=True,timeout=240,check=False)
        self.assertEqual(p.returncode,2,p.stderr)
        r=json.loads(p.stdout)
        self.assertEqual(r["reason"],
                         "REAL_SIGNED_R6A_RESPONSE_PROVENANCE_NEGATIVE_CONTROLS_PASS")
        self.assertEqual(r["native_admission_gates_unmet"],12)
        self.assertEqual(r["native_admission"],"BLOCK")
        self.assertFalse(r["native_authority_granted"])

if __name__=="__main__":
    unittest.main()
