#!/usr/bin/env python3
"""NORDOI RoadMapGuardian v1 — offline, read-only, research-only fail-closed gate.

This checks *declared* Bureau and sequence governance, not real operator identity,
agent execution, hardware custody or authority. No file writes, network, model
inference or privileges. CERTIFIED_RESEARCH / native admission cannot be
established by this tool from locally editable evidence.
"""
import argparse
import json
from pathlib import Path
import sys

PARENT_R6E = "12e050d9b8698f4f902baf44390f1108a8e237ee"
ROLE_IDS = (
    "ATLAS", "FERRUM", "BOUNDARY", "CIPHER", "LEDGER",
    "CAPSTONE", "ADVERSARY", "VERITY", "GUARDIAN",
)
GATE_IDS = (
    "owner_oob_authenticated_root",
    "actual_external_custodian_identity",
    "distinct_administrative_control_domains",
    "guardian_key_pinned_out_of_band",
    "independently_signed_os_attestation",
    "signed_fresh_challenge_and_exact_history",
    "independent_os_domain_and_host_witness",
    "externally_monotonic_key_epoch",
    "protected_global_revocation_floor",
    "protected_global_spent_nonce_floor",
    "independent_quorum",
    "independent_recovery_rotation_audit",
)
GATE_UNMET = "UNMET_INDEPENDENT_EXTERNAL_EVIDENCE"
PLAN = "governance/NORDOI_BUREAU_PLAN_v1.json"
HANDOFF = "governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json"
SOURCE = "research/G03_R6E_OOB_ORIGIN_NO_EXTERNAL_AUTHORITY.json"
BASE_DOCS = ("RoadMap.md", "Bureau.md", "CONSTITUTION.md",
             "governance/future_native_gate_v1.tsv")
REQUIRED_CONSTITUTION = ("C3", "C8", "C14", "C20")
REQUIRED_FNG = tuple("FNG"+str(i) for i in range(1, 7))

class Blocked(ValueError):
    pass

def require(condition, reason):
    if not condition:
        raise Blocked(reason)

def read(root, rel):
    target = root / rel
    require(not target.is_symlink(), "SYMLINK_FORBIDDEN:"+rel)
    require(target.is_file(), "MISSING_REQUIRED_FILE:"+rel)
    return target.read_text(encoding="utf-8")

def read_json(root, rel):
    try:
        return json.loads(read(root, rel))
    except (UnicodeError, ValueError) as e:
        raise Blocked("INVALID_JSON:"+rel) from e

def distinct_ids(items, expected, kind):
    require(type(items) is list, "NOT_A_LIST:"+kind)
    ids = [item.get("id") if type(item) is dict else None for item in items]
    require(tuple(ids) == tuple(expected), "MISSING_DUPLICATE_REORDERED_"+kind)
    return items

def audit(root):
    """Validate the canonical DRAFT handoff, never attest to executed agent reviews."""
    root = Path(root).resolve()
    p = read_json(root, PLAN)
    h = read_json(root, HANDOFF)
    src = read_json(root, SOURCE)
    roadmap = read(root, "RoadMap.md")
    bureau = read(root, "Bureau.md")
    constitution = read(root, "CONSTITUTION.md")
    fng = read(root, "governance/future_native_gate_v1.tsv")

    require(p.get("schema") == "NORDOI-BUREAU-ROADMAP-V1", "BAD_PLAN_PROTOCOL")
    require(h.get("schema") == "NORDOI-BUREAU-HANDOFF-V1", "BAD_HANDOFF_PROTOCOL")
    require(p.get("project") == h.get("project") == "eddarsglobal/nordoi", "WRONG_PROJECT")
    require(p.get("scope") == "RESEARCH_GOVERNANCE_ONLY", "UNAUTHORIZED_SCOPE")
    require(p.get("parent_r6e_head") == h.get("source_parent_r6e_head")
            == PARENT_R6E, "SOURCE_COMMIT_NOT_PINNED")

    require(tuple(p.get("constitutional_rules", ())) == REQUIRED_CONSTITUTION,
            "CONSTITUTIONAL_RULES_MISSING")
    require(tuple(p.get("future_native_gate", ())) == REQUIRED_FNG,
            "FUTURE_NATIVE_GATES_MISSING")
    for x in REQUIRED_CONSTITUTION:
        require(("## "+x+" ") in constitution, "CONSTITUTION_MISSING:"+x)
    for x in REQUIRED_FNG:
        require((x+"\t") in fng, "FNG_POLICY_MISSING:"+x)

    roles = distinct_ids(p.get("bureau"), ROLE_IDS, "BUREAU_ROLES")
    specialties = [r.get("specialty") for r in roles]
    require(all(type(x) is str and x for x in specialties) and
            len(set(specialties)) == len(roles), "BUREAU_SPECIALTIES_NOT_DISTINCT")
    for role in roles:
        require(role.get("role_type") == "AI_REVIEW_FUNCTION_NOT_AN_INDEPENDENT_SIGNER"
                and bool(role.get("mandate")), "BUREAU_ROLE_ESCALATED_OR_EMPTY")
    ap = p.get("approval_policy", {})
    require(tuple(ap.get("required_review_functions", ())) == ROLE_IDS
            and ap.get("unsubmitted_review") == "NOT_SUBMITTED"
            and ap.get("no_ai_self_certification") is True
            and ap.get("owner_authorization_required_for_privileged_changes") is True
            and ap.get("independent_external_signatures_required_for_native_admission") is True
            and ap.get("release_requires_independent_evidence_and_governance_decision") is True,
            "UNSAFE_APPROVAL_POLICY")

    m = p.get("milestones")
    require(type(m) is list and len(m) == 3, "NONSEQUENTIAL_MILESTONES")
    require([x.get("id") for x in m] == ["G03_R6E","G03_R6F","G03_R6G"],
            "MILESTONE_ORDER_CHANGED")
    require([x.get("status") for x in m] == [
        "TECHNICAL_VALIDATED_RESEARCH_ONLY","IN_PROGRESS","LOCKED_BY_SEQUENCE"
    ], "MILESTONE_UNJUSTIFIED_PROMOTION")
    require(m[0].get("certification") == "NOT_CERTIFIED_FOR_NATIVE"
            and m[0].get("owner_terminal_observation") ==
            "1039_REGULAR_PASS_188_SOURCE_BLOBS_PASS_SCRIPT_EXIT_0_USER_LOG"
            and m[0].get("observed_github_ci") == "9_OF_9_SUCCESS",
            "R6E_TECHNICAL_SCOPE_MISREPRESENTED")
    require(m[1].get("mission") == h.get("mission_id") == "BUREAU_001"
            and m[1].get("parent") == "G03_R6E"
            and m[1].get("certification") == "PENDING",
            "R6F_MISSION_OR_CERTIFICATION_CHANGED")
    require(m[2].get("parent") == "G03_R6F"
            and m[2].get("external_evidence") == "NOT_ENROLLED"
            and m[2].get("certification") == "PENDING",
            "R6G_EARLY_UNLOCK")

    admission = p.get("native_admission", {})
    current_gates = distinct_ids(admission.get("gates"), GATE_IDS,
                                 "NATIVE_ADMISSION_GATES")
    original_gates = distinct_ids(src.get("required_gates"), GATE_IDS,
                                  "R6E_SOURCE_ADMISSION_GATES")
    require(all(g.get("status") == GATE_UNMET for g in current_gates+original_gates),
            "ADMISSION_GATE_SILENTLY_PROMOTED")
    require(admission.get("remaining") == src.get("native_admission_gates_unmet") == 12
            and admission.get("guardian") == "NOT_ENROLLED"
            and admission.get("owner_oob_root") == "NOT_AUTHENTICATED"
            and admission.get("independent_monotonic_antirollback") == "NOT_VERIFIED"
            and admission.get("independent_quorum") == "NOT_PROVEN"
            and admission.get("production") == src.get("production") ==
            "NOT_AUTHORIZED", "PRODUCTION_OR_CUSTODY_QUIETLY_AUTHORIZED")
    require("NO_AUTOMATIC_AGENT_EXECUTION" in p.get("non_goals", [])
            and "NO_PR_MERGE" in p.get("non_goals", [])
            and "NO_PRODUCTION_OR_NATIVE_RUNTIME_CHANGES" in p.get("non_goals", []),
            "NO_MUTATION_CONTRACT_MISSING")

    require(h.get("assigned_milestone") == "G03_R6F"
            and h.get("mission_state") == "ASSIGNED_REVIEWS_NOT_SUBMITTED"
            and h.get("execution_state") ==
            "NO_AI_AGENTS_OR_EXTERNAL_OPERATORS_LAUNCHED_BY_THIS_DOCUMENT"
            and h.get("formal_bureau_certification") == "PENDING"
            and h.get("governance_verdict") == "PENDING"
            and h.get("next_milestone") == "G03_R6G"
            and h.get("next_milestone_state") == "LOCKED_BY_SEQUENCE",
            "FAKE_HANDOFF_EXECUTION_OR_CERTIFICATION")
    reviews = h.get("review_functions")
    require(type(reviews) is list and
            [x.get("role") for x in reviews] == list(ROLE_IDS),
            "BUREAU_HANDOFF_INCOMPLETE")
    for review in reviews:
        require(review.get("review_status") == "NOT_SUBMITTED"
                and review.get("evidence_reference") is None
                and review.get("actual_execution_log") is None
                and review.get("formal_verdict") is None
                and review.get("findings") == []
                and bool(review.get("task_id")) and bool(review.get("objective"))
                and type(review.get("source_loci")) is list
                and bool(review["source_loci"]),
                "UNSUBMITTED_REVIEW_FAKED_AS_COMPLETE:"+str(review.get("role")))
    ctrl = h.get("controls", {})
    require(ctrl.get("all_native_gates_unmet") == 12
            and ctrl.get("owner_oob_root") == "NOT_AUTHENTICATED"
            and ctrl.get("independent_guardian") == "NOT_ENROLLED"
            and ctrl.get("global_monotonic_antirollback") == "NOT_VERIFIED"
            and ctrl.get("independent_quorum") == "NOT_PROVEN"
            and ctrl.get("github_merge") == "NONE"
            and ctrl.get("production") == "NOT_AUTHORIZED",
            "HANDOFF_OVERRIDE_OF_SECURITY_DENIAL")

    for role in ROLE_IDS:
        require(role in roadmap and role in bureau,
                "BUREAU_ROLE_NOT_DOCUMENTED:"+role)
    for name in ("G0.3 R6E","G0.3 R6F","G0.3 R6G"):
        require(name in roadmap, "ROADMAP_JALON_MISSING:"+name)
    require("RoadMapGuardian" in roadmap and "BUREAU-001" in roadmap
            and "NOT_SUBMITTED" in roadmap and "NATIVE_ADMISSION_GATES=12_UNMET"
            in roadmap, "ROADMAP_DID_NOT_DEFINE_DENIAL_AND_HANDOFF")
    require("NOT_EXECUTED" in bureau and "NOT_AUTHORIZED" in bureau
            and "BUREAU-001" in bureau, "BUREAU_FALSE_CERTIFICATION_RISK")
    return {
        "BUREAU_GUARDIAN": "PASS_WITH_CONTROLS",
        "SOURCE_R6E": PARENT_R6E,
        "REVIEW_FUNCTIONS": 9,
        "REVIEWS_SUBMITTED": 0,
        "R6E": "TECHNICAL_VALIDATED_RESEARCH_ONLY",
        "R6F": "IN_PROGRESS_REVIEWS_NOT_SUBMITTED",
        "R6G": "LOCKED_BY_SEQUENCE",
        "NATIVE_ADMISSION_GATES": "12_UNMET",
        "OWNER_OOB_ROOT": "NOT_AUTHENTICATED",
        "INDEPENDENT_CUSTODIAN": "NOT_ENROLLED",
        "BUREAU_CERTIFICATION": "PENDING",
        "PRODUCTION": "NOT_AUTHORIZED",
        "GITHUB_MUTATION": "NONE",
    }

def deny_transition(target):
    """No locally editable review is a verified external authority or self-cert."""
    return {"TRANSITION": "BLOCK", "TARGET": target,
            "REASON": "EXTERNAL_AND_INDEPENDENT_CERTIFICATION_NOT_AVAILABLE",
            "PRODUCTION": "NOT_AUTHORIZED",
            "NATIVE_ADMISSION_GATES": "12_UNMET"}

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path,
                        default=Path(__file__).resolve().parents[1])
    actions = parser.add_mutually_exclusive_group(required=True)
    actions.add_argument("--audit", action="store_true")
    actions.add_argument("--attempt-promote", choices=(
        "CERTIFIED_RESEARCH", "NATIVE_ADMISSION", "PRODUCTION"))
    args = parser.parse_args(argv)
    try:
        outcome = audit(args.root)
    except (Blocked, OSError, TypeError, KeyError, AttributeError) as exc:
        print(json.dumps({"BUREAU_GUARDIAN": "BLOCK",
                          "reason": str(exc) if isinstance(exc, Blocked)
                          else type(exc).__name__}, sort_keys=True))
        return 2
    if args.attempt_promote:
        print(json.dumps(deny_transition(args.attempt_promote), sort_keys=True))
        return 2
    print(json.dumps(outcome, sort_keys=True))
    return 0

if __name__ == "__main__":
    sys.exit(main())
