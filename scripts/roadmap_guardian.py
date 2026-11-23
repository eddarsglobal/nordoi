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
TRACK = "governance/NORDOI_APP_RESEARCH_TRACK_V1.json"
APP_CHAIN = (
    ("APP-016", "e988b214c8daa2b37c6abc36e351ce988bea7880", "f8455e602076ffabc49abf4f123cb8d1e70bc639", 168),
    ("APP-017", "c048f1e17590b8d863d38a067fe6ac00d78e61c2", "e988b214c8daa2b37c6abc36e351ce988bea7880", 169),
    ("APP-018", "0aa1fe01fe9391ed67a0fa20f8af7280af56a1ad", "c048f1e17590b8d863d38a067fe6ac00d78e61c2", 170),
    ("APP-019", "fa89752c1a62f3577134017d6a90b62d46a25cf7", "0aa1fe01fe9391ed67a0fa20f8af7280af56a1ad", 171),
    ("APP-020", "6f0f5625e282b4cd5a9ad5d527087502f5842f71", "fa89752c1a62f3577134017d6a90b62d46a25cf7", 172),
    ("APP-021", "6e9c300922348d0ebab4318c7f9c2b4dd4cd76b4", "6f0f5625e282b4cd5a9ad5d527087502f5842f71", 173),
)

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

def audit_app_track(root, plan, roadmap, bureau):
    """Reject any unproved APP chain promotion or retroactive invented Bureau."""
    track = read_json(root, TRACK)
    p = plan.get("research_application_track")
    require(type(p) is dict and p.get("schema") ==
            "NORDOI_APP_RESEARCH_TRACK_POINTER_V1" and
            p.get("register") == TRACK,
            "APP_RESEARCH_OFFICIAL_PLAN_POINTER_MISSING")
    ids = [entry[0] for entry in APP_CHAIN]
    require(p.get("ids_in_strict_c20_order") == ids and
            p.get("reference_predecessor_head") == APP_CHAIN[0][2] and
            p.get("qualified_recent_head") == APP_CHAIN[-1][1] and
            p.get("observed_history") == "SIX_DRAFT_PRS_WITH_13_OF_13_EXACT_HEAD_CI" and
            p.get("app_track_technical_verdict") == "TECHNICAL_VALIDATED_RESEARCH_ONLY" and
            p.get("app_track_formal_bureau_certification") == "NOT_CERTIFIED" and
            p.get("historical_bureau_reviews") == "NOT_SUBMITTED" and
            p.get("app_next_candidate") == "APP-022" and
            p.get("app_next_status") == "LOCKED_BY_GOVERNANCE_REVIEW" and
            p.get("governance_correction") == "GOV-FIX1_IN_PROGRESS" and
            p.get("governance_correction_pr") == "DRAFT_PENDING" and
            p.get("native_admission") == "DENIED" and
            p.get("production") == "NOT_AUTHORIZED",
            "APP_RESEARCH_PLAN_PROMOTION_OR_REORDER")
    require(type(track) is dict and
            track.get("schema") == "NORDOI_APP_RESEARCH_GOVERNANCE_RECONCILIATION_V1" and
            track.get("project") == "eddarsglobal/nordoi" and
            track.get("source_of_truth") == PLAN and
            track.get("scope") == "NONPRIVILEGED_AGENT_RESEARCH_ONLY" and
            track.get("audit_date") == "2026-11-23" and
            track.get("chronology_rule") == "C20_EXACT_COMMIT_PARENT_AND_ALL_CHECKS_SUCCESS_AT_HEAD",
            "APP_RESEARCH_REGISTER_SCHEMA_INVALID")
    base = track.get("reference_baseline", {})
    require(base.get("milestone") == "CERT-015-R1" and
            base.get("head") == APP_CHAIN[0][2] and
            base.get("state") == "RESEARCH_ONLY",
            "APP_RESEARCH_BASELINE_INCORRECT")
    require(track.get("bureau_roles") == list(ROLE_IDS),
            "APP_RESEARCH_BUREAU_FUNCTIONS_REORDERED")
    semantics = track.get("bureau_semantics", {})
    require(semantics.get("role_report_requirement") ==
            "EXACT_SOURCE_PIN_EXECUTION_LOG_ADVERSARIAL_EVIDENCE_FORMAL_VERDICT" and
            semantics.get("machine_function_checks") ==
            "SEPARATE_DETERMINISTIC_AUTOMATED_CHECKS_NOT_INDEPENDENT_AI_AGENTS" and
            semantics.get("independent_agent_reports") == "NOT_SUBMITTED" and
            semantics.get("formal_certification") == "NOT_CERTIFIED" and
            semantics.get("previous_pr_comments") ==
            "SINGLE_COORDINATOR_TECHNICAL_SUMMARIES_NOT_FORMAL_BUREAU_SIGNOFF",
            "APP_RESEARCH_FAKE_BUREAU_CERTIFICATION")
    require(track.get("observed_checks_source") ==
            "GITHUB_CHECK_RUNS_EXACT_HEAD_VERIFIED_ON_2026_11_23_NOT_AN_INDEPENDENT_CUSTODIAN",
            "APP_RESEARCH_CI_ATTESTATION_OVERCLAIMED")
    records = track.get("milestones")
    require(type(records) is list and len(records) == len(APP_CHAIN),
            "APP_RESEARCH_MILESTONE_COUNT_INVALID")
    for i, ((id_, head, parent, pr), item) in enumerate(zip(APP_CHAIN, records)):
        require(type(item) is dict and
                (item.get("id"), item.get("head_exact_sha"),
                 item.get("parent_exact_sha"), item.get("pr_number")) ==
                (id_, head, parent, pr),
                "APP_RESEARCH_C20_EXACT_CHAIN_INVALID:" + id_)
        require(type(item.get("objective")) is str
                and len(item["objective"]) > 15,
                "APP_RESEARCH_OBJECTIVE_UNDEFINED:" + id_)
        require(item.get("current_technical_status") ==
                "TECHNICAL_VALIDATED_RESEARCH_ONLY" and
                item.get("pr_state") == "DRAFT_OPEN_UNMERGED" and
                type(item.get("exact_head_github_success")) is int and
                item.get("exact_head_github_success") == 13 and
                type(item.get("exact_head_github_total")) is int and
                item.get("exact_head_github_total") == 13 and
                item.get("independent_bureau_review") == "NOT_SUBMITTED" and
                item.get("formal_bureau_certification") == "NOT_CERTIFIED" and
                item.get("external_security_certification") == "NOT_PROVEN" and
                item.get("production") == "NOT_AUTHORIZED",
                "APP_RESEARCH_UNSUPPORTED_PROMOTION:" + id_)
        require(id_ in roadmap and ("#"+str(pr)) in roadmap and
                head in roadmap, "APP_RESEARCH_ROADMAP_NOT_MATCHED:" + id_)
        if i > 0:
            require(parent == APP_CHAIN[i-1][1],
                    "APP_RESEARCH_PARENT_LINK_BROKEN:" + id_)
    nxt = track.get("next_candidate", {})
    require(nxt.get("id") == "APP-022" and
            nxt.get("state") == "LOCKED_BY_GOVERNANCE_REVIEW" and
            nxt.get("no_external_custody_claim") is True,
            "APP022_UNLOCKED_WITHOUT_GOVERNANCE")
    protected = track.get("protected_baseline", {})
    require(protected.get("r6f") == "IN_PROGRESS_UNCERTIFIED" and
            protected.get("r6g") == "LOCKED_BY_SEQUENCE" and
            protected.get("native_gates_remaining") == 12 and
            protected.get("global_monotonic_antirollback") == "NOT_VERIFIED" and
            protected.get("release") == "NOT_AUTHORIZED" and
            protected.get("merge") == "NONE",
            "APP_RESEARCH_PROTECTED_GATES_CHANGED")
    require("## 7. Branche applicative IA" in roadmap and
            "FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED" in bureau and
            "APP-022" in roadmap and "LOCKED_BY_GOVERNANCE_REVIEW" in roadmap,
            "APP_RESEARCH_GOVERNANCE_DOCS_MISSING")
    return {"APP_TRACK": "TECHNICAL_VALIDATED_RESEARCH_ONLY",
            "APP_MILESTONES": len(records),
            "APP_C20_PARENT_CHAIN": "EXACT_PINNED_METADATA",
            "APP_EXACT_HEAD_CI_OBSERVATIONS": 13*len(records),
            "APP_BUREAU_FORMAL_CERTIFICATION": "NOT_CERTIFIED",
            "APP_NEXT": "LOCKED_BY_GOVERNANCE_REVIEW",
            "APP_MACHINE_REVIEW_IS_INDEPENDENT_AGENT": False}


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

    app_research = audit_app_track(root, p, roadmap, bureau)

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
        **app_research,
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
