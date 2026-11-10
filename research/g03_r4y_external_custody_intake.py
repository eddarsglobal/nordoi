#!/usr/bin/env python3
"""NORDOI R4Y: read-only untrusted custody evidence intake. NO admission path."""
import argparse
import json
import os
import re
import stat
import sys
VERSION = "NORDOI-R4Y-CUSTODY-INTAKE-V1"
PROJECT = "eddarsglobal/nordoi"
GATES = ("G1", "G2", "G3", "G4", "G5", "G6")
MAX_BYTES = 32768
MAX_ITEMS = 24
HASH_RE = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
IDENT_RE = re.compile(r"[A-Za-z0-9._:/@-]{1,160}\Z", re.ASCII)
PROVENANCE = ("REPOSITORY_ONLY", "OWNER_LOCAL_RECORD", "CLAIMED_OUT_OF_BAND",
              "CLAIMED_INDEPENDENT_CONTROLLER", "CLAIMED_PROTECTED_WITNESS")
class InvalidIntake(ValueError):
    pass
def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise InvalidIntake("DUPLICATE_JSON_KEY")
        result[key] = value
    return result
def _reject_constant(value):
    raise InvalidIntake("NONFINITE_JSON_NUMBER:" + value)
def _exact_fields(value, expected):
    if not isinstance(value, dict) or set(value) != set(expected):
        raise InvalidIntake("SCHEMA_FIELDS_NOT_EXACT")
def parse_candidate_inventory(data):
    if not isinstance(data, bytes) or not data or len(data) > MAX_BYTES:
        raise InvalidIntake("INPUT_MISSING_OR_OVERSIZED")
    try:
        doc = json.loads(data.decode("utf-8"), object_pairs_hook=_unique_object,
                         parse_constant=_reject_constant)
    except UnicodeError as exc:
        raise InvalidIntake("NOT_UTF8") from exc
    except json.JSONDecodeError as exc:
        raise InvalidIntake("INVALID_JSON") from exc
    _exact_fields(doc, ("version", "project", "evidence"))
    if doc["version"] != VERSION or doc["project"] != PROJECT:
        raise InvalidIntake("UNEXPECTED_PROJECT_OR_SCHEMA_VERSION")
    items = doc["evidence"]
    if not isinstance(items, list) or len(items) > MAX_ITEMS:
        raise InvalidIntake("EVIDENCE_LIST_INVALID_OR_OVERSIZED")
    seen = set()
    for item in items:
        _exact_fields(item, ("gate", "source", "reference", "sha256"))
        gate = item["gate"]
        if not isinstance(gate, str) or gate not in GATES:
            raise InvalidIntake("UNKNOWN_EVIDENCE_GATE")
        if gate in seen:
            raise InvalidIntake("DUPLICATE_GATE_CLAIMS")
        seen.add(gate)
        if not isinstance(item["source"], str) or item["source"] not in PROVENANCE:
            raise InvalidIntake("SOURCE_CLASS_INVALID")
        if not isinstance(item["reference"], str) or not IDENT_RE.fullmatch(item["reference"]):
            raise InvalidIntake("UNSAFE_REFERENCE_IDENTIFIER")
        if not isinstance(item["sha256"], str) or not HASH_RE.fullmatch(item["sha256"]):
            raise InvalidIntake("EVIDENCE_DIGEST_NOT_CANONICAL")
    return tuple(sorted(seen))
def audit_bytes(data):
    try:
        supplied = parse_candidate_inventory(data)
    except (InvalidIntake, TypeError) as exc:
        return {"status": "BLOCK", "reason": "MALFORMED_UNTRUSTED_INTAKE",
                "detail": str(exc), "admission": "NOT_AUTHORIZED"}
    missing = [g for g in GATES if g not in supplied]
    return {"status": "BLOCK",
            "reason": "INDEPENDENT_CUSTODY_EVIDENCE_ABSENT" if missing else "CLAIMS_NOT_EXTERNALLY_AUTHENTICATED",
            "submitted_gates": list(supplied), "missing_gates": missing,
            "external_operator": "NOT_VERIFIED", "owner_out_of_band_root": "NOT_VERIFIED",
            "independent_monotonic_witness": "NOT_VERIFIED",
            "durable_nonce_ledger": "NOT_VERIFIED",
            "protected_verifier_and_recovery": "NOT_VERIFIED", "admission": "NOT_AUTHORIZED"}
def audit_file(path):
    if stat.S_ISLNK(os.lstat(path).st_mode):
        raise InvalidIntake("SYMLINK_EVIDENCE_FILE_FORBIDDEN")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    fd = os.open(path, flags)
    try:
        meta = os.fstat(fd)
        if not stat.S_ISREG(meta.st_mode) or meta.st_size > MAX_BYTES:
            raise InvalidIntake("NOT_BOUNDED_REGULAR_FILE")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            data = stream.read(MAX_BYTES + 1)
        return audit_bytes(data)
    finally:
        os.close(fd)
def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True)
    args = parser.parse_args(argv)
    try:
        result = audit_file(args.manifest)
    except (OSError, InvalidIntake) as exc:
        result = {"status": "BLOCK", "reason": "EVIDENCE_FILE_UNAVAILABLE_OR_UNSAFE",
                  "detail": type(exc).__name__, "admission": "NOT_AUTHORIZED"}
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2
if __name__ == "__main__":
    sys.exit(main())
