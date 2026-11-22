#!/usr/bin/env python3
"""APP-011: fail-closed native DAG continuation after a verified clean prefix.

Resumes ONLY an incomplete, clean APP-009 event prefix. Before any effect:
- independently verifies the original graph, historical native receipts,
  historical published bytes, and exact caller-supplied SHA256 pins;
- rejects terminal ERROR, an unfinished job directory, unexpected files, or
  completed graphs, rather than guessing if a native operation committed.
Then runs ONLY missing DAG nodes through the genuine NORDOI Rust binary.
Never overwrites historical tasks or the original event file. New events
are emitted as a suffix on stdout, with the prefix's newline convention.
No power-loss durability, OS sandbox, signatures, concurrent-host defense,
cross-job rollback, or ability to recover ambiguous in-flight commits.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import stat
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app010_readonly_dag_audit"))
import audit_app010 as independent
sys.path.insert(0, str(HERE.parent / "app009_agent_dag"))
import run_app009 as producer

pinned = producer.pinned
native_audit = producer.audit
EVENT = "NORDOI_AGENT_DAG_EVENT_V1"
RESUME_ERROR = "NORDOI_AGENT_DAG_RESUME_ERROR_V1"


def require(yes: bool, code: str) -> None:
    if not yes:
        raise pinned.Denied(code)


def emit(record: dict, newline: bytes) -> None:
    encoded = json.dumps(record, ensure_ascii=True, sort_keys=True,
                         separators=(",", ":")).encode("utf-8") + newline
    sys.stdout.buffer.write(encoded)
    sys.stdout.buffer.flush()


def run(plan: Path, prefix: Path, root: Path, engine: Path, source: Path,
        expected_plan: str, expected_prefix: str,
        expected_engine: str, expected_source: str) -> int:
    # ALL preflight checks precede any write, including a new job directory.
    for digest in (expected_plan, expected_prefix, expected_engine, expected_source):
        require(type(digest) is str and pinned.PIN.fullmatch(digest) is not None,
                "INVALID_PIN_FORMAT")
    nodes, plan_hash = independent.plan_nodes(plan, expected_plan)
    original = native_audit.bounded_regular_file(prefix, independent.MAX_EVENTS_BYTES,
                                                "PREFIX_NOT_BOUNDED_REGULAR_FILE")
    require(hashlib.sha256(original).hexdigest() == expected_prefix,
            "PREFIX_SHA256_PIN_MISMATCH")
    events, actual_raw = independent.event_records(prefix)
    require(actual_raw == original, "PREFIX_CHANGED_DURING_READ")
    require(len(events) < len(nodes), "DAG_ALREADY_FINISHED")
    require(all(event.get("status") in ("COMMITTED", "SKIPPED")
                for event in events), "UNCERTAIN_OR_FAILED_PREFIX_DENIED")
    native_audit.real_directory(root, "GRANT_NOT_REAL_DIRECTORY")
    try:
        source_real = source.resolve(strict=True)
        grant_real = root.resolve(strict=True)
        require(grant_real != source_real and source_real not in grant_real.parents,
                "GRANT_OVERLAPS_SOURCE")
    except OSError:
        raise pinned.Denied("SOURCE_ROOT_RESOLUTION_FAILED") from None
    require(pinned.engine_sha256(engine) == expected_engine, "ENGINE_PIN_MISMATCH")
    require(pinned.source_tree_sha256(source) == expected_source,
            "SOURCE_PIN_MISMATCH")
    try:
        report, status = independent.audit(root, plan, prefix, expected_plan)
    except independent.REJECTED as error:
        raise pinned.Denied("HISTORY_" + error.code) from None
    require(status == 3 and report["status"] == "INCONCLUSIVE"
            and report["events_checked"] == len(events)
            and report["plan_sha256"] == plan_hash,
            "HISTORY_NOT_CLEAN_INCOMPLETE_PREFIX")

    # Recover previous decisions exclusively from the independently verified
    # historical *native* records. A skipped node is not a committed node.
    decisions: list[int | None] = [
        event["native_receipt"]["decision"] if event["status"] == "COMMITTED"
        else None for event in events
    ]
    newline = b"\r\n" if original.endswith(b"\r\n") else b"\n"
    for index in range(len(events) + 1, len(nodes) + 1):
        node = nodes[index - 1]
        gate, deps = node["gate"], node["depends_on"]
        # Detect modifications to the trusted historical checkpoint and plan.
        # After any prior effect we MUST NOT advertise "NOT_STARTED" on failure.
        try:
            require(hashlib.sha256(
                native_audit.bounded_regular_file(prefix, independent.MAX_EVENTS_BYTES,
                                                 "PREFIX_FILE_CHANGED")
            ).hexdigest() == expected_prefix, "PREFIX_CHANGED_DURING_RESUME")
            require(hashlib.sha256(
                native_audit.bounded_regular_file(plan, independent.MAX_PLAN_BYTES,
                                                 "PLAN_FILE_CHANGED")
            ).hexdigest() == expected_plan, "PLAN_CHANGED_DURING_RESUME")
        except pinned.Denied as error:
            emit(producer.make_event(index, "ERROR", gate, deps, plan_hash,
                                    error_code="TRUSTED_INPUT_CHANGED",
                                    publication="UNKNOWN"), newline)
            return 3
        if not independent.gate_true(gate, deps, decisions):
            decisions.append(None)
            emit(producer.make_event(index, "SKIPPED", gate, deps, plan_hash,
                                    reason="DEPENDENCY_GATE_FALSE"), newline)
            continue

        target = root / f"job-{index:04d}"
        # Atomic exclusive mkdir: never modify any earlier task.
        try:
            target.mkdir(mode=0o700)
        except OSError:
            emit(producer.make_event(index, "ERROR", gate, deps, plan_hash,
                                    error_code="JOB_DIRECTORY_CREATE_FAILED",
                                    publication="UNKNOWN"), newline)
            return 3
        request = json.dumps(
            {"schema": "NORDOI_AGENT_REQUEST_V1", "key_code": node["key_code"]},
            separators=(",", ":")).encode("utf-8") + b"\n"
        try:
            native = subprocess.run(
                [str(engine.resolve(strict=True)), "agent-request-json",
                 str(source.resolve(strict=True)), "app.main",
                 "--grant-output-dir", str(target)],
                input=request, capture_output=True,
                cwd=HERE.parents[2], timeout=120,
            )
        except (OSError, subprocess.TimeoutExpired):
            emit(producer.make_event(index, "ERROR", gate, deps, plan_hash,
                                    error_code="NATIVE_EXECUTION_INCOMPLETE",
                                    publication="UNKNOWN"), newline)
            return 3
        if native.returncode != 0:
            emit(producer.make_event(index, "ERROR", gate, deps, plan_hash,
                                    error_code="NATIVE_OPERATION_FAILED",
                                    native_exit_code=native.returncode,
                                    publication="UNKNOWN"), newline)
            return 3
        try:
            result = producer.previous.native_json(native.stdout)
            native_audit.verify_native_record(result, index, root)
            require(type(result["key_code"]) is int
                    and result["key_code"] == node["key_code"],
                    "NATIVE_KEY_CODE_CONTRADICTION")
            require(pinned.engine_sha256(engine) == expected_engine,
                    "ENGINE_CHANGED_DURING_RESUME")
            require(pinned.source_tree_sha256(source) == expected_source,
                    "SOURCE_CHANGED_DURING_RESUME")
        except (native_audit.AuditRejected, pinned.Denied) as error:
            emit(producer.make_event(index, "ERROR", gate, deps, plan_hash,
                                    error_code="NATIVE_RESULT_NOT_VERIFIED",
                                    error_detail=error.code,
                                    publication="UNKNOWN"), newline)
            return 3
        decisions.append(result["decision"])
        emit(producer.make_event(index, "COMMITTED", gate, deps, plan_hash,
                                job=f"job-{index:04d}",
                                decision=result["decision"],
                                native_receipt=result), newline)
    return 0


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--plan", required=True)
    p.add_argument("--prefix-events", required=True)
    p.add_argument("--grant-output-dir", required=True)
    p.add_argument("--engine", required=True)
    p.add_argument("--source-root", required=True)
    p.add_argument("--expected-plan-sha256", required=True)
    p.add_argument("--expected-prefix-sha256", required=True)
    p.add_argument("--expected-engine-sha256", required=True)
    p.add_argument("--expected-source-sha256", required=True)
    args = p.parse_args(argv)
    try:
        return run(Path(args.plan), Path(args.prefix_events),
                   Path(args.grant_output_dir), Path(args.engine),
                   Path(args.source_root), args.expected_plan_sha256,
                   args.expected_prefix_sha256, args.expected_engine_sha256,
                   args.expected_source_sha256)
    except (pinned.Denied, native_audit.AuditRejected) as e:
        # APP-010 and APP-006 deliberately use AuditRejected for untrusted
        # plan, JSONL and file input. Reject in the SAME machine contract,
        # rather than leaking a traceback or claiming an effect occurred.
        emit({"schema": RESUME_ERROR, "status": "REJECTED",
              "error_code": e.code, "publication": "NOT_STARTED"}, b"\n")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
