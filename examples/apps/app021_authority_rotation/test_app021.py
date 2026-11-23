"""APP-021: 40 real Ed25519/SSHSIG authority-chain and native Rust tests.

The ephemeral keys belong to CI fixtures, not independent custodians. Dual
signatures and cryptographic revocation are real. Coherent full rollback with
rollback of every externally supplied trust reference is deliberately shown
as a limitation, not reported as prevention.
"""
from __future__ import annotations

import base64
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import unittest

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
import receiver_app021 as authority
sys.path.insert(0, str(HERE.parent / "app020_signed_head_provenance"))
import test_app020 as baseline


class AuthorityRotation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        baseline.SignedProvenance.setUpClass()

    @classmethod
    def tearDownClass(cls):
        baseline.SignedProvenance.tearDownClass()

    def setUp(self):
        self.fx = baseline.SignedProvenance(
            "test_01_propose_is_exact_canonical_unsigned_candidate")
        self.fx.setUp()
        self.work = self.fx.work
        self.root = self.fx.fx.root
        self.root_public = self.fx.key_public
        self.root_private = self.fx.private
        self.root_sha = self.fx.key_hash
        self.authority_id = self.fx.AUTHORITY
        self.receiver_id = self.fx.RECEIVER
        self.chain_file = self.work / "receiver-independent-authorities.json"
        self.records = []
        root_event = self.make_event(
            "BOOTSTRAP", 1, "ACTIVE", self.root_public, [], 1, 1)
        self.records.append(self.record(root_event, self.root_private))
        self.write_chain()

    def tearDown(self):
        self.fx.tearDown()

    def make_event(self, transition, epoch, state, key, retired, seq, gen):
        prev = (authority.anchor.digest(authority.anchor.canonical(self.records[-1]))
                if self.records else None)
        return {
            "schema": authority.EVENT_SCHEMA,
            "epoch": epoch,
            "previous_record_sha256": prev,
            "authority_id": self.authority_id,
            "receiver_id": self.receiver_id,
            "state": state,
            "active_public_key_b64":
                base64.b64encode(key).decode("ascii") if key else None,
            "retired_public_key_sha256": sorted(retired),
            "minimum_sequence": seq,
            "minimum_generation": gen,
            "transition": transition,
            "release": "RESEARCH_ONLY",
            "production": "NOT_AUTHORIZED",
        }

    def signature(self, event, private, namespace=None):
        p = subprocess.run(
            ["ssh-keygen", "-Y", "sign", "-f", str(private), "-n",
             authority.NAMESPACE if namespace is None else namespace],
            input=authority.anchor.canonical(event),
            capture_output=True, timeout=30)
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertIn(b"-----BEGIN SSH SIGNATURE-----", p.stdout)
        return base64.b64encode(p.stdout).decode("ascii")

    def record(self, event, outgoing, incoming=None):
        return {
            "event": event,
            "outgoing_signature_b64": self.signature(event, outgoing),
            "incoming_signature_b64":
                self.signature(event, incoming) if incoming is not None else None,
        }

    def write_chain(self):
        self.chain = {
            "schema": authority.CHAIN_SCHEMA,
            "records": self.records,
            "release": "RESEARCH_ONLY",
            "production": "NOT_AUTHORIZED",
        }
        raw = authority.anchor.canonical(self.chain)
        self.chain_file.write_bytes(raw)
        self.chain_sha = hashlib.sha256(raw).hexdigest()
        return self.chain_sha

    def rotate(self):
        old_statement = self.fx.document.read_bytes()
        old_signature = self.fx.signature.read_bytes()
        self.fx.rotated_signed()
        self.old_signed_document = old_statement
        self.old_signed_signature = old_signature
        successor = self.work / "authority-successor-private"
        p = subprocess.run(
            ["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(successor)],
            capture_output=True, timeout=30)
        self.assertEqual(p.returncode, 0, p.stderr)
        self.successor_private = successor
        parts = Path(str(successor) + ".pub").read_text(encoding="ascii").split()
        self.successor_public = (parts[0] + " " + parts[1] + "\n").encode("ascii")
        self.successor_sha = hashlib.sha256(self.successor_public).hexdigest()
        self.fx.sign(key=self.successor_private)
        new_event = self.make_event(
            "ROTATE", 2, "ACTIVE", self.successor_public,
            [self.root_sha], 2, 2)
        self.records.append(
            self.record(new_event, self.root_private, self.successor_private))
        self.write_chain()

    def freeze(self):
        self.fx.frozen_signed()
        event = self.make_event(
            "FREEZE", 2, "FROZEN", None, [self.root_sha], 2, 2)
        self.records.append(self.record(event, self.root_private))
        self.write_chain()

    def call(self, action="status", *, pin=None, root_pin=None, epoch=1,
             seq=1, gen=1, chain_path=None, statement=None, signature=None,
             source=None, package=None, dest=None, root=None,
             authority_id=None, receiver_id=None, omit=()):
        args = [
            "--anchor-root", str(self.root if root is None else root),
            "--authority-chain-file",
                str(self.chain_file if chain_path is None else chain_path),
            "--expected-chain-sha256", self.chain_sha if pin is None else pin,
            "--expected-root-key-sha256",
                self.root_sha if root_pin is None else root_pin,
            "--authority-id",
                self.authority_id if authority_id is None else authority_id,
            "--receiver-id",
                self.receiver_id if receiver_id is None else receiver_id,
            "--minimum-epoch", str(epoch),
            "--minimum-sequence", str(seq),
            "--minimum-generation", str(gen),
            "--signed-statement",
                str(self.fx.document if statement is None else statement),
            "--detached-signature",
                str(self.fx.signature if signature is None else signature),
        ]
        chosen = []
        for i in range(0, len(args), 2):
            if args[i] not in omit:
                chosen.extend(args[i:i + 2])
        if action in ("inspect", "verify", "restore", "resume"):
            chosen.extend([
                "--receiver-roster", str(self.fx.fx.fx.fx.roster_file),
                "--receiver-policy", str(self.fx.fx.fx.fx.fx.policy_file),
                "--portable-dir",
                    str(self.fx.fx.fx.fx.fx.portable if package is None else package),
            ])
        elif package is not None:
            chosen.extend(["--portable-dir", str(package)])
        if action in ("verify", "restore", "resume"):
            chosen.extend([
                "--engine", str(baseline.predecessor.fixture.original_fixture.ENGINE),
                "--source-root", str(
                    baseline.predecessor.fixture.original_fixture.SOURCE
                    if source is None else source),
            ])
        if action in ("restore", "resume"):
            chosen.extend([
                "--restore-root", str(
                    self.fx.fx.fx.fx.fx.dest if dest is None else dest),
            ])
        result = subprocess.run(
            [sys.executable, "-B", str(HERE / "receiver_app021.py"), action]
            + chosen, cwd=REPO, capture_output=True, timeout=190)
        try:
            lines = [json.loads(x) for x in result.stdout.splitlines()]
        except ValueError:
            self.fail(f"Non-JSON: {result.stdout!r} stderr={result.stderr!r}")
        return result, lines

    def good(self, action="status", **kwargs):
        p, rows = self.call(action, **kwargs)
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        if action != "resume":
            self.assertEqual(len(rows), 1)
        return p, rows

    def denied(self, expected=None, action="status", **kwargs):
        p, rows = self.call(action, **kwargs)
        self.assertEqual(p.returncode, 2, (rows, p.stderr))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["status"], "REJECTED")
        self.assertEqual(rows[0]["publication"], "NOT_STARTED")
        if expected is not None:
            self.assertEqual(rows[0]["error_code"], expected)
        return rows[0]

    def test_01_initial_chain_genuine_root_signature(self):
        self.assertEqual(self.good()[1][0]["status"], "ACTIVE")

    def test_02_initial_key_is_pinned_by_receiver(self):
        self.assertEqual(self.good()[1][0]["active_key_sha256"], self.root_sha)

    def test_03_epoch_starts_at_one(self):
        self.assertEqual(self.good()[1][0]["authority_epoch"], 1)

    def test_04_chain_pin_is_checked(self):
        self.denied("AUTHORITY_CHAIN_EXTERNAL_PIN_MISMATCH", pin="0"*64)

    def test_05_external_root_key_pin_is_checked(self):
        self.denied("AUTHORITY_BOOTSTRAP_EXTERNAL_ROOT_INVALID", root_pin="f"*64)

    def test_06_invalid_epoch_minimum_is_refused(self):
        self.denied("AUTHORITY_RECEIVER_FLOOR_INVALID", epoch=0)

    def test_07_missing_chain_sha_refused(self):
        self.denied(omit=("--expected-chain-sha256",))

    def test_08_canonical_chain_is_required(self):
        self.chain_file.write_bytes(self.chain_file.read_bytes()+b" ")
        self.chain_sha = hashlib.sha256(self.chain_file.read_bytes()).hexdigest()
        self.denied("AUTHORITY_CHAIN_NOT_CANONICAL")

    def test_09_signed_status_never_runs_native_engine(self):
        self.assertEqual(self.good()[1][0]["native_engine"], "NOT_EXECUTED")

    def test_10_native_inspect_is_read_only(self):
        self.assertEqual(self.good("inspect")[1][0]["native_engine"], "NOT_EXECUTED")

    def test_11_native_real_rust_replay(self):
        row = self.good("verify")[1][0]
        self.assertEqual(row["native_jobs_recomputed"], 2)

    def test_12_native_real_restore(self):
        self.assertEqual(self.good("restore")[1][0]["status"], "RESTORED")

    def test_13_real_native_resume_suffix(self):
        self.good("restore")
        _, rows = self.good("resume")
        self.assertEqual([r["step"] for r in rows], [3,4])

    def test_14_native_second_resume_denied(self):
        self.good("restore")
        self.good("resume")
        self.denied("ROOT_UNEXPECTED_JOB_OR_FILE", "resume")

    def test_15_initial_state_reports_no_independent_key_custody(self):
        self.assertEqual(self.good()[1][0]["independent_root_custody"], "NOT_PROVEN")

    def test_16_chain_embedded_inside_checkpoint_refused(self):
        package = self.fx.fx.fx.fx.fx.portable
        malicious = package/"attacker_authorities.json"
        malicious.write_bytes(self.chain_file.read_bytes())
        self.denied("SSHSIG_TRUST_FILES_OVERLAP_UNTRUSTED_ROOT",
                    chain_path=malicious, package=package)

    def test_17_missing_current_signed_statement_refused(self):
        self.fx.document.unlink()
        self.denied()

    def test_18_rotation_two_signatures_admits_new_key(self):
        self.rotate()
        row = self.good(epoch=2, seq=2, gen=2)[1][0]
        self.assertEqual(row["authority_epoch"], 2)
        self.assertEqual(row["active_key_sha256"], self.successor_sha)

    def test_19_rotation_cumulatively_retires_old_root(self):
        self.rotate()
        self.assertEqual(self.good(epoch=2, seq=2, gen=2)[1][0]["retired_authorities"], 1)

    def test_20_rotation_keeps_original_external_root_trust(self):
        self.rotate()
        self.assertEqual(self.good(epoch=2, seq=2, gen=2)[1][0]["status"], "ACTIVE")

    def test_21_previous_valid_statement_cannot_authorize_new_chain(self):
        self.rotate()
        self.fx.document.write_bytes(self.old_signed_document)
        self.fx.signature.write_bytes(self.old_signed_signature)
        self.denied("SSHSIG_SIGNATURE_VERIFICATION_FAILED",
                    epoch=2, seq=2, gen=2)

    def test_22_new_key_replays_genuine_alternative_noi(self):
        self.rotate()
        row = self.good("verify", epoch=2, seq=2, gen=2,
                        source=self.fx.fx.fx.fx.fx.alt_source)[1][0]
        self.assertEqual(row["native_jobs_recomputed"], 2)

    def test_23_old_noi_source_remains_denied_after_rotation(self):
        self.rotate()
        self.denied("SOURCE_PIN_MISMATCH", "verify", epoch=2, seq=2, gen=2)

    def test_24_rotated_authority_restores_real_native_checkpoint(self):
        self.rotate()
        row = self.good("restore", epoch=2, seq=2, gen=2,
                        source=self.fx.fx.fx.fx.fx.alt_source)[1][0]
        self.assertEqual(row["status"], "RESTORED")
        self.assertEqual(row["authority_epoch"], 2)

    def test_25_rotated_authority_resumes_real_native_suffix(self):
        self.rotate()
        source = self.fx.fx.fx.fx.fx.alt_source
        self.good("restore", epoch=2, seq=2, gen=2, source=source)
        _, rows = self.good("resume", epoch=2, seq=2, gen=2, source=source)
        self.assertEqual([x["step"] for x in rows], [3,4])

    def test_26_revoked_root_key_cannot_be_readmitted(self):
        self.rotate()
        event = self.make_event(
            "ROTATE", 3, "ACTIVE", self.root_public,
            [self.root_sha, self.successor_sha], 3, 3)
        self.records.append(self.record(
            event, self.successor_private, self.root_private))
        self.write_chain()
        self.denied("AUTHORITY_REVOKED_KEY_READMITTED",
                    epoch=2, seq=2, gen=2)

    def test_27_rotation_requires_incoming_signature(self):
        self.rotate()
        self.records[1]["incoming_signature_b64"] = None
        self.write_chain()
        self.denied("AUTHORITY_INCOMING_POSSESSION_PROOF_REQUIRED",
                    epoch=2, seq=2, gen=2)

    def test_28_rotation_requires_valid_outgoing_signature(self):
        self.rotate()
        self.records[1]["outgoing_signature_b64"] = self.signature(
            self.records[1]["event"], self.successor_private)
        self.write_chain()
        self.denied("AUTHORITY_SIGNATURE_VERIFICATION_FAILED",
                    epoch=2, seq=2, gen=2)

    def test_29_rotation_requires_valid_incoming_signature(self):
        self.rotate()
        self.records[1]["incoming_signature_b64"] = self.signature(
            self.records[1]["event"], self.root_private)
        self.write_chain()
        self.denied("AUTHORITY_SIGNATURE_VERIFICATION_FAILED",
                    epoch=2, seq=2, gen=2)

    def test_30_chain_wrong_previous_hash_refused(self):
        self.rotate()
        self.records[1]["event"]["previous_record_sha256"] = "0"*64
        self.write_chain()
        self.denied("AUTHORITY_CHAIN_SHA256_LINK_INVALID", epoch=2, seq=2, gen=2)

    def test_31_receiver_old_chain_pin_denied_after_rotation(self):
        old_sha = self.chain_sha
        self.rotate()
        self.denied("AUTHORITY_CHAIN_EXTERNAL_PIN_MISMATCH",
                    pin=old_sha, epoch=2, seq=2, gen=2)

    def test_32_receiver_epoch_floor_blocks_bootstrap_after_enrollment(self):
        self.denied("AUTHORITY_EPOCH_BELOW_RECEIVER_FLOOR", epoch=2)

    def test_33_rotation_nonmonotone_sequence_and_generation_refused(self):
        self.rotate()
        self.records[1]["event"]["minimum_sequence"] = 1
        self.records[1]["event"]["minimum_generation"] = 1
        self.write_chain()
        self.denied("AUTHORITY_NONMONOTONE_RECEIVER_FLOOR",
                    epoch=2, seq=2, gen=2)

    def test_34_tampered_revocation_history_refused(self):
        self.rotate()
        self.records[1]["event"]["retired_public_key_sha256"] = []
        self.write_chain()
        self.denied("AUTHORITY_REVOKED_KEY_HISTORY_MISMATCH",
                    epoch=2, seq=2, gen=2)

    def test_35_signature_of_root_event_cannot_be_forged(self):
        self.records[0]["outgoing_signature_b64"] = self.signature(
            self.records[0]["event"], self.root_private,
            namespace="wrong-authority-ns")
        self.write_chain()
        self.denied("AUTHORITY_SIGNATURE_VERIFICATION_FAILED")

    def test_36_signed_freeze_is_observable(self):
        self.freeze()
        row = self.good(epoch=2, seq=2, gen=2)[1][0]
        self.assertEqual(row["status"], "FROZEN")

    def test_37_signed_freeze_denies_native_inspection(self):
        self.freeze()
        self.denied("AUTHORITY_FROZEN_DENIES_NATIVE_ADMISSION",
                    "inspect", epoch=2, seq=2, gen=2)

    def test_38_signed_freeze_denies_restore_without_effect(self):
        self.freeze()
        self.denied("AUTHORITY_FROZEN_DENIES_NATIVE_ADMISSION",
                    "restore", epoch=2, seq=2, gen=2)
        self.assertFalse(any(self.fx.fx.fx.fx.fx.dest.iterdir()))

    def test_39_complete_old_journal_chain_and_floors_can_be_replayed_limit(self):
        preserved = self.work/"old-entire-journal"
        shutil.copytree(self.root, preserved)
        old_chain = copy.deepcopy(self.records)
        old_doc = self.fx.document.read_bytes()
        old_sig = self.fx.signature.read_bytes()
        self.rotate()
        shutil.rmtree(self.root)
        shutil.copytree(preserved, self.root)
        self.records = old_chain
        self.write_chain()
        self.fx.document.write_bytes(old_doc)
        self.fx.signature.write_bytes(old_sig)
        r = self.good(epoch=1, seq=1, gen=1)[1][0]
        self.assertEqual(r["authority_epoch"], 1)
        self.assertEqual(r["global_anti_rollback"], "NOT_PROVIDED")

    def test_40_report_disclaims_global_rollback_and_production(self):
        row = self.good()[1][0]
        self.assertEqual(row["global_anti_rollback"], "NOT_PROVIDED")
        self.assertEqual(row["externally_trusted_chain_pin"],
                         "RECEIVER_SUPPLIED_NOT_ATTESTED")
        self.assertEqual(row["production"], "NOT_AUTHORIZED")


if __name__ == "__main__":
    unittest.main()
