"""G0.3 R6C — 24 adversarial dual-key, same-admin quorum denial tests.

16 portable exact-source/parser/custody controls, 8 actual native POSIX
OpenSSH cases. Two real Ed25519 signatures are NOT two independent custodians.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r6c_two_key_quorum_negative as lab
import g03_r6b_signed_response_provenance as r6b
import g03_r6a_oob_witness_challenge as r6a
import g03_r5z_preserved_witness_rollback as r5z
import g03_r5y_signed_nonce_rollback as r5y
import g03_r5x_signed_external_provenance as r5x
import g03_r5w_external_admin_intake as r5w
import g03_r5v_admin_separation_evidence as r5v
import g03_r5t_signed_receipt_provenance as ssh

class PortableQuorumDenial(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix=lab.response.LAB_PREFIX)
        self.root = Path(self.temp.name)
        if os.name != "nt":
            os.chmod(self.root, 0o700)
        a = r5v.observe("owner_proxy", "lab-owner-domain", "a"*64,
                        0, "b"*64, "c"*64)
        b = dict(a, role="custodian_proxy", pid=a["pid"]+1,
                 claimed_domain="lab-candidate-domain", key_sha256="d"*64)
        dossier = r5w.canonical(r5w.candidate(r5v.canonical(a),
                                               r5v.canonical(b)))
        source = r5x.canonical(r5x.make_candidate(dossier, "d"*64, 1000))
        r5y.bootstrap(self.root)
        self.assertEqual(r5y.commit_local_denial(self.root, source, 1100, 1)[
                         "reason"], "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.witness = r5z.canonical(r5z.candidate(self.root, source, 1100))
        self.request = r6a.canonical(r6a.create_request(
            self.witness, "d"*64, 1100, challenge="e"*64))
        self.reply = r6b.canonical(r6b.make_response(self.request, 1100))
        self.obj = lab.proposal(self.reply, self.request, self.witness,
                                "a"*64, 1100)
        self.body = lab.canonical(self.obj)

    def tearDown(self):
        self.temp.cleanup()

    def checked(self, obj=None, reply=None, request=None, historic=None,
                now=1200, **kwargs):
        r = lab.inspect(lab.canonical(self.obj if obj is None else obj),
                        self.reply if reply is None else reply,
                        self.request if request is None else request,
                        self.witness if historic is None else historic,
                        now, **kwargs)
        self.assertEqual(r["status"], "BLOCK")
        self.assertEqual(r["admission"], "NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        self.assertEqual(r["native_admission_gates_unmet"], 12)
        return r

    def rejected(self, raw):
        with self.assertRaises(lab.Refused):
            lab.parse(raw)

    def test_01_exact_frozen_protocol_parent_roles_and_operation(self):
        self.assertEqual(set(self.obj), lab.FIELDS)
        self.assertEqual(self.obj["protocol"], lab.PROTOCOL)
        self.assertEqual(self.obj["parent"], lab.PARENT)
        self.assertEqual(self.obj["operation"], lab.OPERATION)
        self.assertEqual((self.obj["owner_role"], self.obj["candidate_role"]),
                         ("owner_proxy", "custodian_proxy"))

    def test_02_exact_source_hashes_and_two_distinct_key_ids(self):
        self.assertEqual(self.obj["response_sha256"], r5y.sha(self.reply))
        self.assertEqual(self.obj["request_sha256"], r5y.sha(self.request))
        self.assertEqual(self.obj["witness_sha256"], r5y.sha(self.witness))
        self.assertEqual(self.obj["challenge"], "e"*64)
        self.assertNotEqual(self.obj["owner_key_sha256"],
                            self.obj["candidate_key_sha256"])

    def test_03_no_signatures_not_quorum_and_always_denied(self):
        r = self.checked()
        self.assertEqual(r["reason"], "REAL_SIGNATURE_EVIDENCE_MISSING_NO_QUORUM")
        self.assertEqual(r["independent_quorum"], "NOT_PROVEN")

    def test_04_duplicate_pretty_and_noncanonical_fields_rejected(self):
        self.rejected(self.body.replace(b'"parent":',
                       b'"parent":"fake","parent":'))
        self.rejected(json.dumps(self.obj, indent=2).encode())
        self.rejected(self.body.rstrip(b"\n"))

    def test_05_extra_missing_oversize_and_nonutf8_refused(self):
        self.rejected(lab.canonical(dict(self.obj, native_grant=True)))
        d = dict(self.obj)
        del d["challenge"]
        self.rejected(lab.canonical(d))
        self.rejected(b"x"*(lab.MAX_BYTES+1))
        self.rejected(b"\xff")

    def test_06_wrong_source_authorization_or_roles_refused(self):
        for key, val in (("parent", "0"*40), ("project", "fake/repo"),
                         ("operation", "GRANT_PRODUCTION"),
                         ("owner_role", "external_certified"),
                         ("candidate_role", "independent_guardian")):
            self.rejected(lab.canonical(dict(self.obj, **{key:val})))

    def test_07_identical_key_ids_and_bad_hashes_refused(self):
        for key, val in (("owner_key_sha256", "d"*64),
                         ("response_sha256", "NOT_A_PIN"),
                         ("nonce_set_sha256", "Z"*64),
                         ("claimed_owner_oob_root_sha256", "invalid"),
                         ("claimed_external_anchor_sha256", "0"*40)):
            self.rejected(lab.canonical(dict(self.obj, **{key:val})))
        with self.assertRaises(lab.Refused):
            lab.proposal(self.reply, self.request, self.witness, "d"*64, 1100)

    def test_08_bool_float_negative_nan_and_missing_counters_refused(self):
        for key, val in (("witness_height", True), ("witness_height", 0),
                         ("revocation_floor", -1),
                         ("claimed_quorum_size", True),
                         ("claimed_remote_spent_nonce_floor", 1.2),
                         ("issued_at_local", None)):
            self.rejected(lab.canonical(dict(self.obj, **{key:val})))
        self.rejected(self.body.replace(b'"issued_at_local":1100',
                      b'"issued_at_local":NaN'))

    def test_09_local_timebox_stale_or_future_refused(self):
        for now in (1099, 1281):
            self.assertEqual(self.checked(now=now)["reason"],
                             "LOCAL_CLOCK_STALE_OR_FUTURE_NOT_EXTERNAL_FRESHNESS")
        with self.assertRaises(lab.Refused):
            lab.proposal(self.reply, self.request, self.witness, "a"*64,
                         1100, ttl=301)

    def test_10_altered_source_reply_request_and_nonce_rejected(self):
        other = r6a.canonical(r6a.create_request(
            self.witness, "d"*64, 1100, challenge="f"*64))
        self.assertEqual(self.checked(request=other)["reason"],
                         "DUAL_WITNESS_PROVENANCE_QUARANTINED")
        self.assertEqual(self.checked(obj=dict(self.obj, challenge="f"*64))[
                         "reason"], "DUAL_WITNESS_PROVENANCE_QUARANTINED")

    def test_11_changed_witness_height_nonce_set_and_revocation_refused(self):
        for key, val in (("witness_sha256", "f"*64),
                         ("witness_height", 2),
                         ("nonce_set_sha256", "f"*64),
                         ("revocation_floor", 2),
                         ("candidate_key_sha256", "b"*64)):
            self.assertEqual(self.checked(obj=dict(self.obj, **{key:val}))[
                             "reason"], "DUAL_WITNESS_PROVENANCE_QUARANTINED")

    def test_12_local_spent_random_challenge_refused(self):
        r = self.checked(local_spent=frozenset(["e"*64]))
        self.assertEqual(r["reason"],
                         "LOCALLY_USED_CHALLENGE_NOT_GLOBAL_ANTIROLLBACK")
        self.assertTrue(r["local_replay_refused"])

    def test_13_owner_or_candidate_local_revocation_refused(self):
        for p in ("a"*64, "d"*64):
            r = self.checked(local_revoked=frozenset([p]))
            self.assertEqual(r["reason"], "LOCALLY_REVOKED_DUAL_SIGNER_REFUSED")
            self.assertFalse(r["global_revocation_proven"])

    def test_14_bad_untrusted_local_nonce_and_revocation_inputs_refused(self):
        for k in ({"local_spent": ["e"*64]},
                  {"local_revoked": frozenset([4])},
                  {"local_spent": frozenset(["invalid"])}):
            self.assertEqual(self.checked(**k)["reason"],
                             "DUAL_WITNESS_PROVENANCE_QUARANTINED")

    def test_15_all_signed_external_root_and_quorum_claims_denied(self):
        for key, val in (
            ("claimed_owner_oob_root_sha256", "c"*64),
            ("claimed_independent_admin_domain", "lab-other-admin"),
            ("claimed_external_anchor_sha256", "f"*64),
            ("claimed_remote_revocation_floor", 999),
            ("claimed_remote_spent_nonce_floor", 999),
            ("claimed_quorum_size", 2),
        ):
            r = self.checked(obj=dict(self.obj, **{key:val}))
            self.assertEqual(r["reason"],
                             "SIGNED_SELF_ASSERTED_INDEPENDENCE_NOT_AUTHORITY")
            self.assertTrue(r["external_claims_not_independently_authenticated"])

    def test_16_distinct_role_labels_not_os_admin_independence(self):
        self.assertEqual(self.checked()["reason"],
                         "REAL_SIGNATURE_EVIDENCE_MISSING_NO_QUORUM")
        self.assertEqual(lab.deny("NONINDEPENDENT")["independent_admin_custody"],
                         "NOT_PROVEN")
        self.assertEqual(lab.deny("NONINDEPENDENT")["independent_quorum"],
                         "NOT_PROVEN")

@unittest.skipUnless(os.name != "nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "actual native POSIX two-key OpenSSH SSHSIG checks required")
class NativeTwoSignatureQuorum(unittest.TestCase):
    def setUp(self):
        self.temp, self.data = lab.create_lab()

    def tearDown(self):
        self.temp.cleanup()

    def checked(self, raw=None, owner_signature=None, guardian_signature=None,
                owner_allowed=None, guardian_allowed=None, now=1200, **kw):
        d = self.data
        return lab.inspect(
            d["body"] if raw is None else raw,
            d["response"], d["request"], d["witness"], now,
            owner_signature=d["owner_signature"] if owner_signature is None else owner_signature,
            owner_allowed=d["owner_allowed"] if owner_allowed is None else owner_allowed,
            owner_public=d["owner_public"],
            guardian_signature=d["guardian_signature"] if guardian_signature is None else guardian_signature,
            guardian_allowed=d["guardian_allowed"] if guardian_allowed is None else guardian_allowed,
            guardian_public=d["guardian_public"],
            response_signature=d["parent_response_sig"],
            response_allowed=d["allowed"],
            response_public=d["public"], **kw
        )

    def test_17_actual_three_ssh_sigs_two_keys_one_uid_no_quorum(self):
        d = self.data
        self.assertNotEqual(d["owner_pid"], d["candidate_pid"])
        self.assertEqual(d["owner_uid"], d["candidate_uid"])
        self.assertNotEqual(d["owner_pin"], d["candidate_pin"])
        r = self.checked()
        self.assertEqual(r["reason"],
                         "TWO_REAL_SSHSIG_KEYS_SAME_ADMIN_NOT_INDEPENDENT_QUORUM")
        self.assertEqual(r["genuine_signatures_verified"], 3)
        self.assertFalse(r["independent_quorum_proven"])

    def test_18_tampered_signed_envelope_invalid_real_openssh(self):
        x = dict(lab.parse(self.data["body"]), issued_at_local=1101)
        r = self.checked(raw=lab.canonical(x))
        self.assertEqual(r["reason"], "DUAL_WITNESS_PROVENANCE_QUARANTINED")
        self.assertIn("GENUINE_DISTINCT_ED25519_SSHSIG_INVALID_", r["detail"])

    def test_19_wrong_true_owner_allowed_signer_refused(self):
        r = self.checked(owner_allowed=self.data["guardian_allowed"])
        self.assertEqual(r["reason"], "DUAL_WITNESS_PROVENANCE_QUARANTINED")
        self.assertIn("GENUINE_DISTINCT_ED25519_SSHSIG_INVALID_", r["detail"])

    def test_20_wrong_true_guardian_namespace_refused(self):
        d = self.data
        p = d["root"] / "r6c_wrong_ns.json"
        p.write_bytes(d["body"])
        os.chmod(p, 0o600)
        wrong = lab.sign_local(p, d["candidate_key"], "nordoi-r6c-wrong")
        r = self.checked(guardian_signature=wrong)
        self.assertEqual(r["reason"], "DUAL_WITNESS_PROVENANCE_QUARANTINED")

    def test_21_swapped_owner_and_guardian_real_sigs_refused(self):
        d = self.data
        r = self.checked(owner_signature=d["guardian_signature"],
                         guardian_signature=d["owner_signature"])
        self.assertEqual(r["reason"], "DUAL_WITNESS_PROVENANCE_QUARANTINED")

    def test_22_real_locally_signed_false_independent_quorum_denied(self):
        d = self.data
        forged = dict(lab.parse(d["body"]), claimed_quorum_size=2,
                      claimed_owner_oob_root_sha256=d["owner_pin"])
        raw = lab.canonical(forged)
        p = d["root"] / "r6c_signed_false_quorum.json"
        p.write_bytes(raw)
        os.chmod(p, 0o600)
        sig = lab.sign_local(p, d["owner_key"], lab.OWNER_NS)
        self.assertEqual(ssh.run_ssh([
            "-Y", "verify", "-f", str(d["owner_allowed"]),
            "-I", ssh.IDENTITY, "-n", lab.OWNER_NS,
            "-s", str(sig)], raw), 0)
        self.assertEqual(self.checked(raw=raw)["reason"],
                         "SIGNED_SELF_ASSERTED_INDEPENDENCE_NOT_AUTHORITY")

    def test_23_real_owner_or_guardian_key_revoke_and_replay_refused(self):
        d = self.data
        for k in (d["owner_pin"], d["candidate_pin"]):
            r = self.checked(local_revoked=frozenset([k]))
            self.assertEqual(r["reason"], "LOCALLY_REVOKED_DUAL_SIGNER_REFUSED")
        r = self.checked(local_spent=frozenset(
            [lab.parse(d["body"])["challenge"]]))
        self.assertEqual(r["reason"],
                         "LOCALLY_USED_CHALLENGE_NOT_GLOBAL_ANTIROLLBACK")

    def test_24_full_real_two_key_exercise_exit_two(self):
        p = subprocess.run(
            [sys.executable, "-B", str(Path(lab.__file__)), "--exercise"],
            capture_output=True, text=True, timeout=240, check=False)
        self.assertEqual(p.returncode, 2, p.stderr)
        result = json.loads(p.stdout)
        self.assertEqual(result["reason"],
                         "REAL_TWO_KEY_SAME_ADMIN_QUORUM_NEGATIVE_CONTROLS_PASS")
        self.assertEqual(result["authentic_owner_ed25519"], "PASS_LOCAL_ONLY")
        self.assertEqual(result["native_admission"], "BLOCK")
        self.assertEqual(result["native_admission_gates_unmet"], 12)

if __name__ == "__main__":
    unittest.main()
