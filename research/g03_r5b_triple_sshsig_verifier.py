#!/usr/bin/env python3
"""NORDOI G0.3 R5B: real OpenSSH triple-SSHSIG provenance preflight, DENY ONLY.

Verifies three genuine Ed25519 SSHSIG signatures and message/pin binding when
native OpenSSH is present. A validated signature authenticates key possession,
NOT real operator identity, owner-approved out-of-band roots or independent
monotonic storage. No NORDOI production admission path exists in this module.
"""
import argparse
import base64
import binascii
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile
import time

PROTOCOL = "NORDOI-G03-R5B-TRIPLE-SSHSIG-PROVENANCE-V1"
PROJECT = "eddarsglobal/nordoi"
SOURCE_HEAD = "3f03f119fe00756d89b610295de23479d433c6a1"
MIN_EPOCH = 4
MAX_TTL = 300
MAX_SKEW = 30
MAX_BYTES = 24576
SHA256_HEX = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
B64 = re.compile(r"[A-Za-z0-9+/]+={0,2}\Z", re.ASCII)
ROLES = ("owner", "guardian", "witness")
NAMESPACES = {
    "owner": "nordoi-r5b-owner-approval@eddarsglobal",
    "guardian": "nordoi-r5b-guardian-possession@eddarsglobal",
    "witness": "nordoi-r5b-witness-head@eddarsglobal",
}
PRINCIPALS = {r: "nordoi-r5b-" + r for r in ROLES}
PAYLOAD_FIELDS = frozenset((
    "protocol", "project", "source_head", "r4z_request_sha256",
    "owner_key_sha256", "guardian_key_sha256", "witness_key_sha256",
    "nonce", "issued_unix", "expires_unix", "floor_epoch"
))
BUNDLE_FIELDS = frozenset(("payload", "public_keys", "signatures"))
PINS_FIELDS = frozenset(("owner", "guardian", "witness"))

class InvalidEvidence(ValueError):
    pass

def digest(data):
    return hashlib.sha256(data).hexdigest()

def is_digest(value):
    return isinstance(value, str) and bool(SHA256_HEX.fullmatch(value)) and value != "0" * 64

def unique_pairs(pairs):
    out = {}
    for key, value in pairs:
        if key in out:
            raise InvalidEvidence("DUPLICATE_JSON_KEY")
        out[key] = value
    return out

def no_constant(_):
    raise InvalidEvidence("NONFINITE_JSON_VALUE")

def canonical(obj):
    try:
        return (json.dumps(obj, sort_keys=True, separators=(",", ":"),
                           ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")
    except (TypeError, ValueError, OverflowError) as error:
        raise InvalidEvidence("UNSERIALIZABLE_EVIDENCE") from error

def parse_json(data):
    if not isinstance(data, bytes) or not 0 < len(data) <= MAX_BYTES:
        raise InvalidEvidence("INPUT_MISSING_OR_OVERSIZED")
    try:
        obj = json.loads(data.decode("utf-8"), object_pairs_hook=unique_pairs,
                         parse_constant=no_constant)
    except (UnicodeError, json.JSONDecodeError) as error:
        raise InvalidEvidence("NOT_CANONICAL_UTF8_JSON") from error
    if canonical(obj) != data:
        raise InvalidEvidence("NONCANONICAL_JSON")
    return obj

def key_canonical(value):
    if not isinstance(value, str):
        raise InvalidEvidence("KEY_NOT_STRING")
    parts = value.split(" ")
    if len(parts) != 2 or parts[0] != "ssh-ed25519" or not B64.fullmatch(parts[1]):
        raise InvalidEvidence("PUBLIC_KEY_NONCANONICAL")
    try:
        raw = base64.b64decode(parts[1], validate=True)
    except (binascii.Error, ValueError) as error:
        raise InvalidEvidence("PUBLIC_KEY_B64_INVALID") from error
    if len(raw) != 51 or raw[0:4] != b"\x00\x00\x00\x0b" or raw[4:15] != b"ssh-ed25519" or raw[15:19] != b"\x00\x00\x00\x20":
        raise InvalidEvidence("NOT_ED25519_SSH_WIRE_KEY")
    if base64.b64encode(raw).decode("ascii") != parts[1]:
        raise InvalidEvidence("PUBLIC_KEY_B64_NOT_CANONICAL")
    return value

def signature_bytes(encoded):
    if not isinstance(encoded, str) or not 20 < len(encoded) < 6000:
        raise InvalidEvidence("SIGNATURE_MISSING_OR_OVERSIZED")
    try:
        binary = base64.b64decode(encoded, validate=True)
    except (binascii.Error, ValueError) as error:
        raise InvalidEvidence("SIGNATURE_B64_INVALID") from error
    if not binary.startswith(b"-----BEGIN SSH SIGNATURE-----\n"):
        raise InvalidEvidence("NOT_AN_SSHSIG_SIGNATURE")
    if not binary.rstrip(b"\n").endswith(b"-----END SSH SIGNATURE-----"):
        raise InvalidEvidence("SSHSIG_FOOTER_MISSING")
    if len(binary) > 4096:
        raise InvalidEvidence("SSHSIG_TOO_LARGE")
    return binary

def check_bundle(bundle_data, expected_pins_data, expected_nonce, expected_epoch, now=None):
    bundle = parse_json(bundle_data)
    expected = parse_json(expected_pins_data)
    if type(bundle) is not dict or bundle.keys() != BUNDLE_FIELDS:
        raise InvalidEvidence("BUNDLE_FIELDS_INVALID")
    if type(expected) is not dict or expected.keys() != PINS_FIELDS:
        raise InvalidEvidence("PINS_FIELDS_INVALID")
    payload, keys, signs = bundle["payload"], bundle["public_keys"], bundle["signatures"]
    if type(payload) is not dict or payload.keys() != PAYLOAD_FIELDS:
        raise InvalidEvidence("PAYLOAD_FIELDS_INVALID")
    if type(keys) is not dict or keys.keys() != PINS_FIELDS:
        raise InvalidEvidence("KEY_ROLE_FIELDS_INVALID")
    if type(signs) is not dict or signs.keys() != PINS_FIELDS:
        raise InvalidEvidence("SIGNATURE_ROLE_FIELDS_INVALID")
    if payload["protocol"] != PROTOCOL or payload["project"] != PROJECT or payload["source_head"] != SOURCE_HEAD:
        raise InvalidEvidence("UNEXPECTED_SOURCE_OR_PROTOCOL")
    if not is_digest(payload["r4z_request_sha256"]):
        raise InvalidEvidence("R4Z_BINDING_INVALID")
    if not isinstance(expected_nonce, str) or not is_digest(expected_nonce) or payload["nonce"] != expected_nonce:
        raise InvalidEvidence("FRESH_NONCE_NOT_MATCHED")
    if type(expected_epoch) is not int or expected_epoch < MIN_EPOCH:
        raise InvalidEvidence("EXPECTED_EPOCH_INVALID")
    if type(payload["floor_epoch"]) is not int or payload["floor_epoch"] != expected_epoch:
        raise InvalidEvidence("EPOCH_DOWNGRADE_OR_MISMATCH")
    issued, expires = payload["issued_unix"], payload["expires_unix"]
    if type(issued) is not int or type(expires) is not int or issued <= 0 or not 1 <= expires - issued <= MAX_TTL:
        raise InvalidEvidence("INVALID_TIMING_WINDOW")
    if now is None:
        now = int(time.time())
    if type(now) is not int or now < issued - MAX_SKEW or now > expires:
        raise InvalidEvidence("OUTSIDE_VALIDITY_WINDOW")
    values = []
    for role in ROLES:
        key = key_canonical(keys[role])
        pin = digest(key.encode("ascii"))
        field = role + "_key_sha256"
        if not is_digest(expected[role]) or pin != expected[role] or payload[field] != pin:
            raise InvalidEvidence("PUBLIC_ROOT_PIN_MISMATCH")
        signature_bytes(signs[role])
        values.append(key)
    if len(set(values)) != 3:
        raise InvalidEvidence("SAME_PUBLIC_KEY_MULTIPLE_ROLES")
    return bundle, canonical(payload)

def verify_sshsig(payload, signature, public_key, role, ssh_keygen="/usr/bin/ssh-keygen"):
    """Actual cryptographic verification; never a claim of administrative control."""
    if role not in ROLES:
        return False
    if os.path.realpath(ssh_keygen) != "/usr/bin/ssh-keygen" or not Path(ssh_keygen).is_file():
        return False
    try:
        key_canonical(public_key)
        with tempfile.TemporaryDirectory(prefix="nordoi-r5b-verify-") as folder:
            allow = Path(folder) / "allowed_signers"
            sig = Path(folder) / "receipt.sig"
            allow.write_text(PRINCIPALS[role] + " " + public_key + "\n", encoding="ascii")
            sig.write_bytes(signature)
            result = subprocess.run(
                [ssh_keygen, "-Y", "verify", "-f", str(allow),
                 "-I", PRINCIPALS[role], "-n", NAMESPACES[role], "-s", str(sig)],
                input=payload, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                timeout=20, check=False
            )
            return result.returncode == 0
    except (OSError, ValueError, subprocess.SubprocessError, InvalidEvidence):
        return False

def assessment(bundle_data, expected_pins_data, expected_nonce, expected_epoch, now=None):
    try:
        bundle, payload = check_bundle(bundle_data, expected_pins_data, expected_nonce,
                                       expected_epoch, now=now)
        for role in ROLES:
            if not verify_sshsig(payload, signature_bytes(bundle["signatures"][role]),
                                 bundle["public_keys"][role], role):
                raise InvalidEvidence("SSHSIG_NOT_VERIFIED_" + role.upper())
    except (InvalidEvidence, TypeError, OverflowError) as error:
        return {"status": "BLOCK", "reason": "INVALID_OR_UNVERIFIED_CRYPTOGRAPHIC_RECEIPT",
                "detail": str(error), "admission": "NOT_AUTHORIZED",
                "cryptographic_verification": "NOT_VERIFIED"}
    return {"status": "BLOCK", "reason": "OPERATOR_CUSTODY_PROVENANCE_NOT_ESTABLISHED",
            "admission": "NOT_AUTHORIZED", "cryptographic_verification": "THREE_SSHSIG_VALID",
            "administrative_separation": "NOT_VERIFIED",
            "out_of_band_root": "NOT_ENROLLED",
            "protected_monotonic_witness": "NOT_IMPLEMENTED",
            "durable_consumed_nonce_ledger": "NOT_IMPLEMENTED",
            "protected_verifier_recovery": "NOT_VERIFIED"}

def read_regular(path):
    mode = os.lstat(path).st_mode
    if not stat.S_ISREG(mode):
        raise InvalidEvidence("NONREGULAR_OR_SYMLINK_INPUT")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
    fd = os.open(path, flags)
    try:
        meta = os.fstat(fd)
        if not stat.S_ISREG(meta.st_mode) or not 0 < meta.st_size <= MAX_BYTES:
            raise InvalidEvidence("UNSAFE_FILE_SIZE_OR_TYPE")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            result = stream.read(MAX_BYTES + 1)
        if len(result) > MAX_BYTES:
            raise InvalidEvidence("FILE_TOO_LARGE")
        return result
    finally:
        os.close(fd)

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", required=True)
    parser.add_argument("--expected-public-pins", required=True,
                        help="UNTRUSTED input; only owner-verified OOB pins could be authoritative")
    parser.add_argument("--expected-nonce", required=True)
    parser.add_argument("--expected-epoch", type=int, required=True)
    arguments = parser.parse_args(argv)
    try:
        result = assessment(read_regular(arguments.bundle),
                            read_regular(arguments.expected_public_pins),
                            arguments.expected_nonce, arguments.expected_epoch)
    except (InvalidEvidence, OSError) as error:
        result = {"status": "BLOCK", "reason": "NO_USABLE_EXTERNAL_EVIDENCE",
                  "detail": type(error).__name__, "admission": "NOT_AUTHORIZED",
                  "cryptographic_verification": "NOT_VERIFIED"}
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
