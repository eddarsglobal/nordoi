#!/usr/bin/env python3
"""R4Z: offline enrollment handoff INTEROPERABILITY only, NEVER an admission oracle.

Requests/replies carry untrusted reference hashes, not verified authority.  This
utility never reads private keys, checks remote identities or grants host effects.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import re
import secrets
import stat
import sys
import time

PROJECT = "eddarsglobal/nordoi"
PROTOCOL = "NORDOI-G03-R4Z-OFFLINE-HANDOFF-V1"
ORIGIN_HEAD = "be462ead174dec4332882cce7620448a9db41d34"
MAX_TTL = 300
MAX_CLOCK_SKEW = 30
FLOOR_EPOCH = 4
MAX_BYTES = 8192
HEX64 = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
CONTROLLER = re.compile(r"[A-Za-z0-9][A-Za-z0-9._@/-]{0,79}\Z", re.ASCII)
REQ_FIELDS = ("protocol", "project", "origin_commit", "publisher_controller",
              "owner_pin_sha256", "guardian_pin_sha256", "nonce", "issued_unix",
              "expires_unix", "floor_epoch", "binding_sha256")
REPLY_FIELDS = ("protocol", "request_binding_sha256", "guardian_pin_sha256",
                "candidate_controller", "guardian_signature_sha256",
                "owner_oob_receipt_sha256", "witness_head_sha256", "witness_epoch")

class Invalid(ValueError):
    pass

def unique_pairs(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise Invalid("DUPLICATE_FIELD")
        obj[key] = value
    return obj

def reject_constant(_):
    raise Invalid("NONFINITE_NUMBER")

def canonical(obj):
    return (json.dumps(obj, ensure_ascii=True, sort_keys=True,
                       separators=(",", ":"), allow_nan=False) + "\n").encode("ascii")

def digest(data):
    return hashlib.sha256(data).hexdigest()

def is_sha(value):
    return isinstance(value, str) and bool(HEX64.fullmatch(value)) and value != "0" * 64

def is_controller(value):
    return isinstance(value, str) and bool(CONTROLLER.fullmatch(value))

def strict_object(data, fields):
    if not isinstance(data, bytes) or not data or len(data) > MAX_BYTES:
        raise Invalid("MISSING_OR_OVERSIZED")
    try:
        doc = json.loads(data.decode("utf-8"), object_pairs_hook=unique_pairs,
                         parse_constant=reject_constant)
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise Invalid("INVALID_UTF8_OR_JSON") from exc
    if not isinstance(doc, dict) or set(doc) != set(fields):
        raise Invalid("UNEXPECTED_FIELDS")
    if canonical(doc) != data:
        raise Invalid("NONCANONICAL_ENCODING")
    return doc

def exact_int(v):
    return type(v) is int

def make_request(publisher, owner_pin, guardian_pin, epoch=FLOOR_EPOCH,
                 lifetime=120, now=None, nonce=None):
    if not is_controller(publisher):
        raise Invalid("PUBLISHER_ID_INVALID")
    if not is_sha(owner_pin) or not is_sha(guardian_pin) or owner_pin == guardian_pin:
        raise Invalid("UNTRUSTED_KEY_PINS_INVALID_OR_SAME")
    if not exact_int(epoch) or epoch < FLOOR_EPOCH:
        raise Invalid("EPOCH_DOWNGRADE")
    if not exact_int(lifetime) or not 1 <= lifetime <= MAX_TTL:
        raise Invalid("TTL_OUT_OF_BOUNDS")
    if now is None:
        now = int(time.time())
    if not exact_int(now) or now <= 0:
        raise Invalid("CLOCK_INVALID")
    if nonce is None:
        nonce = secrets.token_hex(32)
    if not is_sha(nonce):
        raise Invalid("CHALLENGE_INVALID")
    payload = dict(protocol=PROTOCOL, project=PROJECT, origin_commit=ORIGIN_HEAD,
                   publisher_controller=publisher, owner_pin_sha256=owner_pin,
                   guardian_pin_sha256=guardian_pin, nonce=nonce, issued_unix=now,
                   expires_unix=now + lifetime, floor_epoch=epoch)
    payload["binding_sha256"] = digest(canonical(payload))
    return canonical(payload)

def verify_request(data, now=None):
    req = strict_object(data, REQ_FIELDS)
    if req["protocol"] != PROTOCOL or req["project"] != PROJECT or req["origin_commit"] != ORIGIN_HEAD:
        raise Invalid("UNEXPECTED_PROJECT_VERSION_OR_PARENT")
    if not is_controller(req["publisher_controller"]):
        raise Invalid("PUBLISHER_ID_INVALID")
    if not is_sha(req["owner_pin_sha256"]) or not is_sha(req["guardian_pin_sha256"]):
        raise Invalid("PINS_INVALID")
    if req["owner_pin_sha256"] == req["guardian_pin_sha256"]:
        raise Invalid("SAME_CRYPTOGRAPHIC_PIN")
    if not is_sha(req["nonce"]):
        raise Invalid("CHALLENGE_INVALID")
    if not exact_int(req["floor_epoch"]) or req["floor_epoch"] < FLOOR_EPOCH:
        raise Invalid("EPOCH_DOWNGRADE")
    start, end = req["issued_unix"], req["expires_unix"]
    if not exact_int(start) or not exact_int(end) or start <= 0 or not 1 <= end - start <= MAX_TTL:
        raise Invalid("TTL_OUT_OF_BOUNDS")
    if now is None:
        now = int(time.time())
    if not exact_int(now) or now < start - MAX_CLOCK_SKEW or now > end:
        raise Invalid("CHALLENGE_NOT_CURRENT")
    binding = req.pop("binding_sha256")
    if not is_sha(binding) or digest(canonical(req)) != binding:
        raise Invalid("BINDING_MISMATCH")
    req["binding_sha256"] = binding
    return req

def verify_reply(data, req, publisher_witness_floor):
    rep = strict_object(data, REPLY_FIELDS)
    if rep["protocol"] != PROTOCOL:
        raise Invalid("WRONG_REPLY_PROTOCOL")
    for field in ("request_binding_sha256", "guardian_pin_sha256", "guardian_signature_sha256",
                  "owner_oob_receipt_sha256", "witness_head_sha256"):
        if not is_sha(rep[field]):
            raise Invalid("REPLY_REFERENCE_INVALID")
    if rep["request_binding_sha256"] != req["binding_sha256"]:
        raise Invalid("DIFFERENT_REQUEST_OR_REPLAY")
    if rep["guardian_pin_sha256"] != req["guardian_pin_sha256"]:
        raise Invalid("GUARDIAN_KEY_SUBSTITUTION")
    if not is_controller(rep["candidate_controller"]):
        raise Invalid("CONTROLLER_ID_INVALID")
    if rep["candidate_controller"].casefold() == req["publisher_controller"].casefold():
        raise Invalid("SAME_DECLARED_ADMINISTRATOR")
    if not exact_int(publisher_witness_floor) or publisher_witness_floor < FLOOR_EPOCH:
        raise Invalid("UNTRUSTED_WITNESS_FLOOR")
    if not exact_int(rep["witness_epoch"]) or rep["witness_epoch"] < max(req["floor_epoch"], publisher_witness_floor):
        raise Invalid("WITNESS_EPOCH_ROLLBACK")
    return rep

def assessment(request_bytes, reply_bytes=None, now=None, witness_floor=FLOOR_EPOCH):
    try:
        req = verify_request(request_bytes, now=now)
        if reply_bytes is None:
            return {"status": "BLOCK", "reason": "REAL_GUARDIAN_RECEIPT_MISSING",
                    "admission": "NOT_AUTHORIZED"}
        verify_reply(reply_bytes, req, publisher_witness_floor=witness_floor)
    except (Invalid, TypeError, OverflowError) as exc:
        return {"status": "BLOCK", "reason": "INVALID_UNTRUSTED_HANDOFF",
                "detail": str(exc), "admission": "NOT_AUTHORIZED"}
    return {"status": "BLOCK", "reason": "INDEPENDENT_PROVENANCE_NOT_EXTERNALLY_VERIFIED",
            "admission": "NOT_AUTHORIZED", "owner_oob_root": "NOT_VERIFIED",
            "guardian_independence": "NOT_VERIFIED", "real_signatures": "NOT_VERIFIED",
            "independent_monotonic_highwater": "NOT_VERIFIED",
            "durable_replay_ledger": "NOT_VERIFIED"}

def read_regular(path):
    if stat.S_ISLNK(os.lstat(path).st_mode):
        raise Invalid("SYMLINK_FORBIDDEN")
    fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
    try:
        if not stat.S_ISREG(os.fstat(fd).st_mode) or os.fstat(fd).st_size > MAX_BYTES:
            raise Invalid("UNSAFE_FILE")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            return stream.read(MAX_BYTES + 1)
    finally:
        os.close(fd)

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--request", required=True, help="canonical nonsecret request file")
    parser.add_argument("--reply", help="optional UNTRUSTED candidate references (NOT proof)")
    args = parser.parse_args(argv)
    try:
        req = read_regular(args.request)
        rep = read_regular(args.reply) if args.reply else None
        result = assessment(req, rep)
    except (Invalid, OSError) as exc:
        result = {"status": "BLOCK", "reason": "FILE_NOT_AVAILABLE_OR_UNSAFE",
                  "detail": type(exc).__name__, "admission": "NOT_AUTHORIZED"}
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
