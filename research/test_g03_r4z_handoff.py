"""R4Z offline handoff correctness and fail-closed independent custody tests."""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path(__file__).parent))
import g03_r4z_handoff as lab
T = 1800000000
OWNER = "1" * 64
GUARDIAN = "2" * 64
NONCE = "3" * 64

def request(**kw):
    params = dict(publisher="eddarsglobal", owner_pin=OWNER, guardian_pin=GUARDIAN,
                  epoch=4, lifetime=120, now=T, nonce=NONCE)
    params.update(kw)
    return lab.make_request(**params)

def mutate(base, **kw):
    d = json.loads(base)
    d.update(kw)
    return lab.canonical(d)

def reply(req=None, **kw):
    if req is None:
        req = json.loads(request())
    payload = dict(protocol=lab.PROTOCOL, request_binding_sha256=req["binding_sha256"],
                   guardian_pin_sha256=GUARDIAN, candidate_controller="external-operator",
                   guardian_signature_sha256="4" * 64, owner_oob_receipt_sha256="5" * 64,
                   witness_head_sha256="6" * 64, witness_epoch=4)
    payload.update(kw)
    return lab.canonical(payload)

class R4ZTests(unittest.TestCase):
    def blocked(self, req, receipt=None, now=T, floor=4):
        outcome = lab.assessment(req, receipt, now, floor)
        self.assertEqual(outcome["status"], "BLOCK")
        self.assertEqual(outcome["admission"], "NOT_AUTHORIZED")
        return outcome
    def invalid(self, req, receipt=None, now=T, floor=4):
        result = self.blocked(req, receipt, now, floor)
        self.assertEqual(result["reason"], "INVALID_UNTRUSTED_HANDOFF")
        return result
    def test_01_correct_canonical_request_is_readable_but_not_admitted(self):
        self.assertEqual(lab.verify_request(request(), now=T)["floor_epoch"], 4)
        self.assertEqual(self.blocked(request())["reason"], "REAL_GUARDIAN_RECEIPT_MISSING")
    def test_02_all_claimed_approval_refs_still_cannot_admit(self):
        self.assertEqual(self.blocked(request(), reply())["reason"],
                         "INDEPENDENT_PROVENANCE_NOT_EXTERNALLY_VERIFIED")
    def test_03_challenge_randomness_from_os_source(self):
        a = lab.make_request("eddarsglobal", OWNER, GUARDIAN, now=T)
        b = lab.make_request("eddarsglobal", OWNER, GUARDIAN, now=T)
        self.assertNotEqual(json.loads(a)["nonce"], json.loads(b)["nonce"])
    def test_04_replay_of_old_request_binding_rejected(self):
        other = request(nonce="7"*64)
        self.invalid(other, reply())
    def test_05_wrong_guardian_key_in_reply_rejected(self):
        self.invalid(request(), reply(guardian_pin_sha256="8"*64))
    def test_06_declared_same_administrator_rejected(self):
        res = self.invalid(request(), reply(candidate_controller="EDDARSGLOBAL"))
        self.assertIn("SAME_DECLARED_ADMINISTRATOR", res["detail"])
    def test_07_claimed_external_identity_not_proven(self):
        self.assertEqual(self.blocked(request(), reply(candidate_controller="unknown-third-party"))["guardian_independence"], "NOT_VERIFIED")
    def test_08_epoch_floor_rollback_rejected(self):
        self.invalid(request(), reply(witness_epoch=4), floor=5)
    def test_09_floor_epoch_downgrade_rejected(self):
        with self.assertRaises(lab.Invalid):
            request(epoch=3)
    def test_10_receipt_expired_at_current_clock_rejected(self):
        self.invalid(request(), reply(), now=T+121)
    def test_11_future_dated_receipt_clock_rejected(self):
        self.invalid(request(), reply(), now=T-31)
    def test_12_overlong_or_zero_ttl_rejected(self):
        for ttl in (0, 301, -1, True):
            with self.assertRaises(lab.Invalid):
                request(lifetime=ttl)
    def test_13_wrong_protocol_parent_or_project_rejected(self):
        for field, value in (("protocol", "bad"), ("project", "x/y"), ("origin_commit", "a"*40)):
            self.invalid(mutate(request(), **{field: value}), reply())
    def test_14_modified_nonce_breaks_binding(self):
        self.invalid(mutate(request(), nonce="9"*64), reply())
    def test_15_same_public_root_pin_does_not_prove_separation(self):
        with self.assertRaises(lab.Invalid):
            request(guardian_pin=OWNER)
    def test_16_duplicate_fields_and_extra_fields_rejected(self):
        raw = request().decode().replace('"project":', '"project":"x", "project":', 1).encode()
        self.invalid(raw)
        self.invalid(mutate(request(), admitted=True))
    def test_17_noncanonical_encoding_rejected(self):
        self.invalid(json.dumps(json.loads(request()), indent=2).encode())
    def test_18_invalid_utf8_and_nonfinite_rejected(self):
        self.invalid(b"\xff")
        self.invalid(request().replace(b'"floor_epoch":4', b'"floor_epoch":NaN'))
    def test_19_bad_claimed_signature_digest_rejected(self):
        self.invalid(request(), reply(guardian_signature_sha256="BAD"))
    def test_20_noninteger_epoch_and_confused_boolean_rejected(self):
        self.invalid(request(), reply(witness_epoch=True))
        self.invalid(mutate(request(), floor_epoch=True))
    def test_21_cli_read_only_block_exit_two(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/"request.json"
            blob=lab.make_request("eddarsglobal", OWNER, GUARDIAN)
            path.write_bytes(blob)
            p=subprocess.run([sys.executable, "-B", str(Path(lab.__file__)), "--request", str(path)], capture_output=True, text=True, check=False)
            self.assertEqual(p.returncode,2)
            self.assertEqual(json.loads(p.stdout)["status"],"BLOCK")
            self.assertEqual(path.read_bytes(),blob)
    def test_22_symlink_request_cannot_be_followed(self):
        with tempfile.TemporaryDirectory() as tmp:
            target=Path(tmp)/"real.json"; target.write_bytes(request())
            alias=Path(tmp)/"alias.json"
            try:
                alias.symlink_to(target)
            except OSError:
                self.skipTest("No symlink support")
            with self.assertRaises(lab.Invalid):
                lab.read_regular(alias)
    def test_23_missing_untrusted_external_file_returns_block(self):
        with tempfile.TemporaryDirectory() as tmp:
            result=subprocess.run([sys.executable, "-B", str(Path(lab.__file__)), "--request", str(Path(tmp)/"missing")],capture_output=True,text=True)
            self.assertEqual(result.returncode,2)
            self.assertEqual(json.loads(result.stdout)["reason"],"FILE_NOT_AVAILABLE_OR_UNSAFE")
    def test_24_no_authorization_path_in_current_module(self):
        self.assertEqual(self.blocked(request(), reply())["admission"],"NOT_AUTHORIZED")
        self.assertEqual(self.blocked(request(), None)["admission"],"NOT_AUTHORIZED")

if __name__=="__main__":
    unittest.main()
