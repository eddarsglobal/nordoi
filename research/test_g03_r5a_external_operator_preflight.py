"""R5A: 24 adversarial tests of actual parser, input and deny-only custody boundary."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r5a_external_operator_preflight as gate

def full_candidate():
    return {
        "publisher_controller": "eddarsglobal",
        "operator_controller": "third-party-external",
        "witness_controller": "third-party-witness",
        "operator_identity_evidence_sha256": "1" * 64,
        "candidate_public_key_sha256": "2" * 64,
        "r4z_request_binding_sha256": "3" * 64,
        "owner_oob_receipt_sha256": "4" * 64,
        "protected_witness_head_sha256": "5" * 64,
        "witness_epoch": 4,
        "revocation_policy_sha256": "6" * 64,
        "recovery_policy_sha256": "7" * 64,
    }

def manifest(candidate=None):
    return {"protocol": gate.SCHEMA, "project": gate.PROJECT,
            "r4z_head": gate.R4Z_HEAD, "candidate": candidate}

def blob(candidate=None, **changes):
    obj = manifest(candidate)
    obj.update(changes)
    return json.dumps(obj, separators=(",", ":"), sort_keys=True).encode()

class ExternalCustodyDenialTests(unittest.TestCase):
    def blocked(self, data):
        result = gate.assess(data)
        self.assertEqual(result["status"], "BLOCK")
        self.assertEqual(result["admission"], "NOT_AUTHORIZED")
        return result

    def bad(self, data):
        result = self.blocked(data)
        self.assertEqual(result["reason"], "UNTRUSTED_EVIDENCE_INVALID")
        return result

    def test_01_no_candidate_is_truly_unenrolled(self):
        self.assertEqual(self.blocked(blob())["reason"], "NO_REAL_EXTERNAL_CUSTODIAN_EVIDENCE")

    def test_02_full_independent_claims_are_not_real_authority(self):
        result = self.blocked(blob(full_candidate()))
        self.assertEqual(result["reason"], "INDEPENDENT_CUSTODY_NOT_EXTERNALLY_AUTHENTICATED")
        self.assertEqual(result["owner_oob_root"], "NOT_VERIFIED")
        self.assertEqual(result["independent_witness"], "NOT_VERIFIED")

    def test_03_operator_same_publisher_casefold_rejected(self):
        c = full_candidate()
        c["operator_controller"] = "EDDARSGLOBAL"
        self.assertIn("COLLISION", self.bad(blob(c))["detail"])

    def test_04_witness_same_publisher_rejected(self):
        c = full_candidate()
        c["witness_controller"] = "eddarsglobal"
        self.bad(blob(c))

    def test_05_witness_same_operator_rejected(self):
        c = full_candidate()
        c["witness_controller"] = c["operator_controller"]
        self.bad(blob(c))

    def test_06_owner_controller_spoof_rejected(self):
        c = full_candidate()
        c["publisher_controller"] = "forged-publisher"
        self.bad(blob(c))

    def test_07_name_difference_is_not_administrative_independence(self):
        c = full_candidate()
        c["operator_controller"] = "other-account-name"
        self.assertEqual(self.blocked(blob(c))["external_operator"], "NOT_VERIFIED")

    def test_08_invalid_identity_reference_rejected(self):
        for name in ("", "../../secret", "name space", "/absolute", "@hidden", "用户"):
            c = full_candidate()
            c["operator_controller"] = name
            self.bad(blob(c))

    def test_09_missing_custodian_digest_rejected(self):
        c = full_candidate()
        del c["operator_identity_evidence_sha256"]
        self.bad(blob(c))

    def test_10_invalid_or_uppercase_digest_rejected(self):
        for value in ("F"*64, "a"*63, "prefix:abc", True, 8, None):
            c = full_candidate()
            c["candidate_public_key_sha256"] = value
            self.bad(blob(c))

    def test_11_null_hash_reference_is_not_evidence(self):
        c = full_candidate()
        c["owner_oob_receipt_sha256"] = "0" * 64
        self.bad(blob(c))

    def test_12_reused_hash_for_identity_and_owner_oob_denied(self):
        c = full_candidate()
        c["owner_oob_receipt_sha256"] = c["operator_identity_evidence_sha256"]
        self.bad(blob(c))

    def test_13_epoch_boolean_must_fail(self):
        c = full_candidate()
        c["witness_epoch"] = True
        self.bad(blob(c))

    def test_14_old_epoch_must_fail(self):
        c = full_candidate()
        c["witness_epoch"] = 3
        self.bad(blob(c))

    def test_15_extreme_epoch_must_fail(self):
        c = full_candidate()
        c["witness_epoch"] = 9223372036854775808
        self.bad(blob(c))

    def test_16_untrusted_root_fields_are_exact(self):
        self.bad(blob(None, admitted=True))
        self.bad(blob(None, protocol="WRONG"))
        self.bad(blob(None, project="wrong/repo"))
        self.bad(blob(None, r4z_head="a"*40))

    def test_17_unexpected_privileged_candidate_flag_refused(self):
        c = full_candidate()
        c["production_allowed"] = True
        self.bad(blob(c))

    def test_18_duplicate_outer_key_denied(self):
        raw = b'{"protocol":"x","protocol":"x","project":"x","r4z_head":"x","candidate":null}'
        self.assertIn("DUPLICATE", self.bad(raw)["detail"])

    def test_19_duplicate_candidate_key_denied(self):
        raw = blob(full_candidate()).replace(
            b'"publisher_controller":', b'"publisher_controller":"fake","publisher_controller":')
        self.assertIn("DUPLICATE", self.bad(raw)["detail"])

    def test_20_invalid_unicode_json_or_nonfinite(self):
        for v in (b"\xff", b"{", b"", b'{"x":NaN}', b'[]'):
            self.bad(v)

    def test_21_oversized_input_rejected(self):
        self.bad(b"x" * (gate.MAX_BYTES + 1))

    def test_22_real_regular_file_read_only(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "candidate.json"
            data = blob(full_candidate())
            p.write_bytes(data)
            self.assertEqual(gate.read_regular(p), data)
            self.assertEqual(self.blocked(gate.read_regular(p))["status"], "BLOCK")
            self.assertEqual(p.read_bytes(), data)

    def test_23_symlink_and_fifo_denied_without_blocking(self):
        with tempfile.TemporaryDirectory() as tmp:
            real = Path(tmp) / "real"
            real.write_bytes(blob())
            alias = Path(tmp) / "alias"
            try:
                alias.symlink_to(real)
            except OSError:
                pass
            else:
                with self.assertRaises(gate.InvalidEvidence):
                    gate.read_regular(alias)
            if hasattr(os, "mkfifo"):
                fifo = Path(tmp) / "pipe"
                try:
                    os.mkfifo(fifo)
                except OSError:
                    pass
                else:
                    with self.assertRaises(gate.InvalidEvidence):
                        gate.read_regular(fifo)

    def test_24_cli_returns_block_exit_two_without_mutation(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "manifest.json"
            data = blob(full_candidate())
            p.write_bytes(data)
            before = sorted(x.name for x in Path(tmp).iterdir())
            cmd = [sys.executable, "-B", str(Path(gate.__file__)), "--manifest", str(p)]
            result = subprocess.run(cmd, capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(json.loads(result.stdout)["admission"], "NOT_AUTHORIZED")
            self.assertEqual(result.stderr, "")
            self.assertEqual(before, sorted(x.name for x in Path(tmp).iterdir()))
            self.assertEqual(p.read_bytes(), data)

if __name__ == "__main__":
    unittest.main()
