#!/usr/bin/env python3
"""APP-009: bounded dependency-DAG workflows using the actual NORDOI Rust engine.

This is a sequential topological scheduler, NOT parallelism or a transaction.
Agent-provided JSON is data only: source, engine, SHA-256 pins and FileWrite
grant arrive as explicit CLI arguments. Every node and edge is validated
before any output is created. The native Rust APP-004 request decides each
business result and publishes each P2.7 bundle. APP-006 checks actual files.
No inference, external signer, operating-system sandbox or global rollback.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app008_conditional_workflow"))
import run_app008 as previous

pinned = previous.pinned
audit = previous.audit

SCHEMA = "NORDOI_AGENT_DAG_V1"
EVENT = "NORDOI_AGENT_DAG_EVENT_V1"
MAX_PLAN_BYTES = 32768
MAX_NODES = 8
GATES = frozenset((
    "always", "all_committed", "all_approved", "all_rejected",
    "any_approved", "any_rejected",
))


def require(condition: bool, code: str) -> None:
    if not condition:
        raise pinned.Denied(code)


def load_plan(path: Path) -> tuple[list[dict], str]:
    raw = pinned.regular_bytes(path, MAX_PLAN_BYTES, "DAG_PLAN_NOT_BOUNDED_REGULAR_FILE")
    require(bool(raw), "EMPTY_DAG_PLAN")
    try:
        plan = json.loads(raw.decode("utf-8", "strict"),
                          object_pairs_hook=pinned.no_duplicates,
                          parse_constant=pinned.reject_constant)
    except pinned.Denied:
        raise
    except (UnicodeError, ValueError, TypeError, OverflowError):
        raise pinned.Denied("DAG_PLAN_INVALID_JSON") from None
    require(type(plan) is dict and set(plan) == {"schema", "nodes"}
            and plan["schema"] == SCHEMA, "DAG_SCHEMA_INVALID")
    nodes = plan["nodes"]
    require(type(nodes) is list and 1 <= len(nodes) <= MAX_NODES,
            "DAG_NODE_COUNT_INVALID")

    for index, node in enumerate(nodes, 1):
        require(type(node) is dict and set(node) == {"key_code", "depends_on", "gate"},
                "DAG_NODE_FIELDS_INVALID")
        code = node["key_code"]
        require(type(code) is int and 0 <= code <= 0xffffffff,
                "DAG_KEY_CODE_INVALID")
        gate = node["gate"]
        require(type(gate) is str and gate in GATES, "DAG_GATE_INVALID")
        dependencies = node["depends_on"]
        require(type(dependencies) is list and len(dependencies) <= MAX_NODES,
                "DAG_DEPENDENCY_LIST_INVALID")
        previous_ref = 0
        for ref in dependencies:
            require(type(ref) is int and 1 <= ref < index, "DAG_REFERENCE_NOT_PRIOR")
            require(ref > previous_ref, "DAG_REFERENCES_NOT_STRICTLY_ASCENDING")
            previous_ref = ref
        require((gate == "always") == (len(dependencies) == 0),
                "DAG_GATE_DEPENDENCIES_CONTRADICT")
    return nodes, hashlib.sha256(raw).hexdigest()


def should_execute(gate: str, dependencies: list[int], decisions: list[int | None]) -> bool:
    if gate == "always":
        return True
    values = [decisions[i - 1] for i in dependencies]
    if gate == "all_committed":
        return all(value is not None for value in values)
    if gate == "all_approved":
        return all(value == 1 for value in values)
    if gate == "all_rejected":
        return all(value == 0 for value in values)
    if gate == "any_approved":
        return any(value == 1 for value in values)
    if gate == "any_rejected":
        return any(value == 0 for value in values)
    raise AssertionError("validated DAG must never carry an unknown gate")


def make_event(index: int, status: str, gate: str | None,
               dependencies: list[int], plan_sha256: str | None,
               **extras) -> dict:
    return {"schema": EVENT, "step": index, "status": status,
            "gate": gate, "depends_on": dependencies,
            "plan_sha256": plan_sha256, **extras}


def emit(record: dict) -> None:
    print(json.dumps(record, ensure_ascii=True, sort_keys=True,
                     separators=(",", ":")), flush=True)


def run(plan_file: Path, engine: Path, source: Path, grant: Path,
        expected_engine: str, expected_source: str) -> int:
    # All checks and complete graph validation run BEFORE the first effect.
    for digest in (expected_engine, expected_source):
        require(pinned.PIN.fullmatch(digest) is not None, "INVALID_PIN_FORMAT")
    nodes, plan_hash = load_plan(plan_file)
    require(pinned.engine_sha256(engine) == expected_engine, "ENGINE_PIN_MISMATCH")
    require(pinned.source_tree_sha256(source) == expected_source,
            "SOURCE_PIN_MISMATCH")
    previous.validate_grant(grant, source)

    decisions: list[int | None] = []
    for index, node in enumerate(nodes, 1):
        gate, deps = node["gate"], node["depends_on"]
        if not should_execute(gate, deps, decisions):
            decisions.append(None)
            emit(make_event(index, "SKIPPED", gate, deps, plan_hash,
                            reason="DEPENDENCY_GATE_FALSE"))
            continue

        # Paths are fixed internally; data can never select an output path.
        target = grant / f"job-{index:04d}"
        try:
            target.mkdir(mode=0o700)
        except OSError:
            emit(make_event(index, "ERROR", gate, deps, plan_hash,
                            error_code="JOB_DIRECTORY_CREATE_FAILED",
                            publication="UNKNOWN"))
            return 3

        request = json.dumps(
            {"schema": "NORDOI_AGENT_REQUEST_V1", "key_code": node["key_code"]},
            separators=(",", ":")).encode("utf-8") + b"\n"
        try:
            process = subprocess.run(
                [str(engine.resolve(strict=True)), "agent-request-json",
                 str(source.resolve(strict=True)), "app.main",
                 "--grant-output-dir", str(target)],
                input=request, capture_output=True,
                cwd=HERE.parents[2], timeout=120,
            )
        except (OSError, subprocess.TimeoutExpired):
            emit(make_event(index, "ERROR", gate, deps, plan_hash,
                            error_code="NATIVE_EXECUTION_INCOMPLETE",
                            publication="UNKNOWN"))
            return 3

        if process.returncode != 0:
            emit(make_event(index, "ERROR", gate, deps, plan_hash,
                            error_code="NATIVE_OPERATION_FAILED",
                            native_exit_code=process.returncode,
                            publication="UNKNOWN"))
            return 3

        try:
            native = previous.native_json(process.stdout)
            audit.verify_native_record(native, index, grant)
            require(native["key_code"] == node["key_code"],
                    "NATIVE_KEY_CODE_CONTRADICTION")
            require(pinned.engine_sha256(engine) == expected_engine,
                    "ENGINE_CHANGED_DURING_WORKFLOW")
            require(pinned.source_tree_sha256(source) == expected_source,
                    "SOURCE_CHANGED_DURING_WORKFLOW")
        except (pinned.Denied, audit.AuditRejected) as error:
            emit(make_event(index, "ERROR", gate, deps, plan_hash,
                            error_code="NATIVE_RESULT_NOT_VERIFIED",
                            error_detail=error.code, publication="UNKNOWN"))
            return 3

        decisions.append(native["decision"])
        emit(make_event(index, "COMMITTED", gate, deps, plan_hash,
                        job=f"job-{index:04d}",
                        decision=native["decision"], native_receipt=native))
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--engine", required=True)
    parser.add_argument("--source-root", required=True)
    parser.add_argument("--grant-output-dir", required=True)
    parser.add_argument("--expected-engine-sha256", required=True)
    parser.add_argument("--expected-source-sha256", required=True)
    args = parser.parse_args(argv)
    try:
        return run(Path(args.plan), Path(args.engine), Path(args.source_root),
                   Path(args.grant_output_dir), args.expected_engine_sha256,
                   args.expected_source_sha256)
    except pinned.Denied as err:
        emit(make_event(0, "REJECTED", None, [], None,
                        error_code=err.code, publication="NOT_STARTED"))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
