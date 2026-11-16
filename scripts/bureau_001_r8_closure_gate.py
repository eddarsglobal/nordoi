#!/usr/bin/env python3
"""NORDOI G0.3 R6F BUREAU-001 R8 — closure eligibility adversarial audit.

This is an additive, source-bound coordinator test of requirements to CLOSE
the R6F Bureau mission. It does NOT actually perform a nine-agent independent
review, enroll any external custodian, authenticate OOB key fingerprints or
grant privileges. All outcomes DENY, even when research was technically green.

Real --exercise reuses existing R7 native cryptographic and R5Q rollback
negative controls, with actual installed OpenSSH on POSIX, not simulations.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

import bureau_001_r7_contradiction_guard as r7
import bureau_001_r2_review_intake as r2
import roadmap_guardian as guardian

PROJECT = "eddarsglobal/nordoi"
PARENT = "ee4975e42d4364dd3d2bc78ae2d7a08623042298"
PROTOCOL = "NORDOI-G03-R6F-BUREAU001-R8-CLOSURE-READONLY-DENY-V1"
DOSSIER = "governance/NORDOI_R6F_BUREAU001_R8_CLOSURE_DECISION.json"
CLAIM_PROTOCOL = "NORDOI-G03-R6F-R8-UNVERIFIED-CLOSURE-ASSERTION-V1"
CLAIM_FIELDS = frozenset((
    "protocol","project","parent","requested_operation","claimed_ci_green",
    "claimed_owner_replay_green","claimed_independent_reviewer_count",
    "claimed_external_custodian","claimed_oob_root",
    "claimed_global_monotonic_antirollback","claimed_native_gate_status",
    "claimed_formal_certification"
))
EXPECTED_COUNTS = (1087,1111,1135,1159,1183,1207,1231)
STAGES = ("R1","R2","R3","R4","R5","R6","R7")
EXPECTED_CLOSURE_IDS = (
    "R8-SOURCE-001","R8-CI-002","R8-REVIEW-003","R8-OOB-004",
    "R8-OPERATOR-005","R8-LEDGER-006","R8-DECISION-007"
)
EXPECTED_CI_RUNS = (
    (None,None),(None,None),(43694308826,43694308782),
    (43741889168,43741889124),(43814788301,43814788261),
    (43825369772,43825369666),(43844811936,43844811856)
)
EXPECTED_EXTERNAL_CATEGORY = "EXTERNAL_REQUIRED"
MAX_CLAIM_BYTES = 4096

class Blocked(ValueError):
    pass

def require(condition,reason):
    if not condition:
        raise Blocked(reason)

def deny(reason,**more):
    return dict(
        status="BLOCK",admission="NOT_AUTHORIZED",
        native_authority_granted=False,production="NOT_AUTHORIZED",
        r6f="IN_PROGRESS_UNCERTIFIED",r6g="LOCKED_BY_SEQUENCE",
        native_admission_gates_unmet=12,independent_specialist_reviews=0,
        formal_bureau_certification="PENDING",
        independent_custodian="NOT_ENROLLED",
        owner_oob_root="NOT_AUTHENTICATED",
        global_monotonic_antirollback="NOT_VERIFIED",
        reason=reason,**more
    )

def load(root,rel):
    return r7.load(root,rel)

def canonical(obj):
    return r7.canonical(obj)

def strict_claim(raw):
    obj=r7.strict(raw,max_bytes=MAX_CLAIM_BYTES,canonical_only=True)
    require(obj.keys()==CLAIM_FIELDS,"R8_UNEXPECTED_CLOSURE_CLAIM_FIELDS")
    require((obj["protocol"],obj["project"],obj["parent"],
             obj["requested_operation"])==(
             CLAIM_PROTOCOL,PROJECT,PARENT,
             "STAGE_UNAUTHENTICATED_CLOSURE_CLAIM_ONLY"),
            "R8_PRIVILEGED_OR_UNPINNED_CLOSURE_OPERATION")
    for name in (
        "claimed_external_custodian","claimed_oob_root",
        "claimed_global_monotonic_antirollback","claimed_formal_certification"
    ):
        v=obj[name]
        require(v is None or (
            type(v) is str and 1<=len(v)<=128 and "\x00" not in v
        ),"R8_UNTRUSTED_CLOSURE_FIELD_"+name)
    for name in ("claimed_ci_green","claimed_owner_replay_green"):
        v=obj[name]
        require(type(v) is bool,"R8_INVALID_SELF_REPORTED_TEST_RESULT_"+name)
    count=obj["claimed_independent_reviewer_count"]
    require(type(count) is int and 0<=count<=9,
            "R8_INVALID_SELF_REPORTED_REVIEW_COUNT")
    statuses=obj["claimed_native_gate_status"]
    require(type(statuses) is list and len(statuses)==12 and
            all(type(x) is str and len(x)<=90 for x in statuses),
            "R8_INVALID_SELF_REPORTED_NATIVE_GATES")
    return obj

def candidate_template():
    return dict(
        protocol=CLAIM_PROTOCOL,project=PROJECT,parent=PARENT,
        requested_operation="STAGE_UNAUTHENTICATED_CLOSURE_CLAIM_ONLY",
        claimed_ci_green=True,claimed_owner_replay_green=True,
        claimed_independent_reviewer_count=0,
        claimed_external_custodian=None,claimed_oob_root=None,
        claimed_global_monotonic_antirollback=None,
        claimed_formal_certification=None,
        claimed_native_gate_status=[guardian.GATE_UNMET]*12
    )

def assess_candidate(raw):
    """Untrusted local claims, even 'all gates met', cannot close mission."""
    try:
        claim=strict_claim(raw)
        asserted_external=(
            claim["claimed_independent_reviewer_count"]>0 or
            any(claim[k] is not None for k in (
                "claimed_external_custodian","claimed_oob_root",
                "claimed_global_monotonic_antirollback",
                "claimed_formal_certification"
            )) or
            any(x != guardian.GATE_UNMET for x in
                claim["claimed_native_gate_status"])
        )
        return deny(
            "R8_SELF_ASSERTED_INDEPENDENT_CLOSURE_UNAUTHENTICATED"
            if asserted_external else "R8_LOCAL_RESEARCH_TESTS_CANNOT_CLOSE_R6F",
            local_test_claims_are_not_external_approvals=True,
            independent_operator_proven=False,
            independently_signed_evidence_verified=False,
            independently_authenticated_custodian_enrolled=False,
            native_gates_promoted=0,
            formal_closure_granted=False
        )
    except (Blocked,r7.Blocked,TypeError,ValueError,KeyError) as e:
        return deny(
            "R8_MALFORMED_OR_PRIVILEGED_CLOSURE_CLAIM_QUARANTINED",
            detail=str(e) if isinstance(e,(Blocked,r7.Blocked))
                   else type(e).__name__
        )

def check_sources(r7plan,registry,original,r6,dossier):
    require(dossier["schema"]==
            "NORDOI-G03-R6F-BUREAU001-R8-CLOSURE-DECISION-RESEARCH-V1" and
            dossier["parent_r7_exact_head"]==PARENT and
            dossier["project"]==PROJECT and
            dossier["research_only"] is not True if "research_only" in dossier
            else dossier["audit_scope"]==
                 "SOURCE_BASED_COORDINATOR_CHECK_OF_CLOSURE_PRECONDITIONS_NOT_INDEPENDENT_EXECUTED_REVIEWS",
            "R8_INVALID_CLOSURE_DOSSIER_IDENTITY")
    # Avoid implicit approvals: exact R7 upstream no external real-world proofs.
    require(r7plan["parent_r6_exact_head"]==r7.PARENT and
            r7plan["independently_executed_specialist_reviews"]==0 and
            r7plan["r2_specialist_submissions_count"]==0 and
            r7plan["r6g"]=="LOCKED_BY_SEQUENCE" and
            r7plan["production"]=="NOT_AUTHORIZED" and
            r7plan["native_admission_gates_unmet"]==12,
            "R7_UPSTREAM_NOT_UNCERTIFIED")
    require(tuple(z["id"] for z in original["required_gates"])==guardian.GATE_IDS and
            tuple(z["id"] for z in r6["required_native_gates"])==guardian.GATE_IDS,
            "R8_REAL_SOURCE_TWELVE_GATES_NOT_EXACT")
    require(len(registry["roles"])==9 and
            tuple(r["role"] for r in registry["roles"])==guardian.ROLE_IDS and
            registry["submissions"]==[] and
            registry["independent_specialist_agents_executed"]==0,
            "R8_FAKE_INDEPENDENT_SPECIALIST_CLOSURE")
    timeline=dossier["evidence_timeline"]
    require(type(timeline) is list and len(timeline)==7 and
            tuple(t["stage"] for t in timeline)==STAGES and
            tuple(t["owner_reported_regular_tests"] for t in timeline)==EXPECTED_COUNTS and
            tuple((t["dedicated_github_run"],t["general_github_run"])
                  for t in timeline)==EXPECTED_CI_RUNS and
            all(t["owner_evidence_class"]==
                "OWNER_SUPPLIED_TERMINAL_NOT_INDEPENDENT_HARDWARE_ATTESTATION"
                for t in timeline) and
            tuple(t["head"] for t in timeline[:6])==
            tuple(t["head"] for t in r7plan["historical_reconciliation"]) and
            timeline[-1]["head"]==PARENT,
            "R8_HISTORICAL_TECHNICAL_EVIDENCE_FALSE_OR_WRONG_SOURCE")
    criteria=dossier["closure_criteria"]
    require(type(criteria) is list and len(criteria)==7 and
            tuple(row["id"] for row in criteria)==EXPECTED_CLOSURE_IDS,
            "R8_CLOSURE_CRITERIA_MISSING_DUPLICATED_OR_REORDERED")
    require(criteria[0]["status"]=="OWNER_REPORTED_PASS_WITH_CONTROLS" and
            criteria[0]["category"]=="LOCAL_TECHNICAL_SUPPORTED_OWNER_LOG" and
            criteria[1]["status"]=="PRIOR_CONNECTOR_VERIFIED_9_OF_9_SUCCESS" and
            criteria[1]["category"]=="TECHNICAL_CI_REVIEW" and
            all(row["status"]=="UNMET" and
                row["category"]==EXPECTED_EXTERNAL_CATEGORY
                for row in criteria[2:]),
            "R8_LOCAL_TESTS_FABRICATED_EXTERNAL_CLOSURE_PROOF")
    gates=dossier["twelve_native_gates"]
    require(type(gates) is list and len(gates)==12 and
            tuple(g["id"] for g in gates)==guardian.GATE_IDS and
            all(g["status"]==guardian.GATE_UNMET and
                g["candidate_external_evidence"] is None and
                g["independent_signer"] is None and
                g["authenticated_custodian"] is None and
                g["admitted"] is False for g in gates),
            "R8_CUSTODIAN_OR_NATIVE_GATE_FABRICATED")
    roles=dossier["nine_bureau_roles"]
    require(type(roles) is list and len(roles)==9 and
            tuple(r["role"] for r in roles)==guardian.ROLE_IDS and
            all(r["review_submitted"] is False and
                r["independent_signature"] is None and
                r["independent_execution_log"] is None and
                r["formal_verdict"] is None for r in roles),
            "R8_FORGED_SPECIALIST_ATTESTATION")
    require(tuple(dossier["contradictions_carried_from_r7"])==r7.CONTRADICTIONS,
            "R8_OLD_CONTRADICTIONS_NOT_PRESERVED")
    require(dossier["source_original_R1_blobs"]==99 and
            dossier["source_historical_blobs"]==188 and
            dossier["technical_ci_r7"]=="9/9_SUCCESS_AT_EXACT_PARENT_HEAD" and
            dossier["independently_executed_specialist_reviews"]==0 and
            dossier["independent_custodian"]=="NOT_ENROLLED" and
            dossier["owner_oob_root"]=="NOT_AUTHENTICATED" and
            dossier["global_monotonic_antirollback"]=="NOT_VERIFIED" and
            dossier["independent_quorum"]=="NOT_PROVEN" and
            dossier["formal_bureau_certification"]=="PENDING" and
            dossier["native_admission_gates_unmet"]==12 and
            dossier["research_dossier_closed"] is False and
            dossier["formal_r6f_closure"]==
                "BLOCKED_MISSING_INDEPENDENT_EVIDENCE" and
            dossier["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
            dossier["r6g"]=="LOCKED_BY_SEQUENCE" and
            dossier["production"]=="NOT_AUTHORIZED" and
            dossier["github_merge"]=="NONE" and
            dossier["owner_authorized_privileged_transition"] is False and
            dossier["automatic_grant_on_local_technical_success"] is False and
            dossier["external_custodian_contacted"] is False and
            dossier["external_custody_tests_performed"] is False,
            "R8_PREMATURE_CLOSURE_CERTIFICATION_OR_NATIVE_AUTHORITY")
    return {"technical_supported":2,"external_unmet":5,
            "native_unmet":12,"independent_roles_unsubmitted":9}

def audit(root):
    root=Path(root).resolve()
    before=r7.audit(root)
    require(before["reason"]==
            "R7_SOURCE_CONTRADICTIONS_RECONCILED_NO_EXTERNAL_CUSTODY"
            and before["verified_r1_frozen_git_objects"]==99
            and before["cross_audit_findings_reconciled"]==7,
            "R7_99_SOURCE_BLOBS_OR_SEVEN_FINDINGS_NOT_REAL")
    registry=load(root,r7.R2)
    original=load(root,r7.FROZEN)
    r6=load(root,r7.R6)
    plan=load(root,r7.PLAN)
    dossier=load(root,DOSSIER)
    counts=check_sources(plan,registry,original,r6,dossier)
    return deny(
        "R8_RESEARCH_DOSSIER_RECONCILED_FORMAL_R6F_CLOSURE_BLOCKED",
        research_technical_evidence_classified=counts["technical_supported"],
        external_closure_criteria_unmet=counts["external_unmet"],
        independent_native_gates_unmet=counts["native_unmet"],
        actual_independent_specialists_missing=counts["independent_roles_unsubmitted"],
        seven_prior_contradictions_reconciled=True,
        earlier_owner_R7_reported_regular_tests=1231,
        r7_github_jobs_prior_verified="9/9_SUCCESS",
        locally_supported_is_not_externally_certified=True,
        bureau_research_dossier_formal_closure_granted=False
    )

def exercise(root):
    # Reuse the real R7 POSIX OpenSSH/R6 two-key/R5Q three-file negative tests.
    x=audit(root)
    require(x["reason"]=="R8_RESEARCH_DOSSIER_RECONCILED_FORMAL_R6F_CLOSURE_BLOCKED",
            "R8_RESEARCH_SOURCE_DOSSIER_DID_NOT_RECONCILE")
    if not (sys.platform.startswith(("linux","darwin")) and
            Path("/usr/bin/ssh-keygen").is_file()):
        return deny("R8_REAL_POSIX_NATIVE_EVIDENCE_REQUIRED"),2
    p=subprocess.run([sys.executable,"-B",
        str(Path(root)/"scripts/bureau_001_r7_contradiction_guard.py"),
        "--exercise"],cwd=root,text=True,capture_output=True,timeout=360)
    require(p.returncode==2,"R8_REAL_R7_SIGNED_NEGATIVE_NOT_DENIAL_EXIT_2")
    try: evidence=json.loads(p.stdout)
    except (ValueError,UnicodeError) as e:
        raise Blocked("R8_REAL_R7_SIGNED_EVIDENCE_JSON_MISSING") from e
    require(evidence["reason"]==
            "R7_REAL_R6_LOCAL_SIGNATURES_R5Q_ROLLBACK_AND_FALSE_CUSTODY_REFUSED" and
            evidence["authentic_local_ed25519_signatures_verified"]==2 and
            evidence["real_original_r5q_three_file_cross_epoch_rollback"] is True and
            evidence["independent_external_gates_promoted"]==0,
            "R8_REAL_PRIOR_SIGNED_LOCAL_EVIDENCE_WAS_PROMOTED")
    fake=candidate_template()
    fake.update(
        claimed_independent_reviewer_count=9,
        claimed_external_custodian="fake-local-operator",
        claimed_oob_root="same-admin-forged-oob-check",
        claimed_global_monotonic_antirollback="false-coherent-history",
        claimed_formal_certification="signed-by-owner-local-process",
        claimed_native_gate_status=["VERIFIED"]*12
    )
    rejected=assess_candidate(canonical(fake))
    require(rejected["reason"]==
            "R8_SELF_ASSERTED_INDEPENDENT_CLOSURE_UNAUTHENTICATED" and
            rejected["native_gates_promoted"]==0 and
            rejected["formal_closure_granted"] is False,
            "R8_SELF_CERTIFIED_FORMAL_CLOSURE_NOT_DENIED")
    return deny(
        "R8_REAL_SIGNED_LOCAL_PROOFS_CANNOT_CLOSE_BUREAU_WITHOUT_EXTERNAL_AUTHORITY",
        r7_true_two_local_ed25519_signatures=2,
        r5q_real_three_file_coherent_rollback=True,
        forged_nine_specialists_self_reported_but_refused=True,
        forged_external_oob_and_monotonic_custody_refused=True,
        formal_R6F_closure_granted=False,
        external_admission_gates_promoted=0
    ),2

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root",type=Path,
                        default=Path(__file__).resolve().parents[1])
    group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--audit",action="store_true")
    group.add_argument("--exercise",action="store_true")
    args=parser.parse_args(argv)
    try:
        result,code=(audit(args.root),2) if args.audit else exercise(args.root)
    except (Blocked,r7.Blocked,r7.r1.Blocked,r2.Blocked,guardian.Blocked,
            OSError,ValueError,TypeError,KeyError,subprocess.SubprocessError) as e:
        result,code=deny("R8_CLOSURE_SOURCE_OR_NATIVE_NEGATIVE_QUARANTINED",
            detail=str(e) if isinstance(e,(
                Blocked,r7.Blocked,r7.r1.Blocked,r2.Blocked,guardian.Blocked
            )) else type(e).__name__),3
    print(json.dumps(result,sort_keys=True))
    return code

if __name__=="__main__":
    sys.exit(main())
