"""NORDOI G0.3 R6F: 24 deterministic Bureau/RoadMapGuardian negative controls.

Runs with no network, inference or mutation of repository files. All altered
governance inputs are disposable copies; no independent external authority is
claimed by a test or a passing static check.
"""
import copy
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from scripts import roadmap_guardian as guard

FILES = (guard.PLAN, guard.HANDOFF, guard.SOURCE, "RoadMap.md", "Bureau.md",
         "CONSTITUTION.md", "governance/future_native_gate_v1.tsv", guard.TRACK)

class BureauGuardianTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nordoi-bureau-guardian-")
        self.root = Path(self.temp.name)
        for rel in FILES:
            target = self.root / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / rel, target)

    def tearDown(self):
        self.temp.cleanup()

    def change(self, rel, mutate):
        path = self.root / rel
        obj = json.loads(path.read_text(encoding="utf-8"))
        mutate(obj)
        path.write_text(json.dumps(obj, indent=2) + "\n", encoding="utf-8")

    def blocked(self):
        with self.assertRaises(guard.Blocked):
            guard.audit(self.root)

    def test_01_valid_initial_bureau_read_only_pass_with_controls(self):
        result = guard.audit(self.root)
        self.assertEqual(result["BUREAU_GUARDIAN"], "PASS_WITH_CONTROLS")
        self.assertEqual(result["REVIEW_FUNCTIONS"], 9)
        self.assertEqual(result["REVIEWS_SUBMITTED"], 0)
        self.assertEqual(result["BUREAU_CERTIFICATION"], "PENDING")

    def test_02_12_gate_ids_in_exact_source_order(self):
        p = json.loads((self.root / guard.PLAN).read_text(encoding="utf-8"))
        self.assertEqual(tuple(x["id"] for x in p["native_admission"]["gates"]),
                         guard.GATE_IDS)

    def test_03_sequential_scope_research_only_no_production(self):
        r = guard.audit(self.root)
        self.assertEqual(r["R6E"], "TECHNICAL_VALIDATED_RESEARCH_ONLY")
        self.assertEqual(r["R6G"], "LOCKED_BY_SEQUENCE")
        self.assertEqual(r["PRODUCTION"], "NOT_AUTHORIZED")

    def test_04_handoff_is_not_independent_approved_agents(self):
        h = json.loads((self.root / guard.HANDOFF).read_text(encoding="utf-8"))
        self.assertEqual(h["mission_state"], "ASSIGNED_REVIEWS_NOT_SUBMITTED")
        self.assertTrue(all(x["evidence_reference"] is None and
                            x["review_status"] == "NOT_SUBMITTED"
                            for x in h["review_functions"]))

    def test_05_missing_role_in_plan_blocks(self):
        self.change(guard.PLAN, lambda p: p["bureau"].pop())
        self.blocked()

    def test_06_duplicate_or_reordered_role_blocks(self):
        self.change(guard.PLAN, lambda p: p["bureau"].__setitem__(
            1, copy.deepcopy(p["bureau"][0])))
        self.blocked()

    def test_07_fake_independent_signer_label_blocks(self):
        self.change(guard.PLAN, lambda p: p["bureau"][0].__setitem__(
            "role_type", "SEPARATE_EXTERNAL_CUSTODIAN"))
        self.blocked()

    def test_08_removed_approval_role_blocks(self):
        self.change(guard.PLAN, lambda p: p["approval_policy"][
            "required_review_functions"].pop())
        self.blocked()

    def test_09_explicit_owner_privilege_approval_cannot_be_removed(self):
        self.change(guard.PLAN, lambda p: p["approval_policy"].__setitem__(
            "owner_authorization_required_for_privileged_changes", False))
        self.blocked()

    def test_10_ai_self_certification_allowed_is_blocked(self):
        self.change(guard.PLAN, lambda p: p["approval_policy"].__setitem__(
            "no_ai_self_certification", False))
        self.blocked()

    def test_11_parent_sha_cannot_change(self):
        self.change(guard.PLAN, lambda p: p.__setitem__(
            "parent_r6e_head", "0"*40))
        self.blocked()

    def test_12_fake_ci_count_blocked(self):
        self.change(guard.PLAN, lambda p: p["milestones"][0].__setitem__(
            "observed_github_ci", "99_OF_99_SUCCESS"))
        self.blocked()

    def test_13_claim_r6f_certified_without_evidence_is_blocked(self):
        self.change(guard.PLAN, lambda p: p["milestones"][1].__setitem__(
            "status", "CERTIFIED_RESEARCH"))
        self.blocked()

    def test_14_early_r6g_unlocked_is_blocked(self):
        self.change(guard.PLAN, lambda p: p["milestones"][2].__setitem__(
            "status", "IN_PROGRESS"))
        self.blocked()

    def test_15_unsafe_native_gate_promotion_blocked(self):
        self.change(guard.PLAN, lambda p: p["native_admission"]["gates"][
            0].__setitem__("status", "PASS"))
        self.blocked()

    def test_16_missing_native_gate_blocked(self):
        self.change(guard.PLAN, lambda p: p["native_admission"]["gates"].pop())
        self.blocked()

    def test_17_source_gate_rollback_is_detected(self):
        self.change(guard.SOURCE, lambda p: p["required_gates"][
            0].__setitem__("status", "SATISFIED"))
        self.blocked()

    def test_18_silent_production_flip_is_blocked(self):
        self.change(guard.PLAN, lambda p: p["native_admission"].__setitem__(
            "production", "AUTHORIZED"))
        self.blocked()

    def test_19_false_external_guardian_enrollment_blocked(self):
        self.change(guard.HANDOFF, lambda h: h["controls"].__setitem__(
            "independent_guardian", "ENROLLED"))
        self.blocked()

    def test_20_deleted_review_assignee_blocked(self):
        self.change(guard.HANDOFF, lambda h: h["review_functions"].pop(3))
        self.blocked()

    def test_21_fake_review_pass_without_log_blocked(self):
        self.change(guard.HANDOFF, lambda h: h["review_functions"][0].update(
            review_status="PASS", formal_verdict="CERTIFIED"))
        self.blocked()

    def test_22_fake_log_or_provenance_submission_blocked(self):
        self.change(guard.HANDOFF, lambda h: h["review_functions"][5].update(
            evidence_reference="self-signed:test", actual_execution_log="fake.log"))
        self.blocked()

    def test_23_removed_roadmap_document_and_missing_bureau_role_block(self):
        (self.root / "RoadMap.md").unlink()
        self.blocked()

    def test_24_all_three_unauthorized_transitions_deny_without_writes(self):
        before = sorted((str(p.relative_to(self.root)), p.read_bytes())
                        for p in self.root.rglob("*") if p.is_file())
        for target in ("CERTIFIED_RESEARCH", "NATIVE_ADMISSION", "PRODUCTION"):
            p = subprocess.run([
                sys.executable, "-B", str(ROOT / "scripts/roadmap_guardian.py"),
                "--root", str(self.root), "--attempt-promote", target
            ], text=True, capture_output=True, check=False)
            self.assertEqual(p.returncode, 2, p.stderr)
            verdict = json.loads(p.stdout)
            self.assertEqual(verdict["TRANSITION"], "BLOCK")
            self.assertEqual(verdict["PRODUCTION"], "NOT_AUTHORIZED")
        after = sorted((str(p.relative_to(self.root)), p.read_bytes())
                       for p in self.root.rglob("*") if p.is_file())
        self.assertEqual(before, after, "GUARDIAN_MUST_NOT_WRITE_FILES")

if __name__ == "__main__":
    unittest.main()
