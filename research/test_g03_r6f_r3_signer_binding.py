"""G0.3 R6F BUREAU-001 R3 — 24 adversarial cross-boundary provenance tests.

16 portable strict signer/allowlist checks and eight real OpenSSH SSHSIG
negative controls on POSIX. An unbound valid signer is a *local provenance
attribution defect*, never a native admission (always BLOCK).
"""
import base64
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r6f_r3_signer_binding as lab

def public(comment=b"test-comment", raw_key=b"\x11"*32):
    blob = (b"\x00\x00\x00\x0bssh-ed25519\x00\x00\x00\x20"
            + raw_key)
    return b"ssh-ed25519 " + base64.b64encode(blob) + (
        b" " + comment if comment else b""
    ) + b"\n"

class PortableSignerBinding(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="nordoi-r6f-r3-fixture-")
        self.root = Path(self.tmp.name)
        if os.name != "nt":
            os.chmod(self.root, 0o700)
        self.pub = self.root / "expected.pub"
        self.allow = self.root / "allowed_signers"
        self.pub.write_bytes(public())
        self.allow.write_bytes(lab.expected_allowed_bytes(self.pub.read_bytes()))
        if os.name != "nt":
            os.chmod(self.allow, 0o600)
        self.pin = lab.keys.public_sha(self.pub)

    def tearDown(self):
        self.tmp.cleanup()

    def test_01_exact_real_ed25519_ssh_wire_type(self):
        x = lab.parse_ed25519_public(self.pub.read_bytes())
        self.assertTrue(x.startswith(b"ssh-ed25519 "))
        self.assertEqual(len(base64.b64decode(x.split()[1])), 51)

    def test_02_identity_allowlist_exact_format(self):
        self.assertEqual(self.allow.read_bytes(), (
            lab.ssh.IDENTITY.encode()+b" "+self.pub.read_bytes()
        ))

    def test_03_strict_public_to_allowlist_binding(self):
        self.assertTrue(lab.bind(self.pub, self.allow, self.pin))

    def test_04_mismatched_public_key_with_same_allowlist_refused(self):
        alternate = self.root/"alternate.pub"
        alternate.write_bytes(public(raw_key=b"\x22"*32))
        with self.assertRaises(lab.Refused):
            lab.bind(alternate, self.allow, lab.keys.public_sha(alternate))

    def test_05_wrong_exact_public_sha_pin_refused(self):
        with self.assertRaises(lab.Refused):
            lab.bind(self.pub, self.allow, "0"*64)

    def test_06_allowlist_one_additional_authorized_key_refused(self):
        with self.allow.open("ab") as f:
            f.write(lab.ssh.IDENTITY.encode()+b" "+public(raw_key=b"\x22"*32))
        with self.assertRaises(lab.Refused):
            lab.bind(self.pub, self.allow, self.pin)

    def test_07_replaced_allowed_signer_principal_refused(self):
        self.allow.write_bytes(b"other-principal "+self.pub.read_bytes())
        with self.assertRaises(lab.Refused):
            lab.bind(self.pub, self.allow, self.pin)

    def test_08_options_or_metadata_prepended_refused(self):
        self.allow.write_bytes(lab.ssh.IDENTITY.encode()+b" namespaces=\"x\" "+self.pub.read_bytes())
        with self.assertRaises(lab.Refused):
            lab.bind(self.pub, self.allow, self.pin)

    def test_09_invalid_public_key_algorithm_refused(self):
        with self.assertRaises(lab.Refused):
            lab.parse_ed25519_public(public().replace(b"ssh-ed25519 ",b"ssh-rsa ",1))

    def test_10_bad_base64_refused(self):
        with self.assertRaises(lab.Refused):
            lab.parse_ed25519_public(b"ssh-ed25519 !!!!\n")

    def test_11_extra_public_key_line_refused(self):
        with self.assertRaises(lab.Refused):
            lab.parse_ed25519_public(public()+public())

    def test_12_wrong_ssh_wire_type_refused(self):
        wire=b"\x00\x00\x00\x07ssh-rsa"+b"\x00\x00\x00\x20"+b"x"*32
        with self.assertRaises(lab.Refused):
            lab.parse_ed25519_public(b"ssh-ed25519 "+base64.b64encode(wire)+b"\n")

    def test_13_too_many_key_bytes_refused(self):
        with self.assertRaises(lab.Refused):
            lab.parse_ed25519_public(b"ssh-ed25519 "+b"A"*4096+b"\n")

    def test_14_crlf_and_nul_key_bytes_refused(self):
        for blob in (public().replace(b"\n",b"\r\n"),public()+b"\x00"):
            with self.assertRaises(lab.Refused):
                lab.parse_ed25519_public(blob)

    def test_15_symlink_allowlist_proof_refused(self):
        if os.name == "nt":
            self.skipTest("windows symlink privileges not presumed")
        link = self.root/"linked_allowed"
        link.symlink_to(self.allow.name)
        with self.assertRaises(lab.Refused):
            lab.bind(self.pub,link,self.pin)

    def test_16_no_claimed_native_authority_or_unlocked_r6g(self):
        d=lab.deny("PORTABLE_STRICT_BINDING_NO_EXTERNAL_AUTHORITY")
        self.assertEqual(d["status"],"BLOCK")
        self.assertEqual(d["admission"],"NOT_AUTHORIZED")
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["production"],"NOT_AUTHORIZED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "requires genuine POSIX OpenSSH and OS process signing")
class RealNativeSignerBinding(unittest.TestCase):
    def setUp(self):
        self.temp,self.data=lab.r6e.create_lab()

    def tearDown(self):
        self.temp.cleanup()

    def rogue(self):
        return lab.create_rogue_signed_receipt(self.data)

    def test_17_genuine_original_six_pinned_local_key_signatures(self):
        x=lab.assess_lab(self.data)
        self.assertEqual(x["reason"],"R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY")
        self.assertFalse(x["native_authority_granted"])
        self.assertEqual(x["native_admission_gates_unmet"],12)

    def test_18_real_rogue_ed25519_sshsig_old_attribution_gap(self):
        rogue_sig,rogue_allowed,rogue_pub=self.rogue()
        self.assertNotEqual(lab.keys.public_sha(rogue_pub),
                            lab.keys.public_sha(self.data["guardian_public"]))
        old=lab.r6e.assess_lab(
            self.data,local_signature=rogue_sig,local_allowed=rogue_allowed)
        self.assertEqual(old["reason"],
                         "SIX_VALID_LOCAL_SIGNATURES_NOT_OWNER_OOB_ENROLLMENT")
        self.assertEqual(old["admission"],"NOT_AUTHORIZED")

    def test_19_same_real_rogue_signature_refused_by_r3_fix(self):
        rogue_sig,rogue_allowed,_=self.rogue()
        x=lab.assess_lab(self.data,local_signature=rogue_sig,
                         local_allowed=rogue_allowed)
        self.assertEqual(x["reason"],"R3_SIGNATURE_ATTRIBUTION_QUARANTINED")
        self.assertIn("ALLOWLIST_NOT_EXACT_ONE_DECLARED_PINNED_ED25519_KEY",
                      x["detail"])

    def test_20_extra_subequivalent_local_signer_in_allowlist_refused(self):
        _,rogue_allowed,rogue_pub=self.rogue()
        extra=(self.data["root"]/"combined.allowed")
        extra.write_bytes(self.data["guardian_allowed"].read_bytes()
                          +rogue_allowed.read_bytes())
        os.chmod(extra,0o600)
        x=lab.assess_lab(self.data,local_allowed=extra)
        self.assertEqual(x["reason"],"R3_SIGNATURE_ATTRIBUTION_QUARANTINED")

    def test_21_wrong_owner_allowed_signer_refused(self):
        x=lab.assess_lab(self.data,owner_allowed=self.data["guardian_allowed"])
        self.assertEqual(x["reason"],"R3_SIGNATURE_ATTRIBUTION_QUARANTINED")

    def test_22_wrong_candidate_allowed_signer_refused(self):
        x=lab.assess_lab(self.data,candidate_allowed=self.data["owner_allowed"])
        self.assertEqual(x["reason"],"R3_SIGNATURE_ATTRIBUTION_QUARANTINED")

    def test_23_invalid_signature_still_refused_after_pinning(self):
        wrong=(self.data["root"]/"wrong_sig_r3")
        wrong.write_bytes(b"not a sshsig")
        os.chmod(wrong,0o600)
        x=lab.assess_lab(self.data,local_signature=wrong)
        self.assertNotEqual(x["reason"],"R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY")
        self.assertEqual(x["status"],"BLOCK")
        self.assertEqual(x["native_admission_gates_unmet"],12)

    def test_24_native_independent_exercise_proves_gap_and_fix(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),
                          "--exercise"],capture_output=True,text=True,timeout=240)
        self.assertEqual(p.returncode,2,p.stderr)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],"R3_GENUINE_ROGUE_SSHSIG_ATTRIBUTION_GAP_REPRODUCED_AND_BLOCKED")
        self.assertTrue(x["old_r6e_attribution_gap_reproduced"])
        self.assertTrue(x["corrected_r3_allowlist_binding"])
        self.assertEqual(x["native_admission_gates_unmet"],12)

if __name__=="__main__":
    unittest.main()
