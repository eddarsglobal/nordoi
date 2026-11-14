"""NORDOI G0.3 R6F BUREAU-001 R1 — 24 real offline source/audit challenges.

All mutation tests edit disposable copies of actual Git-pinned source files.
The check does not grant native authority, run independent reviewer agents,
contact an external custodian, or certify an owner's submitted log.
"""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from scripts import bureau_001_r1_audit as lab

ADD_FILES = (
    "CONSTITUTION.md", "governance/future_native_gate_v1.tsv",
    lab.MANIFEST, lab.CHRONOLOGY,
)

class Bureau001EvidenceAudit(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="nordoi-bureau001-r1-")
        self.root = Path(self.tmp.name)
        m = json.loads((ROOT / lab.MANIFEST).read_text(encoding="utf-8"))
        for p in [entry["path"] for entry in m["pinned_files"]] + list(ADD_FILES):
            src = ROOT / p
            dest = self.root / p
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dest)

    def tearDown(self):
        self.tmp.cleanup()

    def mutate(self, rel, fn):
        p = self.root / rel
        obj = json.loads(p.read_text(encoding="utf-8"))
        fn(obj)
        p.write_text(json.dumps(obj, indent=2)+"\n", encoding="utf-8")

    def blocked(self):
        with self.assertRaises((lab.Blocked, lab.guardian.Blocked)):
            lab.audit(self.root)

    def test_01_complete_parent_manifest_audit_is_research_only(self):
        result = lab.audit(self.root)
        self.assertEqual(result["BUREAU001_R1"],
                         "PASS_WITH_CONTROLS_SOURCE_AUDIT_ONLY")
        self.assertEqual(result["FROZEN_SOURCE_BLOBS"], "99/99_PASS")
        self.assertEqual(result["NATIVE_ADMISSION_GATES"], "12_UNMET")

    def test_02_24_rust_and_33_python_source_test_pairs_pinned(self):
        x = lab.audit(self.root)
        self.assertEqual(x["R4A_R4X_NATIVE_RUST_TEST_FILES"], 24)
        self.assertEqual(x["R4Y_R6E_PYTHON_SOURCE_TEST_PAIRS"], 33)

    def test_03_four_original_source_snapshots_reconciled_additively(self):
        x = lab.audit(self.root)
        self.assertEqual(x["R6B_R6E_HISTORICAL_STATUS_RECONCILED"],
                         "4/4_APPEND_ONLY")
        self.assertEqual(x["INITIAL_FINDINGS"], "6_DOCUMENTED")

    def test_04_no_nine_independent_agent_reviews_or_guardian_root(self):
        x = lab.audit(self.root)
        self.assertEqual(x["BUREAU_INDEPENDENT_REVIEWS_COMPLETED"], 0)
        self.assertEqual(x["ACTUAL_EXTERNAL_CUSTODIAN"], "NOT_ENROLLED")
        self.assertEqual(x["R6G"], "LOCKED_BY_SEQUENCE")

    def test_05_historical_gitblob_digest_matches_actual_source_bytes(self):
        m = json.loads((self.root / lab.MANIFEST).read_text())
        source = next(e for e in m["pinned_files"] if
                      e["path"] == "research/g03_r6e_oob_origin_intake.py")
        self.assertEqual(lab.gitblob((self.root / source["path"]).read_bytes()),
                         source["git_blob_sha1"])

    def test_06_owner_chat_log_not_marked_independent_attestation(self):
        d = json.loads((self.root / lab.CHRONOLOGY).read_text())
        self.assertIn("NOT_AN_INDEPENDENT_MACHINE_SIGNATURE",
                      d["owner_evidence_limitation"])
        self.assertEqual(d["record_kind"],
                         "APPEND_ONLY_RECONCILIATION_NOT_MUTATION_OF_ORIGINAL_SNAPSHOT")

    def test_07_modified_r6e_python_source_blob_quarantined(self):
        with (self.root / "research/g03_r6e_oob_origin_intake.py").open("ab") as f:
            f.write(b"\n# tampered\n")
        self.blocked()

    def test_08_modified_r4m_signed_native_test_blob_quarantined(self):
        with (self.root / "tests/g03_r4m_pinned_code_identity_lab.rs").open("ab") as f:
            f.write(b"\n// changed\n")
        self.blocked()

    def test_09_missing_original_native_test_blocks_entire_audit(self):
        (self.root / "tests/g03_r4m_pinned_code_identity_lab.rs").unlink()
        self.blocked()

    @unittest.skipIf(os.name == "nt", "Windows symlink creation may require privilege")
    def test_10_symlink_substitution_rejected_without_following_target(self):
        p = self.root / "research/g03_r6a_oob_witness_challenge.py"
        p.unlink()
        p.symlink_to("g03_r6e_oob_origin_intake.py")
        self.blocked()

    def test_11_extra_unpinned_manifest_record_fails_closed(self):
        self.mutate(lab.MANIFEST, lambda o: o["pinned_files"].append(
            copy.deepcopy(o["pinned_files"][0])))
        self.blocked()

    def test_12_deleted_frozen_manifest_path_fails_closed(self):
        self.mutate(lab.MANIFEST, lambda o: o["pinned_files"].pop())
        self.blocked()

    def test_13_duplicate_path_and_invalid_sort_fails_closed(self):
        def edit(o):
            o["pinned_files"][1] = copy.deepcopy(o["pinned_files"][0])
        self.mutate(lab.MANIFEST, edit)
        self.blocked()

    def test_14_false_git_blob_sha1_detected(self):
        self.mutate(lab.MANIFEST, lambda o: o["pinned_files"][10].__setitem__(
            "git_blob_sha1", "0"*40))
        self.blocked()

    def test_15_path_traversal_manifest_rejected(self):
        self.mutate(lab.MANIFEST, lambda o: o["pinned_files"][0].__setitem__(
            "path", "../private-keys"))
        self.blocked()

    def test_16_faked_owner_1039_replay_count_not_accepted(self):
        self.mutate(lab.CHRONOLOGY, lambda o: o["stages"][3].__setitem__(
            "owner_regular_tests", 99999))
        self.blocked()

    def test_17_forged_github_workflow_run_or_status_detected(self):
        self.mutate(lab.CHRONOLOGY, lambda o: o["stages"][2].__setitem__(
            "dedicated_run_id", 99999999999))
        self.blocked()

    def test_18_frozen_original_pending_status_must_remain_visible(self):
        self.mutate(lab.CHRONOLOGY, lambda o: o["stages"][1].__setitem__(
            "source_owner_status_expected", "PASS"))
        self.blocked()

    def test_19_false_external_custodian_certification_is_blocked(self):
        self.mutate(lab.CHRONOLOGY, lambda o: o["stages"][0].__setitem__(
            "independent_custodian", "CERTIFIED_EXTERNAL"))
        self.blocked()

    def test_20_r6g_cannot_unlock_via_later_chronology(self):
        self.mutate(lab.CHRONOLOGY, lambda o: o["r6f"].__setitem__(
            "r6g", "IN_PROGRESS"))
        self.blocked()

    def test_21_nine_independent_review_claim_cannot_be_forged(self):
        self.mutate(lab.CHRONOLOGY, lambda o: o["r6f"].__setitem__(
            "formal_bureau_independent_reviews", 9))
        self.blocked()

    def test_22_cannot_substitute_different_source_parent_commit(self):
        self.mutate(lab.MANIFEST, lambda o: o.__setitem__(
            "parent_head", "f"*40))
        self.blocked()

    def test_23_original_bureau_fake_review_certification_still_blocked(self):
        self.mutate("governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json",
                    lambda o: o["review_functions"][0].update(
                        review_status="PASS", formal_verdict="CERTIFIED"))
        self.blocked()

    def test_24_attempt_unlock_r6g_returns_block_without_file_writes(self):
        before = sorted((str(p.relative_to(self.root)), p.read_bytes())
                        for p in self.root.rglob("*") if p.is_file())
        p = subprocess.run([
            sys.executable, "-B", str(ROOT / "scripts/bureau_001_r1_audit.py"),
            "--root", str(self.root), "--attempt-unlock-r6g",
        ], capture_output=True, text=True, timeout=45, check=False)
        self.assertEqual(p.returncode, 2, p.stderr)
        x = json.loads(p.stdout)
        self.assertEqual(x["TRANSITION"], "BLOCK")
        self.assertEqual(x["NATIVE_ADMISSION_GATES"], "12_UNMET")
        self.assertEqual(x["PRODUCTION"], "NOT_AUTHORIZED")
        after = sorted((str(p.relative_to(self.root)), p.read_bytes())
                       for p in self.root.rglob("*") if p.is_file())
        self.assertEqual(before, after)

if __name__ == "__main__":
    unittest.main()
