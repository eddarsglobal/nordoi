#!/usr/bin/env python3
"""APP-008: bounded conditional agent workflow backed by REAL NORDOI Rust calls.

AI agents supply bounded *data* only. The source, native executable, grant,
and external SHA-256 pins are explicit CLI parameters outside that data.
All steps are validated before creating output. Conditions inspect only the
previous actually committed native decision, never recompute business rules.
Each executed step invokes the true Rust APP-004/R2 request protocol.
No network, human test, signature, rollback, or production authority.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app007_pinned_replay"))
sys.path.insert(0, str(HERE.parent / "app006_readonly_audit"))
import verify_app007 as pinned
import audit_app006 as audit

SCHEMA = "NORDOI_AGENT_WORKFLOW_V1"
EVENT = "NORDOI_AGENT_WORKFLOW_EVENT_V1"
WHEN = frozenset(("always", "if_previous_approved", "if_previous_rejected"))
MAX_PLAN_BYTES = 32768
MAX_STEPS = 8


def required(condition: bool, code: str) -> None:
    if not condition:
        raise pinned.Denied(code)


def load_steps(plan_path: Path) -> list[dict]:
    raw = pinned.regular_bytes(plan_path, MAX_PLAN_BYTES, "PLAN_NOT_BOUNDED_REGULAR_FILE")
    required(bool(raw), "EMPTY_WORKFLOW_PLAN")
    try:
        obj = json.loads(raw.decode("utf-8", "strict"),
                         object_pairs_hook=pinned.no_duplicates,
                         parse_constant=pinned.reject_constant)
    except pinned.Denied:
        raise
    except (UnicodeError, ValueError, TypeError, OverflowError):
        raise pinned.Denied("INVALID_WORKFLOW_JSON") from None
    required(type(obj) is dict and set(obj) == {"schema", "steps"}
             and obj["schema"] == SCHEMA, "INVALID_WORKFLOW_SCHEMA")
    steps = obj["steps"]
    required(type(steps) is list and 1 <= len(steps) <= MAX_STEPS,
             "INVALID_WORKFLOW_STEP_COUNT")
    for i, step in enumerate(steps):
        required(type(step) is dict and set(step) == {"key_code", "when"},
                 "INVALID_WORKFLOW_STEP_FIELDS")
        code = step["key_code"]
        required(type(code) is int and 0 <= code <= 0xffffffff,
                 "INVALID_WORKFLOW_KEY_CODE")
        required(type(step["when"]) is str and step["when"] in WHEN,
                 "INVALID_WORKFLOW_CONDITION")
        if i == 0:
            required(step["when"] == "always", "FIRST_WORKFLOW_STEP_MUST_RUN")
    return steps


def validate_grant(root: Path, source: Path) -> None:
    try:
        mode = root.lstat().st_mode
        required(stat.S_ISDIR(mode) and not stat.S_ISLNK(mode),
                 "EXPLICIT_GRANT_NOT_REAL_DIRECTORY")
        required(not list(root.iterdir()), "EXPLICIT_GRANT_NOT_EMPTY")
        # Never write a job underneath the source tree containing pinned .noi.
        source_real = source.resolve(strict=True)
        grant_real = root.resolve(strict=True)
        required(grant_real != source_real
                 and source_real not in grant_real.parents,
                 "GRANT_OVERLAPS_SOURCE")
    except OSError:
        raise pinned.Denied("EXPLICIT_GRANT_NOT_REAL_DIRECTORY") from None


def native_json(raw: bytes) -> dict:
    required(0 < len(raw) <= audit.MAX_RECEIPTS_BYTES and raw.endswith(b"\n"),
             "NATIVE_RESPONSE_BOUNDS")
    required(raw.count(b"\n") == 1, "NATIVE_RESPONSE_LINE_COUNT")
    try:
        obj = json.loads(raw.decode("utf-8", "strict"),
                         object_pairs_hook=pinned.no_duplicates,
                         parse_constant=pinned.reject_constant)
    except (UnicodeError, ValueError, TypeError, OverflowError):
        raise pinned.Denied("NATIVE_RESPONSE_INVALID_JSON") from None
    required(type(obj) is dict, "NATIVE_RESPONSE_INVALID_JSON")
    return obj


def event(step: int, status: str, when: str | None, **other) -> dict:
    return {"schema": EVENT, "step": step, "status": status, "when": when, **other}


def emit(record: dict) -> None:
    print(json.dumps(record, sort_keys=True, separators=(",", ":"),
                     ensure_ascii=True), flush=True)


def run(plan_path: Path, engine: Path, source: Path, output: Path,
        expected_engine: str, expected_source: str) -> int:
    # ALL preflight checks and ALL requests precede the first effect.
    for value in (expected_engine, expected_source):
        required(pinned.PIN.fullmatch(value) is not None, "INVALID_PIN_FORMAT")
    steps = load_steps(plan_path)
    required(pinned.engine_sha256(engine) == expected_engine, "ENGINE_PIN_MISMATCH")
    required(pinned.source_tree_sha256(source) == expected_source,
             "SOURCE_PIN_MISMATCH")
    validate_grant(output, source)

    previous_decision: int | None = None
    committed = 0
    for i, step in enumerate(steps, 1):
        condition = step["when"]
        should_run = (condition == "always"
                      or (previous_decision == 1 and condition == "if_previous_approved")
                      or (previous_decision == 0 and condition == "if_previous_rejected"))
        if not should_run:
            emit(event(i, "SKIPPED", condition, reason="CONDITION_NOT_MET"))
            previous_decision = None  # no stale decision may authorize a later branch
            continue

        destination = output / f"job-{i:04d}"
        try:
            destination.mkdir(mode=0o700)  # exclusive; no overwrite
        except OSError:
            emit(event(i, "ERROR", condition, error_code="JOB_DIRECTORY_CREATE_FAILED",
                       publication="UNKNOWN"))
            return 3
        request = json.dumps(
            {"schema": "NORDOI_AGENT_REQUEST_V1", "key_code": step["key_code"]},
            separators=(",", ":")).encode("utf-8") + b"\n"
        try:
            proc = subprocess.run(
                [str(engine.resolve(strict=True)), "agent-request-json",
                 str(source.resolve(strict=True)), "app.main",
                 "--grant-output-dir", str(destination)],
                input=request, cwd=HERE.parents[2], capture_output=True,
                timeout=120,
            )
        except (OSError, subprocess.TimeoutExpired):
            emit(event(i, "ERROR", condition, error_code="NATIVE_EXECUTION_INCOMPLETE",
                       publication="UNKNOWN"))
            return 3
        if proc.returncode != 0:
            emit(event(i, "ERROR", condition, error_code="NATIVE_OPERATION_FAILED",
                       native_exit_code=proc.returncode, publication="UNKNOWN"))
            return 3

        try:
            native = native_json(proc.stdout)
            audit.verify_native_record(native, i, output)
            required(native["key_code"] == step["key_code"],
                     "NATIVE_KEY_CODE_CONTRADICTION")
            required(pinned.engine_sha256(engine) == expected_engine,
                     "ENGINE_CHANGED_DURING_WORKFLOW")
            required(pinned.source_tree_sha256(source) == expected_source,
                     "SOURCE_CHANGED_DURING_WORKFLOW")
        except (audit.AuditRejected, pinned.Denied) as exc:
            emit(event(i, "ERROR", condition, error_code="NATIVE_RESULT_NOT_VERIFIED",
                       error_detail=exc.code, publication="UNKNOWN"))
            return 3
        previous_decision = native["decision"]
        committed += 1
        emit(event(i, "COMMITTED", condition, job=f"job-{i:04d}",
                   decision=previous_decision, native_receipt=native))
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--engine", required=True)
    parser.add_argument("--source-root", required=True)
    parser.add_argument("--grant-output-dir", required=True)
    parser.add_argument("--expected-engine-sha256", required=True)
    parser.add_argument("--expected-source-sha256", required=True)
    a = parser.parse_args(argv)
    try:
        return run(Path(a.plan), Path(a.engine), Path(a.source_root),
                   Path(a.grant_output_dir), a.expected_engine_sha256,
                   a.expected_source_sha256)
    except pinned.Denied as err:
        emit(event(0, "REJECTED", None, error_code=err.code,
                   publication="NOT_STARTED"))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
