#!/usr/bin/env python3
"""APP-014: portable, content-addressed native NORDOI agent checkpoint.

The *only* trust anchor is the separately supplied SHA-256 of portable.json.
Pack: audit an authentic APP-013 handoff and native P2.7 files, then copy the
exact bounded bytes into an exclusive portable directory; marker LAST.
Verify: independent read-only audit of every packaged native file, original
DAG/JSONL and all nested manifest pins, without a native engine.
Restore: copy verified original files only into an EXISTING EMPTY directory,
never overwrite. Resume: audit portable and restored histories before calling
APP-011 with externally pinned real Rust executable and .noi sources.
Not a signature, durable backup, hostile-host sandbox, or crash recovery from
unknown in-flight effects. No rollback across native jobs or partial copies.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app013_verified_handoff"))
import handoff_app013 as inner

audit = inner.native_audit
pinned = inner.pinned
independent = inner.independent
SCHEMA = "NORDOI_AGENT_PORTABLE_CHECKPOINT_V1"
RESULT = "NORDOI_AGENT_PORTABLE_RESULT_V1"
PORTABLE_FILES = {"handoff", "native", "portable.json", "PORTABLE.sha256"}
MAX_MANIFEST = 16384


class Refused(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


class Unknown(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


def need(ok: bool, code: str) -> None:
    if not ok:
        raise Refused(code)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def serialize(obj: object) -> bytes:
    return json.dumps(obj, sort_keys=True, ensure_ascii=True,
                      separators=(",", ":")).encode("utf-8") + b"\n"


def emit(obj: dict) -> None:
    sys.stdout.buffer.write(serialize(obj))
    sys.stdout.buffer.flush()


def safe_write(path: Path, data: bytes) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    flags |= getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    fd = os.open(str(path), flags, 0o600)
    try:
        with os.fdopen(fd, "wb", closefd=True) as f:
            fd = -1
            f.write(data)
            f.flush()
            os.fsync(f.fileno())
    finally:
        if fd != -1:
            os.close(fd)


def safe_dir(parent: Path, name: str) -> Path:
    target = parent / name
    target.mkdir(mode=0o700)
    return target


def fsync_dir(path: Path) -> None:
    if os.name != "nt" and hasattr(os, "O_DIRECTORY"):
        fd = os.open(str(path), os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(fd)
        finally:
            os.close(fd)


def native_inventory(root: Path, records: list[dict]) -> list[dict]:
    # Reconstruct ONLY from verified records and internal fixed P2.7 names.
    items = []
    for index, record in enumerate(records, 1):
        if record["status"] != "COMMITTED":
            continue
        job = root / f"job-{index:04d}"
        audit.real_directory(job, "JOB_DIRECTORY_INVALID")
        b = job / "quality-decision"
        audit.real_directory(b, "NATIVE_BUNDLE_INVALID")
        for name in sorted(audit.FILE_NAMES):
            data = audit.bounded_regular_file(
                b / name, audit.MAX_FILES_BYTES, "NATIVE_FILE_INVALID")
            items.append({"path": f"job-{index:04d}/quality-decision/{name}",
                          "bytes": len(data), "sha256": digest(data)})
    return items


def parse_native_rel(path: str) -> tuple[str, str]:
    # Never use untrusted paths as filesystem references without exact matching.
    import re
    need(type(path) is str and re.fullmatch(
        r"job-000[1-8]/quality-decision/(decision\.txt|report\.txt)", path) is not None,
        "PORTABLE_NATIVE_PATH_INVALID")
    return path.split("/")[0], path.split("/")[-1]


def prepare(source_handoff: Path, source_root: Path,
            trusted_inner_sha: str) -> tuple[dict, list[dict], dict]:
    manifest, report = inner.verify_bundle(
        source_handoff, source_root, trusted_inner_sha)
    events, _ = independent.event_records(source_handoff / "native-prefix.jsonl")
    items = native_inventory(source_root, events)
    need(items == manifest["native_files"], "NATIVE_INVENTORY_MISMATCH")
    portable = {
        "schema": SCHEMA,
        "inner_manifest_sha256": trusted_inner_sha,
        "plan_sha256": manifest["plan_sha256"],
        "prefix_sha256": manifest["prefix_sha256"],
        "native_files": items,
        "events": len(events),
        "committed": report["committed_jobs_verified"],
        "skipped": report["skipped_nodes_verified"],
        "signature": "NOT_PRESENT",
        "external_pin_provenance": "NOT_ATTESTED",
        "crash_durability": "UNCLAIMED",
    }
    return portable, events, report


def pack(target: Path, source_handoff: Path,
         source_root: Path, inner_sha: str) -> dict:
    manifest, events, report = prepare(source_handoff, source_root, inner_sha)
    data = serialize(manifest)
    need(len(data) <= MAX_MANIFEST, "PORTABLE_MANIFEST_TOO_LARGE")
    destination = target.absolute()
    parent = destination.parent
    audit.real_directory(parent, "DESTINATION_PARENT_NOT_REAL_DIRECTORY")
    audit.real_directory(source_handoff, "SOURCE_HANDOFF_NOT_REAL_DIRECTORY")
    audit.real_directory(source_root, "SOURCE_ROOT_NOT_REAL_DIRECTORY")
    p = parent.resolve(strict=True)
    roots = (source_handoff.resolve(strict=True), source_root.resolve(strict=True))
    need(all(p != item and item not in p.parents for item in roots),
         "DESTINATION_OVERLAPS_SOURCE")
    need(not destination.exists() and not destination.is_symlink(),
         "PORTABLE_DESTINATION_EXISTS")
    # Collect checked source data BEFORE creating destination. If malicious
    # concurrent modification occurs, this is not a hostile-host guarantee.
    inner_files = {
        name: audit.bounded_regular_file(
            source_handoff / name,
            MAX_MANIFEST if name == "manifest.json" else independent.MAX_EVENTS_BYTES,
            "SOURCE_HANDOFF_FILE_INVALID")
        for name in sorted(inner.FILES)
    }
    original_files = {}
    for entry in manifest["native_files"]:
        job, name = parse_native_rel(entry["path"])
        raw = audit.bounded_regular_file(
            source_root / job / "quality-decision" / name,
            audit.MAX_FILES_BYTES, "SOURCE_NATIVE_FILE_INVALID")
        need(len(raw) == entry["bytes"] and digest(raw) == entry["sha256"],
             "SOURCE_NATIVE_FILE_HASH_CHANGED")
        original_files[(job, name)] = raw
    try:
        target = destination
        target.mkdir(mode=0o700)
    except FileExistsError:
        raise Refused("PORTABLE_DESTINATION_EXISTS") from None
    except OSError:
        raise Refused("PORTABLE_CREATION_DENIED") from None

    # Everything after exclusive mkdir is a publication with UNKNOWN outcome
    # on error. Never falsely claim no side effect or automatic rollback.
    try:
        handoff = safe_dir(target, "handoff")
        for name, raw in inner_files.items():
            safe_write(handoff / name, raw)
        native = safe_dir(target, "native")
        for job, name in sorted(original_files):
            job_path = native / job
            if not job_path.exists():
                safe_dir(native, job)
                safe_dir(job_path, "quality-decision")
            safe_write(job_path / "quality-decision" / name,
                       original_files[(job, name)])
        safe_write(target / "portable.json", data)
        # Prove complete nested exact content BEFORE publishing final marker.
        verified, _, _ = prepare(handoff, native, inner_sha)
        need(verified == manifest, "COPIED_PORTABLE_CHECK_FAILED")
        marker = digest(data)
        safe_write(target / "PORTABLE.sha256", (marker + "\n").encode("ascii"))
        fsync_dir(handoff)
        fsync_dir(native)
        fsync_dir(target)
    except (OSError, Refused, audit.AuditRejected, inner.Refused) as exc:
        raise Unknown("PORTABLE_PUBLICATION_INCOMPLETE") from exc
    return {"schema": RESULT, "status": "PACKED",
            "portable_sha256": marker, "native_files": len(original_files),
            "native_jobs": report["committed_jobs_verified"],
            "external_authentication": "NOT_PERFORMED",
            "crash_durability": "UNCLAIMED"}


def verify(portable: Path, expected_sha: str) -> tuple[dict, dict, list[dict]]:
    need(type(expected_sha) is str and pinned.PIN.fullmatch(expected_sha) is not None,
         "PORTABLE_EXTERNAL_PIN_REQUIRED")
    audit.real_directory(portable, "PORTABLE_NOT_REAL_DIRECTORY")
    need(audit.listed_names(portable, "PORTABLE_LISTING_FAILED") == PORTABLE_FILES,
         "PORTABLE_FILE_SET_CHANGED")
    raw = audit.bounded_regular_file(
        portable / "portable.json", MAX_MANIFEST, "PORTABLE_MANIFEST_INVALID")
    need(digest(raw) == expected_sha, "PORTABLE_EXTERNAL_PIN_MISMATCH")
    marker = audit.bounded_regular_file(
        portable / "PORTABLE.sha256", 65, "PORTABLE_MARKER_NOT_REGULAR")
    need(marker == (expected_sha + "\n").encode("ascii"),
         "PORTABLE_MARKER_MISMATCH")
    data = independent.decode_json(raw, "PORTABLE_JSON_INVALID")
    need(type(data) is dict and set(data) == {
        "schema", "inner_manifest_sha256", "plan_sha256", "prefix_sha256",
        "native_files", "events", "committed", "skipped", "signature",
        "external_pin_provenance", "crash_durability"
    } and data.get("schema") == SCHEMA, "PORTABLE_SCHEMA_INVALID")
    need(data["signature"] == "NOT_PRESENT"
         and data["external_pin_provenance"] == "NOT_ATTESTED"
         and data["crash_durability"] == "UNCLAIMED",
         "PORTABLE_TRUST_CLAIM_INVALID")
    handoff = portable / "handoff"
    native = portable / "native"
    audit.real_directory(handoff, "PORTABLE_HANDOFF_NOT_REAL_DIRECTORY")
    audit.real_directory(native, "PORTABLE_NATIVE_NOT_REAL_DIRECTORY")
    independently, events, report = prepare(
        handoff, native, data["inner_manifest_sha256"])
    need(independently == data, "PORTABLE_CONTENT_CONTRADICTION")
    return data, report, events


def restored_root_matches(root: Path, portable: Path, records: list[dict],
                          manifest: dict) -> None:
    audit.real_directory(root, "RESTORE_ROOT_NOT_REAL_DIRECTORY")
    # APP-010 is the stricter historical semantics/receipt/file audit.
    report, code = independent.audit(
        root, portable / "handoff" / "plan.json",
        portable / "handoff" / "native-prefix.jsonl",
        manifest["plan_sha256"])
    need(code == 3 and report["status"] == "INCONCLUSIVE"
         and report["committed_jobs_verified"] == manifest["committed"],
         "RESTORED_NATIVE_AUDIT_FAILED")
    need(native_inventory(root, records) == manifest["native_files"],
         "RESTORED_NATIVE_HASH_CHANGED")


def restore(portable: Path, expected_sha: str, dest: Path) -> dict:
    manifest, _, events = verify(portable, expected_sha)
    audit.real_directory(dest, "RESTORE_ROOT_NOT_REAL_DIRECTORY")
    need(not audit.listed_names(dest, "RESTORE_LISTING_FAILED"),
         "RESTORE_ROOT_NOT_EMPTY")
    a = portable.resolve(strict=True)
    b = dest.resolve(strict=True)
    need(a != b and a not in b.parents and b not in a.parents,
         "RESTORE_OVERLAPS_PORTABLE")
    original = {}
    for item in manifest["native_files"]:
        job, name = parse_native_rel(item["path"])
        raw = audit.bounded_regular_file(
            portable / "native" / job / "quality-decision" / name,
            audit.MAX_FILES_BYTES, "PORTABLE_NATIVE_CHANGED")
        need(digest(raw) == item["sha256"] and len(raw) == item["bytes"],
             "PORTABLE_NATIVE_HASH_CHANGED")
        original[(job, name)] = raw

    try:
        for job, name in sorted(original):
            p = dest / job
            if not p.exists():
                safe_dir(dest, job)
                safe_dir(p, "quality-decision")
            safe_write(p / "quality-decision" / name, original[(job, name)])
        restored_root_matches(dest, portable, events, manifest)
        fsync_dir(dest)
    except (OSError, Refused, audit.AuditRejected) as exc:
        raise Unknown("RESTORE_PARTIAL_OR_UNKNOWN") from exc
    return {"schema": RESULT, "status": "RESTORED",
            "native_jobs": manifest["committed"],
            "native_files": len(original),
            "external_authentication": "NOT_PERFORMED",
            "crash_durability": "UNCLAIMED"}


def resume_native(portable: Path, expected_sha: str, restored: Path,
                  engine: Path, source: Path,
                  engine_sha: str, source_sha: str) -> int:
    manifest, _, events = verify(portable, expected_sha)
    need(type(engine_sha) is str and pinned.PIN.fullmatch(engine_sha) is not None
         and type(source_sha) is str and pinned.PIN.fullmatch(source_sha) is not None,
         "ENGINE_SOURCE_PIN_INVALID")
    restored_root_matches(restored, portable, events, manifest)
    # Real APP-011 will itself authenticate engine/source PINs and check the
    # clean historical prefix and file bundle again before effect.
    cmd = [sys.executable, "-B", str(HERE.parent /
            "app011_verified_resume/run_app011.py"),
           "--plan", str(portable / "handoff" / "plan.json"),
           "--prefix-events", str(portable / "handoff" / "native-prefix.jsonl"),
           "--grant-output-dir", str(restored),
           "--engine", str(engine), "--source-root", str(source),
           "--expected-plan-sha256", manifest["plan_sha256"],
           "--expected-prefix-sha256", manifest["prefix_sha256"],
           "--expected-engine-sha256", engine_sha,
           "--expected-source-sha256", source_sha]
    try:
        proc = subprocess.run(cmd, cwd=HERE.parents[2],
                              capture_output=True, timeout=990)
    except (OSError, subprocess.TimeoutExpired):
        raise Unknown("RESUME_NATIVE_OUTCOME_UNKNOWN") from None
    sys.stdout.buffer.write(proc.stdout)
    sys.stdout.buffer.flush()
    if proc.returncode in (0, 2, 3):
        return proc.returncode
    raise Unknown("RESUME_NATIVE_ABNORMAL_EXIT")


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("action", choices=("pack", "verify", "restore", "resume"))
    p.add_argument("--portable-dir", required=True)
    p.add_argument("--expected-portable-sha256")
    p.add_argument("--handoff-dir")
    p.add_argument("--native-root")
    p.add_argument("--expected-handoff-sha256")
    p.add_argument("--restore-root")
    p.add_argument("--engine")
    p.add_argument("--source-root")
    p.add_argument("--expected-engine-sha256")
    p.add_argument("--expected-source-sha256")
    a = p.parse_args(argv)
    portable = Path(a.portable_dir)
    try:
        if a.action == "pack":
            need(bool(a.handoff_dir and a.native_root
                      and a.expected_handoff_sha256),
                 "PACK_ARGUMENTS_MISSING")
            emit(pack(portable, Path(a.handoff_dir), Path(a.native_root),
                      a.expected_handoff_sha256))
            return 0
        need(bool(a.expected_portable_sha256), "PORTABLE_PIN_REQUIRED")
        if a.action == "verify":
            manifest, _, _ = verify(portable, a.expected_portable_sha256)
            emit({"schema": RESULT, "status": "VERIFIED",
                  "portable_sha256": a.expected_portable_sha256,
                  "native_files": len(manifest["native_files"]),
                  "native_jobs": manifest["committed"],
                  "external_authentication": "NOT_PERFORMED"})
            return 0
        need(bool(a.restore_root), "RESTORE_ROOT_REQUIRED")
        if a.action == "restore":
            emit(restore(portable, a.expected_portable_sha256,
                         Path(a.restore_root)))
            return 0
        need(bool(a.engine and a.source_root
                  and a.expected_engine_sha256
                  and a.expected_source_sha256),
             "RESUME_ARGUMENTS_MISSING")
        return resume_native(portable, a.expected_portable_sha256,
                             Path(a.restore_root), Path(a.engine),
                             Path(a.source_root),
                             a.expected_engine_sha256,
                             a.expected_source_sha256)
    except Unknown as exc:
        emit({"schema": RESULT, "status": "ERROR", "error_code": exc.code,
              "publication": "UNKNOWN"})
        return 3
    except (Refused, inner.Refused, pinned.Denied,
            audit.AuditRejected, OSError, ValueError) as exc:
        emit({"schema": RESULT, "status": "REJECTED",
              "error_code": getattr(exc, "code", "PORTABLE_INPUT_REJECTED"),
              "publication": "NOT_STARTED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
