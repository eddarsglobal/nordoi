"""NORDOI G0.3 R6A — 24 adversarial fresh external-witness pre-enrollment cases.

16 portable canonical/source/nonce/custody controls; eight genuine POSIX
OpenSSH Ed25519 native cases. Absolutely no external trust is enrolled.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r6a_oob_witness_challenge as lab
import g03_r5z_preserved_witness_rollback as witness
import g03_r5y_signed_nonce_rollback as ledger
import g03_r5x_signed_external_provenance as receipts
import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as identity
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as ssh

class PortableExternalWitness(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(prefix=lab.LAB_PREFIX)
        self.root = Path(self.t.name)
        if os.name != "nt":
            os.chmod(self.root, 0o700)
        owner = identity.observe("owner_proxy", "lab-owner-domain",
                                 "a" * 64, 0, "b" * 64, "c" * 64)
        candidate = dict(owner, role="custodian_proxy",
                         claimed_domain="lab-guardian-domain",
                         pid=owner["pid"] + 1, key_sha256="d" * 64)
        oraw, craw = identity.canonical(owner), identity.canonical(candidate)
        dossier = intake.canonical(intake.candidate(oraw, craw))
        self.receipt = receipts.canonical(receipts.make_candidate(
            dossier, "d" * 64, 1000))
        ledger.bootstrap(self.root)
        self.assertEqual(ledger.commit_local_denial(self.root, self.receipt,
                         1100, 1)["reason"], "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.wraw = witness.canonical(witness.candidate(self.root, self.receipt, 1100))
        self.x = lab.create_request(self.wraw, "d" * 64, 1100,
                                    challenge="e" * 64)
        self.raw = lab.canonical(self.x)

    def tearDown(self):
        self.t.cleanup()

    def verified(self, x=None, now=1150, witness_raw=None, **kwargs):
        result = lab.assess(lab.canonical(self.x if x is None else x),
                            self.wraw if witness_raw is None else witness_raw,
                            now, **kwargs)
        self.assertEqual(result["status"], "BLOCK")
        self.assertEqual(result["admission"], "NOT_AUTHORIZED")
        self.assertFalse(result["native_authority_granted"])
        self.assertEqual(result["native_admission_gates_unmet"], 12)
        return result

    def refused(self, raw):
        with self.assertRaises(lab.Refused):
            lab.parse(raw)

    def test_01_exact_frozen_source_and_deny_only_operation(self):
        self.assertEqual(self.x["parent"], lab.PARENT)
        self.assertEqual(self.x["protocol"], lab.PROTOCOL)
        self.assertEqual(self.x["operation"], lab.OPERATION)
        self.assertEqual(set(self.x), lab.FIELDS)

    def test_02_real_os_random_256_bit_challenges_are_distinct(self):
        a, b = lab.create_request(self.wraw, "d" * 64, 1100), lab.create_request(
            self.wraw, "d" * 64, 1100)
        self.assertEqual(len(a["challenge"]), 64)
        self.assertEqual(len(b["challenge"]), 64)
        self.assertNotEqual(a["challenge"], b["challenge"])
        self.assertNotEqual(a["challenge"], "e" * 64)

    def test_03_exact_witness_sha_height_nonce_digest_floor(self):
        w = witness.parse(self.wraw)
        self.assertEqual(self.x["source_witness_sha256"], ledger.sha(self.wraw))
        self.assertEqual(self.x["source_witness_height"], w["local_height"])
        self.assertEqual(self.x["source_nonce_set_sha256"], w["nonce_set_sha256"])
        self.assertEqual(self.x["source_local_revocation_floor"], 1)
        self.assertEqual(self.x["source_receipt_sha256"], w["source_receipt_sha256"])

    def test_04_duplicate_or_noncanonical_json_rejected(self):
        self.refused(self.raw.replace(b'"protocol":',
                       b'"protocol":"forged","protocol":'))
        self.refused(json.dumps(self.x, indent=2).encode())
        self.refused(self.raw.rstrip(b"\n"))

    def test_05_extra_missing_field_or_invalid_utf8_rejected(self):
        self.refused(lab.canonical(dict(self.x, native_grant=True)))
        d = dict(self.x)
        del d["source_witness_sha256"]
        self.refused(lab.canonical(d))
        self.refused(b"\xff")
        self.refused(b"x" * (lab.MAX_BYTES + 1))

    def test_06_bad_source_grant_operation_nonfinite_rejected(self):
        for k, value in (("project", "forged/repo"),
                         ("parent", "0" * 40), ("operation", "GRANT"),
                         ("protocol", "FAKE")):
            self.refused(lab.canonical(dict(self.x, **{k: value})))
        self.refused(self.raw.replace(b'"issued_at_local":1100',
                      b'"issued_at_local":NaN'))

    def test_07_malformed_digest_and_root_pin_rejected(self):
        for k, value in (("challenge", "x"), ("source_witness_sha256", "0" * 40),
                         ("candidate_signer_key_sha256", "z" * 64),
                         ("asserted_owner_oob_root_sha256", "wrong"),
                         ("asserted_external_history_anchor_sha256", "not-a-hash")):
            self.refused(lab.canonical(dict(self.x, **{k: value})))

    def test_08_bool_float_negative_counter_rejected(self):
        for k, value in (("source_witness_height", True),
                         ("source_witness_height", -1),
                         ("source_local_revocation_floor", -2),
                         ("issued_at_local", True),
                         ("asserted_external_spent_nonce_floor", 0.5),
                         ("asserted_external_revocation_floor", -1)):
            self.refused(lab.canonical(dict(self.x, **{k: value})))

    def test_09_zero_or_excessive_local_timebox_refused(self):
        for ttl in (0, 301, True, -2):
            with self.assertRaises(lab.Refused):
                lab.create_request(self.wraw, "d" * 64, 1100, ttl=ttl)
        self.refused(lab.canonical(dict(self.x, expires_at_local=999)))

    def test_10_stale_and_future_challenge_not_externally_fresh(self):
        for now in (1099, 1281):
            self.assertEqual(self.verified(now=now)["reason"],
                             "LOCAL_UNTRUSTED_CHALLENGE_TIMEBOX_STALE_OR_FUTURE")

    def test_11_same_local_spent_challenge_refused_not_global(self):
        result = self.verified(locally_spent=frozenset(["e" * 64]))
        self.assertEqual(result["reason"], "LOCALLY_SPENT_CHALLENGE_REPLAY_BLOCKED")
        self.assertFalse(result["globally_protected_spent_nonce"])

    def test_12_unsigned_structural_request_cannot_authenticate_guardian(self):
        result = self.verified()
        self.assertEqual(result["reason"], "EXTERNAL_WITNESS_SIGNATURE_OR_ROOT_MISSING")
        self.assertTrue(result["structural_witness_binding"])

    def test_13_self_asserted_owner_root_domain_anchor_floor_quarantined(self):
        for k, value in (
            ("asserted_owner_oob_root_sha256", "a" * 64),
            ("asserted_guardian_admin_domain", "lab-false-separate-domain"),
            ("asserted_external_history_anchor_sha256", "f" * 64),
            ("asserted_external_revocation_floor", 99),
            ("asserted_external_spent_nonce_floor", 100),
        ):
            result = self.verified(dict(self.x, **{k: value}))
            self.assertEqual(result["reason"],
                             "SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_AUTHENTICATED")
            self.assertTrue(result["claimed_external_fields_quarantined"])

    def test_14_tampered_witness_source_digest_or_floor_quarantined(self):
        for k, v in (("source_witness_sha256", "f" * 64),
                     ("source_witness_height", 2),
                     ("source_nonce_set_sha256", "e" * 64),
                     ("source_local_revocation_floor", 4),
                     ("source_receipt_sha256", "f" * 64)):
            self.assertEqual(self.verified(dict(self.x, **{k: v}))["reason"],
                             "EXTERNAL_WITNESS_CHALLENGE_QUARANTINED")

    def test_15_malformed_local_used_nonce_register_quarantined(self):
        for val in (["e" * 64], frozenset(["fake"]), frozenset([12])):
            self.assertEqual(self.verified(locally_spent=val)["reason"],
                             "EXTERNAL_WITNESS_CHALLENGE_QUARANTINED")

    def test_16_locally_advanced_history_still_no_external_anchor(self):
        next_receipt = receipts.canonical(dict(receipts.parse(self.receipt),
                                               challenge="f" * 64))
        ledger.commit_local_denial(self.root, next_receipt, 1100, 2)
        self.assertEqual(witness.compare(self.root, self.wraw)["reason"],
                         "PRIMARY_AHEAD_OF_OLDER_LOCAL_WITNESS")
        self.assertEqual(self.verified()["reason"],
                         "EXTERNAL_WITNESS_SIGNATURE_OR_ROOT_MISSING")

@unittest.skipUnless(os.name != "nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "Actual POSIX OpenSSH Ed25519 signature verification required")
class NativeExternalWitness(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(prefix=lab.LAB_PREFIX)
        self.root = Path(self.t.name)
        os.chmod(self.root, 0o700)
        self.ok, self.oa = ssh.create_same_admin_key(self.root, "r6a_owner")
        self.sk, self.allow = ssh.create_same_admin_key(self.root, "r6a_candidate")
        self.op = keys.public_sha(self.root / "r6a_owner.pub")
        self.sp = keys.public_sha(self.root / "r6a_candidate.pub")
        self.public = self.root / "r6a_candidate.pub"
        self.assertNotEqual(self.op, self.sp)
        a = identity.child_observe("owner_proxy", "lab-owner-domain",
                                   self.op, 0, "b" * 64, "c" * 64)
        b = identity.child_observe("custodian_proxy", "lab-guardian-domain",
                                   self.sp, 0, "b" * 64, "c" * 64)
        self.assertNotEqual(a["pid"], b["pid"])
        self.assertEqual(a["uid"], b["uid"])
        self.assertEqual(a["euid"], b["euid"])
        ar, br = identity.canonical(a), identity.canonical(b)
        d = intake.canonical(intake.candidate(ar, br))
        signed_receipt = receipts.canonical(receipts.make_candidate(d, self.sp, 1000))
        rp = self.root / "r6a_source_receipt.json"
        rp.write_bytes(signed_receipt)
        os.chmod(rp, 0o600)
        rs = receipts.sign_local(rp, self.sk)
        verified = receipts.verify(signed_receipt, d, ar, br, 1100, rs,
                                   self.allow, self.public)
        self.assertEqual(verified["reason"],
                         "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN")
        ledger.bootstrap(self.root)
        ledger.commit_local_denial(self.root, signed_receipt, 1100, 1)
        self.wraw = witness.canonical(witness.candidate(
            self.root, signed_receipt, 1100))
        wp = witness.save_witness(self.root, self.wraw, witness.WITNESS)
        w_sig = witness.sign_local(wp, self.sk)
        self.assertTrue(witness.verify_local(
            self.wraw, w_sig, self.allow)["cryptographic_signature_valid"])
        self.x = lab.create_request(self.wraw, self.sp, 1100,
                                    challenge="e" * 64)
        self.raw = lab.canonical(self.x)
        self.p = self.root / "r6a_challenge.json"
        self.p.write_bytes(self.raw)
        os.chmod(self.p, 0o600)
        self.sig = lab.sign_local(self.p, self.sk)

    def tearDown(self):
        self.t.cleanup()

    def assess(self, raw=None, now=1200, signature=None, allowed=None,
               public_key=None, **kwargs):
        return lab.assess(self.raw if raw is None else raw,
                          self.wraw, now,
                          self.sig if signature is None else signature,
                          self.allow if allowed is None else allowed,
                          self.public if public_key is None else public_key,
                          **kwargs)

    def test_17_actual_signed_source_witness_and_candidate_no_authority(self):
        out = self.assess()
        self.assertEqual(out["reason"], "REAL_SSHSIG_VALID_LOCAL_ONLY_NO_EXTERNAL_CUSTODY")
        self.assertTrue(out["cryptographic_signature_valid"])
        self.assertFalse(out["signer_is_independently_authenticated"])
        self.assertEqual(out["native_admission_gates_unmet"], 12)

    def test_18_modified_candidate_body_real_signature_refused(self):
        changed = lab.canonical(dict(self.x, expires_at_local=1270))
        out = self.assess(raw=changed)
        self.assertEqual(out["reason"], "EXTERNAL_WITNESS_CHALLENGE_QUARANTINED")
        self.assertIn("ACTUAL_OPENSSH_SIGNED_CANDIDATE_INVALID", out["detail"])

    def test_19_wrong_real_signer_allowlist_refused(self):
        out = self.assess(allowed=self.oa)
        self.assertEqual(out["reason"], "EXTERNAL_WITNESS_CHALLENGE_QUARANTINED")
        self.assertIn("ACTUAL_OPENSSH_SIGNED_CANDIDATE_INVALID", out["detail"])

    def test_20_wrong_ssh_signature_namespace_refused(self):
        wrong = self.root / "r6a_wrong_namespace.json"
        wrong.write_bytes(self.raw)
        os.chmod(wrong, 0o600)
        badsig = lab.sign_local(wrong, self.sk, "nordoi-r6a-wrong")
        out = self.assess(signature=badsig)
        self.assertEqual(out["reason"], "EXTERNAL_WITNESS_CHALLENGE_QUARANTINED")

    def test_21_fake_oob_claim_even_with_genuine_signature_quarantined(self):
        fake = dict(self.x, asserted_owner_oob_root_sha256=self.op,
                    asserted_guardian_admin_domain="lab-fake-external",
                    asserted_external_history_anchor_sha256=self.sp)
        path = self.root / "r6a_false_oob.json"
        raw = lab.canonical(fake)
        path.write_bytes(raw)
        os.chmod(path, 0o600)
        sig = lab.sign_local(path, self.sk)
        out = self.assess(raw=raw, signature=sig)
        self.assertEqual(out["reason"],
                         "SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_AUTHENTICATED")

    def test_22_local_spent_challenge_untrusted_denial_not_global(self):
        out = self.assess(locally_spent=frozenset([self.x["challenge"]]))
        self.assertEqual(out["reason"], "LOCALLY_SPENT_CHALLENGE_REPLAY_BLOCKED")
        self.assertFalse(out["globally_protected_spent_nonce"])

    def test_23_real_signed_untrusted_local_clock_rejects_future_or_stale(self):
        for now in (1099, 1281):
            self.assertEqual(self.assess(now=now)["reason"],
                             "LOCAL_UNTRUSTED_CHALLENGE_TIMEBOX_STALE_OR_FUTURE")

    def test_24_full_native_os_crypto_exercise_exit_two(self):
        p = subprocess.run([sys.executable, "-B", str(Path(lab.__file__)),
                            "--exercise"], capture_output=True, text=True,
                           timeout=240, check=False)
        self.assertEqual(p.returncode, 2, p.stderr)
        result = json.loads(p.stdout)
        self.assertEqual(result["reason"],
                         "REAL_R5Z_WITNESS_OOB_CHALLENGE_NEGATIVE_CONTROL_QUALIFIED")
        self.assertEqual(result["actual_fresh_os_random_challenge"],
                         "256_BIT_VALUE")
        self.assertEqual(result["independently_verified_custodian"], "NONE")
        self.assertEqual(result["native_admission"], "BLOCK")

if __name__ == "__main__":
    unittest.main()
