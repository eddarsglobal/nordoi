#!/usr/bin/env python3
"""NORDOI R5C: offline root-pin consistency check, NEVER custody enrollment.

Both candidate public key and purported owner-OOB anchor are UNTRUSTED.
A hash match cannot prove how the owner acquired the anchor. Always BLOCK.
"""
import argparse
import base64
import binascii
import hashlib
import hmac
import json
import os
from pathlib import Path
import re
import stat
import sys
import time

PROJECT = "eddarsglobal/nordoi"
R5B_HEAD = "2752eb9f5fe71ce212d9728c43a4fc486728e86c"
GUARDIAN_PROTOCOL = "NORDOI-R5C-GUARDIAN-KEY-REF-V1"
ANCHOR_PROTOCOL = "NORDOI-R5C-OWNER-OOB-ANCHOR-V1"
MAX_BYTES = 8192
MAX_TTL = 300
HEX64 = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
IDENT = re.compile(r"[A-Za-z0-9][A-Za-z0-9._@/-]{0,79}\Z", re.ASCII)
B64 = re.compile(r"[A-Za-z0-9+/]+={0,2}\Z", re.ASCII)
GUARDIAN_FIELDS = frozenset((
    "protocol", "project", "r5b_head", "guardian_controller",
    "guardian_public_key", "r5b_bundle_sha256", "r4z_request_sha256"
))
ANCHOR_FIELDS = frozenset((
    "protocol", "project", "r5b_head", "guardian_controller",
    "key_pin_sha256", "r5b_bundle_sha256", "r4z_request_sha256",
    "challenge_nonce", "issued_unix", "expires_unix", "epoch",
    "acquisition_channel_claim"
))

class InvalidEvidence(ValueError):
    pass

def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise InvalidEvidence("DUPLICATE_JSON_FIELD")
        result[key] = value
    return result

def reject_nonfinite(_):
    raise InvalidEvidence("NONFINITE_NUMBER")

def sha256(value):
    return hashlib.sha256(value).hexdigest()

def is_digest(value):
    return isinstance(value, str) and bool(HEX64.fullmatch(value)) and value != "0"*64

def canonical(data):
    return (json.dumps(data, ensure_ascii=True, sort_keys=True,
                       separators=(",", ":"), allow_nan=False) + "\n").encode("ascii")

def read_json(data, fields):
    if not isinstance(data, bytes) or not data or len(data) > MAX_BYTES:
        raise InvalidEvidence("INPUT_SIZE_INVALID")
    try:
        obj = json.loads(data.decode("utf-8"), object_pairs_hook=unique_object,
                         parse_constant=reject_nonfinite)
    except (UnicodeError, json.JSONDecodeError) as e:
        raise InvalidEvidence("INVALID_UTF8_OR_JSON") from e
    if type(obj) is not dict or obj.keys() != fields:
        raise InvalidEvidence("EXACT_FIELDS_REQUIRED")
    if data != canonical(obj):
        raise InvalidEvidence("NONCANONICAL_JSON")
    return obj

def public_key(value):
    if not isinstance(value, str):
        raise InvalidEvidence("KEY_NOT_STRING")
    parts = value.split(" ")
    if len(parts) != 2 or parts[0] != "ssh-ed25519" or not B64.fullmatch(parts[1]):
        raise InvalidEvidence("NONCANONICAL_OPENSSH_KEY")
    try:
        raw = base64.b64decode(parts[1], validate=True)
    except (binascii.Error, ValueError) as e:
        raise InvalidEvidence("INVALID_KEY_BASE64") from e
    if (len(raw) != 51 or raw[:4] != b"\x00\x00\x00\x0b" or
        raw[4:15] != b"ssh-ed25519" or raw[15:19] != b"\x00\x00\x00\x20" or
        base64.b64encode(raw).decode("ascii") != parts[1]):
        raise InvalidEvidence("INVALID_ED25519_WIRE_KEY")
    return value

def check(owner_anchor, candidate, expected_nonce, expected_epoch, now=None):
    c = read_json(candidate, GUARDIAN_FIELDS)
    a = read_json(owner_anchor, ANCHOR_FIELDS)
    if (a["protocol"] != ANCHOR_PROTOCOL or c["protocol"] != GUARDIAN_PROTOCOL or
        a["project"] != PROJECT or c["project"] != PROJECT or
        a["r5b_head"] != R5B_HEAD or c["r5b_head"] != R5B_HEAD):
        raise InvalidEvidence("WRONG_PROJECT_OR_BASE")
    ctrl = c["guardian_controller"]
    if (not isinstance(ctrl, str) or not IDENT.fullmatch(ctrl) or
        ctrl.casefold() == "eddarsglobal"):
        raise InvalidEvidence("SELF_PUBLISHER_OR_INVALID_CONTROLLER")
    if a["guardian_controller"] != ctrl:
        raise InvalidEvidence("DIFFERENT_GUARDIAN_CONTROLLER")
    for key in ("r5b_bundle_sha256", "r4z_request_sha256"):
        if not is_digest(a[key]) or not is_digest(c[key]) or not hmac.compare_digest(a[key], c[key]):
            raise InvalidEvidence("HANDOFF_BINDING_MISMATCH")
    if not is_digest(expected_nonce) or a["challenge_nonce"] != expected_nonce:
        raise InvalidEvidence("OWNER_NONCE_MISMATCH")
    epoch = a["epoch"]
    if type(expected_epoch) is not int or expected_epoch < 4 or type(epoch) is not int or epoch != expected_epoch:
        raise InvalidEvidence("EPOCH_DOWNGRADE_OR_MISMATCH")
    begin, end = a["issued_unix"], a["expires_unix"]
    if type(begin) is not int or type(end) is not int or begin < 1 or not 1 <= end-begin <= MAX_TTL:
        raise InvalidEvidence("INVALID_CHALLENGE_TTL")
    if now is None:
        now = int(time.time())
    if type(now) is not int or now < begin or now > end:
        raise InvalidEvidence("EXPIRED_OR_FUTURE_CHALLENGE")
    if a["acquisition_channel_claim"] != "OWNER_OFFLINE_RECORD":
        raise InvalidEvidence("NO_EXPLICIT_UNVERIFIED_CHANNEL_CLAIM")
    key = public_key(c["guardian_public_key"])
    calculated = sha256(key.encode("ascii"))
    if not is_digest(a["key_pin_sha256"]) or not hmac.compare_digest(a["key_pin_sha256"], calculated):
        raise InvalidEvidence("OWNER_ANCHOR_PIN_MISMATCH")
    return {"pin_consistency": "MATCH_ONLY_NOT_AUTHENTICATED",
            "r5b_bundle_binding": "MATCH_ONLY_NOT_AUTHENTICATED",
            "guardian_control": "NOT_INDEPENDENTLY_VERIFIED",
            "owner_oob_channel": "NOT_AUTHENTICATED",
            "external_witness": "NOT_PROTECTED",
            "nonce_consumption": "NOT_DURABLY_RECORDED"}

def assessment(owner_anchor, candidate, nonce, epoch, now=None):
    try:
        consistency = check(owner_anchor, candidate, nonce, epoch, now)
    except (InvalidEvidence, TypeError, ValueError, OverflowError) as exc:
        return {"status": "BLOCK", "reason": "INVALID_UNTRUSTED_OOB_CANDIDATE",
                "detail": str(exc), "admission": "NOT_AUTHORIZED"}
    return {"status": "BLOCK", "reason": "ROOT_CONSISTENCY_CANNOT_PROVE_OOB_PROVENANCE",
            "admission": "NOT_AUTHORIZED", **consistency}

def read_regular(path):
    mode = os.lstat(path).st_mode
    if not stat.S_ISREG(mode):
        raise InvalidEvidence("NONREGULAR_OR_SYMLINK")
    fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
                 | getattr(os, "O_NONBLOCK", 0))
    try:
        meta = os.fstat(fd)
        if not stat.S_ISREG(meta.st_mode) or not 0 < meta.st_size <= MAX_BYTES:
            raise InvalidEvidence("UNSAFE_FILE")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            data = stream.read(MAX_BYTES + 1)
        if len(data) > MAX_BYTES:
            raise InvalidEvidence("OVERSIZE_AFTER_OPEN")
        return data
    finally:
        os.close(fd)

def owner_anchor_outside_repo(anchor_path, repo_root):
    """Directory disjointness is a hygiene check, NOT proof of channel provenance."""
    anchor = Path(anchor_path).resolve(strict=True)
    root = Path(repo_root).resolve(strict=True)
    if anchor == root or root in anchor.parents:
        raise InvalidEvidence("OWNER_ANCHOR_INSIDE_REPOSITORY")
    return True

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--owner-anchor", required=True)
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--expected-nonce", required=True)
    parser.add_argument("--expected-epoch", type=int, required=True)
    args = parser.parse_args(argv)
    try:
        owner_anchor_outside_repo(args.owner_anchor, args.repo_root)
        result = assessment(read_regular(args.owner_anchor), read_regular(args.candidate),
                            args.expected_nonce, args.expected_epoch)
    except (OSError, InvalidEvidence) as exc:
        result = {"status": "BLOCK", "reason": "UNSAFE_OR_MISSING_OOB_SOURCE",
                  "detail": type(exc).__name__, "admission": "NOT_AUTHORIZED"}
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
