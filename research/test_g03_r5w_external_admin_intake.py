"""NORDOI G0.3 R5W: 24 fail-closed external custody intake cases.

16 portable structural/source/attestation controls and eight genuine POSIX
OpenSSH cases. Local signatures and self-declared remote identifiers NEVER
qualify legal or operational independence; no admission path exists.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r5w_external_admin_intake as lab
import g03_r5v_admin_separation_evidence as r5v
import g03_r5t_signed_receipt_provenance as signed
import g03_r5u_key_rotation_custody as rotate

class PortableExternalIntake(unittest.TestCase):
    def setUp(self):
        self.owner = r5v.observe("owner_proxy", "lab-owner-domain", "a" * 64,
                                 0, "b" * 64, "c" * 64)
        self.custodian = dict(self.owner)
        self.custodian.update(role="custodian_proxy",
                              claimed_domain="lab-guardian-domain",
                              pid=self.owner["pid"] + 1, key_sha256="d" * 64)
        self.oraw = r5v.canonical(self.owner)
        self.craw = r5v.canonical(self.custodian)
        self.d = lab.candidate(self.oraw, self.craw)

    def assess(self, d=None, owner=None, custodian=None):
        r = lab.inspect(lab.canonical(self.d if d is None else d),
                        self.oraw if owner is None else owner,
                        self.craw if custodian is None else custodian)
        self.assertEqual(r["status"], "BLOCK")
        self.assertEqual(r["admission"], "NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        self.assertEqual(r["native_admission_gates_unmet"], 12)
        return r

    def rejected(self, data):
        with self.assertRaises(lab.Refused):
            lab.parse(data)

    def test_01_protocol_source_operation_frozen(self):
        self.assertEqual(self.d["parent"], r5v.__dict__["PARENT"] if False else lab.PARENT)
        self.assertEqual(self.d["protocol"], lab.PROTOCOL)
        self.assertEqual(self.d["operation"], lab.OPERATION)
        self.assertEqual(set(self.d), lab.FIELDS)

    def test_02_reports_bound_by_exact_bytes_and_key_history(self):
        self.assertEqual(self.d["owner_report_sha256"], lab.digest(self.oraw))
        self.assertEqual(self.d["custodian_report_sha256"], lab.digest(self.craw))
        self.assertEqual(self.d["history_head"], "b" * 64)
        self.assertEqual(self.d["challenge"], "c" * 64)

    def test_03_true_same_admin_report_is_never_external(self):
        self.assertEqual(self.assess()["reason"],
                         "NO_INDEPENDENT_EXTERNALLY_AUTHENTICATED_EVIDENCE")

    def test_04_hypothetical_distinct_uid_does_not_certify_custody(self):
        if os.name == "nt":
            self.assertEqual(self.assess()["native_admission_gates_unmet"], 12)
        else:
            alt = dict(self.custodian)
            alt["uid"] += 1
            alt["euid"] += 1
            craw = r5v.canonical(alt)
            d = lab.candidate(self.oraw, craw)
            self.assertEqual(self.assess(d, custodian=craw)["reason"],
                             "NO_INDEPENDENT_EXTERNALLY_AUTHENTICATED_EVIDENCE")

    def test_05_duplicate_json_keys_quarantined(self):
        raw = lab.canonical(self.d)
        self.rejected(raw.replace(b'"protocol":', b'"protocol":"forged","protocol":'))

    def test_06_extra_or_missing_json_field_refused(self):
        self.rejected(lab.canonical(dict(self.d, native_grant=True)))
        x = dict(self.d)
        del x["challenge"]
        self.rejected(lab.canonical(x))

    def test_07_noncanonical_pretty_and_no_newline_refused(self):
        self.rejected(json.dumps(self.d, indent=2).encode())
        self.rejected(lab.canonical(self.d).rstrip(b"\n"))

    def test_08_oversized_invalid_utf8_or_nonfinite_refused(self):
        for data in (b"x" * (lab.MAX_BYTES + 1), b"\xff",
                     lab.canonical(self.d).replace(b'"key_epoch":0',
                                                    b'"key_epoch":NaN')):
            self.rejected(data)

    def test_09_wrong_frozen_parent_and_project_refused(self):
        for key, value in (("parent", "0" * 40), ("project", "other/repo"),
                           ("protocol", "UNKNOWN")):
            self.rejected(lab.canonical(dict(self.d, **{key: value})))

    def test_10_grant_or_enrollment_operation_rejected(self):
        for name in ("GRANT", "ENROLL_EXTERNAL_GUARDIAN", "AUTHORIZE_PRODUCTION"):
            self.rejected(lab.canonical(dict(self.d, operation=name)))

    def test_11_changed_report_bytes_without_manifest_change(self):
        b = dict(self.custodian, claimed_domain="lab-spoofed-domain")
        self.assertEqual(self.assess(custodian=r5v.canonical(b))["reason"],
                         "EXTERNAL_INTAKE_QUARANTINED")

    def test_12_forged_challenge_or_history_digest_refused(self):
        for key in ("challenge", "history_head"):
            d = dict(self.d, **{key: "f" * 64})
            self.assertEqual(self.assess(d)["reason"], "EXTERNAL_INTAKE_QUARANTINED")

    def test_13_forged_report_signer_pin_or_epoch_quarantined(self):
        for key, val in (("owner_key_sha256", "f" * 64),
                         ("custodian_key_sha256", "e" * 64),
                         ("key_epoch", 1)):
            self.assertEqual(self.assess(dict(self.d, **{key: val}))["reason"],
                             "EXTERNAL_INTAKE_QUARANTINED")

    def test_14_self_asserted_external_root_and_floors_never_grant(self):
        for key, val in (("owner_oob_root_pin", "e" * 64),
                         ("guardian_oob_key_pin", "f" * 64),
                         ("independent_os_attestation_sha256", "e" * 64),
                         ("external_history_anchor_sha256", "f" * 64),
                         ("quorum_attestation_sha256", "e" * 64),
                         ("external_key_epoch_floor", 1),
                         ("external_revocation_floor", 2),
                         ("external_spent_nonce_floor", 3)):
            r = self.assess(dict(self.d, **{key: val}))
            self.assertEqual(r["reason"],
                             "SELF_ASSERTED_EXTERNAL_PINS_AND_FLOORS_NOT_AUTHORITY")
            self.assertTrue(r["self_declared_external_fields_quarantined"])

    def test_15_invalid_external_pin_floor_and_bool_refused(self):
        for key, val in (("owner_oob_root_pin", "not-a-pin"),
                         ("external_key_epoch_floor", -1),
                         ("external_revocation_floor", True),
                         ("external_spent_nonce_floor", "5"),
                         ("key_epoch", True)):
            self.rejected(lab.canonical(dict(self.d, **{key: val})))

    def test_16_mismatched_owner_custodian_history_preflight_fails(self):
        bad = dict(self.custodian, nonce="f" * 64)
        with self.assertRaises(lab.Refused):
            lab.candidate(self.oraw, r5v.canonical(bad))
        self.assertEqual(self.assess(custodian=r5v.canonical(bad))["reason"],
                         "EXTERNAL_INTAKE_QUARANTINED")

@unittest.skipUnless(os.name != "nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "genuine native POSIX OS identity and OpenSSH required")
class NativeExternalIntake(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(
            prefix="nordoi-r5l-r5q-r5s-r5t-r5u-r5w-native-")
        self.root = Path(self.t.name)
        os.chmod(self.root, 0o700)
        self.ok, self.oa = signed.create_same_admin_key(self.root, "r5w_owner")
        self.ck, self.ca = signed.create_same_admin_key(self.root, "r5w_guardian")
        self.opub = self.root / "r5w_owner.pub"
        self.cpub = self.root / "r5w_guardian.pub"
        self.opin = rotate.public_sha(self.opub)
        self.cpin = rotate.public_sha(self.cpub)
        self.a = r5v.child_observe("owner_proxy", "lab-owner-domain",
                                   self.opin, 0, "b" * 64, "c" * 64)
        self.b = r5v.child_observe("custodian_proxy", "lab-guardian-domain",
                                   self.cpin, 0, "b" * 64, "c" * 64)
        self.ar, self.br = r5v.canonical(self.a), r5v.canonical(self.b)
        self.af, self.bf = self.root / "owner.json", self.root / "candidate.json"
        self.af.write_bytes(self.ar)
        self.bf.write_bytes(self.br)
        os.chmod(self.af, 0o600)
        os.chmod(self.bf, 0o600)
        self.asig = r5v.signature(self.af, self.ok)
        self.bsig = r5v.signature(self.bf, self.ck)
        self.d = lab.candidate(self.ar, self.br)

    def tearDown(self):
        self.t.cleanup()

    def test_17_real_two_processes_same_uid_euid_with_distinct_keys(self):
        self.assertNotEqual(self.a["pid"], self.b["pid"])
        self.assertEqual(self.a["uid"], self.b["uid"])
        self.assertEqual(self.a["euid"], self.b["euid"])
        self.assertNotEqual(self.opin, self.cpin)
        self.assertEqual(lab.inspect(lab.canonical(self.d), self.ar, self.br)["status"],
                         "BLOCK")

    def test_18_real_both_local_ed25519_signatures_valid_not_external(self):
        for path, sig, allow, public, pin in (
            (self.af, self.asig, self.oa, self.opub, self.opin),
            (self.bf, self.bsig, self.ca, self.cpub, self.cpin),
        ):
            r = r5v.verify(path, sig, allow, public, pin)
            self.assertEqual(r["reason"], "REAL_SIGNED_LOCAL_OS_OBSERVATION_NOT_EXTERNAL")
            self.assertTrue(r["signature_valid_for_local_key"])
            self.assertFalse(r["independently_authenticated_signer"])

    def test_19_real_signature_changed_report_is_rejected(self):
        self.bf.write_bytes(r5v.canonical(dict(self.b, ppid=self.b["ppid"] + 1)))
        r = r5v.verify(self.bf, self.bsig, self.ca, self.cpub, self.cpin)
        self.assertEqual(r["reason"], "OS_REPORT_SIGNATURE_QUARANTINED")

    def test_20_wrong_real_signer_allowlist_refused(self):
        r = r5v.verify(self.bf, self.bsig, self.oa, self.cpub, self.cpin)
        self.assertEqual(r["reason"], "OS_REPORT_SIGNATURE_QUARANTINED")

    def test_21_real_signed_local_root_pin_does_not_enroll_guardian(self):
        d = dict(self.d, guardian_oob_key_pin=self.cpin,
                 owner_oob_root_pin=self.opin)
        r = lab.inspect(lab.canonical(d), self.ar, self.br)
        self.assertEqual(r["reason"],
                         "SELF_ASSERTED_EXTERNAL_PINS_AND_FLOORS_NOT_AUTHORITY")
        self.assertEqual(r["native_admission_gates_unmet"], 12)

    def test_22_real_signed_local_reports_with_unprotected_floor_denied(self):
        self.assertEqual(r5v.verify(self.af, self.asig, self.oa, self.opub,
                                    self.opin)["status"], "BLOCK")
        r = lab.inspect(lab.canonical(dict(self.d, external_revocation_floor=9)),
                        self.ar, self.br)
        self.assertEqual(r["status"], "BLOCK")
        self.assertEqual(r["global_monotonic_antirollback"], "NOT_VERIFIED")

    def test_23_actual_second_process_replay_challenge_mismatch_rejected(self):
        other = r5v.child_observe("custodian_proxy", "lab-guardian-domain",
                                  self.cpin, 0, "b" * 64, "d" * 64)
        raw = r5v.canonical(other)
        r = lab.inspect(lab.canonical(self.d), self.ar, raw)
        self.assertEqual(r["reason"], "EXTERNAL_INTAKE_QUARANTINED")
        self.assertFalse(r["native_authority_granted"])

    def test_24_cli_actual_native_crypto_exercise_exit_denial(self):
        p = subprocess.run([sys.executable, "-B", str(Path(lab.__file__)),
                            "--exercise"], capture_output=True, text=True,
                           timeout=180, check=False)
        self.assertEqual(p.returncode, 2, p.stderr)
        d = json.loads(p.stdout)
        self.assertEqual(d["reason"],
                         "REAL_SIGNED_LOCAL_CUSTODY_EXTERNAL_INTAKE_DENIED")
        self.assertEqual(d["genuine_local_ssh_ed25519"],
                         "TWO_VALID_SAME_ADMIN_SIGNATURES")
        self.assertEqual(d["native_admission_gates_unmet"], 12)
        self.assertEqual(d["native_admission"], "BLOCK")

if __name__ == "__main__":
    unittest.main()
