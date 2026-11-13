"""G0.3 R5X — 24 fail-closed tests for REAL SSHSIG and local provenance limits.

Sixteen portable schema/challenge/source/rollback controls; eight POSIX native
OpenSSH negative experiments. Nothing tests an enrolled external guardian.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r5x_signed_external_provenance as lab
import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as r5v
import g03_r5u_key_rotation_custody as rotate
import g03_r5t_signed_receipt_provenance as signed

class PortableSignedProvenance(unittest.TestCase):
    def setUp(self):
        self.a = r5v.observe("owner_proxy", "lab-owner-domain", "a" * 64,
                              0, "b" * 64, "c" * 64)
        self.b = dict(self.a, role="custodian_proxy",
                      claimed_domain="lab-guardian-domain",
                      pid=self.a["pid"] + 1, key_sha256="d" * 64)
        self.ar, self.br = r5v.canonical(self.a), r5v.canonical(self.b)
        self.d = intake.canonical(intake.candidate(self.ar, self.br))
        self.x = lab.make_candidate(self.d, "e" * 64, 1000)

    def denied(self, x=None, **kwargs):
        raw = lab.canonical(x if x is not None else self.x)
        result = lab.verify(raw, self.d, self.ar, self.br, 1100, **kwargs)
        self.assertEqual(result["status"], "BLOCK")
        self.assertFalse(result["native_authority_granted"])
        self.assertEqual(result["native_admission_gates_unmet"], 12)
        self.assertEqual(result["admission"], "NOT_AUTHORIZED")
        return result

    def refused(self, x):
        with self.assertRaises(lab.Refused):
            lab.parse(x)

    def test_01_exact_frozen_source_schema_and_operation(self):
        self.assertEqual(set(self.x), lab.FIELDS)
        self.assertEqual(self.x["parent"], lab.PARENT)
        self.assertEqual(self.x["protocol"], lab.PROTOCOL)
        self.assertEqual(self.x["operation"], lab.OPERATION)

    def test_02_actual_prior_dossier_hash_challenge_and_history(self):
        self.assertEqual(self.x["dossier_sha256"], intake.digest(self.d))
        self.assertEqual(self.x["challenge"], "c" * 64)
        self.assertEqual(self.x["history_head"], "b" * 64)
        self.assertEqual(self.x["key_epoch"], 0)

    def test_03_unsigned_provenance_never_establishes_custody(self):
        self.assertEqual(self.denied()["reason"],
                         "NO_AUTHENTICATED_EXTERNAL_SIGNATURE_OR_ROOT")

    def test_04_duplicate_json_and_extra_field_refused(self):
        b = lab.canonical(self.x)
        self.refused(b.replace(b'"parent":', b'"parent":"fake","parent":'))
        self.refused(lab.canonical(dict(self.x, native_grant=True)))

    def test_05_missing_field_and_noncanonical_refused(self):
        d = dict(self.x)
        del d["challenge"]
        self.refused(lab.canonical(d))
        self.refused(json.dumps(self.x, indent=2).encode())
        self.refused(lab.canonical(self.x).strip())

    def test_06_oversized_malformed_encoding_and_nonfinite_refused(self):
        for b in (b"a" * (lab.MAX_BYTES + 1), b"\xff",
                  lab.canonical(self.x).replace(b'"issued_at":1000',
                                                b'"issued_at":NaN')):
            self.refused(b)

    def test_07_wrong_parent_project_and_operation_quarantined(self):
        for k, v in (("parent", "0" * 40), ("project", "fake/repo"),
                     ("operation", "GRANT_NATIVE")):
            self.refused(lab.canonical(dict(self.x, **{k: v})))

    def test_08_invalid_hash_and_claimed_domain_refused(self):
        for k, v in (("challenge", "bad"), ("guardian_key_sha256", "Z" * 64),
                     ("dossier_sha256", "0" * 40),
                     ("claimed_admin_domain", "../../external")):
            self.refused(lab.canonical(dict(self.x, **{k: v})))

    def test_09_invalid_bool_negative_or_float_counters(self):
        for k, v in (("key_epoch", True), ("issued_at", -1),
                     ("claimed_remote_revocation_floor", 0.5),
                     ("claimed_remote_spent_nonce_floor", True)):
            self.refused(lab.canonical(dict(self.x, **{k: v})))

    def test_10_excessive_or_reverse_local_timebox_refused(self):
        for a, b in ((2000, 1999), (0, lab.MAX_WINDOW + 1)):
            self.refused(lab.canonical(dict(self.x, issued_at=a, expires_at=b)))

    def test_11_stale_receipt_blocks_even_before_crypto(self):
        r = lab.verify(lab.canonical(self.x), self.d, self.ar, self.br, 1301)
        self.assertEqual(r["reason"], "UNTRUSTED_CLOCK_TIMEBOX_STALE_OR_FUTURE")

    def test_12_future_receipt_blocks_even_before_crypto(self):
        r = lab.verify(lab.canonical(self.x), self.d, self.ar, self.br, 999)
        self.assertEqual(r["reason"], "UNTRUSTED_CLOCK_TIMEBOX_STALE_OR_FUTURE")

    def test_13_unauthenticated_root_and_host_claim_cannot_grant(self):
        x = dict(self.x, claimed_owner_oob_root_sha256="f" * 64,
                 claimed_host_attestation_sha256="1" * 64)
        self.assertEqual(self.denied(x)["reason"],
                         "NO_AUTHENTICATED_EXTERNAL_SIGNATURE_OR_ROOT")

    def test_14_signed_receipt_same_nonce_in_local_spent_set_rejected(self):
        r = self.denied(local_spent_challenges=frozenset(["c" * 64]))
        self.assertEqual(r["reason"], "LOCAL_NONCE_REPLAY_BLOCKED")
        self.assertFalse(r["global_spent_nonce_proven"])

    def test_15_declared_remote_floor_below_local_floor_rejected(self):
        r = self.denied(local_revocation_floor=2)
        self.assertEqual(r["reason"], "LOCAL_REVOCATION_FLOOR_REGRESSION")
        self.assertTrue(r["negative_local_floor_comparison"])

    def test_16_forced_dossier_source_history_nonce_mismatch_quarantined(self):
        for k, v in (("dossier_sha256", "f" * 64),
                     ("challenge", "f" * 64),
                     ("history_head", "a" * 64),
                     ("key_epoch", 1)):
            self.assertEqual(self.denied(dict(self.x, **{k:v}))["reason"],
                             "SIGNED_PROVENANCE_EVIDENCE_QUARANTINED")
        b2 = r5v.canonical(dict(self.b, nonce="f" * 64))
        result = lab.verify(lab.canonical(self.x), self.d, self.ar, b2, 1100)
        self.assertEqual(result["reason"], "SIGNED_PROVENANCE_EVIDENCE_QUARANTINED")

@unittest.skipUnless(os.name != "nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "actual POSIX OpenSSH native signatures required")
class NativeSignedProvenance(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(
            prefix="nordoi-r5l-r5q-r5s-r5t-r5u-r5w-r5x-test-")
        self.root = Path(self.t.name)
        os.chmod(self.root, 0o700)
        self.owner, self.owner_allow = signed.create_same_admin_key(
            self.root, "r5x_owner")
        self.guardian, self.guardian_allow = signed.create_same_admin_key(
            self.root, "r5x_guardian")
        self.owner_pub = self.root / "r5x_owner.pub"
        self.guardian_pub = self.root / "r5x_guardian.pub"
        self.opin = rotate.public_sha(self.owner_pub)
        self.gpin = rotate.public_sha(self.guardian_pub)
        self.a = r5v.child_observe("owner_proxy", "lab-owner-domain",
                                   self.opin, 0, "b" * 64, "c" * 64)
        self.b = r5v.child_observe("custodian_proxy", "lab-guardian-domain",
                                   self.gpin, 0, "b" * 64, "c" * 64)
        self.ar, self.br = r5v.canonical(self.a), r5v.canonical(self.b)
        self.d = intake.canonical(intake.candidate(self.ar, self.br))
        self.x = lab.make_candidate(self.d, self.gpin, 1000)
        self.p = self.root / "receipt.json"
        self.raw = lab.canonical(self.x)
        self.p.write_bytes(self.raw)
        os.chmod(self.p, 0o600)
        self.sig = lab.sign_local(self.p, self.guardian)

    def tearDown(self):
        self.t.cleanup()

    def verify(self, raw=None, **kwargs):
        return lab.verify(raw if raw is not None else self.raw,
                          self.d, self.ar, self.br, 1100,
                          signature_path=kwargs.pop("signature_path", self.sig),
                          allowed_signers=kwargs.pop("allowed_signers", self.guardian_allow),
                          public_key=kwargs.pop("public_key", self.guardian_pub),
                          **kwargs)

    def test_17_real_ssh_ed25519_signed_receipt_valid_not_external(self):
        self.assertNotEqual(self.a["pid"], self.b["pid"])
        self.assertEqual(self.a["uid"], self.b["uid"])
        self.assertEqual(self.a["euid"], self.b["euid"])
        self.assertNotEqual(self.opin, self.gpin)
        out = self.verify()
        self.assertEqual(out["reason"], "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN")
        self.assertTrue(out["cryptographic_signature_valid"])
        self.assertFalse(out["signed_receipt_is_external_authority"])

    def test_18_actual_modified_signed_body_real_crypto_rejected(self):
        changed = lab.canonical(dict(self.x, claimed_admin_domain="lab-fake-domain"))
        out = self.verify(raw=changed)
        self.assertEqual(out["reason"], "SIGNED_PROVENANCE_EVIDENCE_QUARANTINED")
        self.assertIn("REAL_ED25519_SSHSIG_INVALID", out["detail"])

    def test_19_actual_wrong_allowed_signer_crypto_rejected(self):
        out = self.verify(allowed_signers=self.owner_allow)
        self.assertEqual(out["reason"], "SIGNED_PROVENANCE_EVIDENCE_QUARANTINED")
        self.assertIn("REAL_ED25519_SSHSIG_INVALID", out["detail"])

    def test_20_real_wrong_ssh_signature_namespace_rejected(self):
        wrongfile = self.root / "wrong_namespace.json"
        wrongfile.write_bytes(self.raw)
        os.chmod(wrongfile, 0o600)
        sig = lab.sign_local(wrongfile, self.guardian, "nordoi-r5x-wrong")
        out = self.verify(signature_path=sig)
        self.assertEqual(out["reason"], "SIGNED_PROVENANCE_EVIDENCE_QUARANTINED")

    def test_21_real_signed_local_oob_root_claim_is_no_authority(self):
        self.x["claimed_owner_oob_root_sha256"] = self.opin
        self.x["claimed_host_attestation_sha256"] = self.gpin
        path = self.root / "claim.json"
        raw = lab.canonical(self.x)
        path.write_bytes(raw)
        os.chmod(path, 0o600)
        sig = lab.sign_local(path, self.guardian)
        out = self.verify(raw=raw, signature_path=sig)
        self.assertEqual(out["reason"], "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN")
        self.assertTrue(out["locally_declared_root_not_independently_authenticated"])

    def test_22_genuine_signed_receipt_local_replay_quarantined(self):
        out = self.verify(local_spent_challenges=frozenset([self.x["challenge"]]))
        self.assertEqual(out["reason"], "LOCAL_NONCE_REPLAY_BLOCKED")
        self.assertFalse(out["global_spent_nonce_proven"])

    def test_23_genuine_signed_old_remote_floor_vs_local_floor_rejected(self):
        out = self.verify(local_revocation_floor=9)
        self.assertEqual(out["reason"], "LOCAL_REVOCATION_FLOOR_REGRESSION")
        self.assertEqual(out["global_monotonic_revocation"], "NOT_VERIFIED")

    def test_24_full_native_ssh_and_os_lab_returns_exit_two(self):
        p = subprocess.run([sys.executable, "-B", str(Path(lab.__file__)),
                            "--exercise"], capture_output=True, text=True,
                           timeout=150, check=False)
        self.assertEqual(p.returncode, 2, p.stderr)
        result = json.loads(p.stdout)
        self.assertEqual(result["reason"],
                         "REAL_SIGNED_SAME_ADMIN_PROVENANCE_NEGATIVE_CONTROLS_PASS")
        self.assertEqual(result["native_admission"], "BLOCK")
        self.assertEqual(result["native_admission_gates_unmet"], 12)
        self.assertFalse(result["native_authority_granted"])

if __name__ == "__main__":
    unittest.main()
