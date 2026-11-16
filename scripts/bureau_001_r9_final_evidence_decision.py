#!/usr/bin/env python3
"""NORDOI G0.3 R6F BUREAU-001 R9 — final evidence gap no-go check.

Read-only, deny-only coordinator audit. Not an independently executed Bureau
review, a third-party custody certificate, a human signature or an enrollment.
A real POSIX --exercise reuses original R8 OpenSSH Ed25519 / R5Q history
rollback negatives. No fake human, network connection or privileged operation.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import bureau_001_r8_closure_gate as r8
import bureau_001_r7_contradiction_guard as r7
import roadmap_guardian as guardian

PROJECT="eddarsglobal/nordoi"
PARENT="cad5140206725ed45e84a445176360be13ae8a36"
SCHEMA="NORDOI-G03-R6F-BUREAU001-R9-FINAL-EVIDENCE-DECISION-GAP-V1"
PROTOCOL="NORDOI-G03-R6F-BUREAU001-R9-FINAL-MISSING-EVIDENCE-DENY-V1"
DOSSIER="governance/NORDOI_R6F_BUREAU001_R9_FINAL_EVIDENCE_DECISION.json"
CLOSURE_IDS=r8.EXPECTED_CLOSURE_IDS
GAP_IDS=("IND-SPECIALISTS","IND-OPERATOR","IND-OOB","IND-LEDGER","IND-FORMAL")
GAP_STATES=("NOT_RECEIVED","NOT_ENROLLED","NOT_AUTHENTICATED","NOT_VERIFIED","NOT_DECIDED")
DEPENDENCIES=((),(),("IND-OPERATOR",),
              ("IND-OPERATOR","IND-OOB"),
              ("IND-SPECIALISTS","IND-OPERATOR","IND-OOB","IND-LEDGER"))
ATTACK_IDS=tuple("A"+str(i) for i in range(1,9))
MAX_CLAIM=4096
CLAIM_PROTOCOL="NORDOI-G03-R6F-R9-SELF_ASSERTED_EXTERNAL_CLEARANCE-V1"
CLAIM_FIELDS=frozenset((
    "protocol","parent","requested_operation","claims_nine_independent_reviews",
    "claims_owner_oob_confirmed","claims_distinct_external_operator",
    "claims_global_monotonic_history","claims_formal_owner_authorization",
    "claims_all_twelve_native_gates"
))
class Blocked(ValueError):pass

def require(x,reason):
    if not x:raise Blocked(reason)

def deny(reason,**more):
    return dict(status="BLOCK",admission="NOT_AUTHORIZED",
                native_authority_granted=False,r6f="IN_PROGRESS_UNCERTIFIED",
                r6g="LOCKED_BY_SEQUENCE",production="NOT_AUTHORIZED",
                native_admission_gates_unmet=12,
                independent_specialist_reviews=0,
                real_independent_custodian="NOT_ENROLLED",
                owner_oob_root="NOT_AUTHENTICATED",
                global_monotonic_antirollback="NOT_VERIFIED",
                bureau_formal_certification="PENDING",
                reason=reason,**more)

def load(root,rel):return r7.load(root,rel)
def canonical(obj):return r7.canonical(obj)

def validate_sources(r7plan,r8plan,original,register,d):
    """Strict claims can never be upgraded to authorization."""
    require(d["schema"]==SCHEMA and d["project"]==PROJECT and
            d["parent_r8_exact_head"]==PARENT and
            d["mandate"]=="FINAL_ADVERSARIAL_REVIEW_OF_MISSING_REAL_EVIDENCE_AND_INDEPENDENT_DECISION_PRECONDITIONS",
            "R9_WRONG_SOURCE_IDENTITY_OR_PARENT")
    require(tuple(d["source_of_truth"])==(
             "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py",
             "governance/NORDOI_R6F_BUREAU001_R2_REVIEW_REGISTER.json",
             "research/G03_R6E_OOB_ORIGIN_NO_EXTERNAL_AUTHORITY.json",
             r7.PLAN,r8.DOSSIER
             ),"R9_SOURCE_BASIS_UNPINNED_OR_MISSING")
    require(r8plan["parent_r7_exact_head"]==r8.PARENT and
            r8plan["formal_r6f_closure"]=="BLOCKED_MISSING_INDEPENDENT_EVIDENCE" and
            r8plan["research_dossier_closed"] is False and
            r8plan["independently_executed_specialist_reviews"]==0 and
            r8plan["r6g"]=="LOCKED_BY_SEQUENCE" and
            r7plan["parent_r6_exact_head"]==r7.PARENT,
            "R8_RESEARCH_STATUS_FROZEN_SOURCE_MISMATCH")
    require(tuple(g["id"] for g in original["required_gates"])==guardian.GATE_IDS
            and tuple(g["id"] for g in r8plan["twelve_native_gates"])==guardian.GATE_IDS
            and all(g["admitted"] is False for g in r8plan["twelve_native_gates"]),
            "R9_HISTORICAL_TWELVE_GATES_TAMPERED")
    require(register["submissions"]==[] and
            register["independent_specialist_agents_executed"]==0 and
            tuple(r["role"] for r in register["roles"])==guardian.ROLE_IDS,
            "R9_NINE_ROLES_FAKE_INDEPENDENT_EXECUTION")
    obs=d["post_r8_source_observation"]
    require(obs["source_head"]==PARENT and
            obs["owner_mac_intel_report"]==
               "OWNER_SUPPLIED_TERMINAL_PASS_WITH_CONTROLS_RESEARCH_ONLY"
            and obs["owner_mac_os"]=="14.5" and
            obs["architecture"]=="x86_64" and
            obs["owner_python_regular_tests"]==1002 and
            obs["owner_rust_regular_tests"]==253 and
            obs["owner_rust_ignored"]==3 and
            obs["owner_total_regular_tests"]==1255 and
            obs["owner_script_exit"]==0 and
            obs["r8_persisted_source_snapshot_still"]==
                "PENDING_AT_COMMIT_TIME_DO_NOT_REWRITE" and
            obs["owner_transcript_is_independent_attestation"] is False and
            obs["github_ci_exact_head"]=="9/9_COMPLETED_SUCCESS" and
            obs["github_dedicated_run"]==43874942792 and
            obs["general_rust_run"]==43874942657 and
            obs["github_results_are_independent_custodian_attestation"] is False and
            r8plan["owner_mac_intel_r8"]=="PENDING",
            "R9_SOURCE_TIME_SNAPSHOT_VS_LATER_OWNER_EVIDENCE_CONFUSED")
    require(tuple(d["prior_r7_seven_findings"])==r7.CONTRADICTIONS,
            "R9_PRIOR_SEVEN_FINDINGS_LOST")
    criteria=d["closure_criteria"]
    require(type(criteria) is list and len(criteria)==7 and
            tuple(c["id"] for c in criteria)==CLOSURE_IDS and
            [c["status"] for c in criteria[:2]]==[
               "TECHNICAL_SUPPORTED_WITH_OWNER_LOG_LIMITS",
               "TECHNICAL_SUPPORTED_GITHUB_ACTIONS"] and
            all(c["status"]=="UNMET" for c in criteria[2:]) and
            all(type(c["source_limit"]) is str and len(c["source_limit"])>20
                for c in criteria) and
            tuple(c["id"] for c in r8plan["closure_criteria"])==CLOSURE_IDS,
            "R9_FORMAL_CLOSURE_CRITERIA_FALSE_OR_ABSENT")
    blockers=d["missing_external_evidence_dependency_graph"]
    require(type(blockers) is list and len(blockers)==5 and
            tuple(b["id"] for b in blockers)==GAP_IDS and
            tuple(b["state"] for b in blockers)==GAP_STATES and
            tuple(tuple(b["depends_on"]) for b in blockers)==DEPENDENCIES and
            all(b["evidence_sha256"] is None and
                b["verified_by_independent_operator"] is False and
                type(b["evidence_requirements"]) is str and
                len(b["evidence_requirements"])>35
                for b in blockers),
            "R9_FAKE_EXTERNAL_CERTIFICATES_OR_UNPROVEN_DEPENDENCY")
    require(tuple(tuple(c["depends_on"]) for c in criteria)==(
              (),(),("IND-SPECIALISTS",),("IND-OOB",),
              ("IND-OPERATOR","IND-OOB"),
              ("IND-OPERATOR","IND-LEDGER"),
              ("IND-SPECIALISTS","IND-OOB","IND-OPERATOR",
               "IND-LEDGER","IND-FORMAL")),
            "R9_CONTRADICTORY_DEPENDENCY_GRAPH_CHANGED")
    attacks=d["red_team_scenarios"]
    require(type(attacks) is list and len(attacks)==8 and
            tuple(x["id"] for x in attacks)==ATTACK_IDS and
            all(x["required_action"]=="DENY" and
                len(x["mechanism"])>25 for x in attacks),
            "R9_EIGHT_MISSING_ANTI_FRAUD_NEGATIVES")
    roles=d["nine_roles_not_independently_reviewed"]
    gates=d["twelve_native_gates"]
    require(tuple(r["role"] for r in roles)==guardian.ROLE_IDS and
            all(r["execution"]=="NOT_SUBMITTED" and
                r["independent_signature"] is None and
                r["review_log"] is None and r["completed"] is False
                for r in roles) and
            tuple(g["id"] for g in gates)==guardian.GATE_IDS and
            all(g["status"]==guardian.GATE_UNMET and
                g["independent_evidence"] is None and
                g["admitted"] is False for g in gates),
            "R9_LOCAL_CLAIM_PROMOTED_TO_INDEPENDENT_PROOF")
    require(d["technical_research_classification"]==
                "R1_TO_R8_OWNER_LOGS_AND_CI_CONSOLIDATED_AS_TECHNICAL_ONLY" and
            d["external_evidence_classification"]==
                "FIVE_OF_SEVEN_FORMAL_CLOSURE_CRITERIA_STILL_UNMET" and
            d["decision"]==
                "HOLD_FORMAL_R6F_CLOSURE_MISSING_INDEPENDENT_EVIDENCE" and
            d["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
            d["r6g"]=="LOCKED_BY_SEQUENCE" and
            d["native_admission_gates_unmet"]==12 and
            d["bureau_independent_specialist_reviews"]==0 and
            d["formal_bureau_certification"]=="PENDING" and
            d["owner_oob_root"]=="NOT_AUTHENTICATED" and
            d["real_independent_custodian"]=="NOT_ENROLLED" and
            d["global_monotonic_antirollback"]=="NOT_VERIFIED" and
            d["independent_quorum"]=="NOT_PROVEN" and
            d["real_external_operator_invited"] is False and
            d["independent_external_enrollment_completed"] is False and
            d["external_network_called"] is False and
            d["original_source_changed"] is False and
            d["github_merge"]=="NONE" and d["production"]=="NOT_AUTHORIZED" and
            d["owner_mac_intel_r9"]=="PENDING" and
            d["r9_recommendation"]==
                "STOP_REPEAT_ONLY_LOCAL_CERTIFICATION_LOOPS_UNTIL_NEW_INDEPENDENT_EVIDENCE_OR_FALSIFIABLE_SECURITY_FINDING; MAINTAIN_DENY_ONLY_RESEARCH",
            "R9_FALSE_BUREAU_DECISION_OR_PRODUCTION_AUTHORIZATION")
    return len(criteria),len(blockers),len(attacks)

def audit(root):
    root=Path(root).resolve()
    baseline=r8.audit(root)
    require(baseline["reason"]==
            "R8_RESEARCH_DOSSIER_RECONCILED_FORMAL_R6F_CLOSURE_BLOCKED"
            and baseline["external_closure_criteria_unmet"]==5
            and baseline["native_admission_gates_unmet"]==12,
            "R8_EXACT_FROZEN_PARENTS_NOT_VERIFIED")
    plan=load(root,DOSSIER)
    for source in plan["source_of_truth"]:
        r7.file(root,source)
    c,b,a=validate_sources(
        load(root,r7.PLAN),load(root,r8.DOSSIER),
        load(root,r7.FROZEN),load(root,r7.R2),plan)
    return deny(
        "R9_FINAL_EVIDENCE_DEPENDENCIES_RECONCILED_FORMAL_CLOSURE_HELD",
        technical_evidence_categories=2,
        formal_closure_criteria=c,external_evidence_classes_missing=b,
        tested_fraud_scenarios_documented=a,
        final_independent_admission_evidence_received=0,
        r8_owner_reported_regular_tests=1255,
        source_time_r8_pending_snapshot_preserved=True,
        formal_r6f_closure_granted=False,
        independent_native_gates_promoted=0,
        recommendation="STOP_REPEAT_ONLY_LOCAL_CERTIFICATION_LOOPS_UNTIL_NEW_INDEPENDENT_EVIDENCE_OR_FALSIFIABLE_SECURITY_FINDING"
    )

def candidate_template():
    return dict(protocol=CLAIM_PROTOCOL,parent=PARENT,
       requested_operation="RECORD_UNAUTHENTICATED_RESEARCH_CLOSURE_CLAIM_ONLY",
       claims_nine_independent_reviews=False,
       claims_owner_oob_confirmed=False,
       claims_distinct_external_operator=False,
       claims_global_monotonic_history=False,
       claims_formal_owner_authorization=False,
       claims_all_twelve_native_gates=False)

def assess_candidate(raw):
    try:
        obj=r7.strict(raw,max_bytes=MAX_CLAIM,canonical_only=True)
        require(obj.keys()==CLAIM_FIELDS and obj["protocol"]==CLAIM_PROTOCOL
                and obj["parent"]==PARENT and obj["requested_operation"]==
                "RECORD_UNAUTHENTICATED_RESEARCH_CLOSURE_CLAIM_ONLY",
                "R9_ILLEGAL_OR_UNPINNED_DECISION_OPERATION")
        keys=tuple(x for x in CLAIM_FIELDS if x.startswith("claims_"))
        require(all(type(obj[k]) is bool for k in keys),
                "R9_UNTRUSTED_DECISION_CLAIM_INVALID_BOOLEAN")
        asserted=any(obj[k] for k in keys)
        return deny(
           "R9_SELF_ATTESTED_EXTERNAL_BUREAU_APPROVAL_REJECTED"
           if asserted else "R9_NO_NEW_INDEPENDENT_EVIDENCE_HOLD",
           self_attested_external_claims_did_not_change_authority=True,
           formal_r6f_closure_granted=False,independent_native_gates_promoted=0
        )
    except (Blocked,r7.Blocked,ValueError,TypeError,KeyError) as e:
        return deny("R9_INVALID_OR_PRIVILEGED_DECISION_PACKET_QUARANTINED",
              detail=str(e) if isinstance(e,(Blocked,r7.Blocked))
                    else type(e).__name__)

def exercise(root):
    current=audit(root)
    require(current["reason"]==
        "R9_FINAL_EVIDENCE_DEPENDENCIES_RECONCILED_FORMAL_CLOSURE_HELD",
        "R9_SOURCE_AUDIT_DID_NOT_RECONCILE")
    if not (sys.platform.startswith(("linux","darwin")) and
            Path("/usr/bin/ssh-keygen").is_file()):
        return deny("R9_REAL_NATIVE_POSIX_OPENSSH_REQUIRED"),2
    # Previous R8 actually invokes genuine local OpenSSH Ed25519 and R5Q
    # real new-process history rollback; do not simulate any external actor.
    p=subprocess.run([sys.executable,"-B",
         str(Path(root)/"scripts/bureau_001_r8_closure_gate.py"),"--exercise"],
         cwd=root,capture_output=True,text=True,timeout=400)
    require(p.returncode==2,"R9_REAL_R8_NEGATIVE_EXIT_NOT_TWO")
    try: result=json.loads(p.stdout)
    except (ValueError,UnicodeError) as e:
        raise Blocked("R9_REAL_R8_EVIDENCE_JSON_MISSING") from e
    require(result["reason"]==
        "R8_REAL_SIGNED_LOCAL_PROOFS_CANNOT_CLOSE_BUREAU_WITHOUT_EXTERNAL_AUTHORITY"
        and result["r7_true_two_local_ed25519_signatures"]==2
        and result["r5q_real_three_file_coherent_rollback"] is True
        and result["external_admission_gates_promoted"]==0,
        "R9_REAL_SIGNED_LOCAL_AND_ROLLBACK_NOT_NEGATIVE")
    claim=candidate_template()
    for k in claim:
        if k.startswith("claims_"):claim[k]=True
    rejected=assess_candidate(r7.canonical(claim))
    require(rejected["reason"]=="R9_SELF_ATTESTED_EXTERNAL_BUREAU_APPROVAL_REJECTED"
            and rejected["independent_native_gates_promoted"]==0,
            "R9_FAKE_BUREAU_CLAIMS_SELF_CERTIFIED")
    return deny(
        "R9_REAL_LOCAL_SIGNATURES_ROLLBACK_AND_FAKE_FINAL_DECISION_REFUSED",
        real_local_ed25519_signatures_verified=2,
        historical_r5q_three_file_coherent_rollback=True,
        counterfeit_nine_reviews_and_twelve_native_gates_refused=True,
        final_bureau_closure_granted=False,
        independent_external_evidence_items_received=0
    ),2

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[1])
    m=parser.add_mutually_exclusive_group(required=True)
    m.add_argument("--audit",action="store_true")
    m.add_argument("--exercise",action="store_true")
    args=parser.parse_args(argv)
    try:
        result,rc=(audit(args.root),2) if args.audit else exercise(args.root)
    except (Blocked,r7.Blocked,r8.Blocked,
            r7.r1.Blocked,r8.r2.Blocked,guardian.Blocked,
            OSError,ValueError,TypeError,KeyError,
            subprocess.SubprocessError) as e:
        result=deny("R9_SOURCE_INCONSISTENCY_OR_REAL_EXERCISE_QUARANTINED",
           detail=str(e) if isinstance(e,(Blocked,r7.Blocked,r8.Blocked,
              r7.r1.Blocked,r8.r2.Blocked,guardian.Blocked))
                 else type(e).__name__)
        rc=3
    print(json.dumps(result,sort_keys=True))
    return rc

if __name__=="__main__":
    sys.exit(main())
