"""NORDOI G0.3 R6D — 24 independent-domain provenance negative controls.

Sixteen portable source/schema/identity/replay failures, eight genuine POSIX
OpenSSH and real OS-identity controls. No actual external custodian exists.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r6d_admin_domain_provenance as lab
import g03_r6c_two_key_quorum_negative as r6c
import g03_r6b_signed_response_provenance as r6b
import g03_r6a_oob_witness_challenge as r6a
import g03_r5z_preserved_witness_rollback as r5z
import g03_r5y_signed_nonce_rollback as r5y
import g03_r5x_signed_external_provenance as r5x
import g03_r5w_external_admin_intake as r5w
import g03_r5v_admin_separation_evidence as r5v
import g03_r5t_signed_receipt_provenance as ssh

class PortableDomainProvenance(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix=r6b.LAB_PREFIX)
        self.root = Path(self.temp.name)
        if os.name != "nt":
            os.chmod(self.root, 0o700)
        owner = r5v.observe("owner_proxy", "lab-owner-domain",
                             "a"*64, 0, "b"*64, "c"*64)
        candidate = dict(owner, pid=owner["pid"] + 1,
                         role="custodian_proxy",
                         claimed_domain="lab-candidate-domain",
                         key_sha256="d"*64)
        d = r5w.canonical(r5w.candidate(r5v.canonical(owner),
                                         r5v.canonical(candidate)))
        receipt = r5x.canonical(r5x.make_candidate(d, "d"*64, 1000))
        r5y.bootstrap(self.root)
        self.assertEqual(r5y.commit_local_denial(self.root,receipt,1100,1)[
                         "reason"], "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.witness = r5z.canonical(r5z.candidate(self.root,receipt,1100))
        self.request = r6a.canonical(r6a.create_request(
            self.witness,"d"*64,1100,challenge="e"*64))
        self.reply = r6b.canonical(r6b.make_response(self.request,1100))
        self.r6c = r6c.canonical(r6c.proposal(
            self.reply,self.request,self.witness,"a"*64,1100))
        self.x = lab.make(self.r6c,self.reply,self.request,self.witness,
                          1001,1002,501,501,1100)
        self.raw = lab.canonical(self.x)

    def tearDown(self):
        self.temp.cleanup()

    def examine(self, x=None, r6c_raw=None, reply=None, request=None,
                witness=None, now=1200, **kwargs):
        out = lab.inspect(
            lab.canonical(self.x if x is None else x),
            self.r6c if r6c_raw is None else r6c_raw,
            self.reply if reply is None else reply,
            self.request if request is None else request,
            self.witness if witness is None else witness,
            now, **kwargs)
        self.assertEqual(out["status"], "BLOCK")
        self.assertEqual(out["admission"], "NOT_AUTHORIZED")
        self.assertFalse(out["native_authority_granted"])
        self.assertEqual(out["native_admission_gates_unmet"], 12)
        return out

    def rejected(self, raw):
        with self.assertRaises(lab.Refused):
            lab.parse(raw)

    def test_01_exact_protocol_parent_roles_no_native_grant(self):
        self.assertEqual(set(self.x),lab.FIELDS)
        self.assertEqual(self.x["protocol"],lab.PROTOCOL)
        self.assertEqual(self.x["parent"],lab.PARENT)
        self.assertEqual(self.x["operation"],lab.OPERATION)
        self.assertEqual((self.x["owner_role"],self.x["candidate_role"]),
                         ("owner_proxy","custodian_proxy"))

    def test_02_exact_r6c_r6b_r6a_r5z_source_hashes(self):
        for key, raw in (
            ("r6c_envelope_sha256",self.r6c),
            ("r6b_response_sha256",self.reply),
            ("r6a_request_sha256",self.request),
            ("r5z_witness_sha256",self.witness)
        ):
            self.assertEqual(self.x[key],r5y.sha(raw))
        self.assertEqual(self.x["challenge"],"e"*64)

    def test_03_missing_signature_not_real_admin_provenance(self):
        self.assertEqual(self.examine()["reason"],
                         "MISSING_INDEPENDENTLY_VERIFIABLE_PROVENANCE")

    def test_04_duplicate_pretty_and_noncanonical_json_rejected(self):
        self.rejected(self.raw.replace(b'"parent":',
                      b'"parent":"fake","parent":'))
        self.rejected(json.dumps(self.x,indent=2).encode())
        self.rejected(self.raw.rstrip(b"\n"))

    def test_05_extra_missing_large_and_invalid_utf8_rejected(self):
        self.rejected(lab.canonical(dict(self.x,grant_native=True)))
        x = dict(self.x)
        del x["owner_key_sha256"]
        self.rejected(lab.canonical(x))
        self.rejected(b"!"*(lab.MAX_BYTES+1))
        self.rejected(b"\xff")

    def test_06_fake_source_privileged_verb_and_roles_rejected(self):
        for k,v in (("parent","0"*40),("project","other/repo"),
                    ("protocol","FAKE"),("operation","AUTHORIZE_NATIVE"),
                    ("owner_role","independent_custodian"),
                    ("candidate_role","prod_guardian")):
            self.rejected(lab.canonical(dict(self.x,**{k:v})))

    def test_07_invalid_digest_and_external_operator_label_rejected(self):
        for k,v in (("r6c_envelope_sha256","invalid"),("challenge","Z"*64),
                    ("candidate_key_sha256","b"*32),
                    ("claimed_external_operator_id","file:///root"),
                    ("claimed_oob_owner_root_sha256","bad"),
                    ("claimed_owner_domain","external-service")):
            self.rejected(lab.canonical(dict(self.x,**{k:v})))

    def test_08_bad_os_pid_boolean_negative_float_clock_rejected(self):
        for k,v in (("owner_pid",True),("candidate_pid",0),
                    ("owner_uid",-1),("candidate_uid",1.25),
                    ("issued_at_local",None),("expires_at_local",True),
                    ("claimed_global_nonce_floor",-1),
                    ("claimed_independent_quorum",True)):
            self.rejected(lab.canonical(dict(self.x,**{k:v})))
        self.rejected(self.raw.replace(b'"issued_at_local":1100',
                      b'"issued_at_local":NaN'))

    def test_09_two_observed_pids_required_no_zero_or_same(self):
        for d in (dict(owner_pid=0),dict(candidate_pid=1001)):
            self.rejected(lab.canonical(dict(self.x,**d)))
        with self.assertRaises(lab.Refused):
            lab.make(self.r6c,self.reply,self.request,self.witness,
                     1001,1001,501,501,1100)

    def test_10_same_domain_label_is_not_two_domains(self):
        x = dict(self.x,claimed_candidate_domain="lab-owner-domain")
        self.assertEqual(self.examine(x=x)["reason"],
                         "SAME_DOMAIN_LABEL_NOT_SEPARATE_CUSTODY")

    def test_11_different_claimed_uid_does_not_attest_external_domain(self):
        x = dict(self.x,candidate_uid=502)
        self.assertEqual(self.examine(x=x)["reason"],
                         "DIFFERENT_CLAIMED_UID_NOT_EXTERNAL_ADMIN_ATTESTATION")

    def test_12_self_asserted_external_identity_root_anchor_quorum_denied(self):
        for k,v in (
            ("claimed_external_operator_id","lab-fake-independent"),
            ("claimed_oob_owner_root_sha256","a"*64),
            ("claimed_external_admin_attestation_sha256","f"*64),
            ("claimed_external_history_sha256","d"*64),
            ("claimed_global_nonce_floor",99),
            ("claimed_global_revocation_floor",99),
            ("claimed_independent_quorum",2),
        ):
            result=self.examine(x=dict(self.x,**{k:v}))
            self.assertEqual(result["reason"],
                             "SAME_ADMIN_SIGNED_EXTERNAL_CLAIMS_NOT_AUTHENTICATED")
            self.assertTrue(result["signed_claim_is_not_independence"])

    def test_13_old_or_future_local_clock_not_remote_freshness(self):
        for when in (1099,1281):
            self.assertEqual(self.examine(now=when)["reason"],
                             "UNTRUSTED_LOCAL_CLOCK_EXPIRED_OR_FUTURE")

    def test_14_tampered_r6c_envelope_response_request_or_witness_denied(self):
        for k,v in (("r6c_envelope_sha256","f"*64),
                    ("r6b_response_sha256","f"*64),
                    ("r6a_request_sha256","f"*64),
                    ("r5z_witness_sha256","f"*64),
                    ("owner_key_sha256","f"*64)):
            self.assertEqual(self.examine(x=dict(self.x,**{k:v}))["reason"],
                             "ADMIN_DOMAIN_PROVENANCE_QUARANTINED")

    def test_15_local_spent_nonce_and_revoked_key_denied(self):
        self.assertEqual(self.examine(local_spent=frozenset(["e"*64]))["reason"],
                         "LOCAL_SPENT_NONCE_REJECTED_NO_GLOBAL_FLOOR")
        for key in ("a"*64,"d"*64):
            self.assertEqual(self.examine(local_revoked=frozenset([key]))["reason"],
                             "LOCAL_REVOKED_KEY_REJECTED_NO_EXTERNAL_FLOOR")

    def test_16_malformed_local_revocation_sets_quarantined(self):
        for k in ({"local_spent":["e"*64]},
                  {"local_revoked":frozenset(["invalid"])},
                  {"local_spent":frozenset([1])}):
            self.assertEqual(self.examine(**k)["reason"],
                             "ADMIN_DOMAIN_PROVENANCE_QUARANTINED")

@unittest.skipUnless(os.name != "nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "real POSIX OpenSSH, distinct PIDs and same UID required")
class NativeFiveSignedDomainProvenance(unittest.TestCase):
    def setUp(self):
        self.temp,self.data=lab.create_lab()

    def tearDown(self):
        self.temp.cleanup()

    def checked(self, raw=None, **overrides):
        d=self.data
        kwargs=dict(
            original_owner_signature=d["owner_signature"],
            original_guardian_signature=d["guardian_signature"],
            response_signature=d["parent_response_sig"],
            r6d_owner_signature=d["r6d_owner_signature"],
            r6d_candidate_signature=d["r6d_candidate_signature"],
            owner_allowed=d["owner_allowed"],
            candidate_allowed=d["guardian_allowed"],
            owner_public=d["owner_public"],
            candidate_public=d["guardian_public"],
        )
        kwargs.update(overrides)
        return lab.inspect(d["r6d_body"] if raw is None else raw,
                           d["body"],d["response"],d["request"],d["witness"],
                           1200, **kwargs)

    def test_17_five_real_signatures_same_admin_not_external(self):
        d=self.data
        self.assertNotEqual(d["owner_pid"],d["candidate_pid"])
        self.assertEqual(d["owner_uid"],d["candidate_uid"])
        self.assertNotEqual(d["owner_pin"],d["candidate_pin"])
        x=self.checked()
        self.assertEqual(x["reason"],
                         "TRUE_LOCAL_OS_PAIR_AND_FIVE_SSHSIGS_NO_EXTERNAL_DOMAIN")
        self.assertEqual(x["authenticated_local_signatures"],5)
        self.assertFalse(x["real_external_operator_authenticated"])

    def test_18_modified_signed_envelope_fails_actual_crypto(self):
        x=dict(lab.parse(self.data["r6d_body"]),issued_at_local=1101)
        result=self.checked(raw=lab.canonical(x))
        self.assertEqual(result["reason"],"ADMIN_DOMAIN_PROVENANCE_QUARANTINED")
        self.assertIn("REAL_ADMIN_DOMAIN_SSHSIG_INVALID_",result["detail"])

    def test_19_wrong_real_owner_allowed_signer_fails(self):
        result=self.checked(owner_allowed=self.data["guardian_allowed"])
        self.assertEqual(result["reason"],"ADMIN_DOMAIN_PROVENANCE_QUARANTINED")

    def test_20_swapped_signatures_or_namespace_invalid(self):
        d=self.data
        result=self.checked(
            r6d_owner_signature=d["r6d_candidate_signature"],
            r6d_candidate_signature=d["r6d_owner_signature"])
        self.assertEqual(result["reason"],"ADMIN_DOMAIN_PROVENANCE_QUARANTINED")
        path=d["root"]/"r6d_wrong_namespace.json"
        path.write_bytes(d["r6d_body"])
        os.chmod(path,0o600)
        sig=r5x.sign_local(path,d["candidate_key"],"nordoi-r6d-wrong")
        self.assertEqual(self.checked(r6d_candidate_signature=sig)["reason"],
                         "ADMIN_DOMAIN_PROVENANCE_QUARANTINED")

    def test_21_local_signed_false_external_root_still_denied(self):
        d=self.data
        x=dict(lab.parse(d["r6d_body"]),
               claimed_external_operator_id="lab-fake-independent",
               claimed_oob_owner_root_sha256=d["owner_pin"])
        raw=lab.canonical(x)
        path=d["root"]/"r6d_false_claim.json"
        path.write_bytes(raw)
        os.chmod(path,0o600)
        sig=r5x.sign_local(path,d["owner_key"],lab.OWNER_NS)
        self.assertEqual(ssh.run_ssh([
            "-Y","verify","-f",str(d["owner_allowed"]),
            "-I",ssh.IDENTITY,"-n",lab.OWNER_NS,"-s",str(sig)
        ],raw),0)
        self.assertEqual(self.checked(raw=raw)["reason"],
                         "SAME_ADMIN_SIGNED_EXTERNAL_CLAIMS_NOT_AUTHENTICATED")

    def test_22_declared_other_uid_does_not_attest_independence(self):
        d=self.data
        x=dict(lab.parse(d["r6d_body"]),
               candidate_uid=lab.parse(d["r6d_body"])["owner_uid"]+1)
        self.assertEqual(self.checked(raw=lab.canonical(x))["reason"],
                         "DIFFERENT_CLAIMED_UID_NOT_EXTERNAL_ADMIN_ATTESTATION")

    def test_23_actual_spent_nonce_and_key_revocation_remain_local(self):
        d=self.data
        nonce=lab.parse(d["r6d_body"])["challenge"]
        self.assertEqual(self.checked(local_spent=frozenset([nonce]))["reason"],
                         "LOCAL_SPENT_NONCE_REJECTED_NO_GLOBAL_FLOOR")
        self.assertEqual(self.checked(
            local_revoked=frozenset([d["candidate_pin"]]))["reason"],
                         "LOCAL_REVOKED_KEY_REJECTED_NO_EXTERNAL_FLOOR")

    def test_24_full_real_five_signature_experiment_exits_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),"--exercise"],
                         capture_output=True,text=True,timeout=240,check=False)
        self.assertEqual(p.returncode,2,p.stderr)
        result=json.loads(p.stdout)
        self.assertEqual(result["reason"],
                         "R6D_REAL_ADMIN_DOMAIN_NEGATIVE_PROVENANCE_QUALIFIED")
        self.assertEqual(result["five_real_local_ssh_signatures"],
                         "PASS_CRYPTO_ONLY")
        self.assertEqual(result["native_admission_gates_unmet"],12)
        self.assertEqual(result["native_admission"],"BLOCK")

if __name__=="__main__":
    unittest.main()
