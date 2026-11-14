#!/usr/bin/env python3
"""NORDOI G0.3 R6F / BUREAU-001 R1 — offline source-pinned evidence reconciliation.

This *coordinator* check is NOT an independent nine-agent review, verified
external administrative identity, signed release or native authority.
99 old Git blobs are pinned to an exact previously verified GitHub commit.
A SHA-1 Git blob identity is a local consistency check against that manifest,
not independent cryptographic authentication of a GitHub snapshot.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import string
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import roadmap_guardian as guardian

PARENT = "a81bf04576a9e9b3f43c2ef56a1a5eb19fd731ae"
SOURCE_R6E = "12e050d9b8698f4f902baf44390f1108a8e237ee"
MANIFEST = "governance/NORDOI_R6F_BUREAU001_R1_PINNED_SOURCES.json"
CHRONOLOGY = "governance/NORDOI_R6F_BUREAU001_R1_CHRONOLOGY.json"
SNAPSHOTS = (
    "research/G03_R6B_SIGNED_RESPONSE_NO_EXTERNAL_AUTHORITY.json",
    "research/G03_R6C_TWO_KEY_NO_INDEPENDENT_QUORUM.json",
    "research/G03_R6D_ADMIN_DOMAIN_NO_EXTERNAL_CUSTODY.json",
    "research/G03_R6E_OOB_ORIGIN_NO_EXTERNAL_AUTHORITY.json",
)
BUREAU_OLD = (
    "RoadMap.md", "Bureau.md", "scripts/roadmap_guardian.py",
    "governance/NORDOI_BUREAU_PLAN_v1.json",
    "governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json",
)
EXPECTED_STAGES = (
    ("R6B", 967, 173, 119, 43485688247, 43485688279),
    ("R6C", 991, 178, 120, 43495627026, 43495626711),
    ("R6D", 1015, 183, 121, 43563570247, 43563570243),
    ("R6E", 1039, 188, 122, 43588616293, 43588616176),
)
GATE_UNMET = guardian.GATE_UNMET

class Blocked(ValueError):
    pass

def check(yes, reason):
    if not yes:
        raise Blocked(reason)

def unique(pairs):
    d = {}
    for key, value in pairs:
        if key in d:
            raise Blocked("DUPLICATE_JSON_FIELD:" + key)
        d[key] = value
    return d

def nonfinite(value):
    raise Blocked("NONFINITE_JSON_FIELD")

def safe_file(root, rel):
    check(type(rel) is str and len(rel) <= 240 and
          rel and "\\" not in rel, "INVALID_RELATIVE_PATH")
    pp = PurePosixPath(rel)
    check(not pp.is_absolute() and ".." not in pp.parts and
          pp.parts and str(pp) == rel, "PATH_OUTSIDE_FROZEN_WORKTREE")
    current = root
    for part in pp.parts:
        current = current / part
        check(not current.is_symlink(), "SYMLINK_IN_SOURCE_PATH:" + rel)
    check(current.is_file(), "MISSING_FROZEN_SOURCE:" + rel)
    check(current.stat().st_size <= 512_000, "OVERSIZED_FROZEN_FILE:" + rel)
    return current

def raw(root, rel):
    return safe_file(root, rel).read_bytes()

def read_json(root, rel):
    try:
        return json.loads(raw(root, rel).decode("utf-8"),
                          object_pairs_hook=unique, parse_constant=nonfinite)
    except (UnicodeError, ValueError, TypeError) as exc:
        if isinstance(exc, Blocked):
            raise
        raise Blocked("INVALID_FROZEN_JSON:" + rel) from exc

def gitblob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode("ascii") +
                        b"\0" + data).hexdigest()

def expected_code_paths():
    result = set()
    for prefix, letters in (("r4", "yz"), ("r5", string.ascii_lowercase),
                            ("r6", "abcde")):
        for c in letters:
            key = "g03_" + prefix + c + "_"
            # Actual suffixes must be discovered from the frozen manifest.
            result.add(key)
    return result

def verify_manifest(root, manifest):
    check(manifest.get("protocol") ==
          "NORDOI-R6F-BUREAU001-R1-FROZEN-SOURCE-AUDIT-V1",
          "MANIFEST_WRONG_PROTOCOL")
    check(manifest.get("repo") == "eddarsglobal/nordoi" and
          manifest.get("parent_head") == PARENT and
          manifest.get("verified_basis") == "GITHUB_GIT_TREE_COMMIT" and
          manifest.get("source_immutability") ==
          "OFFLINE_PIN_CONSISTENCY_NOT_GITHUB_SIGNATURE_OR_EXTERNAL_AUTHORITY",
          "UNTRUSTED_SOURCE_BASELINE")
    records = manifest.get("pinned_files")
    check(type(records) is list and len(records) == 99
          and manifest.get("total_pinned_files") == 99, "EXPECTED_99_FROZEN_BLOBS")
    paths = []
    for record in records:
        check(type(record) is dict and set(record) == {
            "path", "git_blob_sha1", "bytes"
        }, "INVALID_PIN_RECORD_SCHEMA")
        p = record["path"]
        check(type(p) is str and type(record["git_blob_sha1"]) is str and
              re.fullmatch(r"[0-9a-f]{40}", record["git_blob_sha1"]) is not None and
              type(record["bytes"]) is int and 0 <= record["bytes"] <= 512000,
              "INVALID_SOURCE_PIN")
        content = raw(root, p)
        check(len(content) == record["bytes"], "SOURCE_LENGTH_CHANGED:" + p)
        check(gitblob(content) == record["git_blob_sha1"],
              "GIT_BLOB_MISMATCH:" + p)
        paths.append(p)
    check(len(paths) == len(set(paths)), "DUPLICATE_SOURCE_PINS")
    py_sources = [p for p in paths if re.fullmatch(
        r"research/g03_r(?:4[yz]|5[a-z]|6[a-e])_[a-z0-9_]+\.py", p)]
    py_tests = [p for p in paths if re.fullmatch(
        r"research/test_g03_r(?:4[yz]|5[a-z]|6[a-e])_[a-z0-9_]+\.py", p)]
    rust = [p for p in paths if re.fullmatch(
        r"tests/g03_r4[a-x]_[a-z0-9_]+\.rs", p)]
    check(len(py_sources) == len(py_tests) == 33 and len(rust) == 24,
          "R4A_R6E_CODE_OR_TEST_COVERAGE_INCOMPLETE")
    check({p.replace("research/g03_", "research/test_g03_", 1)
           for p in py_sources} == set(py_tests),
          "SOURCE_MISSING_CORRESPONDING_ADVERSARIAL_TEST")
    check(set(SNAPSHOTS).issubset(paths) and
          set(BUREAU_OLD).issubset(paths),
          "MISSING_BASELINE_GATES_OR_BUREAU_GUARDIAN")
    check(set(paths) == set(py_sources + py_tests + rust) |
          set(SNAPSHOTS) | set(BUREAU_OLD),
          "UNEXPECTED_UNCHECKED_SOURCE_GROUP")
    scope = manifest.get("scopes", {})
    check(scope == {
        "r4a_r4x_rust_native_tests": 24,
        "r4y_r4z_python_sources_and_tests": 4,
        "r5a_r5z_python_sources_and_tests": 52,
        "r6a_r6e_python_sources_and_tests": 10,
        "r6b_r6e_security_gate_evidence": 4,
        "r6f_roadmap_bureau_guardian_basis": 5,
    }, "FROZEN_SOURCE_SCOPE_MISLABELED")
    return {r["path"]: r for r in records}

def verify_history(root, chronology, pins):
    check(chronology.get("schema") ==
          "NORDOI-BUREAU001-R6F-R1-CHRONOLOGY-V1"
          and chronology.get("project") == "eddarsglobal/nordoi"
          and chronology.get("frozen_parent_commit") == PARENT
          and chronology.get("research_source_head") == SOURCE_R6E,
          "CHRONOLOGY_PARENT_NOT_VERIFIED")
    check(chronology.get("source_snapshots_status") == "PENDING_AT_COMMIT_TIME"
          and chronology.get("owner_evidence_source") ==
          "OWNER_SUPPLIED_CHAT_TERMINAL_OUTPUT_AFTER_ORIGINAL_CANDIDATE_CREATION"
          and chronology.get("owner_evidence_limitation") ==
          "OWNER_LOG_NOT_AN_INDEPENDENT_MACHINE_SIGNATURE_OR_EXTERNAL_CUSTODIAN_ATTESTATION"
          and chronology.get("record_kind") ==
          "APPEND_ONLY_RECONCILIATION_NOT_MUTATION_OF_ORIGINAL_SNAPSHOT",
          "OWNER_LOG_FALSIFIED_AS_CUSTODY_CERTIFICATION")
    stages = chronology.get("stages")
    check(type(stages) is list and len(stages) == 4,
          "CHRONOLOGY_NOT_FOUR_STAGES")
    baseline_gate_ids = guardian.GATE_IDS
    for record, (name, total, blobs, pr, did, gid), source in zip(
        stages, EXPECTED_STAGES, SNAPSHOTS
    ):
        snap = read_json(root, source)
        check(record.get("id") == name and record.get("source") == source
              and record.get("source_blob") == pins[source]["git_blob_sha1"]
              and snap.get("owner_mac_intel_replay") == record.get(
                  "source_owner_status_expected") == "PENDING",
              "ORIGINAL_SOURCE_SNAPSHOT_OR_TIMELINE_CHANGED:" + name)
        check(type(record.get("owner_regular_tests")) is int and
              record["owner_regular_tests"] == total and
              type(record.get("owner_source_git_blobs")) is int and
              record["owner_source_git_blobs"] == blobs and
              record.get("pr") == pr and
              record.get("dedicated_run_id") == did and
              record.get("general_run_id") == gid and
              record.get("observed_owner_terminal") ==
              "PASS_WITH_CONTROLS_RESEARCH_ONLY" and
              type(record.get("observed_owner_exit")) is int and
              record["observed_owner_exit"] == 0 and
              record.get("source_correction_authorized") is False and
              record.get("independent_custodian") == "NOT_PROVEN" and
              record.get("dedicated_ci_expected") == "4/4_SUCCESS" and
              record.get("general_ci_expected") == "5/5_SUCCESS",
              "UNVERIFIED_OR_ESCALATED_OWNER_EVIDENCE:" + name)
        gates = snap.get("required_gates")
        check(type(gates) is list and
              tuple(g.get("id") for g in gates) == baseline_gate_ids and
              all(g.get("status") == GATE_UNMET for g in gates) and
              snap.get("native_admission_gates_unmet") == 12 and
              snap.get("production") == "NOT_AUTHORIZED",
              "NATIVE_AUTHORITY_WRONG_IN_HISTORICAL_SOURCE:" + name)

    check(len(chronology.get("findings", [])) == 6 and
          {x["id"] for x in chronology["findings"]} ==
          {"R6F-R1-%03d" % n for n in range(1, 7)} and
          all(x.get("remaining") and x.get("repro") and
              x.get("resolution") for x in chronology["findings"]),
          "SOURCE_ANCHORED_FINDINGS_INCOMPLETE")
    r = chronology.get("r6f", {})
    check(r.get("formal_bureau_independent_reviews") == 0 and
          r.get("review_functions") == 9 and
          r.get("technical_ci_candidate") == "9/9_SUCCESS_AT_PARENT_R6F" and
          r.get("bureau_mission") == "BUREAU_001" and
          r.get("current_status") == "IN_PROGRESS" and
          r.get("r6g") == "LOCKED_BY_SEQUENCE" and
          r.get("independent_operator") == "NOT_ENROLLED" and
          r.get("owner_oob_root") == "NOT_AUTHENTICATED" and
          r.get("independent_antirollback") == "NOT_VERIFIED" and
          type(r.get("native_admission_gates_unmet")) is int and
          r.get("native_admission_gates_unmet") == 12 and
          r.get("production") == "NOT_AUTHORIZED",
          "FALSE_REVIEW_EXECUTION_OR_PREMATURE_R6G_UNBLOCK")

def audit(root):
    root = Path(root).resolve()
    guardian_result = guardian.audit(root)
    check(guardian_result.get("BUREAU_GUARDIAN") == "PASS_WITH_CONTROLS"
          and guardian_result.get("REVIEWS_SUBMITTED") == 0,
          "PARENT_GUARDIAN_NOT_DENY_ONLY")
    manifest = read_json(root, MANIFEST)
    chronology = read_json(root, CHRONOLOGY)
    pins = verify_manifest(root, manifest)
    verify_history(root, chronology, pins)
    return {
        "BUREAU001_R1": "PASS_WITH_CONTROLS_SOURCE_AUDIT_ONLY",
        "FROZEN_PARENT": PARENT,
        "FROZEN_SOURCE_BLOBS": "99/99_PASS",
        "R4A_R4X_NATIVE_RUST_TEST_FILES": 24,
        "R4Y_R6E_PYTHON_SOURCE_TEST_PAIRS": 33,
        "R6B_R6E_HISTORICAL_STATUS_RECONCILED": "4/4_APPEND_ONLY",
        "INITIAL_FINDINGS": "6_DOCUMENTED",
        "BUREAU_INDEPENDENT_REVIEWS_COMPLETED": 0,
        "ACTUAL_EXTERNAL_CUSTODIAN": "NOT_ENROLLED",
        "R6F": "IN_PROGRESS_NOT_CERTIFIED",
        "R6G": "LOCKED_BY_SEQUENCE",
        "NATIVE_ADMISSION_GATES": "12_UNMET",
        "PRODUCTION": "NOT_AUTHORIZED",
    }

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path,
                        default=Path(__file__).resolve().parents[1])
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--audit", action="store_true")
    group.add_argument("--attempt-unlock-r6g", action="store_true")
    args = parser.parse_args(argv)
    try:
        result = audit(args.root)
    except (Blocked, guardian.Blocked, OSError, TypeError, KeyError,
            AttributeError, UnicodeError) as exc:
        print(json.dumps({"BUREAU001_R1": "BLOCK",
                          "reason": str(exc) if isinstance(
                              exc, (Blocked, guardian.Blocked))
                          else type(exc).__name__}, sort_keys=True))
        return 2
    if args.attempt_unlock_r6g:
        print(json.dumps({"TRANSITION": "BLOCK",
                          "R6G": "LOCKED_BY_SEQUENCE",
                          "NATIVE_ADMISSION_GATES": "12_UNMET",
                          "PRODUCTION": "NOT_AUTHORIZED",
                          "REASON": "NO_INDEPENDENT_EXTERNAL_CUSTODY_OR_BUREAU_CERTIFICATION"},
                         sort_keys=True))
        return 2
    print(json.dumps(result, sort_keys=True))
    return 0

if __name__ == "__main__":
    sys.exit(main())
