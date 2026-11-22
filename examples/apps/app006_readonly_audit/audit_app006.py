#!/usr/bin/env python3
"""APP-006: read-only offline verifier for real APP-005 NORDOI JSONL batches.

NO invocation of the compiler, NO file writes, NO model/inference, NO network,
NO human test duty. It checks that each committed native APP-004 receipt is
consistent with exact bytes of files actually published by APP-005.

IMPORTANT: per-file P2.7 proof_sha256 and bundle_receipt_sha256 are historical
engine receipts, NOT simple content checksums. We check their shape but
cannot independently authenticate them without their canonical inputs or a
trusted signature. Our computed file SHA256 hashes attest observed bytes
ONLY. No claim of provenance, hostile-concurrent-filesystem isolation or
durability after crash. This audit does not authorize production.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys

sys.dont_write_bytecode = True

AUDIT_SCHEMA = "NORDOI_AGENT_BATCH_AUDIT_V1"
NATIVE_SCHEMA = "NORDOI_AGENT_BUNDLE_RESULT_V1"
MAX_RECEIPTS_BYTES = 262144
MAX_FILES_BYTES = 65536
MAX_JOBS = 8
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
HEX_V13 = re.compile(r"(?:[0-9a-f]{2}){20,}\Z")
FILE_NAMES = frozenset(("decision.txt", "report.txt"))
COMMIT_FIELDS = frozenset((
    "schema", "status", "entry_module", "key_code", "module_count",
    "decision", "selected", "bundle", "files", "authority", "publication",
    "partial_final_state", "crash_durability", "rule_receipt_hex",
    "bundle_receipt_sha256",
))
FILE_FIELDS = frozenset(("name", "bytes", "content", "proof_sha256"))
ERROR_FIELDS = frozenset((
    "schema", "status", "error_code", "exit_code", "publication",
))
AUTHORITY = {
    "mode": "EXPLICIT",
    "grant_count": 2,
    "ambient": "NONE",
    "overwrite": "DENIED",
}


class AuditRejected(ValueError):
    def __init__(self, code: str):
        super().__init__(code)
        self.code = code


def require(yes: bool, code: str) -> None:
    if not yes:
        raise AuditRejected(code)


def is_int(value: object, minimum: int = 0, maximum: int = 2**32 - 1) -> bool:
    return type(value) is int and minimum <= value <= maximum


def exact_keys(value: object, required: frozenset[str], code: str) -> dict:
    require(type(value) is dict and set(value) == required, code)
    return value


def no_duplicate_keys(pairs: list[tuple[str, object]]) -> dict:
    value = {}
    for key, item in pairs:
        if key in value:
            raise AuditRejected("DUPLICATE_RECEIPT_FIELD")
        value[key] = item
    return value


def no_constants(value: str) -> None:
    raise AuditRejected("NONFINITE_RECEIPT_VALUE")


def bounded_regular_file(path: Path, maximum: int, code: str) -> bytes:
    try:
        mode = path.lstat().st_mode
        require(stat.S_ISREG(mode) and not stat.S_ISLNK(mode), code)
        require(path.stat().st_size <= maximum, code)
        with path.open("rb") as stream:
            data = stream.read(maximum + 1)
        require(len(data) <= maximum, code)
        return data
    except (OSError, OverflowError):
        raise AuditRejected(code) from None


def real_directory(path: Path, code: str) -> None:
    try:
        mode = path.lstat().st_mode
        require(stat.S_ISDIR(mode) and not stat.S_ISLNK(mode), code)
    except OSError:
        raise AuditRejected(code) from None


def listed_names(path: Path, code: str) -> set[str]:
    try:
        return {entry.name for entry in path.iterdir()}
    except OSError:
        raise AuditRejected(code) from None


def load_receipts(path: Path) -> tuple[bytes, list[dict]]:
    raw = bounded_regular_file(path, MAX_RECEIPTS_BYTES, "RECEIPTS_NOT_BOUNDED_REGULAR_FILE")
    try:
        text = raw.decode("utf-8", errors="strict")
    except UnicodeError:
        raise AuditRejected("RECEIPTS_NOT_UTF8") from None
    require(text.endswith("\n"), "RECEIPTS_NOT_NEWLINE_TERMINATED")
    lines = text.splitlines()
    require(1 <= len(lines) <= MAX_JOBS + 1, "INVALID_RECEIPT_LINE_COUNT")
    receipts = []
    for line in lines:
        require(line != "" and "\r" not in line, "INVALID_RECEIPT_LINE")
        try:
            obj = json.loads(
                line, object_pairs_hook=no_duplicate_keys, parse_constant=no_constants
            )
        except AuditRejected:
            # Do not erase an intentionally precise duplicate-key rejection.
            raise
        except (ValueError, TypeError, OverflowError):
            raise AuditRejected("INVALID_RECEIPT_JSON") from None
        require(type(obj) is dict, "INVALID_RECEIPT_JSON")
        receipts.append(obj)
    return raw, receipts


def verify_native_record(record: dict, index: int, root: Path) -> dict:
    exact_keys(record, COMMIT_FIELDS, "COMMIT_FIELDS_INVALID")
    require(record["schema"] == NATIVE_SCHEMA and record["status"] == "COMMITTED",
            "COMMIT_SCHEMA_INVALID")
    require(record["entry_module"] == "app.main", "COMMIT_MODULE_INVALID")
    require(is_int(record["key_code"]), "COMMIT_KEY_CODE_INVALID")
    require(is_int(record["module_count"], 2, 2), "COMMIT_MODULE_COUNT_INVALID")
    require(is_int(record["decision"], 0, 1), "COMMIT_DECISION_INVALID")
    require(record["selected"] == ("approved" if record["decision"] else "rejected"),
            "COMMIT_BRANCH_CONTRADICTION")
    require(record["bundle"] == "quality-decision", "COMMIT_BUNDLE_INVALID")
    require(record["authority"] == AUTHORITY, "COMMIT_AUTHORITY_INVALID")
    require(record["publication"] == "DIRECTORY_RENAME", "COMMIT_PUBLICATION_INVALID")
    require(record["partial_final_state"] == "FORBIDDEN", "COMMIT_PARTIAL_STATE_INVALID")
    require(record["crash_durability"] == "UNCLAIMED", "COMMIT_CRASH_CLAIM_INVALID")
    require(type(record["rule_receipt_hex"]) is str
            and HEX_V13.fullmatch(record["rule_receipt_hex"]) is not None,
            "COMMIT_RULE_RECEIPT_INVALID")
    require(type(record["bundle_receipt_sha256"]) is str
            and HEX64.fullmatch(record["bundle_receipt_sha256"]) is not None,
            "COMMIT_P27_RECEIPT_INVALID")

    files = record["files"]
    require(type(files) is list and len(files) == 2, "COMMIT_FILE_LIST_INVALID")
    job = root / f"job-{index:04d}"
    real_directory(job, "MISSING_OR_SYMLINKED_JOB_DIRECTORY")
    require(listed_names(job, "JOB_LISTING_DENIED") == {"quality-decision"},
            "JOB_HAS_UNEXPECTED_CONTENT")
    bundle = job / "quality-decision"
    real_directory(bundle, "MISSING_OR_SYMLINKED_BUNDLE")
    require(listed_names(bundle, "BUNDLE_LISTING_DENIED") == FILE_NAMES,
            "BUNDLE_FILE_SET_CHANGED")

    audits = []
    seen = set()
    for file in files:
        exact_keys(file, FILE_FIELDS, "FILE_RECEIPT_FIELDS_INVALID")
        name = file["name"]
        require(type(name) is str and name in FILE_NAMES and name not in seen,
                "FILE_RECEIPT_NAME_INVALID")
        seen.add(name)
        require(type(file["content"]) is str, "FILE_RECEIPT_CONTENT_INVALID")
        require(is_int(file["bytes"], 0, MAX_FILES_BYTES),
                "FILE_RECEIPT_BYTES_INVALID")
        require(type(file["proof_sha256"]) is str
                and HEX64.fullmatch(file["proof_sha256"]) is not None,
                "FILE_P27_PROOF_SHAPE_INVALID")
        expected = file["content"].encode("utf-8")
        require(len(expected) == file["bytes"], "FILE_RECEIPT_SIZE_CONTRADICTION")
        actual = bounded_regular_file(
            bundle / name, MAX_FILES_BYTES, "FILE_MISSING_SYMLINK_OR_OVERSIZED"
        )
        require(actual == expected, "REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")
        audits.append({
            "name": name,
            "size_bytes": len(actual),
            "observed_sha256": hashlib.sha256(actual).hexdigest(),
        })
    require(seen == FILE_NAMES, "FILE_RECEIPT_NAMES_INCOMPLETE")
    return {
        "job": f"job-{index:04d}",
        "key_code": record["key_code"],
        "decision": record["decision"],
        "bundle_receipt_sha256_unverified": record["bundle_receipt_sha256"],
        "files": sorted(audits, key=lambda x: x["name"]),
    }


def audit(root: Path, receipt_file: Path) -> tuple[dict, int]:
    real_directory(root, "BATCH_ROOT_NOT_REAL_DIRECTORY")
    raw, receipts = load_receipts(receipt_file)
    committed = []
    error_record = None
    for index, receipt in enumerate(receipts):
        require(receipt.get("schema") == NATIVE_SCHEMA, "RECEIPT_SCHEMA_INVALID")
        state = receipt.get("status")
        if state == "COMMITTED":
            require(error_record is None, "COMMIT_AFTER_ERROR")
            require(len(committed) < MAX_JOBS, "TOO_MANY_COMMITTED_JOBS")
            committed.append(verify_native_record(receipt, len(committed) + 1, root))
        elif state == "ERROR":
            require(error_record is None and index == len(receipts) - 1,
                    "ERROR_MUST_TERMINATE_BATCH")
            exact_keys(receipt, ERROR_FIELDS, "ERROR_FIELDS_INVALID")
            require(type(receipt["error_code"]) is str and bool(receipt["error_code"]),
                    "ERROR_CODE_INVALID")
            require(is_int(receipt["exit_code"], 1, 255), "ERROR_EXIT_CODE_INVALID")
            require(receipt["publication"] == "UNKNOWN",
                    "ERROR_CLAIMS_FALSE_PUBLICATION")
            error_record = receipt
        else:
            raise AuditRejected("INVALID_RECEIPT_STATUS")

    expected_names = {f"job-{i:04d}" for i in range(1, len(committed) + 1)}
    observed_names = listed_names(root, "BATCH_ROOT_LISTING_DENIED")
    # The native APP005 creates a job directory before evaluating its rule.
    # A failing next job may leave its job-NNNN directory empty.
    optional_failed = f"job-{len(committed)+1:04d}"
    extras = observed_names - expected_names
    if error_record is not None and extras == {optional_failed}:
        empty = root / optional_failed
        real_directory(empty, "FAILED_JOB_DIRECTORY_NOT_REAL")
        require(not listed_names(empty, "FAILED_JOB_LISTING_DENIED"),
                "FAILED_JOB_LEFT_UNEXPECTED_FILES")
    else:
        require(not extras, "BATCH_ROOT_UNEXPECTED_FILES_OR_JOBS")
    require(expected_names <= observed_names, "BATCH_ROOT_MISSING_JOB")

    status = ("PASS" if error_record is None else
              "PARTIAL" if committed else "ERROR")
    report = {
        "schema": AUDIT_SCHEMA,
        "status": status,
        "committed_jobs_verified": len(committed),
        "input_receipts_sha256": hashlib.sha256(raw).hexdigest(),
        "jobs": committed,
        "attestation": "READ_ONLY_CONTENT_CONSISTENCY_NOT_AUTHENTICATED_SIGNATURE",
        "p27_proof_verification": "SHAPE_ONLY_NOT_CRYPTOGRAPHICALLY_RECOMPUTED",
        "crash_durability": "UNCLAIMED",
        "concurrent_host_adversary": "NOT_COVERED",
    }
    if error_record is not None:
        report["native_error_code"] = error_record["error_code"]
        report["native_error_exit_code"] = error_record["exit_code"]
    return report, 0 if status == "PASS" else 3


def emit(record: dict) -> None:
    print(json.dumps(record, ensure_ascii=True, sort_keys=True, separators=(",", ":")))


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--batch-root", required=True)
    parser.add_argument("--receipts", required=True)
    args = parser.parse_args(argv)
    try:
        result, code = audit(Path(args.batch_root), Path(args.receipts))
    except AuditRejected as error:
        emit({
            "schema": AUDIT_SCHEMA, "status": "REJECTED",
            "error_code": error.code, "verified": False,
            "attestation": "NONE",
        })
        return 2
    emit(result)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
