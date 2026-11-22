"""APP-014: 40 real Rust-backed portable checkpoint and adversarial tests.

Positive files are published by actual compiled NORDOI Rust via APP-012, then
sealed by APP-013. This suite copies, verifies, restores and resumes those
genuine native P2.7 bundles; negative tests tamper with genuine evidence.
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
SOURCE = REPO / "examples/apps/app003_integrated_quality/src"
ENGINE = REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
sys.path.insert(0, str(HERE))
import portable_app014 as portable
sys.path.insert(0, str(HERE.parent / "app012_clean_checkpoint"))
import run_app012 as checkpoint


def node(code: int, dep=None, gate="always"):
    return {"key_code": code, "depends_on": [] if dep is None else dep,
            "gate": gate}


class PortableNativeEvidence(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nordoi_app014_real_")
        self.base = Path(self.temp.name)
        self.native = self.base / "real-native"
        self.native.mkdir()
        self.sealed = self.base / "original-app013-sealed"
        self.bundle = self.base / "portable-artifact"
        self.restored = self.base / "restored-real-native"
        self.restored.mkdir()
        self.plan = self.base / "original-plan.json"
        self.events = self.base / "authentic-native-prefix.jsonl"
        self.engine = ENGINE
        self.source = SOURCE
        self.engine_pin = checkpoint.pinned.engine_sha256(self.engine)
        self.source_pin = checkpoint.pinned.source_tree_sha256(self.source)
        self.create([node(73), node(59), node(60, [1, 2], "all_committed"),
                     node(80, [3], "all_approved")], stop=2)

    def tearDown(self):
        self.temp.cleanup()

    def create(self, nodes, stop=2):
        if self.native.exists():
            shutil.rmtree(self.native)
        self.native.mkdir()
        self.plan.write_bytes(portable.serialize(
            {"schema": "NORDOI_AGENT_DAG_V1", "nodes": nodes}))
        self.plan_pin = hashlib.sha256(self.plan.read_bytes()).hexdigest()
        p = subprocess.run([
            sys.executable, "-B",
            str(HERE.parent / "app012_clean_checkpoint/run_app012.py"),
            "--plan", str(self.plan), "--engine", str(self.engine),
            "--source-root", str(self.source),
            "--grant-output-dir", str(self.native),
            "--expected-plan-sha256", self.plan_pin,
            "--expected-engine-sha256", self.engine_pin,
            "--expected-source-sha256", self.source_pin,
            "--stop-after", str(stop),
        ], capture_output=True, cwd=REPO, timeout=150)
        self.assertEqual(p.returncode, 0, (p.stdout, p.stderr))
        self.events.write_bytes(p.stdout)
        if self.sealed.exists():
            shutil.rmtree(self.sealed)
        seal = subprocess.run([
            sys.executable, "-B",
            str(HERE.parent / "app013_verified_handoff/handoff_app013.py"),
            "seal", "--bundle-dir", str(self.sealed),
            "--native-root", str(self.native), "--plan", str(self.plan),
            "--prefix-events", str(self.events),
            "--expected-plan-sha256", self.plan_pin
        ], capture_output=True, cwd=REPO, timeout=35)
        self.assertEqual(seal.returncode, 0, (seal.stdout, seal.stderr))
        self.inner_pin = json.loads(seal.stdout)["manifest_sha256"]

    def call(self, action, *, pin=None, bundle=None, dest=None,
             inner_pin=None, sealed=None, native=None):
        cmd = [sys.executable, "-B", str(HERE / "portable_app014.py"),
               action, "--portable-dir", str(self.bundle if bundle is None else bundle)]
        if action == "pack":
            cmd.extend([
                "--handoff-dir", str(self.sealed if sealed is None else sealed),
                "--native-root", str(self.native if native is None else native),
                "--expected-handoff-sha256", self.inner_pin if inner_pin is None else inner_pin,
            ])
        else:
            cmd.extend(["--expected-portable-sha256",
                        self.outer_pin if pin is None else pin])
        if action in ("restore", "resume"):
            cmd.extend(["--restore-root",
                        str(self.restored if dest is None else dest)])
        if action == "resume":
            cmd.extend([
                "--engine", str(self.engine), "--source-root", str(self.source),
                "--expected-engine-sha256", self.engine_pin,
                "--expected-source-sha256", self.source_pin,
            ])
        p = subprocess.run(cmd, capture_output=True, cwd=REPO, timeout=155)
        try:
            rows = [json.loads(x) for x in p.stdout.splitlines()]
        except (ValueError, UnicodeError):
            self.fail(f"non-json output: {p.stdout!r}, {p.stderr!r}")
        return p, rows

    def packed(self):
        p, rows = self.call("pack")
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["status"], "PACKED")
        self.outer_pin = rows[0]["portable_sha256"]
        self.assertEqual({x.name for x in self.bundle.iterdir()},
                         portable.PORTABLE_FILES)
        return rows[0]

    def verified(self, *, bundle=None):
        p, rows = self.call("verify", bundle=bundle)
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        self.assertEqual(rows[0]["status"], "VERIFIED")
        return rows[0]

    def restored_ok(self):
        p, rows = self.call("restore")
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        self.assertEqual(rows[0]["status"], "RESTORED")
        return rows[0]

    def rejected(self, action, code, **kw):
        p, rows = self.call(action, **kw)
        self.assertEqual(p.returncode, 2, (rows, p.stderr))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["status"], "REJECTED")
        self.assertEqual(rows[0]["error_code"], code)
        self.assertEqual(rows[0]["publication"], "NOT_STARTED")

    def rewrite_outer(self, mutation):
        path = self.bundle / "portable.json"
        raw = json.loads(path.read_bytes())
        mutation(raw)
        result = portable.serialize(raw)
        path.write_bytes(result)
        self.outer_pin = hashlib.sha256(result).hexdigest()
        (self.bundle / "PORTABLE.sha256").write_bytes(
            (self.outer_pin + "\n").encode("ascii"))

    def native_bytes(self, root=None):
        if root is None:
            root = self.native
        return {p.relative_to(root).as_posix(): p.read_bytes()
                for p in root.rglob("*") if p.is_file()}

    def test_01_real_native_checkpoint_is_portably_packaged(self):
        r = self.packed()
        self.assertEqual(r["native_jobs"], 2)
        self.assertEqual(r["native_files"], 4)

    def test_02_portable_package_verifies_without_original_native_path(self):
        self.packed()
        self.assertEqual(self.verified()["native_files"], 4)

    def test_03_original_plan_is_byte_identical(self):
        self.packed()
        self.assertEqual((self.bundle / "handoff/plan.json").read_bytes(),
                         self.plan.read_bytes())

    def test_04_original_prefix_is_byte_identical(self):
        self.packed()
        self.assertEqual(
            (self.bundle / "handoff/native-prefix.jsonl").read_bytes(),
            self.events.read_bytes())

    def test_05_genuine_native_p27_file_bytes_are_identical(self):
        self.packed()
        self.assertEqual(self.native_bytes(),
                         self.native_bytes(self.bundle / "native"))

    def test_06_external_portable_digest_matches_exact_manifest_bytes(self):
        self.packed()
        self.assertEqual(self.outer_pin, hashlib.sha256(
            (self.bundle / "portable.json").read_bytes()).hexdigest())

    def test_07_same_authentic_native_checkpoint_produces_same_manifest(self):
        self.packed()
        manifest = (self.bundle / "portable.json").read_bytes()
        outer = self.outer_pin
        self.bundle = self.base / "different-location"
        self.packed()
        self.assertEqual(manifest, (self.bundle / "portable.json").read_bytes())
        self.assertEqual(outer, self.outer_pin)

    def test_08_portable_manifest_contains_no_absolute_machine_paths(self):
        self.packed()
        raw = (self.bundle / "portable.json").read_text()
        self.assertNotIn(str(self.base), raw)
        self.assertNotIn(str(REPO), raw)

    def test_09_copy_portable_directory_then_verify_from_new_location(self):
        self.packed()
        other = self.base / "relocated-copy"
        shutil.copytree(self.bundle, other)
        r = self.verified(bundle=other)
        self.assertEqual(r["native_jobs"], 2)

    def test_10_restore_to_empty_native_root(self):
        self.packed()
        self.assertEqual(self.restored_ok()["native_jobs"], 2)

    def test_11_restored_native_files_identical_to_original(self):
        self.packed()
        self.restored_ok()
        self.assertEqual(self.native_bytes(self.restored), self.native_bytes())

    def test_12_restored_jobs_independently_audited_app010(self):
        self.packed()
        self.restored_ok()
        report, code = portable.independent.audit(
            self.restored, self.bundle / "handoff/plan.json",
            self.bundle / "handoff/native-prefix.jsonl", self.plan_pin)
        self.assertEqual(code, 3, report)
        self.assertEqual(report["status"], "INCONCLUSIVE")
        self.assertEqual(report["committed_jobs_verified"], 2)

    def test_13_resume_after_restore_invokes_real_rust_engine(self):
        self.packed()
        self.restored_ok()
        p, rows = self.call("resume")
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        self.assertEqual([r["step"] for r in rows], [3, 4])
        self.assertEqual([r["status"] for r in rows],
                         ["COMMITTED", "COMMITTED"])

    def test_14_full_dag_passes_independent_app010_after_resume(self):
        self.packed()
        self.restored_ok()
        p, _ = self.call("resume")
        self.assertEqual(p.returncode, 0, p.stderr)
        joined = self.base / "complete-real-dag.jsonl"
        joined.write_bytes((self.bundle / "handoff/native-prefix.jsonl").read_bytes()
                           + p.stdout)
        report, status = portable.independent.audit(
            self.restored, self.bundle / "handoff/plan.json",
            joined, self.plan_pin)
        self.assertEqual(status, 0, report)
        self.assertEqual(report["committed_jobs_verified"], 4)

    def test_15_original_native_files_unchanged_after_export_and_restore(self):
        baseline = self.native_bytes()
        self.packed()
        self.restored_ok()
        self.assertEqual(self.native_bytes(), baseline)

    def test_16_original_native_files_unchanged_after_verified_resume(self):
        original = self.native_bytes()
        self.packed()
        self.restored_ok()
        self.assertEqual(self.call("resume")[0].returncode, 0)
        self.assertEqual(self.native_bytes(), original)

    def test_17_manifest_without_externally_pinned_sha_refused(self):
        self.packed()
        self.rejected("verify", "PORTABLE_EXTERNAL_PIN_MISMATCH",
                      pin="0"*64)

    def test_18_wrong_external_sha_denies_restore_before_effect(self):
        self.packed()
        self.rejected("restore", "PORTABLE_EXTERNAL_PIN_MISMATCH",
                      pin="0"*64)
        self.assertFalse(list(self.restored.iterdir()))

    def test_19_wrong_external_sha_denies_resume_before_effect(self):
        self.packed()
        self.restored_ok()
        self.rejected("resume", "PORTABLE_EXTERNAL_PIN_MISMATCH",
                      pin="0"*64)
        self.assertEqual(len(list(self.restored.iterdir())), 2)

    def test_20_seal_marker_missing_is_rejected(self):
        self.packed()
        (self.bundle / "PORTABLE.sha256").unlink()
        self.rejected("verify", "PORTABLE_FILE_SET_CHANGED")

    def test_21_modified_seal_marker_is_rejected(self):
        self.packed()
        (self.bundle / "PORTABLE.sha256").write_bytes(b"invalid\n")
        self.rejected("verify", "PORTABLE_MARKER_MISMATCH")

    def test_22_modified_portable_manifest_rejected(self):
        self.packed()
        m = self.bundle / "portable.json"
        m.write_bytes(m.read_bytes() + b"forged")
        self.rejected("verify", "PORTABLE_EXTERNAL_PIN_MISMATCH")

    def test_23_forged_native_file_rejected(self):
        self.packed()
        target = self.bundle / "native/job-0001/quality-decision/decision.txt"
        target.write_bytes(target.read_bytes() + b"forged")
        self.rejected("verify", "REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")

    def test_24_native_file_deleted_is_rejected(self):
        self.packed()
        (self.bundle / "native/job-0002/quality-decision/report.txt").unlink()
        self.rejected("verify", "BUNDLE_FILE_SET_CHANGED")

    def test_25_extra_native_file_is_rejected(self):
        self.packed()
        (self.bundle / "native/job-0001/quality-decision/other.txt").write_bytes(b"x")
        self.rejected("verify", "BUNDLE_FILE_SET_CHANGED")

    def test_26_extra_native_job_is_rejected(self):
        self.packed()
        (self.bundle / "native/job-0003").mkdir()
        self.rejected("verify", "ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_27_missing_inner_handoff_marker_is_rejected(self):
        self.packed()
        (self.bundle / "handoff/SEAL.sha256").unlink()
        self.rejected("verify", "HANDOFF_FILE_SET_CHANGED")

    def test_28_modified_inner_plan_is_rejected(self):
        self.packed()
        file = self.bundle / "handoff/plan.json"
        file.write_bytes(file.read_bytes() + b"\n")
        self.rejected("verify", "PLAN_SHA256_PIN_MISMATCH")

    def test_29_modified_inner_native_events_are_rejected(self):
        self.packed()
        file = self.bundle / "handoff/native-prefix.jsonl"
        file.write_bytes(file.read_bytes() + b"\n")
        self.rejected("verify", "EVENT_LINE_INVALID")

    def test_30_extra_file_in_portable_root_is_rejected(self):
        self.packed()
        (self.bundle / "unexpected").write_bytes(b"malicious")
        self.rejected("verify", "PORTABLE_FILE_SET_CHANGED")

    def test_31_forged_native_digest_with_reanchored_manifest_is_rejected(self):
        self.packed()
        self.rewrite_outer(lambda m: m["native_files"][0].__setitem__(
            "sha256", "0"*64))
        self.rejected("verify", "PORTABLE_CONTENT_CONTRADICTION")

    def test_32_forged_signature_claim_even_with_new_anchor_is_rejected(self):
        self.packed()
        self.rewrite_outer(lambda m: m.__setitem__("signature", "SIGNED"))
        self.rejected("verify", "PORTABLE_TRUST_CLAIM_INVALID")

    def test_33_forged_inner_manifest_digest_with_new_anchor_is_rejected(self):
        self.packed()
        self.rewrite_outer(lambda m: m.__setitem__(
            "inner_manifest_sha256", "0"*64))
        self.rejected("verify", "HANDOFF_SEAL_PIN_MISMATCH")

    def test_34_existing_portable_destination_cannot_be_overwritten(self):
        self.packed()
        baseline = (self.bundle / "portable.json").read_bytes()
        self.rejected("pack", "PORTABLE_DESTINATION_EXISTS")
        self.assertEqual((self.bundle / "portable.json").read_bytes(),
                         baseline)

    def test_35_partial_destination_without_marker_cannot_be_reused(self):
        self.bundle.mkdir()
        (self.bundle / "portable.json").write_bytes(b"partial")
        self.rejected("pack", "PORTABLE_DESTINATION_EXISTS")
        self.assertEqual((self.bundle / "portable.json").read_bytes(),
                         b"partial")

    def test_36_restore_into_nonempty_root_refused(self):
        self.packed()
        marker = self.restored / "existing.txt"
        marker.write_bytes(b"preserve")
        self.rejected("restore", "RESTORE_ROOT_NOT_EMPTY")
        self.assertEqual(marker.read_bytes(), b"preserve")

    def test_37_second_restore_refused_without_overwrite(self):
        self.packed()
        self.restored_ok()
        history = self.native_bytes(self.restored)
        self.rejected("restore", "RESTORE_ROOT_NOT_EMPTY")
        self.assertEqual(self.native_bytes(self.restored), history)

    def test_38_tampered_restored_native_file_blocks_resume(self):
        self.packed()
        self.restored_ok()
        path = self.restored / "job-0001/quality-decision/report.txt"
        path.write_bytes(path.read_bytes() + b"x")
        self.rejected("resume", "REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")
        self.assertFalse((self.restored / "job-0003").exists())

    def test_39_replaying_checkpoint_after_success_is_rejected(self):
        self.packed()
        self.restored_ok()
        p, rows = self.call("resume")
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        self.rejected("resume", "ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_40_historical_skipped_node_is_portable_and_resumable(self):
        self.create([node(59), node(73, [1], "all_approved"),
                     node(60, [2], "all_committed"), node(73)], stop=3)
        self.packed()
        self.assertEqual(self.verified()["native_jobs"], 1)
        self.assertEqual(self.restored_ok()["native_jobs"], 1)
        p, rows = self.call("resume")
        self.assertEqual(p.returncode, 0, (rows, p.stderr))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["step"], 4)
        self.assertEqual(rows[0]["status"], "COMMITTED")


if __name__ == "__main__":
    unittest.main()
