#!/usr/bin/env python3
"""APP-020: SSHSIG Ed25519 proof of a receiver HEAD statement before Rust.

Security boundary: a receiver independently provisions the expected public-key
SHA-256, authority identity and minimum generation/sequence. A canonical
HEAD statement is verified with OpenSSH ssh-keygen -Y verify and is compared
under the APP-018 lock with the complete linked journal and APP-017/016 policy.
Cryptographic verification is NOT proof of external key custody, identity
enrollment, global freshness, malicious-host isolation or production admission.
An old genuinely signed statement with an old coherent journal is still valid
if the receiver rolls back its independently kept minimum sequence as well.
"""
from __future__ import annotations

import argparse
import base64
import binascii
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app019_external_head_pin"))
import receiver_app019 as previous

anchor = previous.anchor
roster = previous.roster
portable = previous.portable
pinned = previous.pinned
audit = previous.audit
Denied = previous.Denied
Unknown = previous.Unknown
require = previous.require

SCHEMA = "NORDOI_RECEIVER_SIGNED_HEAD_STATEMENT_V1"
RESULT = "NORDOI_RECEIVER_VERIFIED_SIGNED_HEAD_V1"
NAMESPACE = "nordoi-head@eddarsglobal"
AUTHORITY_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}\Z")
STATEMENT_FIELDS = frozenset((
    "schema", "authority_id", "receiver_id", "head_sha256", "roster_sha256",
    "roster_generation", "journal_sequence", "latest_entry_sha256",
    "release", "production"
))
ACTIONS = ("propose", "status", "inspect", "verify", "restore", "resume")
MAX_STATEMENT = 2048
MAX_SIGNATURE = 8192
MAX_PUBLIC_KEY = 512


def statement_for(head: dict, head_hash: str,
                  authority_id: str, receiver_id: str) -> dict:
    return {
        "schema": SCHEMA,
        "authority_id": authority_id,
        "receiver_id": receiver_id,
        "head_sha256": head_hash,
        "roster_sha256": head["roster_sha256"],
        "roster_generation": head["roster_generation"],
        "journal_sequence": head["sequence"],
        "latest_entry_sha256": head["latest_entry_sha256"],
        "release": "RESEARCH_ONLY",
        "production": "NOT_AUTHORIZED",
    }


def valid_id(s: object) -> bool:
    return type(s) is str and AUTHORITY_PATTERN.fullmatch(s) is not None


def nonoverlap(root: Path, package: Path | None, native: Path | None,
               paths: list[Path]) -> None:
    anchor.external_root(root, *paths)
    for p in paths:
        resolved = p.resolve(strict=True)
        for protected in (package, native):
            if protected is not None and protected.exists():
                bound = protected.resolve(strict=True)
                require(resolved != bound and bound not in resolved.parents
                        and resolved not in bound.parents,
                        "SSHSIG_TRUST_FILES_OVERLAP_UNTRUSTED_ROOT")


def check_pubkey(public_file: Path, expected_hash: str) -> str:
    require(type(expected_hash) is str
            and pinned.PIN.fullmatch(expected_hash) is not None,
            "SSHSIG_PUBLIC_KEY_PIN_INVALID")
    raw = audit.bounded_regular_file(
        public_file, MAX_PUBLIC_KEY, "SSHSIG_PUBLIC_KEY_NOT_REGULAR")
    require(anchor.digest(raw) == expected_hash,
            "SSHSIG_PUBLIC_KEY_PIN_MISMATCH")
    try:
        txt = raw.decode("ascii")
    except UnicodeDecodeError:
        raise Denied("SSHSIG_PUBLIC_KEY_FORMAT_INVALID") from None
    require(txt.endswith("\n") and txt.count("\n") == 1,
            "SSHSIG_PUBLIC_KEY_FORMAT_INVALID")
    fields = txt.rstrip("\n").split(" ")
    require(len(fields) == 2 and fields[0] == "ssh-ed25519"
            and re.fullmatch(r"[A-Za-z0-9+/]+={0,2}", fields[1]) is not None,
            "SSHSIG_PUBLIC_KEY_FORMAT_INVALID")
    try:
        binary = base64.b64decode(fields[1], validate=True)
    except (ValueError, binascii.Error):
        raise Denied("SSHSIG_PUBLIC_KEY_FORMAT_INVALID") from None
    def chunk(payload: bytes, offset: int) -> tuple[bytes, int]:
        if len(payload) - offset < 4:
            raise Denied("SSHSIG_PUBLIC_KEY_FORMAT_INVALID")
        length = int.from_bytes(payload[offset:offset + 4], "big")
        offset += 4
        if length > len(payload) - offset:
            raise Denied("SSHSIG_PUBLIC_KEY_FORMAT_INVALID")
        return payload[offset:offset + length], offset + length
    type_bytes, offset = chunk(binary, 0)
    key_bytes, offset = chunk(binary, offset)
    require(type_bytes == b"ssh-ed25519" and len(key_bytes) == 32
            and offset == len(binary),
            "SSHSIG_PUBLIC_KEY_FORMAT_INVALID")
    return txt


def verified_signed_head(args, root: Path) -> tuple[dict, dict]:
    require(valid_id(args.authority_id) and valid_id(args.receiver_id),
            "SSHSIG_EXPECTED_IDENTITY_INVALID")
    require(type(args.minimum_sequence) is int and
            1 <= args.minimum_sequence <= anchor.MAX_ENTRIES
            and type(args.minimum_generation) is int
            and 1 <= args.minimum_generation <= anchor.MAX_ROSTER_GENERATION,
            "SSHSIG_RECEIVER_FLOOR_INVALID")
    if not all((
        args.signed_statement, args.detached_signature,
        args.trusted_public_key, args.expected_public_key_sha256
    )):
        raise Denied("SSHSIG_RECEIVER_TRUST_INPUTS_REQUIRED")
    statement_path = Path(args.signed_statement)
    signature_path = Path(args.detached_signature)
    key_path = Path(args.trusted_public_key)
    pkg = Path(args.portable_dir) if args.portable_dir else None
    native = Path(args.restore_root) if args.restore_root else None
    nonoverlap(root, pkg, native, [statement_path, signature_path, key_path])
    key = check_pubkey(key_path, args.expected_public_key_sha256)
    raw = audit.bounded_regular_file(
        statement_path, MAX_STATEMENT, "SSHSIG_STATEMENT_NOT_REGULAR")
    claimed = anchor.decode_strict(raw, "SSHSIG_STATEMENT_NOT_CANONICAL")
    require(set(claimed) == STATEMENT_FIELDS
            and claimed.get("schema") == SCHEMA
            and claimed.get("release") == "RESEARCH_ONLY"
            and claimed.get("production") == "NOT_AUTHORIZED",
            "SSHSIG_STATEMENT_SCHEMA_INVALID")
    require(claimed["authority_id"] == args.authority_id
            and claimed["receiver_id"] == args.receiver_id,
            "SSHSIG_SIGNER_OR_RECEIVER_MISMATCH")
    for f in ("head_sha256", "roster_sha256", "latest_entry_sha256"):
        require(type(claimed[f]) is str
                and pinned.PIN.fullmatch(claimed[f]) is not None,
                "SSHSIG_STATEMENT_SHA_INVALID")
    for f, limit in (("roster_generation", anchor.MAX_ROSTER_GENERATION),
                     ("journal_sequence", anchor.MAX_ENTRIES)):
        require(type(claimed[f]) is int and 1 <= claimed[f] <= limit,
                "SSHSIG_STATEMENT_INDEX_INVALID")
    signature = audit.bounded_regular_file(
        signature_path, MAX_SIGNATURE, "SSHSIG_SIGNATURE_NOT_REGULAR")
    require(signature.startswith(b"-----BEGIN SSH SIGNATURE-----\n")
            and signature.rstrip().endswith(b"-----END SSH SIGNATURE-----"),
            "SSHSIG_SIGNATURE_FORMAT_INVALID")
    binary = shutil.which("ssh-keygen")
    require(binary is not None, "SSHSIG_VERIFIER_UNAVAILABLE")
    # The only trust identity is the receiver-supplied pinned key. Do not
    # accept caller-created allowed-signers statements or signer options.
    with tempfile.TemporaryDirectory(prefix="nordoi-sshsig-") as temp:
        allowed = Path(temp) / "allowed_signers"
        allowed.write_text(args.authority_id + " " + key, encoding="ascii")
        try:
            verdict = subprocess.run(
                [binary, "-Y", "verify", "-f", str(allowed),
                 "-I", args.authority_id, "-n", NAMESPACE,
                 "-s", str(signature_path)],
                input=raw, capture_output=True, timeout=20)
        except (OSError, subprocess.TimeoutExpired):
            raise Denied("SSHSIG_VERIFIER_UNAVAILABLE") from None
        require(verdict.returncode == 0, "SSHSIG_SIGNATURE_VERIFICATION_FAILED")
    # Freshness comparison is not derived from a signer-controlled floor.
    require(claimed["journal_sequence"] >= args.minimum_sequence
            and claimed["roster_generation"] >= args.minimum_generation,
            "SSHSIG_BELOW_RECEIVER_FLOOR")
    head, _ = anchor.read_locked(root)
    current = statement_for(
        head, previous.head_sha_locked(root),
        args.authority_id, args.receiver_id)
    require(current == claimed, "SSHSIG_SIGNED_STATE_NOT_CURRENT")
    return head, claimed


def output(status: str, head: dict, payload: dict, **more) -> dict:
    return {
        "schema": RESULT, "status": status,
        "head_sha256": payload["head_sha256"],
        "journal_sequence": head["sequence"],
        "roster_generation": head["roster_generation"],
        "authority_id": payload["authority_id"],
        "receiver_id": payload["receiver_id"],
        "crypto_signature": "SSHSIG_ED25519_VERIFIED",
        "public_key_pin": "RECEIVER_SUPPLIED_SHA256_MATCH",
        "signature_namespace": NAMESPACE,
        "independent_key_custody": "NOT_PROVEN",
        "global_anti_rollback": "NOT_PROVIDED",
        "freshness": "CONDITIONAL_RECEIVER_MINIMUM",
        "external_security_certification": "NOT_CLAIMED",
        "production": "NOT_AUTHORIZED",
        **more,
    }


def native(args, root: Path, head: dict, payload: dict) -> int:
    require(args.receiver_roster is not None
            and args.receiver_policy is not None
            and args.portable_dir is not None,
            "SSHSIG_NATIVE_INPUTS_REQUIRED")
    roster_file = Path(args.receiver_roster)
    policy_file = Path(args.receiver_policy)
    package = Path(args.portable_dir)
    native_root = Path(args.restore_root) if args.restore_root else None
    if args.action in ("restore", "resume"):
        require(native_root is not None,
                "SSHSIG_NATIVE_DESTINATION_REQUIRED")
    checked, current_roster, policy, manifest = anchor.active_locked(
        root, roster_file, args.action, policy_file, package, native_root)
    require(checked == head, "SSHSIG_JOURNAL_CHANGED_AFTER_SIGNATURE")
    if args.action == "inspect":
        anchor.emit(output(
            "POLICY_MATCH", head, payload,
            policy_id=policy["policy_id"],
            native_engine="NOT_EXECUTED", native_jobs_held=manifest["committed"]))
        return 0
    require(args.engine is not None and args.source_root is not None,
            "SSHSIG_ENGINE_SOURCE_REQUIRED")
    engine, source = Path(args.engine), Path(args.source_root)
    proof = roster.previous.admitted_replay(policy, package, engine, source)
    if args.action == "verify":
        anchor.emit(output(
            "PASS", head, payload, native_engine="REAL_RUST_EXECUTED",
            native_jobs_recomputed=proof["native_jobs_recomputed"],
            native_files_bytewise_equal=proof["native_files_bytewise_equal"],
            native_receipts_exactly_equal=proof["native_receipts_exactly_equal"]))
        return 0
    # Reverify detached signature, key, HEAD and APP-018 chain before effect.
    again_head, again_payload = verified_signed_head(args, root)
    after, new_roster, new_policy, _ = anchor.active_locked(
        root, roster_file, args.action, policy_file, package, native_root)
    require(again_head == head and again_payload == payload
            and after == head and new_roster == current_roster
            and new_policy == policy,
            "SSHSIG_CHANGED_DURING_REPLAY")
    if args.action == "restore":
        restored = portable.restore(
            package, policy["portable_sha256"], native_root)
        restored["sshsig_provenance"] = "RECEIVER_KEY_PINNED_VERIFIED"
        restored["independent_key_custody"] = "NOT_PROVEN"
        restored["production"] = "NOT_AUTHORIZED"
        anchor.emit(restored)
        return 0
    # Native APP-011 continuation emits JSONL suffix only.
    return portable.resume_native(
        package, policy["portable_sha256"], native_root, engine, source,
        policy["engine_sha256"], policy["source_tree_sha256"])


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=ACTIONS)
    parser.add_argument("--anchor-root", required=True)
    parser.add_argument("--authority-id", required=True)
    parser.add_argument("--receiver-id", required=True)
    parser.add_argument("--trusted-public-key")
    parser.add_argument("--expected-public-key-sha256")
    parser.add_argument("--signed-statement")
    parser.add_argument("--detached-signature")
    parser.add_argument("--minimum-sequence", type=int)
    parser.add_argument("--minimum-generation", type=int)
    parser.add_argument("--receiver-roster")
    parser.add_argument("--receiver-policy")
    parser.add_argument("--portable-dir")
    parser.add_argument("--engine")
    parser.add_argument("--source-root")
    parser.add_argument("--restore-root")
    args = parser.parse_args(argv)
    root = Path(args.anchor_root)
    try:
        require(valid_id(args.authority_id) and valid_id(args.receiver_id),
                "SSHSIG_EXPECTED_IDENTITY_INVALID")
        with anchor.locked(root):
            if args.action == "propose":
                # Unsigned canonical candidate, NOT an authenticated claim.
                head, _ = anchor.read_locked(root)
                anchor.emit(statement_for(
                    head, previous.head_sha_locked(root),
                    args.authority_id, args.receiver_id))
                return 0
            require(args.minimum_sequence is not None
                    and args.minimum_generation is not None,
                    "SSHSIG_RECEIVER_FLOOR_REQUIRED")
            head, payload = verified_signed_head(args, root)
            if args.action == "status":
                anchor.emit(output(
                    head["roster_state"], head, payload,
                    native_engine="NOT_EXECUTED", local_journal="SHA256_CHAIN_VALID"))
                return 0
            return native(args, root, head, payload)
    except (Unknown, roster.previous.replay.Uncertain,
            portable.Unknown) as exc:
        anchor.emit({"schema": RESULT, "status": "INCONCLUSIVE",
                     "error_code": exc.code, "publication": "UNKNOWN",
                     "production": "NOT_AUTHORIZED"})
        return 3
    except (Denied, roster.Denied, roster.previous.Denied,
            roster.previous.replay.Reject, portable.Refused,
            portable.inner.Refused, pinned.Denied, audit.AuditRejected,
            ValueError, OSError) as exc:
        anchor.emit({"schema": RESULT, "status": "REJECTED",
                     "error_code": getattr(exc, "code", "SSHSIG_INPUT_REJECTED"),
                     "publication": "NOT_STARTED",
                     "production": "NOT_AUTHORIZED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
