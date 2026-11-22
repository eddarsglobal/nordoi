#!/usr/bin/env python3
"""APP-015: independently replay historical portable-DAG jobs with pinned Rust.

Recomputes every COMMITTED historical job using the genuine, independently
pinned NORDOI native binary and .noi source tree in a fresh private directory;
compares FULL native receipts and the exact P2.7 file bytes with the portable
APP-014 original. SKIPPED nodes are not executed; APP-010 verifies their
actual gate conditions. The package and source are rechecked after replay.

verify: no writes to user data; only disposable replay files.
restore: replay first, then delegate exclusive verified restore to APP-014.
resume: replay first, then require an independently audited restored root and
delegate missing tasks to APP-011 through APP-014.

Neither execution isolation from a hostile engine, an independent P2.7 crypto
implementation, externally authenticated SHA pins, signatures, durability,
nor safe recovery of unknown in-flight effects is claimed.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE.parent / "app014_portable_checkpoint"))
import portable_app014 as portable
sys.path.insert(0, str(HERE.parent / "app009_agent_dag"))
import run_app009 as producer

pinned = portable.pinned
native_audit = portable.audit
independent = portable.independent

SCHEMA = "NORDOI_AGENT_PORTABLE_NATIVE_REPLAY_V1"


class Reject(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


class Uncertain(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


def require(condition: bool, code: str) -> None:
    if not condition:
        raise Reject(code)


def emit(obj: dict) -> None:
    sys.stdout.buffer.write(json.dumps(obj, sort_keys=True, ensure_ascii=True,
                                      separators=(",", ":")).encode("utf-8") + b"\n")
    sys.stdout.buffer.flush()


def replay(portable_dir: Path, expected_portable: str,
           engine: Path, source: Path, expected_engine: str,
           expected_source: str) -> dict:
    # Full independent APP-014/013/010/006 package validation BEFORE any
    # engine execution or writes into the caller's destination.
    manifest, _, events = portable.verify(portable_dir, expected_portable)
    for pin in (expected_engine, expected_source):
        require(type(pin) is str and pinned.PIN.fullmatch(pin) is not None,
                "ENGINE_SOURCE_PIN_INVALID")
    require(pinned.engine_sha256(engine) == expected_engine,
            "ENGINE_PIN_MISMATCH")
    require(pinned.source_tree_sha256(source) == expected_source,
            "SOURCE_PIN_MISMATCH")

    plan = portable_dir / "handoff" / "plan.json"
    nodes, actual_plan = independent.plan_nodes(plan, manifest["plan_sha256"])
    require(actual_plan == manifest["plan_sha256"], "DAG_PLAN_CHANGED")
    require(0 < len(events) < len(nodes), "NOT_INCOMPLETE_CHECKPOINT")

    file_count = 0
    replayed = 0
    try:
        with tempfile.TemporaryDirectory(prefix="nordoi_app015_private_native_") as d:
            scratch = Path(d) / "native-replayed"
            scratch.mkdir(mode=0o700)
            for index, (node, event) in enumerate(
                    zip(nodes[:len(events)], events, strict=True), 1):
                if event["status"] == "SKIPPED":
                    # APP-010 already recomputed the gate and proved zero
                    # original native files for this skipped node.
                    continue

                require(event["status"] == "COMMITTED",
                        "NONCOMMITTED_HISTORICAL_EVENT")
                target = scratch / f"job-{index:04d}"
                target.mkdir(mode=0o700)
                request = json.dumps({
                    "schema": "NORDOI_AGENT_REQUEST_V1",
                    "key_code": node["key_code"],
                }, separators=(",", ":")).encode("utf-8") + b"\n"
                try:
                    proc = subprocess.run(
                        [str(engine.resolve(strict=True)), "agent-request-json",
                         str(source.resolve(strict=True)), "app.main",
                         "--grant-output-dir", str(target)],
                        input=request, capture_output=True, cwd=REPO, timeout=120,
                    )
                except (OSError, subprocess.TimeoutExpired):
                    raise Uncertain("INDEPENDENT_NATIVE_REPLAY_INCOMPLETE") from None
                if proc.returncode != 0:
                    raise Uncertain("INDEPENDENT_NATIVE_REPLAY_FAILED")
                try:
                    observed = producer.previous.native_json(proc.stdout)
                    native_audit.verify_native_record(observed, index, scratch)
                except (native_audit.AuditRejected, pinned.Denied):
                    raise Reject("INDEPENDENT_NATIVE_REPLAY_INVALID_RECEIPT") from None

                # This comparison includes the native receipt's P2.7 proof
                # fields and rule witness, not merely key and decision.
                require(observed == event["native_receipt"],
                        "INDEPENDENT_NATIVE_RECEIPT_MISMATCH")
                for name in sorted(native_audit.FILE_NAMES):
                    replayed_bytes = native_audit.bounded_regular_file(
                        scratch / f"job-{index:04d}" / "quality-decision" / name,
                        native_audit.MAX_FILES_BYTES, "REPLAY_FILE_NOT_REGULAR")
                    original_bytes = native_audit.bounded_regular_file(
                        portable_dir / "native" / f"job-{index:04d}"
                        / "quality-decision" / name,
                        native_audit.MAX_FILES_BYTES, "ORIGINAL_FILE_NOT_REGULAR")
                    require(replayed_bytes == original_bytes,
                            "INDEPENDENT_NATIVE_FILE_BYTES_MISMATCH")
                    file_count += 1
                replayed += 1

            # No extra/hidden jobs in the replay directory, and skipped nodes
            # never receive native files.
            expected = {
                f"job-{i:04d}" for i, event in enumerate(events, 1)
                if event["status"] == "COMMITTED"
            }
            require(native_audit.listed_names(scratch, "REPLAY_LISTING_DENIED")
                    == expected, "REPLAY_UNEXPECTED_JOB_DIRECTORY")
    except OSError:
        raise Uncertain("PRIVATE_REPLAY_DIRECTORY_UNAVAILABLE") from None

    # Fail closed if trust inputs changed while executing native jobs.
    require(pinned.engine_sha256(engine) == expected_engine,
            "ENGINE_CHANGED_DURING_REPLAY")
    require(pinned.source_tree_sha256(source) == expected_source,
            "SOURCE_CHANGED_DURING_REPLAY")
    after, _, after_events = portable.verify(portable_dir, expected_portable)
    require(after == manifest and after_events == events,
            "PORTABLE_CHANGED_DURING_REPLAY")
    require(replayed == manifest["committed"]
            and file_count == len(manifest["native_files"]),
            "REPLAY_COUNT_CONTRADICTION")
    return {
        "schema": SCHEMA,
        "status": "PASS",
        "portable_sha256": expected_portable,
        "engine_sha256": expected_engine,
        "source_tree_sha256": expected_source,
        "native_jobs_recomputed": replayed,
        "native_files_bytewise_equal": file_count,
        "skipped_nodes_independently_gated": manifest["skipped"],
        "native_receipts_exactly_equal": True,
        "p27_assurance": "EXACT_PINNED_NATIVE_REPRODUCTION_NOT_SEPARATE_CRYPTO",
        "external_pin_authentication": "NOT_PERFORMED",
        "native_engine_sandbox": "NOT_PROVIDED",
        "crash_durability": "UNCLAIMED",
        "production_admission": "NOT_AUTHORIZED",
    }


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("verify-replay", "restore-replay",
                                            "resume-replay"))
    parser.add_argument("--portable-dir", required=True)
    parser.add_argument("--expected-portable-sha256", required=True)
    parser.add_argument("--engine", required=True)
    parser.add_argument("--source-root", required=True)
    parser.add_argument("--expected-engine-sha256", required=True)
    parser.add_argument("--expected-source-sha256", required=True)
    parser.add_argument("--restore-root")
    args = parser.parse_args(argv)
    p = Path(args.portable_dir)
    try:
        if args.action != "verify-replay":
            require(bool(args.restore_root), "RESTORE_ROOT_REQUIRED")
        qualified = replay(p, args.expected_portable_sha256,
                           Path(args.engine), Path(args.source_root),
                           args.expected_engine_sha256,
                           args.expected_source_sha256)
        if args.action == "verify-replay":
            emit(qualified)
            return 0
        if args.action == "restore-replay":
            result = portable.restore(
                p, args.expected_portable_sha256, Path(args.restore_root))
            result["replay_qualification"] = "PASS_EXACT_NATIVE_RUST_REPLAY"
            result["native_jobs_recomputed"] = qualified["native_jobs_recomputed"]
            emit(result)
            return 0

        # APP-014 independently audits the restored root and invokes APP-011.
        # The returned stdout is ONLY new DAG JSONL events, not this receipt.
        return portable.resume_native(
            p, args.expected_portable_sha256, Path(args.restore_root),
            Path(args.engine), Path(args.source_root),
            args.expected_engine_sha256, args.expected_source_sha256)
    except (Uncertain, portable.Unknown) as error:
        emit({"schema": SCHEMA, "status": "INCONCLUSIVE",
              "error_code": error.code, "publication": "UNKNOWN"})
        return 3
    except (Reject, portable.Refused, portable.inner.Refused,
            portable.pinned.Denied, native_audit.AuditRejected,
            ValueError, OSError) as error:
        emit({"schema": SCHEMA, "status": "REJECTED",
              "error_code": getattr(error, "code", "REPLAY_INPUT_INVALID"),
              "publication": "NOT_STARTED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
