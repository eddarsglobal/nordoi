#!/usr/bin/env python3
"""APP-007: pinned-source, pinned-engine deterministic replay of real APP-005 jobs.

This is an AI-agent research verifier. It recomputes native V1.3/P2.7 results
by executing the explicitly pinned native binary in a NEW PRIVATE temporary
directory. It does not implement independent P2.7 cryptography, authenticate
pins, sign evidence, or certify native/production authority. Source files,
engine, original batch files and receipts remain untouched.
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
import tempfile

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "app006_readonly_audit"))
import audit_app006 as prior

SCHEMA = "NORDOI_AGENT_PINNED_REPLAY_V1"
REQUEST_SCHEMA = "NORDOI_AGENT_REQUEST_V1"
PIN = re.compile(r"[0-9a-f]{64}\Z")
MAX_REQUEST_BYTES = 8 * (4096 + 1)
MAX_SOURCE_FILES = 64
MAX_SOURCE_BYTES = 1_048_576
MAX_ENGINE_BYTES = 134_217_728


class Denied(ValueError):
    def __init__(self, code: str):
        super().__init__(code)
        self.code = code


def require(ok: bool, code: str) -> None:
    if not ok:
        raise Denied(code)


def regular_bytes(path: Path, limit: int, code: str) -> bytes:
    try:
        metadata = path.lstat()
        require(stat.S_ISREG(metadata.st_mode), code)
        require(metadata.st_size <= limit, code)
        with path.open("rb") as handle:
            data = handle.read(limit + 1)
        require(len(data) <= limit, code)
        return data
    except OSError:
        raise Denied(code) from None


def engine_sha256(path: Path) -> str:
    return hashlib.sha256(regular_bytes(path, MAX_ENGINE_BYTES, "ENGINE_NOT_BOUNDED_REGULAR_FILE")).hexdigest()


def source_tree_sha256(root: Path) -> str:
    try:
        require(stat.S_ISDIR(root.lstat().st_mode), "SOURCE_ROOT_INVALID")
        digest = hashlib.sha256(b"NORDOI_APP007_SOURCE_TREE_V1\x00")
        paths = []
        for parent, dirs, files in os.walk(root, followlinks=False):
            for name in dirs + files:
                p = Path(parent) / name
                require(not p.is_symlink(), "SOURCE_SYMLINK_DENIED")
            dirs.sort()
            for name in sorted(files):
                paths.append(Path(parent) / name)
        require(1 <= len(paths) <= MAX_SOURCE_FILES, "SOURCE_FILE_COUNT_INVALID")
        total = 0
        for path in sorted(paths, key=lambda p: p.relative_to(root).as_posix()):
            content = regular_bytes(path, MAX_SOURCE_BYTES, "SOURCE_FILE_INVALID")
            total += len(content)
            require(total <= MAX_SOURCE_BYTES, "SOURCE_TOO_LARGE")
            name = path.relative_to(root).as_posix().encode("utf-8")
            digest.update(len(name).to_bytes(4, "big"))
            digest.update(name)
            digest.update(len(content).to_bytes(8, "big"))
            digest.update(hashlib.sha256(content).digest())
        return digest.hexdigest()
    except OSError:
        raise Denied("SOURCE_ROOT_INVALID") from None


def no_duplicates(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for k, v in pairs:
        require(k not in result, "DUPLICATE_REQUEST_KEY")
        result[k] = v
    return result


def reject_constant(_value: str) -> None:
    raise Denied("INVALID_REQUEST_NUMBER")


def request_codes(raw: bytes) -> list[int]:
    require(0 < len(raw) <= MAX_REQUEST_BYTES, "REQUESTS_SIZE_INVALID")
    if raw.endswith(b"\n"):
        raw = raw[:-1]
    lines = raw.split(b"\n")
    require(1 <= len(lines) <= 8, "REQUEST_COUNT_INVALID")
    codes = []
    for line in lines:
        require(0 < len(line) <= 4096, "REQUEST_LINE_INVALID")
        try:
            record = json.loads(line.decode("utf-8"), object_pairs_hook=no_duplicates,
                                parse_constant=reject_constant)
        except Denied:
            raise
        except (UnicodeError, ValueError, TypeError, OverflowError):
            raise Denied("REQUEST_JSON_INVALID") from None
        require(type(record) is dict and set(record) == {"schema", "key_code"}
                and record["schema"] == REQUEST_SCHEMA, "REQUEST_SCHEMA_INVALID")
        code = record["key_code"]
        require(type(code) is int and 0 <= code <= 2**32 - 1, "REQUEST_CODE_INVALID")
        codes.append(code)
    return codes


def result(status: str, code: str | None = None, **extra) -> dict:
    report = {
        "schema": SCHEMA,
        "status": status,
        "assurance": "PINNED_NATIVE_DETERMINISTIC_REPLAY_NOT_INDEPENDENT_CRYPTO",
        "external_authentication": "NOT_PERFORMED",
        "native_production_admission": "NOT_AUTHORIZED",
    }
    if code is not None:
        report["error_code"] = code
    report.update(extra)
    return report


def replay(batch_root: Path, receipts_path: Path, requests_path: Path,
           engine: Path, source_root: Path,
           expected_engine: str, expected_source: str, expected_requests: str
           ) -> tuple[dict, int]:
    for value in (expected_engine, expected_source, expected_requests):
        require(PIN.fullmatch(value) is not None, "INVALID_PIN_FORMAT")
    actual_engine = engine_sha256(engine)
    require(actual_engine == expected_engine, "ENGINE_PIN_MISMATCH")
    actual_source = source_tree_sha256(source_root)
    require(actual_source == expected_source, "SOURCE_PIN_MISMATCH")
    original_input = regular_bytes(requests_path, MAX_REQUEST_BYTES, "REQUEST_FILE_INVALID")
    actual_input = hashlib.sha256(original_input).hexdigest()
    require(actual_input == expected_requests, "REQUEST_PIN_MISMATCH")
    codes = request_codes(original_input)

    try:
        first, _ = prior.audit(batch_root, receipts_path)
        original_raw, original_records = prior.load_receipts(receipts_path)
    except prior.AuditRejected as error:
        raise Denied("APP006_" + error.code) from None
    if first["status"] != "PASS":
        return result("INCONCLUSIVE", "ORIGINAL_BATCH_NOT_FULLY_COMMITTED",
                      original_status=first["status"]), 3
    require(len(original_records) == len(codes), "RECEIPT_REQUEST_COUNT_MISMATCH")
    require([r["key_code"] for r in original_records] == codes, "RECEIPT_REQUEST_CODES_MISMATCH")

    with tempfile.TemporaryDirectory(prefix="nordoi_app007_private_replay_") as tmp:
        private = Path(tmp)
        root = private / "isolated-output"
        root.mkdir(mode=0o700)
        receipt_copy = private / "native-replay.jsonl"
        try:
            execution = subprocess.run(
                [str(engine.resolve(strict=True)), "agent-batch-jsonl",
                 str(source_root.resolve(strict=True)), "app.main",
                 "--grant-output-dir", str(root)],
                input=original_input, capture_output=True,
                cwd=Path(__file__).resolve().parents[3], timeout=120,
            )
        except (OSError, subprocess.TimeoutExpired):
            return result("INCONCLUSIVE", "NATIVE_REPLAY_COULD_NOT_COMPLETE"), 3
        require(engine_sha256(engine) == expected_engine, "ENGINE_CHANGED_DURING_REPLAY")
        require(source_tree_sha256(source_root) == expected_source, "SOURCE_CHANGED_DURING_REPLAY")
        require(regular_bytes(requests_path, MAX_REQUEST_BYTES, "REQUEST_FILE_CHANGED")
                == original_input, "REQUEST_FILE_CHANGED")
        try:
            current_raw, _ = prior.load_receipts(receipts_path)
        except prior.AuditRejected:
            raise Denied("ORIGINAL_RECEIPTS_CHANGED") from None
        require(current_raw == original_raw, "ORIGINAL_RECEIPTS_CHANGED")
        require(execution.returncode == 0, "NATIVE_REPLAY_FAILED")
        require(0 < len(execution.stdout) <= prior.MAX_RECEIPTS_BYTES, "REPLAY_OUTPUT_INVALID")
        receipt_copy.write_bytes(execution.stdout)
        try:
            second, _ = prior.audit(root, receipt_copy)
        except prior.AuditRejected:
            raise Denied("REPLAY_OUTPUT_NOT_CONSISTENT") from None
        require(second["status"] == "PASS", "REPLAY_OUTPUT_NOT_COMMITTED")
        require(execution.stdout == original_raw, "NATIVE_REPLAY_RECEIPT_MISMATCH")
        try:
            third, _ = prior.audit(batch_root, receipts_path)
        except prior.AuditRejected:
            raise Denied("ORIGINAL_CHANGED_DURING_REPLAY") from None
        require(third == first, "ORIGINAL_CHANGED_DURING_REPLAY")
        return result(
            "PASS",
            jobs_replayed=len(codes),
            files_replayed=2 * len(codes),
            original_receipts_sha256=hashlib.sha256(original_raw).hexdigest(),
            replay_receipts_sha256=hashlib.sha256(execution.stdout).hexdigest(),
            engine_sha256=actual_engine,
            source_tree_sha256=actual_source,
            original_requests_sha256=actual_input,
            p27_proof_validation="EXACT_NATIVE_REPRODUCTION_NOT_STANDALONE_CRYPTO_RECOMPUTATION",
            crash_durability="UNCLAIMED",
            concurrent_host_adversary="NOT_COVERED",
        ), 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--batch-root", required=True)
    parser.add_argument("--receipts", required=True)
    parser.add_argument("--requests", required=True)
    parser.add_argument("--engine", required=True)
    parser.add_argument("--source-root", required=True)
    parser.add_argument("--expected-engine-sha256", required=True)
    parser.add_argument("--expected-source-sha256", required=True)
    parser.add_argument("--expected-requests-sha256", required=True)
    args = parser.parse_args(argv)
    try:
        report, exit_code = replay(
            Path(args.batch_root), Path(args.receipts), Path(args.requests),
            Path(args.engine), Path(args.source_root),
            args.expected_engine_sha256, args.expected_source_sha256,
            args.expected_requests_sha256,
        )
    except Denied as error:
        report, exit_code = result("REJECTED", error.code), 2
    print(json.dumps(report, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
