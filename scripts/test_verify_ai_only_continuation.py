"""Adversarial machine-only tests for NORDOI AI-only owner directive."""
import copy
import importlib.util
from pathlib import Path
import unittest

HERE=Path(__file__).resolve().parent
file=HERE/"verify_ai_only_continuation.py"
spec=importlib.util.spec_from_file_location("verify_ai_only_continuation",file)
v=importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)

class AIAgentDevelopmentPolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.policy=v.load(v.ROOT/v.REL_POLICY)
        cls.e2=v.load(v.ROOT/v.REL_E2)
        cls.e12=v.load(v.ROOT/v.REL_E12)

    def test_01_real_policy_passes(self):
        self.assertEqual(v.validate(self.policy,self.e2,self.e12),
            "PASS_AI_ONLY_CONTINUATION_AND_SEPARATE_NATIVE_CERTIFICATION")

    def test_02_human_tester_reintroduction_is_denied(self):
        p=copy.deepcopy(self.policy)
        p["standing_execution_policy"]["human_testers_allowed"]=True
        with self.assertRaisesRegex(v.PolicyBlocked,"HUMAN_DEPENDENCY"):
            v.validate(p,self.e2,self.e12)

    def test_03_founder_manual_step_cannot_block_development(self):
        p=copy.deepcopy(self.policy)
        p["standing_execution_policy"]["founder_manual_execution_as_quality_gate_allowed"]=True
        with self.assertRaisesRegex(v.PolicyBlocked,"HUMAN_DEPENDENCY"):
            v.validate(p,self.e2,self.e12)

    def test_04_human_signature_solicitation_denied(self):
        p=copy.deepcopy(self.policy)
        p["standing_execution_policy"]["human_signature_solicitation_allowed"]=True
        with self.assertRaisesRegex(v.PolicyBlocked,"HUMAN_DEPENDENCY"):
            v.validate(p,self.e2,self.e12)

    def test_05_forbidden_c20_skip_denied(self):
        p=copy.deepcopy(self.policy)
        p["qualification_tracks"]["agent_development"]["source_and_ci"]="OPTIONAL"
        with self.assertRaisesRegex(v.PolicyBlocked,"DEVELOPMENT_WRONGLY_GATED"):
            v.validate(p,self.e2,self.e12)

    def test_06_historical_e2_no_human_gate_preserved(self):
        e=copy.deepcopy(self.e2)
        e["human_testers_allowed"]=True
        with self.assertRaisesRegex(v.PolicyBlocked,"HISTORICAL_E2"):
            v.validate(self.policy,e,self.e12)

    def test_07_historical_e12_no_human_gate_preserved(self):
        e=copy.deepcopy(self.e12)
        e["human_testers_allowed"]=True
        with self.assertRaisesRegex(v.PolicyBlocked,"HISTORICAL_E12"):
            v.validate(self.policy,self.e2,e)

    def test_08_external_native_gate_cannot_be_self_certified(self):
        e=copy.deepcopy(self.e12)
        e["required_native_gates"][0]["status"]="MET_BY_AI_SELF_DECLARATION"
        with self.assertRaisesRegex(v.PolicyBlocked,"E12_REAL"):
            v.validate(self.policy,self.e2,e)

    def test_09_auto_promotion_to_native_production_is_denied(self):
        p=copy.deepcopy(self.policy)
        p["qualification_tracks"]["privileged_activation_or_production"]["status"]="AUTHORIZED"
        with self.assertRaisesRegex(v.PolicyBlocked,"UNSAFE_PRIVILEGED_ACTIVATION"):
            v.validate(p,self.e2,self.e12)

    def test_10_implicit_agent_independence_is_denied(self):
        p=copy.deepcopy(self.policy)
        p["standing_execution_policy"]["agent_count_must_be_observed_not_inferred"]=False
        with self.assertRaisesRegex(v.PolicyBlocked,"UNSUPPORTED"):
            v.validate(p,self.e2,self.e12)

    def test_11_repetitive_blocked_research_loop_denied(self):
        p=copy.deepcopy(self.policy)
        p["standing_execution_policy"]["no_repeated_research_only_blocker_loops"]=False
        with self.assertRaisesRegex(v.PolicyBlocked,"REPEATED_BLOCKED_LOOP"):
            v.validate(p,self.e2,self.e12)

    def test_12_no_development_without_auto_code_build_and_ci(self):
        p=copy.deepcopy(self.policy)
        p["standing_execution_policy"]["allowed_machine_actions"].remove("REAL_RUNTIME_TEST")
        with self.assertRaisesRegex(v.PolicyBlocked,"AUTOMATED_ACTIONS"):
            v.validate(p,self.e2,self.e12)

    def test_13_unapproved_auto_merge_must_remain_blocked(self):
        p=copy.deepcopy(self.policy)
        p["immutable_predecessor_status"]["merge_of_protected_chain"]="AUTHORIZED"
        with self.assertRaisesRegex(v.PolicyBlocked,"HISTORICAL_GUARDIAN_PROMOTION"):
            v.validate(p,self.e2,self.e12)

    def test_14_ai_required_for_compiled_execution_denied_by_c8(self):
        p=copy.deepcopy(self.policy)
        p["standing_execution_policy"]["normal_compiled_runtime_requires_AI_model"]=True
        with self.assertRaisesRegex(v.PolicyBlocked,"C8_AI_DEPENDENT"):
            v.validate(p,self.e2,self.e12)

if __name__=="__main__":
    unittest.main()
