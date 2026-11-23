"""APP-020 — forty real Ed25519 SSHSIG and native Rust admission tests.

A locally generated test private key is only an ephemeral fixture! It does not
establish independent custody or a trusted third party. Genuine SSHSIG
verification is performed by OpenSSH, not mocked. APP-019 fixture creates
authentic Rust .noi histories and checkpoint/resume demonstrations.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
import receiver_app020 as signed
sys.path.insert(0, str(HERE.parent / "app019_external_head_pin"))
import test_app019 as predecessor  # do not import the TestCase name directly


class SignedProvenance(unittest.TestCase):
    AUTHORITY = "trusted-lab-authority"
    RECEIVER = "nordoi-receiver-1"

    @classmethod
    def setUpClass(cls):
        predecessor.ReceiverExternalHeadPin.setUpClass()
        cls.keytemp = tempfile.TemporaryDirectory(prefix="nordoi-app020-ephemeral-key-")
        cls.private = Path(cls.keytemp.name) / "testkey"
        result = subprocess.run(
            ["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f",
             str(cls.private)], capture_output=True, timeout=30)
        if result.returncode != 0:
            raise RuntimeError("SSH_KEYGEN_ED25519_UNAVAILABLE: "
                               + result.stderr.decode("utf-8", "replace"))
        parts = (Path(str(cls.private) + ".pub")
                 .read_text(encoding="ascii").split())
        assert len(parts) >= 2 and parts[0] == "ssh-ed25519"
        cls.key_public = (parts[0] + " " + parts[1] + "\n").encode("ascii")
        cls.key_hash = hashlib.sha256(cls.key_public).hexdigest()

    @classmethod
    def tearDownClass(cls):
        cls.keytemp.cleanup()
        predecessor.ReceiverExternalHeadPin.tearDownClass()

    def setUp(self):
        self.fx = predecessor.ReceiverExternalHeadPin(
            "test_01_capture_is_explicitly_untrusted")
        self.fx.setUp()
        self.work = self.fx.fx.fx.fx.work
        self.public = self.work / "receiver-trusted-key.pub"
        self.public.write_bytes(self.key_public)
        self.document = self.work / "receiver-signed-statement.json"
        self.signature = self.work / "receiver-detached-sshsig.sig"
        self.statement = self.build_statement()
        self.sign()

    def tearDown(self):
        self.fx.tearDown()

    def build_statement(self):
        root = self.fx.root
        head = json.loads((root / "HEAD.json").read_bytes())
        sha = hashlib.sha256((root / "HEAD.json").read_bytes()).hexdigest()
        return signed.statement_for(
            head, sha, self.AUTHORITY, self.RECEIVER)

    def sign(self, key=None, namespace=None):
        self.document.write_bytes(signed.anchor.canonical(self.statement))
        result = subprocess.run(
            ["ssh-keygen", "-Y", "sign", "-f", str(self.private if key is None else key),
             "-n", signed.NAMESPACE if namespace is None else namespace],
            input=self.document.read_bytes(), capture_output=True, timeout=25)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(b"-----BEGIN SSH SIGNATURE-----", result.stdout)
        self.signature.write_bytes(result.stdout)

    def rotated_signed(self):
        self.fx.rotate()
        self.statement = self.build_statement()
        self.sign()

    def frozen_signed(self):
        self.fx.freeze()
        self.statement = self.build_statement()
        self.sign()

    def call(self, action="status", *, statement=None, signature=None,
             public=None, key_hash=None, minimum_sequence=1,
             minimum_generation=1, authority=None, receiver=None,
             omit=(), package=None, source=None, engine=None, dest=None,
             root=None, path_env=None):
        params = [
            "--anchor-root", str(self.fx.root if root is None else root),
            "--authority-id", self.AUTHORITY if authority is None else authority,
            "--receiver-id", self.RECEIVER if receiver is None else receiver,
            "--trusted-public-key", str(self.public if public is None else public),
            "--expected-public-key-sha256",
            self.key_hash if key_hash is None else key_hash,
            "--signed-statement",
            str(self.document if statement is None else statement),
            "--detached-signature",
            str(self.signature if signature is None else signature),
            "--minimum-sequence", str(minimum_sequence),
            "--minimum-generation", str(minimum_generation),
        ]
        selection = []
        for i in range(0, len(params), 2):
            if params[i] not in omit:
                selection.extend(params[i:i + 2])
        if action in ("inspect", "verify", "restore", "resume"):
            selection.extend([
                "--receiver-roster", str(self.fx.fx.fx.roster_file),
                "--receiver-policy", str(self.fx.fx.fx.fx.policy_file),
                "--portable-dir", str(self.fx.fx.fx.fx.portable
                                      if package is None else package),
            ])
        elif package is not None:
            selection.extend(["--portable-dir", str(package)])
        if action in ("verify", "restore", "resume"):
            selection.extend([
                "--engine", str(predecessor.fixture.original_fixture.ENGINE
                                if engine is None else engine),
                "--source-root", str(predecessor.fixture.original_fixture.SOURCE
                                     if source is None else source),
            ])
        if action in ("restore", "resume"):
            selection.extend([
                "--restore-root", str(self.fx.fx.fx.fx.dest if dest is None else dest),
            ])
        env = dict(os.environ)
        if path_env is not None:
            env["PATH"] = path_env
        process = subprocess.run(
            [sys.executable, "-B", str(HERE / "receiver_app020.py"), action]
            + selection, cwd=REPO, capture_output=True, timeout=190, env=env)
        try:
            rows = [json.loads(s) for s in process.stdout.splitlines()]
        except ValueError:
            self.fail(f"NOT_JSON: {process.stdout!r}, stderr={process.stderr!r}")
        return process, rows

    def good(self, action="status", **kwargs):
        p, rows = self.call(action, **kwargs)
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        if action != "resume":
            self.assertEqual(len(rows), 1)
        return p, rows

    def refused(self, code, action="status", **kwargs):
        p, rows = self.call(action, **kwargs)
        self.assertEqual(p.returncode, 2, (rows, p.stderr))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["status"], "REJECTED")
        self.assertEqual(rows[0]["error_code"], code)
        self.assertEqual(rows[0]["publication"], "NOT_STARTED")

    def test_01_propose_is_exact_canonical_unsigned_candidate(self):
        row = self.good("propose")[1][0]
        self.assertEqual(row["schema"], signed.SCHEMA)
        self.assertEqual(row["head_sha256"], self.statement["head_sha256"])

    def test_02_proposal_not_a_signature(self):
        self.assertNotIn("crypto_signature", self.good("propose")[1][0])

    def test_03_missing_receiver_floors_denied(self):
        self.refused("SSHSIG_RECEIVER_FLOOR_REQUIRED",
                     omit=("--minimum-sequence",))

    def test_04_missing_public_key_pin_denied(self):
        self.refused("SSHSIG_RECEIVER_TRUST_INPUTS_REQUIRED",
                     omit=("--expected-public-key-sha256",))

    def test_05_wrong_receiver_public_key_pin_denied(self):
        self.refused("SSHSIG_PUBLIC_KEY_PIN_MISMATCH", key_hash="0" * 64)

    def test_06_malformed_public_key_even_with_matching_pin_denied(self):
        bad = self.work / "malformed.pub"
        bad.write_bytes(b"ssh-ed25519 NOT-A-KEY\n")
        self.refused("SSHSIG_PUBLIC_KEY_FORMAT_INVALID",
                     public=bad, key_hash=hashlib.sha256(bad.read_bytes()).hexdigest())

    def test_07_trust_file_symlink_is_not_admitted(self):
        bad = self.work / "linked.pub"
        try:
            bad.symlink_to(self.public)
        except (OSError, NotImplementedError):
            # On Windows a regular invalid key still exercises fail closed.
            bad.write_bytes(b"ssh-rsa invalid\n")
            self.refused("SSHSIG_PUBLIC_KEY_PIN_MISMATCH", public=bad)
            return
        self.refused("SSHSIG_PUBLIC_KEY_NOT_REGULAR", public=bad)

    def test_08_tampered_signed_json_rejected_without_effect(self):
        self.statement["receiver_id"] = "wrong-receiver"
        self.document.write_bytes(signed.anchor.canonical(self.statement))
        self.refused("SSHSIG_SIGNER_OR_RECEIVER_MISMATCH")

    def test_09_tampered_detached_signature_fails(self):
        raw = bytearray(self.signature.read_bytes())
        middle = raw.index(b"\n") + 1
        while middle < len(raw) and raw[middle] in (10, 13, 45):
            middle += 1
        raw[middle] = ord("A") if raw[middle] != ord("A") else ord("B")
        self.signature.write_bytes(bytes(raw))
        self.refused("SSHSIG_SIGNATURE_VERIFICATION_FAILED")

    def test_10_different_trusted_key_cannot_verify_signature(self):
        other_key = self.work / "other-private"
        result = subprocess.run(
            ["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f",
             str(other_key)], capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        fields = Path(str(other_key) + ".pub").read_text().split()
        pub = self.work / "other-public.pub"
        pub.write_text(fields[0] + " " + fields[1] + "\n", encoding="ascii")
        self.refused("SSHSIG_SIGNATURE_VERIFICATION_FAILED",
                     public=pub, key_hash=hashlib.sha256(pub.read_bytes()).hexdigest())

    def test_11_expected_authority_mismatch_denied(self):
        self.refused("SSHSIG_SIGNER_OR_RECEIVER_MISMATCH",
                     authority="another-authority")

    def test_12_receiver_identity_mismatch_denied(self):
        self.refused("SSHSIG_SIGNER_OR_RECEIVER_MISMATCH", receiver="other-receiver")

    def test_13_cross_namespace_signature_denied(self):
        self.sign(namespace="another-nordoi-protocol")
        self.refused("SSHSIG_SIGNATURE_VERIFICATION_FAILED")

    def test_14_noncanonical_statement_refused(self):
        self.document.write_bytes(self.document.read_bytes() + b" ")
        self.refused("SSHSIG_STATEMENT_NOT_CANONICAL")

    def test_15_extra_signer_controlled_field_refused_even_signed(self):
        self.statement["fabricated_authority"] = "CERTIFIED"
        self.sign()
        self.refused("SSHSIG_STATEMENT_SCHEMA_INVALID")

    def test_16_missing_statement_refused(self):
        self.document.unlink()
        p, rows = self.call()
        self.assertEqual(p.returncode, 2)
        self.assertEqual(rows[0]["status"], "REJECTED")

    def test_17_valid_sshsig_and_receiver_pin_allow_status(self):
        self.assertEqual(self.good()[1][0]["status"], "ACTIVE")

    def test_18_report_contains_genuine_sshsig_verification(self):
        self.assertEqual(self.good()[1][0]["crypto_signature"],
                         "SSHSIG_ED25519_VERIFIED")

    def test_19_signer_verified_inspection_never_runs_rust(self):
        self.assertEqual(self.good("inspect")[1][0]["native_engine"],
                         "NOT_EXECUTED")

    def test_20_real_rust_replay_after_signed_provenance(self):
        result = self.good("verify")[1][0]
        self.assertEqual(result["native_jobs_recomputed"], 2)
        self.assertEqual(result["native_files_bytewise_equal"], 4)

    def test_21_genuine_native_restore_after_signed_provenance(self):
        self.assertEqual(self.good("restore")[1][0]["status"], "RESTORED")

    def test_22_real_native_resume_uses_only_new_jsonl_rows(self):
        self.good("restore")
        _, rows = self.good("resume")
        self.assertEqual([row["step"] for row in rows], [3, 4])

    def test_23_duplicate_native_resume_denied(self):
        self.good("restore")
        self.good("resume")
        self.refused("ROOT_UNEXPECTED_JOB_OR_FILE", "resume")

    def test_24_old_signed_provenance_rejected_after_rotation(self):
        self.fx.rotate()
        self.refused("SSHSIG_SIGNED_STATE_NOT_CURRENT")

    def test_25_new_signed_generation_admitted(self):
        self.rotated_signed()
        self.assertEqual(self.good(minimum_sequence=2)[1][0]["journal_sequence"], 2)

    def test_26_receiver_minimum_blocks_old_valid_signature(self):
        self.refused("SSHSIG_BELOW_RECEIVER_FLOOR", minimum_sequence=2)

    def test_27_rotated_new_noi_native_replay_admitted(self):
        self.rotated_signed()
        result = self.good(
            "verify", minimum_sequence=2,
            source=self.fx.fx.fx.fx.alt_source)[1][0]
        self.assertEqual(result["native_jobs_recomputed"], 2)

    def test_28_rotated_signature_does_not_authorize_old_noi_source(self):
        self.rotated_signed()
        self.refused("SOURCE_PIN_MISMATCH", "verify", minimum_sequence=2)

    def test_29_rotated_source_genuine_restore_and_resume(self):
        self.rotated_signed()
        alt = self.fx.fx.fx.fx.alt_source
        self.good("restore", minimum_sequence=2, source=alt)
        _, rows = self.good("resume", minimum_sequence=2, source=alt)
        self.assertEqual([r["step"] for r in rows], [3, 4])

    def test_30_signed_frozen_state_can_be_observed(self):
        self.frozen_signed()
        self.assertEqual(self.good(minimum_sequence=2)[1][0]["status"], "FROZEN")

    def test_31_signed_frozen_state_denies_inspect(self):
        self.frozen_signed()
        self.refused("ROSTER_FROZEN_DENIES_ADMISSION",
                     "inspect", minimum_sequence=2)

    def test_32_signed_frozen_state_denies_native_restore_without_effect(self):
        self.frozen_signed()
        self.refused("ROSTER_FROZEN_DENIES_ADMISSION",
                     "restore", minimum_sequence=2)
        self.assertFalse(any(self.fx.fx.fx.fx.dest.iterdir()))

    def test_33_full_coherent_rollback_old_signed_statement_blocked_by_floor(self):
        oldroot = self.work / "original-complete-anchor"
        shutil.copytree(self.fx.root, oldroot)
        self.rotated_signed()
        shutil.rmtree(self.fx.root)
        shutil.copytree(oldroot, self.fx.root)
        self.refused("SSHSIG_SIGNED_STATE_NOT_CURRENT", minimum_sequence=2)

    def test_34_old_complete_journal_and_old_valid_signature_pass_without_floor(self):
        oldroot = self.work / "original-complete-anchor"
        shutil.copytree(self.fx.root, oldroot)
        old_statement = self.document.read_bytes()
        old_signature = self.signature.read_bytes()
        self.rotated_signed()
        self.document.write_bytes(old_statement)
        self.signature.write_bytes(old_signature)
        shutil.rmtree(self.fx.root)
        shutil.copytree(oldroot, self.fx.root)
        result = self.good()[1][0]
        self.assertEqual(result["journal_sequence"], 1)
        self.assertEqual(result["global_anti_rollback"], "NOT_PROVIDED")

    def test_35_unchanged_public_pin_refuses_attacker_alternate_key(self):
        alt = self.work / "untrusted.pub"
        alt.write_bytes(b"ssh-ed25519 changed\n")
        self.refused("SSHSIG_PUBLIC_KEY_PIN_MISMATCH", public=alt)

    def test_36_trust_key_embedded_in_portable_package_is_refused(self):
        package = self.fx.fx.fx.fx.portable
        embedded = package / "attacker-allowed-signers.pub"
        embedded.write_bytes(self.key_public)
        self.refused("SSHSIG_TRUST_FILES_OVERLAP_UNTRUSTED_ROOT",
                     public=embedded, package=package)

    def test_37_statement_embedded_in_journal_is_refused(self):
        embedded = self.fx.root / "attacker.json"
        embedded.write_bytes(self.document.read_bytes())
        self.refused("ANCHOR_NOT_SEPARATE_FROM_UNTRUSTED_INPUT",
                     statement=embedded)

    def test_38_missing_sshsig_verifier_is_fail_closed(self):
        self.refused("SSHSIG_VERIFIER_UNAVAILABLE", path_env="")

    def test_39_invalid_receiver_minimum_never_admitted(self):
        self.refused("SSHSIG_RECEIVER_FLOOR_INVALID", minimum_sequence=0)

    def test_40_report_never_claims_independent_custody_or_global_freshness(self):
        result = self.good()[1][0]
        self.assertEqual(result["independent_key_custody"], "NOT_PROVEN")
        self.assertEqual(result["global_anti_rollback"], "NOT_PROVIDED")
        self.assertEqual(result["external_security_certification"], "NOT_CLAIMED")
        self.assertEqual(result["production"], "NOT_AUTHORIZED")


if __name__ == "__main__":
    unittest.main()
