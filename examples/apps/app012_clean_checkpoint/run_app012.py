#!/usr/bin/env python3
"""APP-012: deliberately pause genuine native NORDOI DAGs at a clean boundary.

Executes a bounded PREFIX of an agent DAG with the authentic Rust engine,
emitting native receipts directly as APP-009 JSONL events. No full DAG run
followed by deleting outputs, no fake native receipts. A successful return
means the exact requested number of DAG events has been written to stdout,
each native COMMITTED event backed by real verified P2.7 files. The prefix is
incomplete by construction, suitable for APP-010 inspection and APP-011
verified continuation, once separately persisted and pinned by the caller.

The application is NOT a durable journal, crash-recovery service, OS sandbox,
external signature authority, atomic multi-job transaction or host-race guard.
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
sys.path.insert(0, str(HERE.parent / "app009_agent_dag"))
import run_app009 as producer
sys.path.insert(0, str(HERE.parent / "app010_readonly_dag_audit"))
import audit_app010 as independent

pinned = producer.pinned
native_audit = producer.audit
SCHEMA = "NORDOI_AGENT_DAG_CHECKPOINT_ERROR_V1"


def require(value: bool, code: str) -> None:
    if not value:
        raise pinned.Denied(code)


def output(record: dict) -> None:
    # Binary stdout is exact LF on all four OS. The caller can pin the exact
    # returned bytes without platform-dependent text-layer translation.
    line = json.dumps(record, ensure_ascii=True, sort_keys=True,
                      separators=(",", ":")).encode("utf-8") + b"\n"
    sys.stdout.buffer.write(line)
    sys.stdout.buffer.flush()


def error_event(index: int, gate: str, deps: list[int], digest: str,
                code: str, **other) -> None:
    output(producer.make_event(index, "ERROR", gate, deps, digest,
                               error_code=code, publication="UNKNOWN", **other))


def execute(plan: Path, engine: Path, source: Path, root: Path,
            plan_pin: str, engine_pin: str, source_pin: str,
            stop_after: int) -> int:
    # Exact full-plan validation before any side effect.
    for value in (plan_pin, engine_pin, source_pin):
        require(type(value) is str and pinned.PIN.fullmatch(value) is not None,
                "INVALID_PIN_FORMAT")
    nodes, digest = independent.plan_nodes(plan, plan_pin)
    require(type(stop_after) is int and 1 <= stop_after < len(nodes),
            "CHECKPOINT_MUST_BE_NONEMPTY_PROPER_PREFIX")
    require(pinned.engine_sha256(engine) == engine_pin,
            "ENGINE_PIN_MISMATCH")
    require(pinned.source_tree_sha256(source) == source_pin,
            "SOURCE_PIN_MISMATCH")
    # Real existing, empty FileWrite grant, enforced outside hostile plan.
    producer.previous.validate_grant(root, source)

    decisions: list[int | None] = []
    for index, node in enumerate(nodes[:stop_after], 1):
        gate, deps = node["gate"], node["depends_on"]
        # Guard against trust-input replacement before each successive task.
        try:
            require(hashlib.sha256(native_audit.bounded_regular_file(
                plan, independent.MAX_PLAN_BYTES, "PLAN_FILE_CHANGED")
            ).hexdigest() == plan_pin, "PLAN_CHANGED_DURING_CHECKPOINT")
            require(pinned.engine_sha256(engine) == engine_pin,
                    "ENGINE_CHANGED_DURING_CHECKPOINT")
            require(pinned.source_tree_sha256(source) == source_pin,
                    "SOURCE_CHANGED_DURING_CHECKPOINT")
        except (native_audit.AuditRejected, pinned.Denied):
            error_event(index, gate, deps, digest, "TRUSTED_INPUT_CHANGED")
            return 3

        if not independent.gate_true(gate, deps, decisions):
            decisions.append(None)
            output(producer.make_event(index, "SKIPPED", gate, deps, digest,
                                       reason="DEPENDENCY_GATE_FALSE"))
            continue

        target = root / f"job-{index:04d}"
        try:
            target.mkdir(mode=0o700)
        except OSError:
            error_event(index, gate, deps, digest, "JOB_DIRECTORY_CREATE_FAILED")
            return 3

        request = json.dumps({
            "schema": "NORDOI_AGENT_REQUEST_V1",
            "key_code": node["key_code"],
        }, separators=(",", ":")).encode("utf-8") + b"\n"

        try:
            process = subprocess.run(
                [str(engine.resolve(strict=True)), "agent-request-json",
                 str(source.resolve(strict=True)), "app.main",
                 "--grant-output-dir", str(target)],
                input=request, capture_output=True, cwd=HERE.parents[2],
                timeout=120,
            )
        except (OSError, subprocess.TimeoutExpired):
            error_event(index, gate, deps, digest, "NATIVE_EXECUTION_INCOMPLETE")
            return 3

        if process.returncode != 0:
            error_event(index, gate, deps, digest, "NATIVE_OPERATION_FAILED",
                        native_exit_code=process.returncode)
            return 3

        try:
            native = producer.previous.native_json(process.stdout)
            native_audit.verify_native_record(native, index, root)
            require(type(native.get("key_code")) is int
                    and native["key_code"] == node["key_code"],
                    "NATIVE_KEY_CODE_CONTRADICTION")
            require(pinned.engine_sha256(engine) == engine_pin,
                    "ENGINE_CHANGED_DURING_CHECKPOINT")
            require(pinned.source_tree_sha256(source) == source_pin,
                    "SOURCE_CHANGED_DURING_CHECKPOINT")
        except (native_audit.AuditRejected, pinned.Denied) as err:
            error_event(index, gate, deps, digest, "NATIVE_RESULT_NOT_VERIFIED",
                        error_detail=err.code)
            return 3

        decisions.append(native["decision"])
        output(producer.make_event(index, "COMMITTED", gate, deps, digest,
                                   job=f"job-{index:04d}",
                                   decision=native["decision"],
                                   native_receipt=native))
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--engine", required=True)
    parser.add_argument("--source-root", required=True)
    parser.add_argument("--grant-output-dir", required=True)
    parser.add_argument("--expected-plan-sha256", required=True)
    parser.add_argument("--expected-engine-sha256", required=True)
    parser.add_argument("--expected-source-sha256", required=True)
    parser.add_argument("--stop-after", type=int, required=True)
    a = parser.parse_args(argv)
    try:
        return execute(Path(a.plan), Path(a.engine), Path(a.source_root),
                       Path(a.grant_output_dir), a.expected_plan_sha256,
                       a.expected_engine_sha256, a.expected_source_sha256,
                       a.stop_after)
    except (pinned.Denied, native_audit.AuditRejected) as err:
        output({"schema": SCHEMA, "status": "REJECTED",
                "error_code": err.code, "publication": "NOT_STARTED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
