"""24 adversarial R5C tests for offline pin consistency and mandatory denial.

Uses a real published prior same-owner R4U public key *as untrusted test data*.
There is deliberately NO independent guardian, owner OOB verification or signing.
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
import g03_r5c_oob_root_consistency as gate

NOW = 1800000000
NONCE = "a" * 64
BUNDLE = "b" * 64
REQUEST = "c" * 64
PUB = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGRMxyrMD2G9xpR3YPmCgtp9cinas6Rxz7oiuYMoS91f"

def candidate(**changes):
    obj = dict(protocol=gate.GUARDIAN_PROTOCOL, project=gate.PROJECT,
               r5b_head=gate.R5B_HEAD, guardian_controller="unverified-other-admin",
               guardian_public_key=PUB, r5b_bundle_sha256=BUNDLE,
               r4z_request_sha256=REQUEST)
    obj.update(changes)
    return gate.canonical(obj)

def anchor(**changes):
    obj = dict(protocol=gate.ANCHOR_PROTOCOL, project=gate.PROJECT,
               r5b_head=gate.R5B_HEAD, guardian_controller="unverified-other-admin",
               key_pin_sha256=gate.sha256(PUB.encode("ascii")),
               r5b_bundle_sha256=BUNDLE, r4z_request_sha256=REQUEST,
               challenge_nonce=NONCE, issued_unix=NOW, expires_unix=NOW+120,
               epoch=4, acquisition_channel_claim="OWNER_OFFLINE_RECORD")
    obj.update(changes)
    return gate.canonical(obj)

class R5COobTrustControls(unittest.TestCase):
    def blocked(self, a=None, c=None, nonce=NONCE, epoch=4, now=NOW):
        report = gate.assessment(a if a is not None else anchor(),
                                 c if c is not None else candidate(), nonce, epoch, now)
        self.assertEqual(report["status"], "BLOCK")
        self.assertEqual(report["admission"], "NOT_AUTHORIZED")
        return report

    def bad(self, a=None, c=None, **kw):
        report = self.blocked(a, c, **kw)
        self.assertEqual(report["reason"], "INVALID_UNTRUSTED_OOB_CANDIDATE")
        return report

    def test_01_actual_published_key_pin_matches_but_never_authorizes(self):
        report = self.blocked()
        self.assertEqual(report["pin_consistency"], "MATCH_ONLY_NOT_AUTHENTICATED")
        self.assertEqual(report["reason"], "ROOT_CONSISTENCY_CANNOT_PROVE_OOB_PROVENANCE")

    def test_02_different_claimed_controller_is_not_independence(self):
        report = self.blocked()
        self.assertEqual(report["guardian_control"], "NOT_INDEPENDENTLY_VERIFIED")

    def test_03_offline_channel_label_does_not_authenticate_origin(self):
        report = self.blocked()
        self.assertEqual(report["owner_oob_channel"], "NOT_AUTHENTICATED")

    def test_04_same_publisher_cannot_claim_independence(self):
        self.bad(anchor(guardian_controller="eddarsglobal"),
                 candidate(guardian_controller="eddarsglobal"))

    def test_05_controller_swap_between_files_refused(self):
        self.bad(anchor(guardian_controller="other-custodian"))

    def test_06_owner_pin_substitution_refused(self):
        self.bad(anchor(key_pin_sha256="d" * 64))

    def test_07_missing_expected_nonce_refused(self):
        self.bad(nonce="d" * 64)

    def test_08_claimed_nonce_swap_refused(self):
        self.bad(anchor(challenge_nonce="d" * 64))

    def test_09_expired_challenge_refused(self):
        self.bad(now=NOW+121)

    def test_10_future_dated_challenge_refused(self):
        self.bad(now=NOW-1)

    def test_11_oversized_ttl_refused(self):
        self.bad(anchor(expires_unix=NOW+301))

    def test_12_zero_or_boolean_ttl_refused(self):
        self.bad(anchor(expires_unix=NOW))
        self.bad(anchor(expires_unix=True))

    def test_13_epoch_mismatch_refused(self):
        self.bad(epoch=5)
        self.bad(anchor(epoch=3), epoch=3)

    def test_14_epoch_boolean_refused(self):
        self.bad(anchor(epoch=True))

    def test_15_r5b_signature_bundle_swap_refused(self):
        self.bad(candidate(r5b_bundle_sha256="e" * 64))

    def test_16_r4z_request_swap_refused(self):
        self.bad(anchor(r4z_request_sha256="e" * 64))

    def test_17_wrong_protocol_project_and_r5b_commit_refused(self):
        self.bad(candidate(protocol="WRONG"))
        self.bad(anchor(project="third-party/other"))
        self.bad(candidate(r5b_head="0"*40))

    def test_18_noncanonical_json_and_duplicate_keys_refused(self):
        self.bad(a=json.dumps(json.loads(anchor()), indent=2).encode())
        self.bad(a=anchor().replace(b'"project":',b'"project":"x","project":'))

    def test_19_invalid_utf8_and_overlarge_manifest_refused(self):
        self.bad(a=b"\xff")
        self.bad(c=b"x" * (gate.MAX_BYTES+1))

    def test_20_wrong_openssh_algorithm_refused(self):
        self.bad(c=candidate(guardian_public_key="ssh-rsa AAAA"))
        self.bad(c=candidate(guardian_public_key="ssh-ed25519 AAAA"))

    def test_21_owner_channel_claim_cannot_be_missing_or_relabelled(self):
        self.bad(anchor(acquisition_channel_claim="VERIFIED_BY_GITHUB"))
        self.bad(anchor(acquisition_channel_claim=None))

    def test_22_reject_symlink_and_fifo_before_reading(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/"real"
            p.write_bytes(anchor())
            alias=Path(tmp)/"alias"
            try:
                alias.symlink_to(p)
            except OSError:
                pass
            else:
                with self.assertRaises(gate.InvalidEvidence):
                    gate.read_regular(alias)
            if hasattr(os, "mkfifo"):
                fifo=Path(tmp)/"fifo"
                try:
                    os.mkfifo(fifo)
                except OSError:
                    pass
                else:
                    with self.assertRaises(gate.InvalidEvidence):
                        gate.read_regular(fifo)

    def test_23_anchor_inside_repo_tree_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            checkout=Path(tmp)/"checkout"
            checkout.mkdir()
            in_checkout=checkout/"anchor.json"
            in_checkout.write_bytes(anchor())
            with self.assertRaises(gate.InvalidEvidence):
                gate.owner_anchor_outside_repo(in_checkout,checkout)
            external=Path(tmp)/"separate.json"
            external.write_bytes(anchor())
            self.assertTrue(gate.owner_anchor_outside_repo(external,checkout))

    def test_24_cli_missing_external_anchor_exit_two_and_no_mutation(self):
        with tempfile.TemporaryDirectory() as tmp:
            checkout=Path(tmp)/"checkout"; checkout.mkdir()
            c=Path(tmp)/"candidate.json"; c.write_bytes(candidate())
            before=sorted(p.name for p in Path(tmp).iterdir())
            cmd=[sys.executable,"-B",str(Path(gate.__file__)),"--candidate",str(c),
                 "--owner-anchor",str(Path(tmp)/"no-oob-anchor"),
                 "--repo-root",str(checkout),"--expected-nonce",NONCE,
                 "--expected-epoch","4"]
            outcome=subprocess.run(cmd,capture_output=True,text=True,check=False)
            self.assertEqual(outcome.returncode,2)
            report=json.loads(outcome.stdout)
            self.assertEqual(report["reason"],"UNSAFE_OR_MISSING_OOB_SOURCE")
            self.assertEqual(report["admission"],"NOT_AUTHORIZED")
            self.assertEqual(sorted(p.name for p in Path(tmp).iterdir()),before)

if __name__=="__main__":
    unittest.main()
