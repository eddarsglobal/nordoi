#!/usr/bin/env python3
"""G0.3 R6F BUREAU-001 R7 — contradiction/reviewer/guardian feasibility audit.

Source-based, READ-ONLY, DENY-ONLY. Cross-checks frozen R1 Git identities,
historical R6B-E snapshots, R2 unsubmitted nine-role evidence, additive
R3-R6 negative findings and exact 12 external admission gates. Separates
later owner-supplied Terminal reports and CI observations from source-time
snapshot PENDING states. No autonomous specialist signatures are invented.

An optional genuine POSIX native exercise reruns R6 local Ed25519 SSHSIG
and the existing R5Q coherent local rollback. Neither creates any
independent custodian, externally authenticated OOB root or authority.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

import bureau_001_r1_audit as r1
import bureau_001_r2_review_intake as r2
import roadmap_guardian as guardian

PROJECT = "eddarsglobal/nordoi"
PARENT = "17614622acb09440dfe2428b85fe2c89491a0b35"
PROTOCOL = "NORDOI-G03-R6F-BUREAU001-R7-CONTRADICTION-EVIDENCE-DENY-V1"
PLAN = "governance/NORDOI_R6F_BUREAU001_R7_CONTRADICTIONS_AND_GUARDIAN_PLAN.json"
R2 = "governance/NORDOI_R6F_BUREAU001_R2_REVIEW_REGISTER.json"
R3 = "governance/NORDOI_R6F_BUREAU001_R3_CROSS_AUDIT.json"
R4 = "governance/NORDOI_R6F_BUREAU001_R4_RACE_AUDIT.json"
R5 = "governance/NORDOI_R6F_BUREAU001_R5_ABA_MULTIFILE_AUDIT.json"
R6 = "governance/NORDOI_R6F_BUREAU001_R6_EXTERNAL_CUSTODY_FEASIBILITY.json"
FROZEN = "research/G03_R6E_OOB_ORIGIN_NO_EXTERNAL_AUTHORITY.json"
EXPECTED_HEADS = (
    ("R1","108548e2021de42e65e8261fadd8184d4d9ff783",1087),
    ("R2","80010bdddd5e4743f023c0eed888a1c3360339da",1111),
    ("R3","2dd46dc07568bf382c49b1c8bca0cba8900b7b1f",1135),
    ("R4","4dd43f37e7a7ac0ca0b0b39fcc88248cd2bad9ad",1159),
    ("R5","4d3bd3cbff8c67cef98481750270a2c6c0ce7b37",1183),
    ("R6",PARENT,1207),
)
CONTRADICTIONS = (
    "R7-TEMPORAL-001","R7-PROVENANCE-002","R7-TOCTOU-003",
    "R7-ABA-004","R7-OOB-005","R7-REVIEW-006","R7-CI-007",
)
PHASES = ("A","B","C","D","E","F")
MAX_CLAIM_BYTES = 4096
CLAIM_PROTOCOL = "NORDOI-G03-R6F-R7-UNVERIFIED-EXTERNAL-CUSTODY-CANDIDATE-V1"
CLAIM_FIELDS = frozenset((
    "protocol","project","parent","requested_operation",
    "asserted_operator_id","asserted_owner_oob_root","asserted_independent_quorum",
    "asserted_external_monotonic_floor","asserted_native_gate_ids"
))

class Blocked(ValueError):
    pass

def require(ok, reason):
    if not ok:
        raise Blocked(reason)

def deny(reason, **extras):
    return dict(status="BLOCK", admission="NOT_AUTHORIZED",
                native_authority_granted=False, production="NOT_AUTHORIZED",
                owner_oob_root="NOT_AUTHENTICATED",
                independent_external_custodian="NOT_ENROLLED",
                independent_specialist_reviews=0,
                external_monotonic_antirollback="NOT_VERIFIED",
                native_admission_gates_unmet=12,
                r6g="LOCKED_BY_SEQUENCE",reason=reason,**extras)

def canonical(obj):
    return (json.dumps(obj,sort_keys=True,ensure_ascii=True,
                       separators=(",",":"),allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    out={}
    for key,value in pairs:
        if key in out:raise Blocked("DUPLICATE_JSON_FIELD:"+key)
        out[key]=value
    return out

def nonfinite(_):
    raise Blocked("NONFINITE_JSON")

def strict(raw,max_bytes=65536,canonical_only=False):
    require(type(raw) is bytes and 0<len(raw)<=max_bytes,"JSON_MISSING_OR_OVERSIZED")
    try:
        value=json.loads(raw.decode("utf-8"),
                         object_pairs_hook=unique,parse_constant=nonfinite)
    except (UnicodeError,json.JSONDecodeError) as e:
        raise Blocked("INVALID_JSON") from e
    require(type(value) is dict,"JSON_NOT_OBJECT")
    if canonical_only:require(raw==canonical(value),"NONCANONICAL_JSON")
    return value

def file(root,rel):
    require(type(rel) is str and rel and "\\" not in rel
            and not rel.startswith("/") and
            ".." not in Path(rel).parts,"UNSAFE_EVIDENCE_PATH")
    p=Path(root)/rel
    require(not p.is_symlink() and p.is_file(),"MISSING_OR_SYMLINK_EVIDENCE:"+rel)
    return p

def load(root,rel):
    return strict(file(root,rel).read_bytes())

def gitblob(raw):
    return hashlib.sha1(b"blob "+str(len(raw)).encode()+b"\0"+raw).hexdigest()

def check_cross_sources(manifest,chronology,register,r3,r4,r5,r6,frozen,plan):
    """No privilege changes: returns a source-bound checklist or fails shut."""
    require(manifest["total_pinned_files"]==99 and
            len(manifest["pinned_files"])==99,"R1_FROZEN_99_NOT_PRESERVED")
    require(chronology["source_snapshots_status"]=="PENDING_AT_COMMIT_TIME"
            and "OWNER_SUPPLIED" in chronology["owner_evidence_source"]
            and len(chronology["stages"])==4,
            "R1_SNAPSHOT_VS_OWNER_LOG_PROVENANCE_LOST")
    for stage in chronology["stages"]:
        require(stage["id"] in ("R6B","R6C","R6D","R6E") and
                stage["source_owner_status_expected"]=="PENDING" and
                stage["observed_owner_terminal"]==
                    "PASS_WITH_CONTROLS_RESEARCH_ONLY" and
                stage["observed_owner_exit"]==0 and
                stage["independent_custodian"]=="NOT_PROVEN" and
                stage["source_correction_authorized"] is False,
                "R1_TEMPORAL_EVIDENCE_CONFUSED_WITH_FROZEN_SOURCE")
    require(register["parent_r1_exact_head"]==EXPECTED_HEADS[0][1]
            and register["review_status"]==
                "COORDINATOR_SOURCE_INSPECTION_ONLY"
            and register["submissions"]==[]
            and register["independent_specialist_agents_executed"]==0,
            "R2_INDEPENDENT_REVIEW_FABRICATED")
    roles=register["roles"]
    require(tuple(role.get("role") for role in roles)==guardian.ROLE_IDS,
            "R2_NINE_BUREAU_ROLES_CHANGED")
    pins={p["path"]:p["git_blob_sha1"] for p in manifest["pinned_files"]}
    for role in roles:
        require(role["source_path"] in pins and
                role["git_blob_sha1"]==pins[role["source_path"]] and
                role["specialist_review_status"]=="NOT_SUBMITTED" and
                role["external_reviewer_identity"] is None and
                role["independent_execution_log"] is None and
                role["certification"] is None,
                "R2_COOPTED_OR_UNPINNED_SPECIALIST_PROVENANCE")
    require(r3["parent_r2_exact_head"]==EXPECTED_HEADS[1][1] and
            r3["formal_independent_agent_reviews_completed"]==0 and
            len(r3["roles"])==9,"R3_OLD_REVIEW_OR_PARENT_TAMPERED")
    require(r4["parent_exact_r3_head"]==EXPECTED_HEADS[2][1] and
            r4["formal_independent_bureau_reviews"]==0 and
            r4["native_admission_gates_unmet"]==12 and
            r4["r6g"]=="LOCKED_BY_SEQUENCE",
            "R4_OLD_REVIEW_OR_PARENT_TAMPERED")
    require(r5["parent_r4_head"]==EXPECTED_HEADS[3][1] and
            r5["reviews_independently_executed"]==0 and
            r5["native_admission_gates_unmet"]==12 and
            r5["r6g"]=="LOCKED_BY_SEQUENCE",
            "R5_OLD_REVIEW_OR_PARENT_TAMPERED")
    require(r6["parent_r5_exact_head"]==EXPECTED_HEADS[4][1] and
            r6["independently_executed_specialist_reviews"]==0 and
            r6["independent_custodian_enrolled"] is False and
            r6["owner_authenticated_oob_root"] is False and
            r6["independent_global_history_verified"] is False and
            r6["r6g"]=="LOCKED_BY_SEQUENCE",
            "R6_EXTERNAL_OPERATOR_STATUS_FABRICATED")
    ids=guardian.GATE_IDS
    require(tuple(g["id"] for g in frozen["required_gates"])==ids and
            frozen["native_admission_gates_unmet"]==12 and
            tuple(g["id"] for g in r6["required_native_gates"])==ids and
            all(g["status"]==guardian.GATE_UNMET for g in frozen["required_gates"])
            and all(g["status"]==guardian.GATE_UNMET
                    for g in r6["required_native_gates"]),
            "R6_TWELVE_EXTERNAL_ADMISSION_GATES_NOT_SOURCE_EXACT")
    require(plan["schema"]==
             "NORDOI-G03-R6F-BUREAU001-R7-CONTRADICTION-AND-EXTERNAL-GUARDIAN-ADMISSION-PLAN-V1"
            and plan["project"]==PROJECT
            and plan["parent_r6_exact_head"]==PARENT
            and plan["historical_sources_mutated"] is False
            and plan["research_only"] is True
            and len(plan["contradictions"])==7
            and tuple(x["code"] for x in plan["contradictions"])==CONTRADICTIONS
            and all(x["authority"]=="NONE"
                    for x in plan["contradictions"]),
            "R7_CONTRADICTORY_CLAIMS_OR_FUTURE_AUTHORITY_INCORRECT")
    stages=plan["historical_reconciliation"]
    require(len(stages)==6 and
            tuple((s["stage"],s["head"],s["owner_reported_regular_tests"])
                  for s in stages)==EXPECTED_HEADS and
            len({s["head"] for s in stages})==6,
            "R7_SIX_STAGE_HISTORICAL_PROVENANCE_NOT_EXACT")
    expected_sources=(
        r1.CHRONOLOGY,R2,R3,R4,R5,R6
    )
    require(tuple(s["source"] for s in stages)==expected_sources,
            "R7_STAGE_TO_SOURCE_BINDING_INCORRECT")
    gates=plan["external_gate_plan"]
    require(tuple(g["id"] for g in gates)==ids
            and all(g["state"]==guardian.GATE_UNMET and
                    g["external_evidence_sha256"] is None and
                    g["independent_operator_signature"] is None and
                    g["owner_out_of_band_verification_receipt"] is None and
                    g["admitted"] is False and
                    isinstance(g["future_real_evidence_required"],str)
                    and g["future_real_evidence_required"]
                    for g in gates),
            "R7_OWN_INVENTED_EXTRINSIC_EVIDENCE_OR_NATIVE_GRANT")
    phases=plan["independent_operator_enrollment_phases"]
    require(tuple(p["phase"] for p in phases)==PHASES and
            all(p["status"]=="NOT_STARTED" for p in phases),
            "R7_FALSE_EXTERNAL_GUARDIAN_ENROLLMENT")
    require(tuple(plan["nine_roles"])==guardian.ROLE_IDS
            and plan["independently_executed_specialist_reviews"]==0
            and plan["r2_specialist_submissions_count"]==0 and
            plan["owner_oob_root"]=="NOT_AUTHENTICATED"
            and plan["independent_custodian"]=="NOT_ENROLLED"
            and plan["external_global_monotonic_antirollback"]=="NOT_VERIFIED"
            and plan["independent_signed_quorum"]=="NOT_PROVEN"
            and plan["native_admission_gates_unmet"]==12
            and plan["r6f"]=="IN_PROGRESS_UNCERTIFIED"
            and plan["r6g"]=="LOCKED_BY_SEQUENCE"
            and plan["bureau_formal_certification"]=="PENDING"
            and plan["production"]=="NOT_AUTHORIZED"
            and plan["real_external_operator_invited"] is False,
            "R7_FALSE_AUTHORITY_CERTIFICATION_OR_PRIVILEGE")
    return 7,12,9

def audit(root):
    root=Path(root).resolve()
    previous=r1.audit(root)
    require(previous["FROZEN_SOURCE_BLOBS"]=="99/99_PASS",
            "R1_REAL_SOURCE_AUDIT_FAIL_CLOSED")
    previous2=r2.audit(root)
    require(previous2.get("R6G")=="LOCKED_BY_SEQUENCE"
            and previous2.get("PRODUCTION")=="NOT_AUTHORIZED",
            "R2_REAL_REVIEW_INTAKE_NOT_DENY_ONLY")
    manifest=load(root,r1.MANIFEST)
    chrono=load(root,r1.CHRONOLOGY)
    reg=load(root,R2)
    r3=load(root,R3);r4=load(root,R4)
    r5=load(root,R5);r6=load(root,R6)
    frozen=load(root,FROZEN);plan=load(root,PLAN)
    n,g,roles=check_cross_sources(manifest,chrono,reg,r3,r4,r5,r6,frozen,plan)
    for c in plan["contradictions"]:
        require(type(c.get("sources")) is list and 1<=len(c["sources"])<=3
                and type(c.get("finding")) is str and len(c["finding"])>18
                and type(c.get("disposition")) is str
                and c["disposition"]
                and all(file(root,rel) for rel in c["sources"]),
                "R7_UNREFERENCED_CROSS_AUDIT_FINDING")
    return deny(
        "R7_SOURCE_CONTRADICTIONS_RECONCILED_NO_EXTERNAL_CUSTODY",
        r7_protocol=PROTOCOL,
        verified_r1_frozen_git_objects=99,
        owner_terminal_evidence_is_not_an_independent_operator_signature=True,
        r6b_to_r6e_snapshot_vs_later_owner_reports="4/4_TIME_SCOPED_NOT_REWRITTEN",
        r1_through_r6_stage_pins=6,
        cross_audit_findings_reconciled=n,
        bureau_source_perspectives=roles,
        independent_specialist_submissions=0,
        exact_external_gates_still_unmet=g,
        future_external_enrollment_phases="6_NOT_STARTED",
        formal_bureau_certification="PENDING"
    )

def candidate_template():
    return dict(
        protocol=CLAIM_PROTOCOL,project=PROJECT,parent=PARENT,
        requested_operation="STAGE_UNAUTHENTICATED_PROVENANCE_CLAIM_ONLY",
        asserted_operator_id=None,asserted_owner_oob_root=None,
        asserted_independent_quorum=None,asserted_external_monotonic_floor=None,
        asserted_native_gate_ids=list(guardian.GATE_IDS)
    )

def assess_candidate(raw):
    """Even syntactically valid outsider-sounding local assertions cannot admit."""
    try:
        obj=strict(raw,MAX_CLAIM_BYTES,canonical_only=True)
        require(obj.keys()==CLAIM_FIELDS,"R7_WRONG_CANDIDATE_SCHEMA")
        require((obj["protocol"],obj["project"],obj["parent"],
                obj["requested_operation"])==(
                 CLAIM_PROTOCOL,PROJECT,PARENT,
                 "STAGE_UNAUTHENTICATED_PROVENANCE_CLAIM_ONLY"),
                 "R7_CANDIDATE_REQUESTED_PRIVILEGED_OPERATION")
        require(type(obj["asserted_native_gate_ids"]) is list and
                tuple(obj["asserted_native_gate_ids"])==guardian.GATE_IDS,
                "R7_CANDIDATE_REORDERED_DUPLICATED_OR_REMOVED_GATES")
        for field in ("asserted_operator_id","asserted_owner_oob_root",
                      "asserted_independent_quorum",
                      "asserted_external_monotonic_floor"):
            val=obj[field]
            require(val is None or (type(val) is str and
                        1<=len(val)<=128 and "\x00" not in val),
                    "R7_INVALID_UNTRUSTED_EXTERNAL_CLAIM_"+field)
        asserted=any(obj[k] is not None for k in (
            "asserted_operator_id","asserted_owner_oob_root",
            "asserted_independent_quorum","asserted_external_monotonic_floor"))
        return deny(
            "R7_SELF_ASSERTED_EXTERNAL_AUTHORITY_STAGED_UNAUTHENTICATED"
            if asserted else "R7_NO_EXTERNAL_AUTHORITY_EVIDENCE_SUBMITTED",
            signed_external_authority_authenticated=False,
            local_claim_source_only=True,
            external_gates_promoted=0
        )
    except (Blocked,TypeError,ValueError,KeyError) as e:
        return deny("R7_EXTERNAL_CLAIM_QUARANTINED",
                    detail=str(e) if isinstance(e,Blocked) else type(e).__name__)

def exercise(root):
    if not (sys.platform.startswith(("linux","darwin")) and
            Path("/usr/bin/ssh-keygen").is_file()):
        return deny("R7_REAL_POSIX_OPENSSH_REQUIRED"),2
    initial=audit(root)
    require(initial["reason"]==
            "R7_SOURCE_CONTRADICTIONS_RECONCILED_NO_EXTERNAL_CUSTODY",
            "R7_CROSS_AUDIT_DID_NOT_RECONCILE")
    program=Path(root)/"research/g03_r6f_r6_external_custody_feasibility.py"
    p=subprocess.run([sys.executable,"-B",str(program),"--exercise"],
                     cwd=root,text=True,capture_output=True,timeout=300)
    require(p.returncode==2,
            "R7_REAL_R6_TWO_LOCAL_SSHSIG_EXERCISE_NOT_DENIAL_EXIT_2")
    try:
        result=json.loads(p.stdout)
    except ValueError as e:
        raise Blocked("R7_REAL_R6_NATIVE_JSON_MISSING") from e
    require(result.get("reason")==
            "R6_REAL_LOCAL_TWO_KEY_OOB_CLAIMS_AND_CROSS_EPOCH_ROLLBACK_NO_EXTERNAL_AUTHORITY"
            and result.get("true_local_ssh_signatures_verified")==2
            and result.get("malicious_same_admin_oob_claim_genuinely_signed") is True
            and result.get("real_r5q_three_file_coherent_rollback_reproduced") is True
            and result.get("native_admission_gates_unmet")==12,
            "R7_ACTUAL_LOCAL_TRUE_SIGNATURE_VS_EXTERNAL_CUSTODY_NOT_REPRODUCED")
    spoof=candidate_template()
    spoof["asserted_operator_id"]="forged-third-party-label"
    spoof["asserted_owner_oob_root"]="forged-locally-authored-oob-proof"
    spoof["asserted_independent_quorum"]="two-self-asserted-local-keys"
    spoof["asserted_external_monotonic_floor"]="locally-asserted-monotonic-history"
    refused=assess_candidate(canonical(spoof))
    require(refused["reason"]==
            "R7_SELF_ASSERTED_EXTERNAL_AUTHORITY_STAGED_UNAUTHENTICATED"
            and refused["external_gates_promoted"]==0,
            "R7_LOCAL_FAKE_EXTERNAL_CLAIMS_WERE_PROMOTED")
    return deny(
        "R7_REAL_R6_LOCAL_SIGNATURES_R5Q_ROLLBACK_AND_FALSE_CUSTODY_REFUSED",
        source_contradictions_reconciled=7,
        owner_reported_tests_not_external_certification=True,
        authentic_local_ed25519_signatures_verified=2,
        real_original_r5q_three_file_cross_epoch_rollback=True,
        self_asserted_external_guardian_quorum_quarantined=True,
        independent_external_gates_promoted=0,
        future_protocol_phases="6_NOT_STARTED"
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
        if args.audit:
            result=audit(args.root);code=2
        else:
            result,code=exercise(args.root)
    except (Blocked,r1.Blocked,r2.Blocked,guardian.Blocked,
            OSError,TypeError,ValueError,KeyError,
            subprocess.SubprocessError) as e:
        result=deny("R7_CONTRADICTION_SOURCE_OR_REAL_EXERCISE_QUARANTINED",
                    detail=str(e) if isinstance(e,(
                        Blocked,r1.Blocked,r2.Blocked,guardian.Blocked
                    )) else type(e).__name__)
        code=3
    print(json.dumps(result,sort_keys=True))
    return code

if __name__=="__main__":
    sys.exit(main())
