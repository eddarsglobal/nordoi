#!/usr/bin/env python3
"""APP-013: seal, verify and resume a genuine NORDOI agent-DAG checkpoint.

An inter-agent handoff is a fixed-format directory containing exact original
plan and native JSONL bytes, a deterministic native-file inventory, a
SHA-256 manifest and a FINAL seal marker. Verification REQUIRES the expected
manifest SHA-256 from a separately trusted channel. A sealed handoff is
neither a cryptographic signature nor crash-durable storage.

Seal uses exclusive creation, best-effort fsync, and writes the marker LAST;
incomplete destinations remain rejected. Verify is read-only. Resume first
verifies every native P2.7 file and all checkpoint contents before delegating
to APP-011; native stdout is passed through unchanged as the DAG suffix.
No model, Internet, privilege escalation or production admission.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app010_readonly_dag_audit"))
import audit_app010 as independent
sys.path.insert(0, str(HERE.parent / "app011_verified_resume"))
import run_app011 as resume

native_audit = independent.native_audit
pinned = resume.pinned
SCHEMA = "NORDOI_AGENT_CHECKPOINT_HANDOFF_V1"
RESPONSE = "NORDOI_AGENT_CHECKPOINT_HANDOFF_RESULT_V1"
MANIFEST = "manifest.json"
FILES = frozenset(("plan.json", "native-prefix.jsonl", MANIFEST, "SEAL.sha256"))
MAX_MANIFEST_BYTES = 16384


class Refused(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


class PublicationUnknown(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


def require(condition: bool, code: str) -> None:
    if not condition:
        raise Refused(code)


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def encode(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, ensure_ascii=True,
                      separators=(",", ":")).encode("utf-8") + b"\n"


def emit(value: dict) -> None:
    sys.stdout.buffer.write(encode(value))
    sys.stdout.buffer.flush()


def verify_checkpoint(root: Path, plan: Path, prefix: Path,
                      expected_plan_sha: str) -> tuple[dict, dict, bytes, bytes]:
    require(type(expected_plan_sha) is str
            and pinned.PIN.fullmatch(expected_plan_sha) is not None,
            "TRUSTED_PLAN_PIN_INVALID")
    plan_bytes = native_audit.bounded_regular_file(
        plan, independent.MAX_PLAN_BYTES, "PLAN_NOT_REGULAR_BOUNDED")
    prefix_bytes = native_audit.bounded_regular_file(
        prefix, independent.MAX_EVENTS_BYTES, "PREFIX_NOT_REGULAR_BOUNDED")
    nodes, observed_plan_sha = independent.plan_nodes(plan, expected_plan_sha)
    events, original = independent.event_records(prefix)
    require(prefix_bytes == original and len(events) < len(nodes)
            and all(event.get("status") in ("COMMITTED", "SKIPPED")
                    for event in events), "NOT_CLEAN_PROPER_PREFIX")
    report, code = independent.audit(root, plan, prefix, expected_plan_sha)
    require(code == 3 and report["status"] == "INCONCLUSIVE"
            and report["events_checked"] == len(events)
            and report["plan_sha256"] == observed_plan_sha,
            "INDEPENDENT_CHECKPOINT_AUDIT_FAILED")

    # Audit APP-010 already validates the exact job directories and filenames;
    # construct paths ONLY from internal constants and its checked step count,
    # never from an untrusted manifest's arbitrary relative path strings.
    inventory = []
    for index, event in enumerate(events, 1):
        if event["status"] != "COMMITTED":
            continue
        for name in sorted(native_audit.FILE_NAMES):
            relative = f"job-{index:04d}/quality-decision/{name}"
            raw = native_audit.bounded_regular_file(
                root / f"job-{index:04d}" / "quality-decision" / name,
                native_audit.MAX_FILES_BYTES, "NATIVE_FILE_NOT_REGULAR_BOUNDED")
            inventory.append({"path": relative, "bytes": len(raw), "sha256": sha(raw)})
    require(len(inventory) == 2 * report["committed_jobs_verified"],
            "NATIVE_INVENTORY_COUNT_INVALID")
    manifest = {
        "schema": SCHEMA,
        "plan_sha256": observed_plan_sha,
        "prefix_sha256": sha(prefix_bytes),
        "events": len(events),
        "committed": report["committed_jobs_verified"],
        "skipped": report["skipped_nodes_verified"],
        "native_files": inventory,
        "pin_provenance": "EXTERNAL_TRUST_NOT_ATTESTED",
        "native_receipt_signature": "NOT_PRESENT",
        "crash_durability": "UNCLAIMED",
    }
    return manifest, report, plan_bytes, prefix_bytes


def exclusive_file(path: Path, payload: bytes) -> None:
    mode = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    mode |= getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    fd = os.open(str(path), mode, 0o600)
    try:
        with os.fdopen(fd, "wb", closefd=True) as file:
            fd = -1
            file.write(payload)
            file.flush()
            os.fsync(file.fileno())
    finally:
        if fd != -1:
            os.close(fd)


def seal(bundle: Path, root: Path, plan: Path, prefix: Path,
         expected_plan_sha: str) -> dict:
    manifest, report, plan_raw, prefix_raw = verify_checkpoint(
        root, plan, prefix, expected_plan_sha)
    manifest_raw = encode(manifest)
    require(len(manifest_raw) <= MAX_MANIFEST_BYTES, "MANIFEST_TOO_LARGE")
    target = bundle.absolute()
    parent = target.parent
    native_audit.real_directory(parent, "SEAL_PARENT_NOT_REAL_DIRECTORY")
    root_real = root.resolve(strict=True)
    parent_real = parent.resolve(strict=True)
    # Keep the handoff OUTSIDE the authenticated native publication tree.
    require(parent_real != root_real and root_real not in parent_real.parents,
            "HANDOFF_OVERLAPS_NATIVE_ROOT")
    require(not target.exists() and not target.is_symlink(),
            "HANDOFF_DESTINATION_ALREADY_EXISTS")

    # Do not imply rollback on error once an exclusive directory is created.
    try:
        target.mkdir(mode=0o700)
    except FileExistsError:
        raise Refused("HANDOFF_DESTINATION_ALREADY_EXISTS") from None
    except OSError:
        raise Refused("HANDOFF_DIRECTORY_CREATION_DENIED") from None
    digest = sha(manifest_raw)
    try:
        exclusive_file(target / "plan.json", plan_raw)
        exclusive_file(target / "native-prefix.jsonl", prefix_raw)
        exclusive_file(target / MANIFEST, manifest_raw)
        # Publish seal last. If interrupted, consumers reject the directory.
        exclusive_file(target / "SEAL.sha256", (digest + "\n").encode("ascii"))
        if os.name != "nt" and hasattr(os, "O_DIRECTORY"):
            fd = os.open(str(target), os.O_RDONLY | os.O_DIRECTORY)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
    except OSError:
        raise PublicationUnknown("HANDOFF_PUBLICATION_INCOMPLETE") from None
    return {"schema": RESPONSE, "status": "SEALED",
            "manifest_sha256": digest,
            "original_events_sha256": manifest["prefix_sha256"],
            "native_jobs_verified": report["committed_jobs_verified"],
            "native_files_verified": len(manifest["native_files"]),
            "external_authentication": "NOT_PERFORMED",
            "publication": "HANDOFF_DIRECTORY_NOT_CRASH_DURABLE"}


def verify_bundle(bundle: Path, root: Path, expected_seal_sha: str
                  ) -> tuple[dict, dict]:
    require(type(expected_seal_sha) is str
            and pinned.PIN.fullmatch(expected_seal_sha) is not None,
            "TRUSTED_SEAL_PIN_INVALID")
    native_audit.real_directory(bundle, "HANDOFF_NOT_REAL_DIRECTORY")
    require(native_audit.listed_names(bundle, "HANDOFF_LISTING_FAILED") == FILES,
            "HANDOFF_FILE_SET_CHANGED")
    manifest_raw = native_audit.bounded_regular_file(
        bundle / MANIFEST, MAX_MANIFEST_BYTES, "HANDOFF_MANIFEST_NOT_REGULAR")
    require(sha(manifest_raw) == expected_seal_sha, "HANDOFF_SEAL_PIN_MISMATCH")
    seal_raw = native_audit.bounded_regular_file(
        bundle / "SEAL.sha256", 65, "HANDOFF_MARKER_NOT_REGULAR")
    require(seal_raw == (expected_seal_sha + "\n").encode("ascii"),
            "HANDOFF_MARKER_MISMATCH")
    saved = independent.decode_json(manifest_raw, "HANDOFF_MANIFEST_INVALID_JSON")
    require(type(saved) is dict and set(saved) == {
        "schema", "plan_sha256", "prefix_sha256", "events",
        "committed", "skipped", "native_files", "pin_provenance",
        "native_receipt_signature", "crash_durability",
    } and saved.get("schema") == SCHEMA, "HANDOFF_MANIFEST_SCHEMA_INVALID")
    require(saved["pin_provenance"] == "EXTERNAL_TRUST_NOT_ATTESTED"
            and saved["native_receipt_signature"] == "NOT_PRESENT"
            and saved["crash_durability"] == "UNCLAIMED",
            "HANDOFF_TRUST_CLAIM_INVALID")
    original, report, _, _ = verify_checkpoint(
        root, bundle / "plan.json", bundle / "native-prefix.jsonl",
        saved["plan_sha256"])
    require(saved == original, "HANDOFF_MANIFEST_CONTENT_CONTRADICTION")
    return original, report


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("seal", "verify", "resume"))
    parser.add_argument("--bundle-dir", required=True)
    parser.add_argument("--native-root", required=True)
    parser.add_argument("--plan")
    parser.add_argument("--prefix-events")
    parser.add_argument("--expected-plan-sha256")
    parser.add_argument("--expected-seal-sha256")
    parser.add_argument("--engine")
    parser.add_argument("--source-root")
    parser.add_argument("--expected-engine-sha256")
    parser.add_argument("--expected-source-sha256")
    args = parser.parse_args(argv)
    bundle, root = Path(args.bundle_dir), Path(args.native_root)
    try:
        if args.action == "seal":
            require(bool(args.plan and args.prefix_events and args.expected_plan_sha256),
                    "SEAL_REQUIRED_ARGUMENT_MISSING")
            result = seal(bundle, root, Path(args.plan),
                          Path(args.prefix_events), args.expected_plan_sha256)
            emit(result)
            return 0

        require(bool(args.expected_seal_sha256), "SEAL_VERIFICATION_PIN_REQUIRED")
        manifest, report = verify_bundle(bundle, root, args.expected_seal_sha256)
        if args.action == "verify":
            emit({"schema": RESPONSE, "status": "VERIFIED",
                  "manifest_sha256": args.expected_seal_sha256,
                  "original_events_sha256": manifest["prefix_sha256"],
                  "native_jobs_verified": report["committed_jobs_verified"],
                  "native_files_verified": len(manifest["native_files"]),
                  "external_authentication": "NOT_PERFORMED"})
            return 0

        require(bool(args.engine and args.source_root
                     and args.expected_engine_sha256
                     and args.expected_source_sha256),
                "RESUME_REQUIRED_ARGUMENT_MISSING")
        for digest in (args.expected_engine_sha256, args.expected_source_sha256):
            require(pinned.PIN.fullmatch(digest) is not None,
                    "INVALID_ENGINE_SOURCE_PIN")
        command = [
            sys.executable, "-B",
            str(HERE.parent / "app011_verified_resume/run_app011.py"),
            "--plan", str(bundle / "plan.json"),
            "--prefix-events", str(bundle / "native-prefix.jsonl"),
            "--grant-output-dir", str(root),
            "--engine", args.engine,
            "--source-root", args.source_root,
            "--expected-plan-sha256", manifest["plan_sha256"],
            "--expected-prefix-sha256", manifest["prefix_sha256"],
            "--expected-engine-sha256", args.expected_engine_sha256,
            "--expected-source-sha256", args.expected_source_sha256,
        ]
        try:
            result = subprocess.run(command, capture_output=True,
                                    cwd=HERE.parents[2], timeout=980)
        except (OSError, subprocess.TimeoutExpired):
            emit({"schema": RESPONSE, "status": "ERROR",
                  "error_code": "RESUME_SUBPROCESS_OUTCOME_UNKNOWN",
                  "publication": "UNKNOWN"})
            return 3
        sys.stdout.buffer.write(result.stdout)
        sys.stdout.buffer.flush()
        if result.returncode in (0, 2, 3):
            return result.returncode
        emit({"schema": RESPONSE, "status": "ERROR",
              "error_code": "RESUME_SUBPROCESS_ABNORMAL_EXIT",
              "publication": "UNKNOWN"})
        return 3
    except PublicationUnknown as error:
        emit({"schema": RESPONSE, "status": "ERROR",
              "error_code": error.code, "publication": "UNKNOWN"})
        return 3
    except (Refused, pinned.Denied, native_audit.AuditRejected,
            OSError, ValueError) as error:
        emit({"schema": RESPONSE, "status": "REJECTED",
              "error_code": getattr(error, "code", "HANDOFF_INPUT_IO_REJECTED"),
              "publication": "NOT_STARTED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
