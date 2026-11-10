"""NORDOI R5B: 24 real negative controls, including native triple Ed25519 SSHSIG.

Mac tests generate real disposable SSH keys only in an isolated test directory.
No real third-party authority, owner-OOB root or monotonic witness is simulated
as operational. Every independent-custody admission remains BLOCK.
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
import g03_r5b_triple_sshsig_verifier as gate

DUMMY_NONCE = "a" * 64
DUMMY_R4Z = "b" * 64
TIME = 1800000000

def canonical(obj):
    return gate.canonical(obj)

def payload(**updates):
    out = dict(
        protocol=gate.PROTOCOL, project=gate.PROJECT, source_head=gate.SOURCE_HEAD,
        r4z_request_sha256=DUMMY_R4Z, owner_key_sha256="1"*64,
        guardian_key_sha256="2"*64, witness_key_sha256="3"*64,
        nonce=DUMMY_NONCE, issued_unix=TIME, expires_unix=TIME+120, floor_epoch=4
    )
    out.update(updates)
    return out

class PortableDenials(unittest.TestCase):
    def test_01_empty_bundle_denied(self):
        report = gate.assessment(b"", b"", DUMMY_NONCE, 4, now=TIME)
        self.assertEqual(report["status"], "BLOCK")
        self.assertEqual(report["admission"], "NOT_AUTHORIZED")

    def test_02_non_utf8_denied(self):
        with self.assertRaises(gate.InvalidEvidence):
            gate.parse_json(b"\xff")

    def test_03_noncanonical_json_denied(self):
        with self.assertRaises(gate.InvalidEvidence):
            gate.parse_json(json.dumps({"foo": 1}, indent=2).encode())

    def test_04_duplicate_json_keys_denied(self):
        with self.assertRaises(gate.InvalidEvidence):
            gate.parse_json(b'{"foo":1,"foo":2}\n')

    def test_05_oversized_input_denied(self):
        with self.assertRaises(gate.InvalidEvidence):
            gate.parse_json(b"x" * (gate.MAX_BYTES + 1))

    def test_06_symlink_and_nonregular_input_denied(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "file"
            target.write_bytes(b"123")
            link = Path(tmp) / "link"
            try:
                link.symlink_to(target)
            except OSError:
                pass
            else:
                with self.assertRaises(gate.InvalidEvidence):
                    gate.read_regular(link)
            if hasattr(os, "mkfifo"):
                fifo = Path(tmp) / "fifo"
                try:
                    os.mkfifo(fifo)
                except OSError:
                    pass
                else:
                    with self.assertRaises(gate.InvalidEvidence):
                        gate.read_regular(fifo)

    def test_07_missing_provenance_is_block_with_exit_two(self):
        with tempfile.TemporaryDirectory() as tmp:
            command = [sys.executable, "-B", str(Path(gate.__file__)),
                       "--bundle", str(Path(tmp) / "missing"),
                       "--expected-public-pins", str(Path(tmp) / "missing-pins"),
                       "--expected-nonce", DUMMY_NONCE,
                       "--expected-epoch", "4"]
            result = subprocess.run(command, capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(json.loads(result.stdout)["admission"], "NOT_AUTHORIZED")

    def test_08_wrong_public_key_type_not_accepted(self):
        for value in ("ssh-rsa AAAA", "ssh-ed25519 AAAA", "ssh-ed25519 /../../", None):
            with self.assertRaises(gate.InvalidEvidence):
                gate.key_canonical(value)

@unittest.skipUnless(sys.platform == "darwin", "native macOS SSHSIG required")
class NativeThreeRoleSSHSIG(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory(prefix="nordoi-r5b-native-")
        cls.dir = Path(cls.temporary.name)
        cls.priv = {}
        cls.pub = {}
        cls.pins = {}
        for role in gate.ROLES:
            path = cls.dir / role
            result = subprocess.run(
                ["/usr/bin/ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(path)],
                capture_output=True, check=False, timeout=20
            )
            if result.returncode:
                raise AssertionError("native keygen failed for " + role)
            cls.priv[role] = path
            parts = Path(str(path) + ".pub").read_text("ascii").split()
            cls.pub[role] = parts[0] + " " + parts[1]
            cls.pins[role] = gate.digest(cls.pub[role].encode("ascii"))

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def make_bundle(self, overrides=None):
        obj = payload(
            owner_key_sha256=self.pins["owner"],
            guardian_key_sha256=self.pins["guardian"],
            witness_key_sha256=self.pins["witness"]
        )
        if overrides:
            obj.update(overrides)
        msg = canonical(obj)
        signatures = {}
        with tempfile.TemporaryDirectory(prefix="nordoi-r5b-sign-") as temp:
            for role in gate.ROLES:
                target = Path(temp) / (role + ".txt")
                target.write_bytes(msg)
                result = subprocess.run(
                    ["/usr/bin/ssh-keygen", "-Y", "sign", "-f", str(self.priv[role]),
                     "-n", gate.NAMESPACES[role], str(target)],
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                    timeout=20, check=False
                )
                if result.returncode:
                    raise AssertionError("real native SSHSIG failed " + role)
                signatures[role] = base64.b64encode(
                    Path(str(target) + ".sig").read_bytes()
                ).decode("ascii")
        result = {"payload": obj, "public_keys": dict(self.pub), "signatures": signatures}
        return result

    def eval(self, data, *, nonce=DUMMY_NONCE, epoch=4, now=TIME, pins=None):
        if pins is None:
            pins = self.pins
        report = gate.assessment(canonical(data), canonical(pins), nonce, epoch, now=now)
        self.assertEqual(report["status"], "BLOCK")
        self.assertEqual(report["admission"], "NOT_AUTHORIZED")
        return report

    def bad(self, data, **kwargs):
        result = self.eval(data, **kwargs)
        self.assertEqual(result["cryptographic_verification"], "NOT_VERIFIED")
        return result

    def test_09_native_real_three_signatures_still_denied_custody(self):
        x = self.make_bundle()
        report = self.eval(x)
        self.assertEqual(report["cryptographic_verification"], "THREE_SSHSIG_VALID")
        self.assertEqual(report["administrative_separation"], "NOT_VERIFIED")

    def test_10_native_owner_root_substitution(self):
        x = self.make_bundle()
        pins = dict(self.pins, owner="e"*64)
        self.bad(x, pins=pins)

    def test_11_native_guardian_root_substitution(self):
        x = self.make_bundle()
        pins = dict(self.pins, guardian="d"*64)
        self.bad(x, pins=pins)

    def test_12_native_witness_root_substitution(self):
        x = self.make_bundle()
        pins = dict(self.pins, witness="c"*64)
        self.bad(x, pins=pins)

    def test_13_native_wrong_domain_signature_replay(self):
        x = self.make_bundle()
        x["signatures"]["owner"] = x["signatures"]["guardian"]
        self.bad(x)

    def test_14_native_payload_tamper_but_pins_intact(self):
        x = self.make_bundle()
        x["payload"]["r4z_request_sha256"] = "c"*64
        self.bad(x)

    def test_15_native_old_challenge_replay_denied(self):
        x = self.make_bundle()
        self.bad(x, nonce="f"*64)

    def test_16_native_expired_receipt_denied(self):
        x = self.make_bundle()
        self.bad(x, now=TIME+121)

    def test_17_native_future_receipt_denied(self):
        x = self.make_bundle()
        self.bad(x, now=TIME-31)

    def test_18_native_epoch_downgrade_denied(self):
        x = self.make_bundle()
        self.bad(x, epoch=5)

    def test_19_native_ttl_over_300_even_if_signed_denied(self):
        x = self.make_bundle({"expires_unix": TIME+301})
        self.bad(x)

    def test_20_native_missing_witness_signature_denied(self):
        x = self.make_bundle()
        x["signatures"]["witness"] = ""
        self.bad(x)

    def test_21_native_same_key_claimed_for_two_roles_denied(self):
        x = self.make_bundle()
        x["public_keys"]["witness"] = x["public_keys"]["guardian"]
        self.bad(x)

    def test_22_native_cross_packet_signature_replay_denied(self):
        a = self.make_bundle()
        b = self.make_bundle({"nonce": "c"*64})
        b["signatures"]["witness"] = a["signatures"]["witness"]
        self.bad(b, nonce="c"*64)

    def test_23_native_valid_all_signatures_cannot_enroll_oob_root(self):
        report = self.eval(self.make_bundle())
        self.assertEqual(report["out_of_band_root"], "NOT_ENROLLED")
        self.assertEqual(report["protected_monotonic_witness"], "NOT_IMPLEMENTED")

    def test_24_native_no_global_antirollback_or_durable_nonce_ledger(self):
        report = self.eval(self.make_bundle())
        self.assertEqual(report["durable_consumed_nonce_ledger"], "NOT_IMPLEMENTED")
        self.assertEqual(report["protected_verifier_recovery"], "NOT_VERIFIED")

if __name__ == "__main__":
    unittest.main()
