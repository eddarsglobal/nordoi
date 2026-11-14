"""NORDOI G0.3 R6E: 24 owner OOB origin and external custodian refusal cases.

16 portable; eight native POSIX real SSHSIG and same-admin process checks.
No actual owner OOB channel or external custodian is enrolled or simulated.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r6e_oob_origin_intake as lab
import g03_r6d_admin_domain_provenance as r6d
import g03_r6c_two_key_quorum_negative as r6c
import g03_r6b_signed_response_provenance as r6b
import g03_r6a_oob_witness_challenge as r6a
import g03_r5z_preserved_witness_rollback as r5z
import g03_r5y_signed_nonce_rollback as r5y
import g03_r5x_signed_external_provenance as r5x
import g03_r5w_external_admin_intake as r5w
import g03_r5v_admin_separation_evidence as r5v
import g03_r5t_signed_receipt_provenance as ssh

class PortableOOBOrigin(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix=r6b.LAB_PREFIX)
        self.root = Path(self.tmp.name)
        if os.name != "nt":
            os.chmod(self.root, 0o700)
        a = r5v.observe("owner_proxy", "lab-owner-domain",
                        "a"*64, 0, "b"*64, "c"*64)
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
        self.r6c = r6c.canonical(r6c.proposal(
            self.reply, self.request, self.witness, "a"*64, 1100))
        self.r6d = r6d.canonical(r6d.make(
            self.r6c, self.reply, self.request, self.witness,
            1001, 1002, 501, 501, 1100))
        self.x = lab.make(self.r6d, self.r6c, self.reply,
                          self.request, self.witness, 1100)
        self.raw = lab.canonical(self.x)

    def tearDown(self):
        self.tmp.cleanup()

    def checked(self, obj=None, r6d_raw=None, r6c_raw=None, r6b_raw=None,
                r6a_raw=None, witness_raw=None, now=1200, **kw):
        result = lab.assess(
            lab.canonical(self.x if obj is None else obj),
            self.r6d if r6d_raw is None else r6d_raw,
            self.r6c if r6c_raw is None else r6c_raw,
            self.reply if r6b_raw is None else r6b_raw,
            self.request if r6a_raw is None else r6a_raw,
            self.witness if witness_raw is None else witness_raw, now, **kw)
        self.assertEqual(result["status"], "BLOCK")
        self.assertEqual(result["admission"], "NOT_AUTHORIZED")
        self.assertFalse(result["native_authority_granted"])
        self.assertEqual(result["native_admission_gates_unmet"], 12)
        return result

    def rejected(self, raw):
        with self.assertRaises(lab.Refused):
            lab.parse(raw)

    def test_01_exact_frozen_parent_protocol_and_deny_operation(self):
        self.assertEqual(set(self.x), lab.FIELDS)
        self.assertEqual(self.x["parent"], lab.PARENT)
        self.assertEqual(self.x["protocol"], lab.PROTOCOL)
        self.assertEqual(self.x["operation"], lab.OPERATION)
        self.assertEqual(self.x["local_claim_transport"], "same-admin-local-file")

    def test_02_exact_all_five_parent_blob_hashes_and_challenge(self):
        for key, raw in (
            ("r6d_evidence_sha256", self.r6d),
            ("r6c_envelope_sha256", self.r6c),
            ("r6b_response_sha256", self.reply),
            ("r6a_request_sha256", self.request),
            ("r5z_witness_sha256", self.witness),
        ):
            self.assertEqual(self.x[key], r5y.sha(raw))
        self.assertEqual(self.x["challenge"], "e"*64)

    def test_03_missing_real_signature_no_external_oob(self):
        self.assertEqual(self.checked()["reason"],
                         "EXTERNAL_OOB_OR_SIGNED_EVIDENCE_MISSING")

    def test_04_duplicate_fields_pretty_json_or_noncanonical_refused(self):
        self.rejected(self.raw.replace(
            b'"parent":', b'"parent":"forged","parent":'))
        self.rejected(json.dumps(self.x, indent=2).encode())
        self.rejected(self.raw.rstrip(b"\n"))

    def test_05_oversized_invalid_utf8_missing_extra_field(self):
        self.rejected(b"\xff")
        self.rejected(b"z"*(lab.MAX_BYTES + 1))
        self.rejected(lab.canonical(dict(self.x, native_grant=True)))
        x = dict(self.x)
        del x["challenge"]
        self.rejected(lab.canonical(x))

    def test_06_wrong_source_or_elevated_operation_quarantined(self):
        for key, val in (("parent", "0"*40), ("protocol", "FAKE"),
                         ("project", "fake/repo"),
                         ("operation", "NATIVE_ADMISSION_GRANTED")):
            self.rejected(lab.canonical(dict(self.x, **{key:val})))

    def test_07_invalid_required_digest_local_transport_and_root_pin(self):
        for key, val in (
            ("r6d_evidence_sha256", "not-a-sha"),
            ("challenge", "z"*64),
            ("candidate_key_sha256", "0"*40),
            ("local_claim_transport", "secure-independent-oob"),
            ("claimed_owner_oob_root_sha256", "invalid"),
            ("claimed_external_history_sha256", "x"*64),
        ):
            self.rejected(lab.canonical(dict(self.x, **{key:val})))

    def test_08_bool_negative_float_and_nan_rejected(self):
        for key, val in (
            ("locally_issued_at", True), ("locally_expires_at", None),
            ("claimed_remote_nonce_floor", -1),
            ("claimed_remote_revocation_floor", 1.5),
            ("claimed_external_quorum", True),
        ):
            self.rejected(lab.canonical(dict(self.x, **{key:val})))
        self.rejected(self.raw.replace(
            b'"locally_issued_at":1100', b'"locally_issued_at":NaN'))

    def test_09_invalid_local_timebox_strict(self):
        for ttl in (0, 301, True, -1):
            with self.assertRaises(lab.Refused):
                lab.make(self.r6d, self.r6c, self.reply,
                         self.request, self.witness, 1100, ttl)
        self.rejected(lab.canonical(dict(self.x, locally_expires_at=1099)))

    def test_10_old_and_future_clock_not_oob_freshness(self):
        for time in (1099, 1281):
            self.assertEqual(self.checked(now=time)["reason"],
                             "UNTRUSTED_LOCAL_TIMEBOX_STALE_OR_FUTURE")

    def test_11_wrong_request_or_parent_r6d_hash_rejected(self):
        altered_request = r6a.canonical(r6a.create_request(
            self.witness, "d"*64, 1100, challenge="f"*64))
        self.assertEqual(self.checked(r6a_raw=altered_request)["reason"],
                         "OOB_ORIGIN_EVIDENCE_QUARANTINED")
        self.assertEqual(self.checked(obj=dict(
            self.x, r6d_evidence_sha256="f"*64))["reason"],
            "OOB_ORIGIN_EVIDENCE_QUARANTINED")

    def test_12_wrong_owner_candidate_key_or_parent_chain_rejected(self):
        for key, val in (
            ("owner_key_sha256", "f"*64),
            ("candidate_key_sha256", "e"*64),
            ("r6c_envelope_sha256", "c"*64),
            ("r5z_witness_sha256", "e"*64),
        ):
            self.assertEqual(self.checked(obj=dict(self.x, **{key:val}))[
                             "reason"], "OOB_ORIGIN_EVIDENCE_QUARANTINED")

    def test_13_all_external_root_operator_guardian_claims_untrusted(self):
        for key, val in (
            ("claimed_external_operator_id", "lab-false-guardian"),
            ("claimed_owner_oob_root_sha256", "a"*64),
            ("claimed_guardian_public_key_sha256", "d"*64),
            ("claimed_external_admin_domain", "lab-false-domain"),
            ("claimed_external_history_sha256", "c"*64),
            ("claimed_remote_nonce_floor", 100),
            ("claimed_remote_revocation_floor", 100),
            ("claimed_external_quorum", 2),
        ):
            out = self.checked(obj=dict(self.x, **{key:val}))
            self.assertEqual(out["reason"],
                "SELF_ASSERTED_OOB_ROOT_OR_EXTERNAL_CUSTODY_NOT_AUTHENTICATED")
            self.assertTrue(out["asserted_external_evidence_is_untrusted"])

    def test_14_local_spent_challenge_not_global_antirollback(self):
        out = self.checked(local_spent=frozenset(["e"*64]))
        self.assertEqual(out["reason"],
                         "LOCAL_SPENT_CHALLENGE_NOT_EXTERNAL_NONCE_FLOOR")

    def test_15_local_key_revocation_not_independent_floor(self):
        for pin in ("a"*64, "d"*64):
            self.assertEqual(self.checked(
                local_revoked=frozenset([pin]))["reason"],
                "LOCAL_REVOKED_SIGNER_NOT_EXTERNAL_REVOCATION_FLOOR")

    def test_16_malformed_local_antirollback_registers_refused(self):
        for params in (
            {"local_spent":["e"*64]},
            {"local_revoked":frozenset(["bad"])},
            {"local_spent":frozenset([4])},
        ):
            self.assertEqual(self.checked(**params)["reason"],
                             "OOB_ORIGIN_EVIDENCE_QUARANTINED")

@unittest.skipUnless(os.name != "nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "true POSIX OpenSSH Ed25519 OS-native proofs required")
class NativeOOBOrigin(unittest.TestCase):
    def setUp(self):
        self.tmp, self.d = lab.create_lab()

    def tearDown(self):
        self.tmp.cleanup()

    def checked(self, **kw):
        return lab.assess_lab(self.d, **kw)

    def test_17_actual_six_valid_signatures_not_owner_oob(self):
        d = self.d
        self.assertNotEqual(d["owner_pid"], d["candidate_pid"])
        self.assertEqual(d["owner_uid"], d["candidate_uid"])
        self.assertNotEqual(d["owner_pin"], d["candidate_pin"])
        out = self.checked()
        self.assertEqual(out["reason"],
                         "SIX_VALID_LOCAL_SIGNATURES_NOT_OWNER_OOB_ENROLLMENT")
        self.assertEqual(out["genuine_signatures_verified"], 6)
        self.assertFalse(out["separately_authenticated_owner_channel"])

    def test_18_mutated_genuinely_signed_receipt_rejected(self):
        d = self.d
        fake = lab.canonical(dict(lab.parse(d["r6e_body"]),
                                  locally_issued_at=1101))
        out = self.checked(raw=fake)
        self.assertEqual(out["reason"], "OOB_ORIGIN_EVIDENCE_QUARANTINED")
        self.assertIn("GENUINE_LOCAL_OOB_LOOKING_SSHSIG_INVALID", out["detail"])

    def test_19_wrong_allowed_signer_provenance_refused(self):
        out = self.checked(local_allowed=self.d["owner_allowed"])
        self.assertEqual(out["reason"], "OOB_ORIGIN_EVIDENCE_QUARANTINED")
        self.assertIn("GENUINE_LOCAL_OOB_LOOKING_SSHSIG_INVALID", out["detail"])

    def test_20_wrong_sshsig_namespace_refused(self):
        d = self.d
        path = d["root"] / "r6e_wrong_namespace.json"
        path.write_bytes(d["r6e_body"])
        os.chmod(path, 0o600)
        sig = r5x.sign_local(path, d["candidate_key"], "nordoi-r6e-wrong")
        self.assertEqual(
            self.checked(local_signature=sig)["reason"],
            "OOB_ORIGIN_EVIDENCE_QUARANTINED")

    def test_21_real_signed_fake_owner_root_still_denied(self):
        d = self.d
        fake = lab.canonical(dict(
            lab.parse(d["r6e_body"]),
            claimed_owner_oob_root_sha256=d["owner_pin"],
            claimed_external_operator_id="lab-false-guardian"))
        path = d["root"] / "r6e_false_root_signed_test.json"
        path.write_bytes(fake)
        os.chmod(path, 0o600)
        sig = r5x.sign_local(path, d["candidate_key"], lab.NAMESPACE)
        self.assertEqual(ssh.run_ssh([
            "-Y", "verify", "-f", str(d["guardian_allowed"]),
            "-I", ssh.IDENTITY, "-n", lab.NAMESPACE, "-s", str(sig)
        ], fake), 0)
        out = self.checked(raw=fake, local_signature=sig)
        self.assertEqual(out["reason"],
            "SELF_ASSERTED_OOB_ROOT_OR_EXTERNAL_CUSTODY_NOT_AUTHENTICATED")

    def test_22_cross_request_signed_replay_refused(self):
        d = self.d
        another_request = r6a.canonical(r6a.create_request(
            d["witness"], d["candidate_pin"], 1100))
        out = self.checked(request_raw=another_request)
        self.assertEqual(out["reason"], "OOB_ORIGIN_EVIDENCE_QUARANTINED")

    def test_23_actual_local_key_revocation_and_spent_nonce(self):
        d = self.d
        ch = lab.parse(d["r6e_body"])["challenge"]
        self.assertEqual(self.checked(local_spent=frozenset([ch]))[
                         "reason"], "LOCAL_SPENT_CHALLENGE_NOT_EXTERNAL_NONCE_FLOOR")
        for k in (d["owner_pin"], d["candidate_pin"]):
            self.assertEqual(self.checked(local_revoked=frozenset([k]))[
                             "reason"], "LOCAL_REVOKED_SIGNER_NOT_EXTERNAL_REVOCATION_FLOOR")

    def test_24_real_full_lab_exercise_exits_two(self):
        p = subprocess.run([sys.executable, "-B", str(Path(lab.__file__)),
                            "--exercise"], capture_output=True, text=True,
                           timeout=240, check=False)
        self.assertEqual(p.returncode, 2, p.stderr)
        x = json.loads(p.stdout)
        self.assertEqual(x["reason"],
                         "R6E_GENUINE_LOCAL_OOB_ORIGIN_NEGATIVE_CONTROL_QUALIFIED")
        self.assertEqual(x["native_admission_gates_unmet"], 12)
        self.assertEqual(x["independently_authenticated_custodian"], "NONE")
        self.assertEqual(x["status"], "BLOCK")

if __name__ == "__main__":
    unittest.main()
