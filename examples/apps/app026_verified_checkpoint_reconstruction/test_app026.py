"""Native integration tests using actual APP-025 Rust-generated evidence."""
from pathlib import Path
import sys
import unittest
import hashlib
import shutil
import subprocess
import json
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent/"app025_interrupted_dag_forensics"))
import test_app025 as previous
import reconstruct_app026 as app


class Reconstruction(unittest.TestCase):
    def setUp(self):
        self.f = previous.NativeInterruptedForensicTests("test_03_single_verified_new_job_requires_new_checkpoint")
        self.f.setUp()
        self.addCleanup(self.f.tearDown)
        self.dest = self.f.base/"reconstructed-app026"

    def run_app(self, **overrides):
        f = self.f
        args = dict(checkpoint=f.bundle, checkpoint_pin=f.pin, root=f.root,
                    suffix=f.suffix, suffix_pin=f.suffix_pin, target=self.dest)
        args.update(overrides)
        return app.reconstruct(**args)

    def reject(self, **overrides):
        with self.assertRaises(Exception):
            self.run_app(**overrides)
        self.assertFalse(self.dest.exists())

    def test_01_one_extension_real_portable(self):
        self.f.fresh_partial(1)
        r = self.run_app()
        m, _, events = app.portable.verify(self.dest, r["new_checkpoint_sha256"])
        self.assertEqual((len(events), m["committed"], r["remaining_nodes"]), (3,3,3))
        self.assertEqual(r["native_processes_started"], 0)
        self.assertEqual(r["next_action"], "INDEPENDENTLY_PIN_AND_VERIFY_NEW_CHECKPOINT_BEFORE_RESUME")

    def test_02_two_extension_restore_and_history(self):
        self.f.fresh_partial(2)
        r = self.run_app()
        restored = self.f.base/"app026-restored"
        restored.mkdir()
        app.portable.restore(self.dest, r["new_checkpoint_sha256"], restored)
        self.assertEqual(self.f.fixture.native_bytes(restored), self.f.fixture.native_bytes(self.f.root))
        self.assertEqual(r["remaining_nodes"],2)

    def test_03_unchanged_checkpoint_rejected(self):
        self.reject()

    def test_04_complete_dag_rejected(self):
        self.f.capture_success()
        self.reject()

    def test_05_orphan_effect_rejected(self):
        self.f.capture_success()
        self.f.write_suffix(b"")
        self.reject()

    def test_06_suffix_pin_rejected(self):
        self.f.fresh_partial()
        self.reject(suffix_pin="0"*64)

    def test_07_checkpoint_pin_rejected(self):
        self.f.fresh_partial()
        self.reject(checkpoint_pin="0"*64)

    def test_08_truncated_suffix_rejected(self):
        self.f.fresh_partial()
        self.f.write_suffix(self.f.suffix.read_bytes()[:-1])
        self.reject()

    def test_09_existing_destination_not_overwritten(self):
        self.f.fresh_partial()
        self.dest.mkdir()
        marker = self.dest/"sentinel"
        marker.write_bytes(b"keep")
        with self.assertRaises(Exception):
            self.run_app()
        self.assertEqual(marker.read_bytes(), b"keep")

    def test_10_no_output_inside_evidence(self):
        self.f.fresh_partial()
        for root in (self.f.root, self.f.bundle):
            target = root/"forbidden"
            with self.assertRaises(Exception):
                self.run_app(target=target)
            self.assertFalse(target.exists())

    def test_11_original_bytes_preserved(self):
        self.f.fresh_partial()
        def snapshot(root):
            return {str(p.relative_to(root)):p.read_bytes() for p in root.rglob("*") if p.is_file()}
        before = (snapshot(self.f.bundle), snapshot(self.f.root), self.f.suffix.read_bytes())
        r = self.run_app()
        self.assertEqual(before,(snapshot(self.f.bundle), snapshot(self.f.root), self.f.suffix.read_bytes()))
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertEqual(r["production"],"NOT_AUTHORIZED")

    def test_12_new_checkpoint_can_be_audited_without_replay(self):
        self.f.fresh_partial()
        r = self.run_app()
        empty = self.f.base/"empty-suffix"
        empty.write_bytes(b"")
        report = app.forensic.reconcile(self.dest,r["new_checkpoint_sha256"],self.f.root,
                                        empty,hashlib.sha256(b"").hexdigest())
        self.assertEqual(report["status"],"ORIGINAL_CHECKPOINT_REUSABLE")
        self.assertEqual(report["verified_events_total"],3)


if __name__ == "__main__":
    unittest.main()
