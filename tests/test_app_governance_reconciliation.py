"""GOV-FIX1 — twenty-four fail-closed APP RoadMap + Bureau controls.

Disposable local copies; cannot prove GitHub CI, real agents, external keys
or independent custody. All reported C20 check evidence is separately verified
from GitHub at exact heads and labeled as historical observation, not an
inferred certificate.
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
from scripts import app_research_guardian as guard

FILES = (guard.PLAN, guard.HANDOFF, guard.SOURCE, guard.TRACK,
         "scripts/roadmap_guardian.py",
         "RoadMap.md", "Bureau.md", "CONSTITUTION.md",
         "governance/future_native_gate_v1.tsv", guard.EXTENSION,
         guard.ROADMAP_DOC, guard.BUREAU_DOC)


class AppGovernanceReconciliation(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nordoi-gov-fix1-")
        self.root = Path(self.temp.name)
        for file in FILES:
            to = self.root / file
            to.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / file, to)

    def tearDown(self):
        self.temp.cleanup()

    def edit(self, path, function):
        p = self.root / path
        data = json.loads(p.read_text("utf-8"))
        function(data)
        p.write_text(json.dumps(data, indent=2)+"\n", "utf-8")

    def refused(self):
        with self.assertRaises(guard.Blocked):
            guard.audit(self.root)

    def test_01_six_c20_parent_pins_pass(self):
        self.assertEqual(guard.audit(self.root)["APP_MILESTONES"], 6)

    def test_02_78_ci_observations_only_not_certification(self):
        data = guard.audit(self.root)
        self.assertEqual(data["APP_EXACT_HEAD_CI_OBSERVATIONS"], 78)
        self.assertEqual(data["APP_BUREAU_FORMAL_CERTIFICATION"], "NOT_CERTIFIED")
        self.assertFalse(data["APP_MACHINE_REVIEW_IS_INDEPENDENT_AGENT"])

    def test_03_app_next_locked_pending_governance(self):
        self.assertEqual(guard.audit(self.root)["APP_NEXT"],
                         "LOCKED_BY_GOVERNANCE_REVIEW")

    def test_04_legacy_r6f_r6g_still_locked(self):
        res = guard.audit(self.root)
        self.assertEqual(res["R6F"], "IN_PROGRESS_REVIEWS_NOT_SUBMITTED")
        self.assertEqual(res["R6G"], "LOCKED_BY_SEQUENCE")
        self.assertEqual(res["NATIVE_ADMISSION_GATES"], "12_UNMET")

    def test_05_delete_app018_from_manifest_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"].pop(2))
        self.refused()

    def test_06_swap_adjacent_app_items_is_rejected(self):
        def swap(d):
            d["milestones"][1],d["milestones"][2]=d["milestones"][2],d["milestones"][1]
        self.edit(guard.TRACK, swap)
        self.refused()

    def test_07_forged_app020_head_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][4].update(
            head_exact_sha="a"*40))
        self.refused()

    def test_08_forged_app017_parent_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][1].update(
            parent_exact_sha="0"*40))
        self.refused()

    def test_09_wrong_pr_number_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][5].update(pr_number=170))
        self.refused()

    def test_10_inflate_13_success_observation_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][2].update(
            exact_head_github_success=14))
        self.refused()

    def test_11_boolean_success_count_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][2].update(
            exact_head_github_success=True))
        self.refused()

    def test_12_fake_independent_agent_review_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][2].update(
            independent_bureau_review="PASS"))
        self.refused()

    def test_13_fake_bureau_certification_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][3].update(
            formal_bureau_certification="CERTIFIED_RESEARCH"))
        self.refused()

    def test_14_fake_production_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][0].update(
            production="AUTHORIZED"))
        self.refused()

    def test_15_promote_next_app_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["next_candidate"].update(
            state="IN_PROGRESS"))
        self.refused()

    def test_16_change_r6f_certification_is_rejected(self):
        self.edit(guard.PLAN, lambda d:d["milestones"][1].update(
            certification="CERTIFIED_RESEARCH"))
        self.refused()

    def test_17_silent_native_gate_removal_is_rejected(self):
        self.edit(guard.PLAN, lambda d:d["native_admission"]["gates"].pop())
        self.refused()

    def test_18_forged_independent_quorum_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["protected_baseline"].update(
            global_monotonic_antirollback="VERIFIED"))
        self.refused()

    def test_19_plan_without_app_pointer_is_rejected(self):
        self.edit(guard.EXTENSION, lambda d:d.pop("register"))
        self.refused()

    def test_20_plan_app_order_mismatch_is_rejected(self):
        self.edit(guard.EXTENSION, lambda d:d.update(
            ids_in_strict_c20_order=["APP-021","APP-020","APP-019","APP-018","APP-017","APP-016"]))
        self.refused()

    def test_21_milestone_title_missing_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["milestones"][0].update(objective=""))
        self.refused()

    def test_22_bureau_independence_fabricated_is_rejected(self):
        self.edit(guard.TRACK, lambda d:d["bureau_semantics"].update(
            machine_function_checks="NINE_INDEPENDENT_AGENTS_CERTIFIED"))
        self.refused()

    def test_23_missing_roadmap_app_section_is_rejected(self):
        p = self.root / guard.ROADMAP_DOC
        s = p.read_text("utf-8")
        p.write_text(s.split("## 7. Branche applicative IA")[0], "utf-8")
        self.refused()

    def test_24_guardian_does_not_mutate_any_input_file(self):
        before = {p:str((self.root/p).read_bytes()) for p in FILES}
        self.assertEqual(guard.audit(self.root)["APP_MILESTONES"], 6)
        self.assertEqual(before,{p:str((self.root/p).read_bytes()) for p in FILES})


if __name__ == "__main__":
    unittest.main()
