#!/usr/bin/env python3
"""APP-010: independent read-only audit of real APP-009 native DAG execution.

No subprocess, no native engine launch, no writes, no network and no model.
The verifier independently checks the DAG topology, gate decisions, event
order, native receipt consistency and actual P2.7 published files. The plan
digest is supplied by a separately trusted caller. No signature or hostile
concurrent filesystem guarantee is claimed.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app006_readonly_audit"))
import audit_app006 as native_audit

REJECTED = native_audit.AuditRejected
require = native_audit.require
SCHEMA = "NORDOI_AGENT_DAG_AUDIT_V1"
PLAN_SCHEMA = "NORDOI_AGENT_DAG_V1"
EVENT_SCHEMA = "NORDOI_AGENT_DAG_EVENT_V1"
MAX_PLAN_BYTES = 32768
MAX_EVENTS_BYTES = 524288
MAX_NODES = 8
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
GATES = frozenset(("always", "all_committed", "all_approved",
                   "all_rejected", "any_approved", "any_rejected"))
BASE_FIELDS = frozenset(("schema", "step", "status", "gate",
                         "depends_on", "plan_sha256"))
COMMITTED_FIELDS = BASE_FIELDS | {"job", "decision", "native_receipt"}
SKIPPED_FIELDS = BASE_FIELDS | {"reason"}
ERROR_FIELDS = BASE_FIELDS | {"error_code", "publication"}
NATIVE_ERROR_FIELDS = ERROR_FIELDS | {"native_exit_code"}
AUDIT_ERROR_FIELDS = ERROR_FIELDS | {"error_detail"}


def decode_json(raw: bytes, code: str) -> object:
    try:
        return json.loads(raw.decode("utf-8", "strict"),
                          object_pairs_hook=native_audit.no_duplicate_keys,
                          parse_constant=native_audit.no_constants)
    except REJECTED:
        raise
    except (ValueError, UnicodeError, TypeError, OverflowError):
        raise REJECTED(code) from None


def plan_nodes(path: Path, expected_sha256: str) -> tuple[list[dict], str]:
    require(type(expected_sha256) is str and HEX64.fullmatch(expected_sha256) is not None,
            "INVALID_PLAN_PIN")
    raw = native_audit.bounded_regular_file(path, MAX_PLAN_BYTES,
                                           "PLAN_NOT_BOUNDED_REGULAR_FILE")
    require(bool(raw), "EMPTY_DAG_PLAN")
    actual = hashlib.sha256(raw).hexdigest()
    require(actual == expected_sha256, "PLAN_SHA256_PIN_MISMATCH")
    plan = decode_json(raw, "DAG_PLAN_INVALID_JSON")
    require(type(plan) is dict and set(plan) == {"schema", "nodes"}
            and plan["schema"] == PLAN_SCHEMA, "DAG_SCHEMA_INVALID")
    nodes = plan["nodes"]
    require(type(nodes) is list and 1 <= len(nodes) <= MAX_NODES,
            "DAG_NODE_COUNT_INVALID")
    for i, node in enumerate(nodes, 1):
        require(type(node) is dict and set(node) == {"key_code", "depends_on", "gate"},
                "DAG_NODE_FIELDS_INVALID")
        code, dep, gate = node["key_code"], node["depends_on"], node["gate"]
        require(type(code) is int and 0 <= code <= 0xffffffff,
                "DAG_KEY_CODE_INVALID")
        require(type(dep) is list and len(dep) <= MAX_NODES,
                "DAG_DEPENDENCY_LIST_INVALID")
        require(type(gate) is str and gate in GATES, "DAG_GATE_INVALID")
        prev = 0
        for ref in dep:
            require(type(ref) is int and 0 < ref < i, "DAG_REFERENCE_NOT_PRIOR")
            require(ref > prev, "DAG_REFERENCES_NOT_ASCENDING")
            prev = ref
        require((gate == "always") == (len(dep) == 0),
                "DAG_GATE_DEPENDENCIES_CONTRADICT")
    return nodes, actual


def event_records(path: Path) -> tuple[list[dict], bytes]:
    raw = native_audit.bounded_regular_file(path, MAX_EVENTS_BYTES,
                                           "EVENTS_NOT_BOUNDED_REGULAR_FILE")
    require(bool(raw) and raw.endswith(b"\n"), "EVENTS_NOT_NEWLINE_TERMINATED")
    lines = raw.split(b"\n")
    require(lines[-1] == b"", "EVENTS_NOT_NEWLINE_TERMINATED")
    lines.pop()
    require(1 <= len(lines) <= MAX_NODES, "EVENT_LINE_COUNT_INVALID")
    # Native Python stdout on Windows uses CRLF. Accept uniform LF or
    # uniform CRLF, but never normalize the original bytes used in SHA-256.
    if all(line.endswith(b"\r") for line in lines):
        lines = [line[:-1] for line in lines]
    records = []
    for line in lines:
        require(bool(line) and b"\r" not in line, "EVENT_LINE_INVALID")
        data = decode_json(line, "EVENT_INVALID_JSON")
        require(type(data) is dict, "EVENT_NOT_OBJECT")
        records.append(data)
    return records, raw


def gate_true(gate: str, dep: list[int], results: list[int | None]) -> bool:
    if gate == "always":
        return True
    vals = [results[ref - 1] for ref in dep]
    if gate == "all_committed":
        return not any(v is None for v in vals)
    if gate == "all_approved":
        return all(v is not None and v == 1 for v in vals)
    if gate == "all_rejected":
        return all(v is not None and v == 0 for v in vals)
    if gate == "any_approved":
        return any(v is not None and v == 1 for v in vals)
    if gate == "any_rejected":
        return any(v is not None and v == 0 for v in vals)
    raise AssertionError("validated DAG contained unknown condition")


def audit(root: Path, plan_path: Path, events_path: Path,
          expected_plan_sha256: str) -> tuple[dict, int]:
    nodes, plan_sha = plan_nodes(plan_path, expected_plan_sha256)
    records, raw = event_records(events_path)
    require(len(records) <= len(nodes), "EVENT_COUNT_EXCEEDS_PLAN")
    native_audit.real_directory(root, "BATCH_ROOT_NOT_REAL_DIRECTORY")

    prior: list[int | None] = []
    commits = []
    skipped = 0
    stopped = False
    for index, record in enumerate(records, 1):
        node = nodes[index - 1]
        require(record.get("schema") == EVENT_SCHEMA
                and type(record.get("step")) is int
                and record["step"] == index,
                "EVENT_SCHEMA_OR_STEP_INVALID")
        require(record.get("gate") == node["gate"]
                and record.get("depends_on") == node["depends_on"],
                "EVENT_DAG_BINDING_MISMATCH")
        require(record.get("plan_sha256") == plan_sha,
                "EVENT_PLAN_DIGEST_MISMATCH")
        expected_run = gate_true(node["gate"], node["depends_on"], prior)
        state = record.get("status")

        if state == "SKIPPED":
            require(set(record) == SKIPPED_FIELDS,
                    "SKIPPED_FIELDS_INVALID")
            require(not expected_run, "UNJUSTIFIED_SKIPPED_NODE")
            require(record["reason"] == "DEPENDENCY_GATE_FALSE",
                    "SKIPPED_REASON_INVALID")
            prior.append(None)
            skipped += 1
            continue

        require(expected_run, "EXECUTED_NODE_WHEN_GATE_FALSE")
        if state == "COMMITTED":
            require(set(record) == COMMITTED_FIELDS,
                    "COMMITTED_FIELDS_INVALID")
            require(record["job"] == f"job-{index:04d}",
                    "JOB_ID_NOT_BOUND_TO_DAG_STEP")
            decision = record["decision"]
            require(type(decision) is int and decision in (0, 1),
                    "DECISION_INVALID")
            native = record["native_receipt"]
            require(type(native) is dict
                    and native.get("key_code") == node["key_code"]
                    and type(native.get("key_code")) is int,
                    "NATIVE_KEY_CODE_MISMATCH")
            observed = native_audit.verify_native_record(native, index, root)
            require(observed["decision"] == decision,
                    "NATIVE_DECISION_CONTRADICTION")
            prior.append(decision)
            commits.append(observed)
            continue

        if state == "ERROR":
            require(index == len(records), "ERROR_NOT_TERMINAL")
            require(set(record) in (ERROR_FIELDS, NATIVE_ERROR_FIELDS,
                                   AUDIT_ERROR_FIELDS), "ERROR_FIELDS_INVALID")
            require(record["publication"] == "UNKNOWN",
                    "ERROR_FALSE_PUBLICATION_CLAIM")
            require(type(record["error_code"]) is str
                    and record["error_code"] in (
                        "JOB_DIRECTORY_CREATE_FAILED",
                        "NATIVE_EXECUTION_INCOMPLETE",
                        "NATIVE_OPERATION_FAILED",
                        "NATIVE_RESULT_NOT_VERIFIED"),
                    "ERROR_CODE_INVALID")
            require((set(record) == NATIVE_ERROR_FIELDS)
                    == (record["error_code"] == "NATIVE_OPERATION_FAILED"),
                    "NATIVE_ERROR_FIELDS_CONTRADICT")
            require((set(record) == AUDIT_ERROR_FIELDS)
                    == (record["error_code"] == "NATIVE_RESULT_NOT_VERIFIED"),
                    "AUDIT_ERROR_FIELDS_CONTRADICT")
            if set(record) == NATIVE_ERROR_FIELDS:
                require(type(record["native_exit_code"]) is int
                        and 1 <= record["native_exit_code"] <= 255,
                        "NATIVE_EXIT_CODE_INVALID")
            if set(record) == AUDIT_ERROR_FIELDS:
                require(type(record["error_detail"]) is str
                        and 0 < len(record["error_detail"]) <= 128,
                        "ERROR_DETAIL_INVALID")
            prior.append(None)
            stopped = True
            break

        raise REJECTED("UNKNOWN_EVENT_STATE")

    expected_dirs = {f"job-{i:04d}" for i, record in enumerate(records, 1)
                     if record["status"] == "COMMITTED"}
    if stopped:
        # The last attempted native job may have created a partial output:
        # it remains UNKNOWN, never silently certified as committed.
        optional_error = f"job-{len(records):04d}"
        expected_dirs.add(optional_error)
        existing = native_audit.listed_names(root, "ROOT_LISTING_FAILED")
        require(existing == expected_dirs or existing == expected_dirs - {optional_error},
                "ROOT_UNEXPECTED_JOB_OR_FILE")
        if optional_error in existing:
            native_audit.real_directory(root / optional_error,
                                        "ERROR_JOB_DIR_INVALID")
    else:
        require(native_audit.listed_names(root, "ROOT_LISTING_FAILED")
                == expected_dirs, "ROOT_UNEXPECTED_JOB_OR_FILE")

    status = "PASS" if not stopped and len(records) == len(nodes) else "INCONCLUSIVE"
    report = {
        "schema": SCHEMA,
        "status": status,
        "plan_sha256": plan_sha,
        "original_events_sha256": hashlib.sha256(raw).hexdigest(),
        "nodes_in_plan": len(nodes),
        "events_checked": len(records),
        "committed_jobs_verified": len(commits),
        "skipped_nodes_verified": skipped,
        "verified_jobs": commits,
        "attestation": "READ_ONLY_DAG_AND_REAL_FILE_CONSISTENCY_NOT_SIGNED",
        "p27_native_proof": "SHAPE_ONLY_NOT_CRYPTOGRAPHICALLY_RECOMPUTED",
        "external_pin_authentication": "NOT_PERFORMED",
        "crash_durability": "UNCLAIMED",
        "concurrent_host_adversary": "NOT_COVERED",
    }
    if stopped:
        report["terminal_error"] = records[-1]["error_code"]
    return report, 0 if status == "PASS" else 3


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dag-root", required=True)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--events", required=True)
    parser.add_argument("--expected-plan-sha256", required=True)
    args = parser.parse_args(argv)
    try:
        record, code = audit(Path(args.dag_root), Path(args.plan),
                             Path(args.events), args.expected_plan_sha256)
    except REJECTED as error:
        record, code = {"schema": SCHEMA, "status": "REJECTED",
                        "verified": False, "error_code": error.code,
                        "attestation": "NONE"}, 2
    print(json.dumps(record, ensure_ascii=True, sort_keys=True,
                     separators=(",", ":")))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
