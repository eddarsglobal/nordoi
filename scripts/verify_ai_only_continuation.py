#!/usr/bin/env python3
"""Fail-closed AI-only development directive verifier (additive governance).

This verifies documented, machine-checkable policy compatibility only.
It cannot prove that autonomous agents have actually run, that external
administrative independence exists, or that releases are authorized.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parent.parent
REL_POLICY="governance/NORDOI_AI_AGENTS_ONLY_CONTINUATION_V1.json"
REL_E2="governance/NORDOI_R6F_BUREAU001_E2_NO_HUMAN_TESTING_POLICY.json"
REL_E12="governance/NORDOI_R6F_BUREAU001_E12_TWELVE_GATE_INTAKE.json"

class PolicyBlocked(ValueError):
    pass

def require(condition: bool, reason: str):
    if not condition:
        raise PolicyBlocked(reason)

def validate(policy:dict, e2:dict, e12:dict) -> str:
    require(policy.get("schema")=="NORDOI-AI-AGENTS-FIRST-AUTONOMOUS-CONTINUATION-V1","BAD_SCHEMA")
    require(policy.get("project")=="eddarsglobal/nordoi","WRONG_PROJECT")
    require(policy.get("basis",{}).get("predecessor_exact_head")=="2d2049a03139ebfb0005d403792e915a35625d99","BAD_C20_PARENT")
    require(e2.get("human_testers_allowed") is False and
            e2.get("human_reviewer_outreach_allowed") is False,"HISTORICAL_E2_POLICY_CHANGED")
    require(e12.get("human_testers_allowed") is False,"HISTORICAL_E12_HUMAN_TEST_CHANGED")
    execution=policy.get("standing_execution_policy",{})
    for key in ("human_testers_allowed","human_manual_tests_required",
                "human_specialist_recruitment_allowed","human_reviewer_outreach_allowed",
                "human_signature_solicitation_allowed",
                "human_intervention_as_development_blocker_allowed",
                "founder_manual_execution_as_quality_gate_allowed"):
        require(execution.get(key) is False,"HUMAN_DEPENDENCY_REINTRODUCED_"+key.upper())
    require(execution.get("agent_count_must_be_observed_not_inferred") is True,
            "UNSUPPORTED_INDEPENDENT_AGENT_ASSERTION")
    require(execution.get("no_repeated_research_only_blocker_loops") is True,
            "REPEATED_BLOCKED_LOOP")
    require(execution.get("normal_compiled_runtime_requires_AI_model") is False,
            "C8_AI_DEPENDENT_RUNTIME")
    required_actions={"IMPLEMENT_CODE","BUILD_COMPILE","REAL_RUNTIME_TEST",
       "CROSS_OS_CI","FUZZ_AND_NEGATIVE_TEST","PATCH_AND_RETEST",
       "ADVANCE_AFTER_C20_PASS","VERIFY_PROVENANCE_AND_SHA"}
    require(required_actions.issubset(set(execution.get("allowed_machine_actions",[]))),
            "AUTOMATED_ACTIONS_MISSING")
    tracks=policy.get("qualification_tracks",{})
    dev=tracks.get("agent_development",{})
    require(dev.get("human_dependency")=="NONE" and
            dev.get("source_and_ci")=="REQUIRED_C20","DEVELOPMENT_WRONGLY_GATED")
    att=tracks.get("native_independent_attestation",{})
    require(att.get("human_testers")=="NEVER_REQUIRED" and
            att.get("status")=="REMAIN_BLOCKED_UNTIL_REAL_EVIDENCE","FAKE_INDEPENDENT_CERTIFICATION")
    activation=tracks.get("privileged_activation_or_production",{})
    require(activation.get("status")=="DENY_UNTIL_RELEVANT_GATES_AND_AUTHORIZATION_VERIFIED" and
            activation.get("fabricated_authority")=="FORBIDDEN",
            "UNSAFE_PRIVILEGED_ACTIVATION")
    require(policy.get("decision_rule")=="NEVER_TREAT_A_MISSING_HUMAN_AS_A_RESEARCH_MILESTONE_STALL; NEVER_TREAT_MACHINE_OUTPUT_AS_INDEPENDENT_REAL_WORLD_AUTHORITY",
            "WRONG_SEPARATION_RULE")
    frozen=policy.get("immutable_predecessor_status",{})
    require(frozen.get("r6f")=="IN_PROGRESS_UNCERTIFIED" and
            frozen.get("r6g")=="LOCKED_BY_SEQUENCE" and
            frozen.get("native_admission_external_gates_unmet")==12 and
            frozen.get("production")=="NOT_AUTHORIZED" and
            frozen.get("merge_of_protected_chain")=="NOT_AUTOMATICALLY_AUTHORIZED",
            "HISTORICAL_GUARDIAN_PROMOTION")
    gates=e12.get("required_native_gates",[])
    require(len(gates)==12 and
            len({x.get("id") for x in gates})==12 and
            all(x.get("status")=="UNMET_INDEPENDENT_EXTERNAL_EVIDENCE" for x in gates),
            "E12_REAL_INDEPENDENCE_GATES_NOT_PRESERVED")
    non_goals=set(policy.get("governance_non_goals",[]))
    for item in ("NO_FALSE_FORMAL_CERTIFICATION",
                 "NO_BYPASS_OF_REAL_NATIVE_SECURITY_GATES",
                 "NO_UNAUTHORIZED_PRODUCTION_RELEASE_OR_MERGE",
                 "NO_REWRITE_OF_PINNED_ROADMAP_BUREAU_OR_CONSTITUTION"):
        require(item in non_goals,"NON_GOAL_MISSING_"+item)
    return "PASS_AI_ONLY_CONTINUATION_AND_SEPARATE_NATIVE_CERTIFICATION"

def load(path:Path)->dict:
    with path.open("r",encoding="utf-8") as stream:
        obj=json.load(stream)
    if not isinstance(obj,dict):
        raise PolicyBlocked("NOT_JSON_OBJECT")
    return obj

def main(argv=None)->int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audit",action="store_true")
    opts=parser.parse_args(argv)
    root=ROOT
    try:
        result=validate(load(root/REL_POLICY),load(root/REL_E2),load(root/REL_E12))
    except (OSError,ValueError,TypeError,KeyError) as error:
        print("AI_ONLY_GUARDIAN=BLOCK",file=sys.stderr)
        print("REASON="+str(error),file=sys.stderr)
        return 2
    print("AI_ONLY_GUARDIAN="+result)
    print("HUMAN_DEVELOPMENT_DEPENDENCY=NONE")
    print("NATIVE_INDEPENDENT_CERTIFICATION=SEPARATE_REAL_EVIDENCE_ONLY")
    print("UNSAFE_PRODUCTION_OR_MERGE=NOT_AUTHORIZED")
    print("POLICY_AUDIT=READ_ONLY")
    return 0

if __name__=="__main__":
    raise SystemExit(main())
