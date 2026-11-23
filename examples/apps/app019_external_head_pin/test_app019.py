"""APP-019: 40 native-backed tests for externally supplied HEAD SHA-256.

These tests use the genuine APP-017/018 Rust checkpoint fixture. The
"complete journal + stale external pin" test deliberately demonstrates the
remaining limitation: NORDOI does not authenticate that external pin.
"""
from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
import receiver_app019 as external
sys.path.insert(0, str(HERE.parent / "app018_local_freshness"))
import test_app018 as fixture  # module alias, NOT a TestCase exported by discovery


class ReceiverExternalHeadPin(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        fixture.LocalReceiverFreshness.setUpClass()

    @classmethod
    def tearDownClass(cls):
        fixture.LocalReceiverFreshness.tearDownClass()

    def setUp(self):
        self.fx = fixture.LocalReceiverFreshness(
            "test_01_bootstrap_creates_exactly_one_local_head")
        self.fx.setUp()
        self.root = self.fx.root
        self.pin = hashlib.sha256((self.root / "HEAD.json").read_bytes()).hexdigest()
        self.old_pin = self.pin

    def tearDown(self):
        self.fx.tearDown()

    def call(self, action="status", *, pin=None, with_pin=True,
             roster_file=None, roster_sha=None, policy_file=None,
             package=None, source=None, engine=None, dest=None, root=None,
             extra=()):
        args = [
            sys.executable, "-B", str(HERE / "receiver_app019.py"), action,
            "--anchor-root", str(self.root if root is None else root),
        ]
        if with_pin and action not in ("bootstrap", "capture"):
            args += ["--expected-head-sha256", self.pin if pin is None else pin]
        if action in ("bootstrap", "advance"):
            args += ["--receiver-roster", str(self.fx.fx.roster_file if roster_file is None else roster_file),
                     "--expected-roster-sha256", self.fx.fx.anchor if roster_sha is None else roster_sha]
        if action in ("inspect", "verify", "restore", "resume"):
            args += ["--receiver-roster", str(self.fx.fx.roster_file if roster_file is None else roster_file),
                     "--receiver-policy", str(self.fx.fx.fx.policy_file if policy_file is None else policy_file),
                     "--portable-dir", str(self.fx.fx.fx.portable if package is None else package)]
        if action in ("verify", "restore", "resume"):
            args += ["--engine", str(fixture.original_fixture.ENGINE if engine is None else engine),
                     "--source-root", str(fixture.original_fixture.SOURCE if source is None else source)]
        if action in ("restore", "resume"):
            args += ["--restore-root", str(self.fx.fx.fx.dest if dest is None else dest)]
        args += list(extra)
        process = subprocess.run(args, cwd=REPO, capture_output=True, timeout=190)
        try:
            rows = [json.loads(row) for row in process.stdout.splitlines()]
        except ValueError:
            self.fail(f"NON-JSON response: {process.stdout!r}; {process.stderr!r}")
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

    def rotate(self):
        self.fx.fx.rotated()
        result = self.good("advance")[1][0]
        self.pin = result["head_sha256"]
        return result

    def freeze(self):
        self.fx.fx.frozen()
        result = self.good("advance")[1][0]
        self.pin = result["head_sha256"]
        return result

    def test_01_capture_is_explicitly_untrusted(self):
        row = self.good("capture")[1][0]
        self.assertEqual(row["status"], "UNAUTHENTICATED_CANDIDATE")

    def test_02_capture_matches_head_bytes(self):
        self.assertEqual(self.good("capture")[1][0]["head_sha256"], self.pin)

    def test_03_status_requires_external_pin(self):
        self.refused("HEAD_PIN_EXTERNAL_REFERENCE_REQUIRED", with_pin=False)

    def test_04_invalid_external_pin_rejected(self):
        self.refused("HEAD_PIN_INVALID", pin="not-a-sha")

    def test_05_well_formed_wrong_external_pin_rejected(self):
        self.refused("HEAD_PIN_MISMATCH", pin="0" * 64)

    def test_06_current_head_pin_allows_status(self):
        self.assertEqual(self.good()[1][0]["status"], "ACTIVE")

    def test_07_status_disclaims_external_authentication(self):
        self.assertEqual(self.good()[1][0]["external_pin_authentication"], "NOT_PERFORMED")

    def test_08_status_does_not_run_rust(self):
        self.assertEqual(self.good()[1][0]["native_engine"], "NOT_EXECUTED")

    def test_09_capture_does_not_run_rust(self):
        self.assertEqual(self.good("capture")[1][0]["native_engine"], "NOT_EXECUTED")

    def test_10_head_pin_survives_separate_python_process(self):
        self.assertEqual(self.good()[1][0]["head_sha256"], self.pin)

    def test_11_original_policy_inspects_with_current_pin(self):
        self.assertEqual(self.good("inspect")[1][0]["status"], "POLICY_MATCH")

    def test_12_inspect_does_not_run_rust(self):
        self.assertEqual(self.good("inspect")[1][0]["native_engine"], "NOT_EXECUTED")

    def test_13_real_pinned_native_replay_succeeds(self):
        row = self.good("verify")[1][0]
        self.assertEqual(row["native_jobs_recomputed"], 2)
        self.assertTrue(row["native_receipts_exactly_equal"])

    def test_14_real_native_restore_is_head_gated(self):
        self.assertEqual(self.good("restore")[1][0]["status"], "RESTORED")

    def test_15_real_native_resume_preserves_suffix_only(self):
        self.good("restore")
        _, rows = self.good("resume")
        self.assertEqual([row["step"] for row in rows], [3, 4])

    def test_16_second_native_resume_is_refused(self):
        self.good("restore")
        self.good("resume")
        self.refused("ROOT_UNEXPECTED_JOB_OR_FILE", "resume")

    def test_17_receiver_pinned_rotation_generates_new_head(self):
        row = self.rotate()
        self.assertNotEqual(row["head_sha256"], self.old_pin)
        self.assertEqual(row["roster_generation"], 2)

    def test_18_old_external_pin_refused_after_rotation(self):
        self.rotate()
        self.refused("HEAD_PIN_MISMATCH", pin=self.old_pin)

    def test_19_new_external_pin_allows_status(self):
        self.rotate()
        self.assertEqual(self.good()[1][0]["roster_generation"], 2)

    def test_20_old_external_pin_refuses_inspect(self):
        self.rotate()
        self.refused("HEAD_PIN_MISMATCH", "inspect", pin=self.old_pin)

    def test_21_old_external_pin_refuses_native_verify(self):
        self.rotate()
        self.refused("HEAD_PIN_MISMATCH", "verify", pin=self.old_pin)

    def test_22_old_external_pin_refuses_native_restore(self):
        self.rotate()
        self.refused("HEAD_PIN_MISMATCH", "restore", pin=self.old_pin)
        self.assertFalse(list(self.fx.fx.fx.dest.iterdir()))

    def test_23_old_external_pin_refuses_native_resume(self):
        self.rotate()
        self.refused("HEAD_PIN_MISMATCH", "resume", pin=self.old_pin)
        self.assertFalse(list(self.fx.fx.fx.dest.iterdir()))

    def test_24_original_policy_stays_revoked_under_new_head(self):
        self.rotate()
        self.refused("RECEIVER_POLICY_EXTERNAL_PIN_MISMATCH",
                     "inspect", policy_file=self.fx.fx.original_policy,
                     package=self.fx.fx.original_pkg)

    def test_25_new_policy_replays_alternate_real_noi_source(self):
        self.rotate()
        row = self.good("verify", source=self.fx.fx.fx.alt_source)[1][0]
        self.assertEqual(row["native_jobs_recomputed"], 2)

    def test_26_new_policy_rejects_original_source(self):
        self.rotate()
        self.refused("SOURCE_PIN_MISMATCH", "verify")

    def test_27_new_policy_restores_and_resumes_real_rust(self):
        self.rotate()
        source = self.fx.fx.fx.alt_source
        self.good("restore", source=source)
        _, rows = self.good("resume", source=source)
        self.assertEqual([row["step"] for row in rows], [3, 4])

    def test_28_frozen_state_refuses_inspect(self):
        self.freeze()
        self.refused("ROSTER_FROZEN_DENIES_ADMISSION", "inspect")

    def test_29_frozen_state_refuses_native_restore(self):
        self.freeze()
        self.refused("ROSTER_FROZEN_DENIES_ADMISSION", "restore")
        self.assertFalse(list(self.fx.fx.fx.dest.iterdir()))

    def test_30_frozen_state_allows_status_not_execution(self):
        self.freeze()
        self.assertEqual(self.good()[1][0]["status"], "FROZEN")

    def test_31_frozen_state_refuses_native_replay(self):
        self.freeze()
        self.refused("ROSTER_FROZEN_DENIES_ADMISSION", "verify")

    def test_32_frozen_state_refuses_native_resume(self):
        self.freeze()
        self.refused("ROSTER_FROZEN_DENIES_ADMISSION", "resume")

    def test_33_advance_requires_old_external_pin(self):
        self.fx.fx.rotated()
        self.refused("HEAD_PIN_EXTERNAL_REFERENCE_REQUIRED",
                     "advance", with_pin=False)

    def test_34_advance_refuses_wrong_old_pin_without_write(self):
        self.fx.fx.rotated()
        self.refused("HEAD_PIN_MISMATCH", "advance", pin="f" * 64)
        self.assertEqual(self.good()[1][0]["journal_sequence"], 1)

    def test_35_advance_refuses_wrong_new_roster_pin(self):
        self.fx.fx.rotated()
        self.refused("ROSTER_EXTERNAL_PIN_MISMATCH", "advance",
                     roster_sha="0" * 64)
        self.assertEqual(self.good()[1][0]["journal_sequence"], 1)

    def test_36_corrupted_head_refuses_even_matching_old_pin(self):
        file = self.root / "HEAD.json"
        file.write_bytes(file.read_bytes() + b"CORRUPTION")
        self.refused("ANCHOR_HEAD_NOT_CANONICAL_JSON")

    def test_37_coherent_full_journal_rollback_refused_with_new_external_pin(self):
        preserved = self.fx.fx.fx.work / "coherent-old-local-anchor"
        shutil.copytree(self.root, preserved)
        self.rotate()
        newest = self.pin
        shutil.rmtree(self.root)
        shutil.copytree(preserved, self.root)
        self.refused("HEAD_PIN_MISMATCH", pin=newest)

    def test_38_coherent_full_journal_plus_old_pin_remains_accepted_limit(self):
        preserved = self.fx.fx.fx.work / "coherent-old-local-anchor"
        shutil.copytree(self.root, preserved)
        self.rotate()
        shutil.rmtree(self.root)
        shutil.copytree(preserved, self.root)
        row = self.good(pin=self.old_pin)[1][0]
        self.assertEqual(row["roster_generation"], 1)
        self.assertEqual(row["external_pin_authentication"], "NOT_PERFORMED")

    def test_39_symlinked_head_fails_closed_when_supported(self):
        replacement = self.fx.fx.fx.work / "elsewhere-head.json"
        head_file = self.root / "HEAD.json"
        replacement.write_bytes(head_file.read_bytes())
        head_file.unlink()
        try:
            head_file.symlink_to(replacement)
        except (OSError, NotImplementedError):
            self.skipTest("runner does not permit symlinks")
        self.refused("ANCHOR_HEAD_NOT_REGULAR_BOUNDED")

    def test_40_report_never_claims_external_certificate_or_production(self):
        row = self.good()[1][0]
        self.assertEqual(row["external_security_certification"], "NOT_CLAIMED")
        self.assertEqual(row["global_anti_rollback"], "NOT_PROVIDED")
        self.assertEqual(row["signature"], "NOT_PRESENT")
        self.assertEqual(row["production"], "NOT_AUTHORIZED")


if __name__ == "__main__":
    unittest.main()
