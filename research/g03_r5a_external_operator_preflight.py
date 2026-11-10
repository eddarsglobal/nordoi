#!/usr/bin/env python3
"""G0.3 R5A: bounded, read-only review of UNTRUSTED external-custodian evidence.

No external operator is enrolled here; this is NOT an identity verifier.
Neither a correct digest nor a separate controller name authenticates custody.
Every execution is BLOCK / NOT_AUTHORIZED with exit status 2.
"""
import argparse
import hashlib
import json
import os
import re
import stat
import sys

SCHEMA = "NORDOI-G03-R5A-EXTERNAL-OPERATOR-PREFLIGHT-V1"
PROJECT = "eddarsglobal/nordoi"
R4Z_HEAD = "3090ceb5346e32d4922c1d55b92e4ab205e0549f"
PUBLISHER = "eddarsglobal"
MAX_BYTES = 16384
HEX = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
IDENT = re.compile(r"[A-Za-z0-9][A-Za-z0-9._@/-]{0,79}\Z", re.ASCII)
FIELDS = frozenset(("protocol", "project", "r4z_head", "candidate"))
CANDIDATE_FIELDS = frozenset((
    "publisher_controller", "operator_controller", "witness_controller",
    "operator_identity_evidence_sha256", "candidate_public_key_sha256",
    "r4z_request_binding_sha256", "owner_oob_receipt_sha256",
    "protected_witness_head_sha256", "witness_epoch",
    "revocation_policy_sha256", "recovery_policy_sha256"
))
class InvalidEvidence(ValueError):
    pass

def no_duplicate_keys(pairs):
    value = {}
    for key, part in pairs:
        if key in value:
            raise InvalidEvidence("DUPLICATE_JSON_FIELD")
        value[key] = part
    return value

def no_nonfinite(_):
    raise InvalidEvidence("NONFINITE_JSON_NUMBER")

def digest(value):
    return hashlib.sha256(value).hexdigest()

def parse(data):
    if not isinstance(data, bytes) or not data or len(data) > MAX_BYTES:
        raise InvalidEvidence("INPUT_MISSING_OR_TOO_LARGE")
    try:
        record = json.loads(data.decode("utf-8"),
                            object_pairs_hook=no_duplicate_keys,
                            parse_constant=no_nonfinite)
    except (UnicodeError, json.JSONDecodeError) as err:
        raise InvalidEvidence("INVALID_UTF8_OR_JSON") from err
    if type(record) is not dict or record.keys() != FIELDS:
        raise InvalidEvidence("INVALID_TOP_LEVEL_FIELDS")
    if record["protocol"] != SCHEMA or record["project"] != PROJECT or record["r4z_head"] != R4Z_HEAD:
        raise InvalidEvidence("WRONG_PROJECT_PROTOCOL_OR_ORIGIN")
    return record

def valid_digest(value):
    return isinstance(value, str) and bool(HEX.fullmatch(value)) and value != "0" * 64

def valid_controller(value):
    return isinstance(value, str) and bool(IDENT.fullmatch(value))

def validate_candidate(candidate):
    if type(candidate) is not dict or candidate.keys() != CANDIDATE_FIELDS:
        raise InvalidEvidence("INVALID_CANDIDATE_FIELDS")
    publisher = candidate["publisher_controller"]
    operator = candidate["operator_controller"]
    witness = candidate["witness_controller"]
    if not all(map(valid_controller, (publisher, operator, witness))):
        raise InvalidEvidence("CONTROLLER_IDENTIFIER_INVALID")
    if publisher.casefold() != PUBLISHER:
        raise InvalidEvidence("PUBLISHER_IDENTITY_SPOOFED")
    if len({publisher.casefold(), operator.casefold(), witness.casefold()}) != 3:
        raise InvalidEvidence("CLAIMED_CONTROLLER_COLLISION")
    for key in (
        "operator_identity_evidence_sha256", "candidate_public_key_sha256",
        "r4z_request_binding_sha256", "owner_oob_receipt_sha256",
        "protected_witness_head_sha256", "revocation_policy_sha256",
        "recovery_policy_sha256"
    ):
        if not valid_digest(candidate[key]):
            raise InvalidEvidence("INVALID_DIGEST_REFERENCE")
    if (candidate["operator_identity_evidence_sha256"]
            == candidate["owner_oob_receipt_sha256"]):
        raise InvalidEvidence("IDENTITY_AND_OWNER_RECEIPT_NOT_DISTINCT")
    epoch = candidate["witness_epoch"]
    if type(epoch) is not int or epoch < 4 or epoch > 9223372036854775807:
        raise InvalidEvidence("WITNESS_EPOCH_INVALID")
    # Deliberately no trust promotion. All fields above are self-asserted.
    return None

def assess(data):
    try:
        record = parse(data)
        if record["candidate"] is None:
            return {"status": "BLOCK", "reason": "NO_REAL_EXTERNAL_CUSTODIAN_EVIDENCE",
                    "admission": "NOT_AUTHORIZED",
                    "manifest_sha256": digest(data),
                    "external_operator": "NOT_ENROLLED"}
        validate_candidate(record["candidate"])
    except (InvalidEvidence, TypeError, OverflowError) as err:
        return {"status": "BLOCK", "reason": "UNTRUSTED_EVIDENCE_INVALID",
                "detail": str(err), "admission": "NOT_AUTHORIZED",
                "manifest_sha256": digest(data) if isinstance(data, bytes) else None}
    return {"status": "BLOCK", "reason": "INDEPENDENT_CUSTODY_NOT_EXTERNALLY_AUTHENTICATED",
            "admission": "NOT_AUTHORIZED", "manifest_sha256": digest(data),
            "external_operator": "NOT_VERIFIED", "owner_oob_root": "NOT_VERIFIED",
            "independent_witness": "NOT_VERIFIED",
            "durable_nonce_consumption": "NOT_VERIFIED",
            "protected_verifier": "NOT_VERIFIED",
            "independent_recovery": "NOT_VERIFIED"}

def read_regular(path):
    if stat.S_ISLNK(os.lstat(path).st_mode):
        raise InvalidEvidence("SYMLINK_INPUT_FORBIDDEN")
    fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
    try:
        meta = os.fstat(fd)
        if not stat.S_ISREG(meta.st_mode) or meta.st_size > MAX_BYTES:
            raise InvalidEvidence("NOT_BOUNDED_REGULAR_FILE")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            return stream.read(MAX_BYTES + 1)
    finally:
        os.close(fd)

def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--manifest", required=True)
    args = ap.parse_args(argv)
    try:
        result = assess(read_regular(args.manifest))
    except (OSError, InvalidEvidence) as err:
        result = {"status": "BLOCK", "reason": "EVIDENCE_SOURCE_UNAVAILABLE_OR_UNSAFE",
                  "detail": type(err).__name__, "admission": "NOT_AUTHORIZED"}
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
