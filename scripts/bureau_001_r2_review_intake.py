#!/usr/bin/env python3
"""NORDOI R6F BUREAU-001 R2: read-only specialist-review evidence intake.

Nine source observations are one coordinator's research observations, not
nine independently executed models/operators. An optional caller-supplied
review submission plus local log can be integrity-checked and STAGED, but
never authenticated as an independently controlled reviewer or certified.
All native admissions, production and R6G remain BLOCKED.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bureau_001_r1_audit as previous

PARENT = "108548e2021de42e65e8261fadd8184d4d9ff783"
REGISTRY = "governance/NORDOI_R6F_BUREAU001_R2_REVIEW_REGISTER.json"
ROLES = previous.guardian.ROLE_IDS
CANDIDATE_SCHEMA = "NORDOI-R6F-R2-REVIEW-SUBMISSION-V1"
CANDIDATE_FIELDS = frozenset((
    "schema", "project", "parent_head", "role", "source_git_blob_sha1",
    "reviewer_claim", "provenance", "execution_command_record",
    "review_result_claim", "evidence_log_relpath", "evidence_log_sha256",
    "requested_decision", "independent_operator_attestation",
    "independent_signer_signature",
))
REVIEW_RESULTS = ("PASS", "REMEDIATE", "BLOCK")
MAX_SUBMISSION = 8192
MAX_LOG = 32768

class Blocked(ValueError):
    pass

def require(condition, error):
    if not condition:
        raise Blocked(error)

def unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise Blocked("DUPLICATE_JSON_KEY:"+str(key))
        result[key] = value
    return result

def not_finite(value):
    raise Blocked("NONFINITE_NUMBER")

def canonical(data):
    return (json.dumps(data, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def strict_json(raw, *, max_len, canonical_required=False):
    require(type(raw) is bytes and 0 < len(raw) <= max_len,
            "JSON_MISSING_OR_OVERSIZED")
    try:
        obj = json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                         parse_constant=not_finite)
    except (UnicodeError, ValueError) as err:
        if isinstance(err, Blocked):
            raise
        raise Blocked("JSON_INVALID_ENCODING_OR_SYNTAX") from err
    require(type(obj) is dict, "JSON_NOT_OBJECT")
    if canonical_required:
        require(canonical(obj) == raw, "JSON_NOT_CANONICAL")
    return obj

def gate(root):
    root = Path(root).resolve()
    result = previous.audit(root)
    require(result["BUREAU001_R1"] == "PASS_WITH_CONTROLS_SOURCE_AUDIT_ONLY"
            and result["BUREAU_INDEPENDENT_REVIEWS_COMPLETED"] == 0,
            "R1_SOURCE_BASELINE_NOT_PASSED")
    manifest = previous.read_json(root, previous.MANIFEST)
    pins = previous.verify_manifest(root, manifest)
    registry = strict_json(previous.raw(root, REGISTRY), max_len=32768)
    require(registry.get("schema") ==
            "NORDOI-G03-R6F-BUREAU001-R2-REVIEW-REGISTER-V1"
            and registry.get("project") == "eddarsglobal/nordoi"
            and registry.get("parent_r1_exact_head") == PARENT,
            "WRONG_R2_REGISTRY_OR_PARENT")
    require(registry.get("review_status") ==
            "COORDINATOR_SOURCE_INSPECTION_ONLY"
            and registry.get("source_evidence") ==
            "GITHUB_SOURCE_FILES_INSPECTED_WITH_COORDINATOR_LIMITATIONS"
            and registry.get("review_scope") ==
            "NINE_SPECIALIST_VIEWPOINTS_SOURCE_ANCHORED_NO_INDEPENDENT_AGENTS_CLAIMED"
            and registry.get("independent_specialist_agents_executed") == 0,
            "FALSE_INDEPENDENT_REVIEW_EXECUTION")
    require(registry.get("independent_administrative_custodian") ==
            "NOT_ENROLLED" and registry.get("owner_oob_root") ==
            "NOT_AUTHENTICATED" and registry.get("global_monotonic_antirollback") ==
            "NOT_VERIFIED" and registry.get("native_admission_gates_unmet") == 12
            and registry.get("production") == "NOT_AUTHORIZED"
            and registry.get("r6f") == "IN_PROGRESS"
            and registry.get("r6g") == "LOCKED_BY_SEQUENCE",
            "REVIEW_REGISTRY_NATIVE_AUTHORITY_ESCALATED")
    require(registry.get("submissions") == [],
            "SELF_ENTERED_REVIEW_SUBMISSION_UNVERIFIED")
    policy = registry.get("guard_policy", {})
    require(type(policy) is dict and policy == {
        "accepted_submission_disposition": "STAGED_SOURCE_AND_LOG_INTEGRITY_ONLY",
        "invalid_or_self_certified_submission": "BLOCK",
        "actual_agent_execution_not_inferred": True,
        "review_certification_not_automated": True,
        "native_admission_not_granted": True,
        "user_owner_privileged_authorization_not_derived": True,
        "r6g_locked": True,
    }, "REVIEW_GATE_ESCALATION_POLICY_CHANGED")
    entries = registry.get("roles")
    require(type(entries) is list and len(entries) == 9 and
            tuple(x.get("role") if type(x) is dict else None for x in entries)
            == ROLES, "NINE_ROLE_SOURCE_EVIDENCE_NOT_COMPLETE")
    seen_paths = set()
    for row in entries:
        p = row.get("source_path")
        require(type(p) is str and p in pins,
                "ROLE_UNPINNED_SOURCE_PATH")
        require(row.get("git_blob_sha1") == pins[p]["git_blob_sha1"],
                "ROLE_EVIDENCE_PIN_MISMATCH")
        anchor = row.get("literal_source_anchor")
        require(type(anchor) is str and 6 <= len(anchor) <= 140
                and "\n" not in anchor and "\r" not in anchor,
                "ROLE_ANCHOR_INVALID")
        src = previous.git_checkout_bytes(previous.raw(root, p))
        require(anchor.encode("utf-8") in src,
                "ROLE_SOURCE_ANCHOR_NOT_PRESENT:"+row["role"])
        require(p not in seen_paths, "ROLE_SOURCE_REUSED_WITHOUT_SEPARATE_SCOPE")
        seen_paths.add(p)
        require(type(row.get("coordinator_observation")) is str and
                len(row["coordinator_observation"]) >= 24 and
                row.get("record_class") ==
                "COORDINATOR_SOURCE_INSPECTED_NOT_A_COMPLETED_SPECIALIST_REVIEW"
                and row.get("specialist_review_status") == "NOT_SUBMITTED"
                and row.get("external_reviewer_identity") is None
                and row.get("independent_execution_log") is None
                and row.get("certification") is None
                and row.get("further_work") ==
                "INDEPENDENT_SOURCE_AND_SECURITY_REVIEW_REQUIRED",
                "COORDINATOR_OBSERVATION_FAKED_AS_COMPLETED_REVIEW")
    return result, registry, {row["role"]: row for row in entries}

def audit(root):
    before, registry, _ = gate(root)
    return {
        "BUREAU001_R2": "PASS_WITH_CONTROLS_COORDINATOR_SOURCE_INTAKE_ONLY",
        "PARENT_R1": PARENT,
        "R1_PINNED_SOURCES": before["FROZEN_SOURCE_BLOBS"],
        "SOURCE_PERSPECTIVES_INSPECTED": "9/9",
        "INDEPENDENT_REVIEWER_EXECUTIONS": 0,
        "INDEPENDENTLY_CERTIFIED_SPECIALIST_REPORTS": 0,
        "REAL_EXTERNAL_GUARDIAN": "NOT_ENROLLED",
        "OWNER_OOB_ROOT": "NOT_AUTHENTICATED",
        "R6F": "IN_PROGRESS_UNCERTIFIED",
        "R6G": "LOCKED_BY_SEQUENCE",
        "NATIVE_ADMISSION_GATES": "12_UNMET",
        "PRODUCTION": "NOT_AUTHORIZED",
    }

def stage_submission(root, relpath):
    root = Path(root).resolve()
    _, reg, rows = gate(root)
    payload = previous.raw(root, relpath)
    item = strict_json(payload, max_len=MAX_SUBMISSION,
                       canonical_required=True)
    require(set(item) == CANDIDATE_FIELDS,
            "UNEXPECTED_MISSING_REVIEW_SUBMISSION_FIELDS")
    require(item["schema"] == CANDIDATE_SCHEMA
            and item["project"] == "eddarsglobal/nordoi"
            and item["parent_head"] == PARENT,
            "REVIEW_NOT_BOUND_TO_EXACT_R1_PARENT")
    role = item.get("role")
    require(type(role) is str and role in rows and
            item["source_git_blob_sha1"] == rows[role]["git_blob_sha1"],
            "UNPINNED_OR_UNKNOWN_REVIEW_ROLE")
    require(item["provenance"] == "LOCAL_SELF_REPORTED_ONLY"
            and item["requested_decision"] == "STAGE_ONLY"
            and item["independent_operator_attestation"] is None
            and item["independent_signer_signature"] is None,
            "FORGED_INDEPENDENT_CERTIFICATION_OR_PRIVILEGE")
    require(type(item["review_result_claim"]) is str
            and item["review_result_claim"] in REVIEW_RESULTS,
            "UNSUPPORTED_REVIEW_VERDICT")
    require(type(item["reviewer_claim"]) is str and
            re.fullmatch(r"[A-Za-z0-9_ .-]{3,80}", item["reviewer_claim"])
            is not None, "INVALID_UNVERIFIED_REVIEWER_LABEL")
    cmd = item.get("execution_command_record")
    require(type(cmd) is str and 6 <= len(cmd) <= 500 and
            "\n" not in cmd and "\r" not in cmd,
            "INVALID_UNVERIFIED_EXECUTION_COMMAND")
    path = item.get("evidence_log_relpath")
    require(type(path) is str and re.fullmatch(
        r"governance/review_logs/[A-Za-z0-9_.-]{3,100}\.log", path
    ) is not None, "UNSAFE_LOCAL_EVIDENCE_LOG_PATH")
    log = previous.raw(root, path)
    require(0 < len(log) <= MAX_LOG and
            type(item.get("evidence_log_sha256")) is str and
            re.fullmatch(r"[0-9a-f]{64}", item["evidence_log_sha256"])
            is not None and
            hashlib.sha256(log).hexdigest() == item["evidence_log_sha256"],
            "LOCAL_REVIEW_LOG_NOT_BYTE_MATCHED")
    require(item["evidence_log_relpath"] != relpath,
            "SELF_REFERENCING_REVIEW_DOCUMENT")
    # A same-admin local log can be fabricated: integrity != execution proof.
    return {
        "BUREAU001_R2": "STAGED_UNVERIFIED_LOCAL_REVIEW_LOG_ONLY",
        "ROLE": role, "LOG_SHA256": item["evidence_log_sha256"],
        "LOCAL_LOG_BYTE_INTEGRITY": "PASS",
        "REVIEWER_IDENTITY_AUTHENTICATED": False,
        "INDEPENDENT_EXECUTION_PROVEN": False,
        "SPECIALIST_CERTIFICATION": "PENDING",
        "NATIVE_ADMISSION_GATES": "12_UNMET",
        "R6G": "LOCKED_BY_SEQUENCE",
        "PRODUCTION": "NOT_AUTHORIZED",
        "WRITES": 0,
    }

def deny_promotion(which):
    return {
        "TRANSITION": "BLOCK",
        "REQUESTED": which,
        "REASON": "SELF_REPORTED_AGENT_OR_LOCAL_LOG_NOT_AN_INDEPENDENT_CERTIFICATION",
        "R6G": "LOCKED_BY_SEQUENCE",
        "NATIVE_ADMISSION_GATES": "12_UNMET",
        "PRODUCTION": "NOT_AUTHORIZED",
    }

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path,
                        default=Path(__file__).resolve().parents[1])
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--audit", action="store_true")
    action.add_argument("--check-submission", type=str)
    action.add_argument("--attempt-certify", choices=(
        "SPECIALIST_REVIEW", "BUREAU", "R6G", "NATIVE_ADMISSION", "PRODUCTION"))
    args = parser.parse_args(argv)
    try:
        if args.audit:
            result = audit(args.root)
        elif args.check_submission is not None:
            result = stage_submission(args.root, args.check_submission)
        else:
            gate(args.root)
            result = deny_promotion(args.attempt_certify)
            print(json.dumps(result, sort_keys=True))
            return 2
    except (Blocked, previous.Blocked, previous.guardian.Blocked,
            TypeError, AttributeError, KeyError, OSError,
            UnicodeError, ValueError) as e:
        result = {
            "BUREAU001_R2": "BLOCK",
            "REASON": str(e) if isinstance(
                e, (Blocked, previous.Blocked, previous.guardian.Blocked)
            ) else type(e).__name__,
            "R6G": "LOCKED_BY_SEQUENCE",
            "NATIVE_ADMISSION_GATES": "12_UNMET",
            "PRODUCTION": "NOT_AUTHORIZED",
        }
        print(json.dumps(result, sort_keys=True))
        return 2
    print(json.dumps(result, sort_keys=True))
    return 0

if __name__ == "__main__":
    sys.exit(main())
