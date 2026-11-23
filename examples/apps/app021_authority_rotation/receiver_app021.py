#!/usr/bin/env python3
"""APP-021 — receiver-pinned signed authority rotation/revocation for APP-020.

A receiver supplies the trusted initial Ed25519 public-key SHA-256, the
SHA-256 of the CURRENT canonical authority chain, and minimum epoch, journal
sequence and roster generation, all OUTSIDE an untrusted checkpoint.

BOOTSTRAP is signed by the root key, ROTATE is signed by BOTH outgoing and
incoming SSHSIG Ed25519 keys, and FREEZE is signed by the outgoing key.
Every transition links the preceding complete record SHA-256 and cumulatively
retires outgoing keys. APP-020 signed native replay is admitted only with
the latest nonretired authority key. The complete chain and receiver floor
are checked twice under APP-018 lock around real APP-014/011 effects.

This does NOT authenticate external distribution of these receiver inputs,
provide independent key custody, protect an attacker-controlled host, or stop
rollback when journal, whole chain, externally supplied hashes AND floors are
all reverted together. No TPM, remote witness, production, or sandbox.
"""
from __future__ import annotations

import argparse
import base64
import binascii
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app020_signed_head_provenance"))
import receiver_app020 as signed

anchor = signed.anchor
roster = signed.roster
portable = signed.portable
pinned = signed.pinned
audit = signed.audit
Denied = signed.Denied
Unknown = signed.Unknown
require = signed.require

CHAIN_SCHEMA = "NORDOI_RECEIVER_AUTHORITY_ROTATION_CHAIN_V1"
EVENT_SCHEMA = "NORDOI_RECEIVER_AUTHORITY_TRANSITION_V1"
RESULT_SCHEMA = "NORDOI_RECEIVER_ROTATING_AUTHORITY_RESULT_V1"
NAMESPACE = "nordoi-authority@eddarsglobal"
MAX_CHAIN_BYTES = 131072
MAX_RECORDS = 16
CHAIN_FIELDS = frozenset(("schema", "records", "release", "production"))
RECORD_FIELDS = frozenset(("event", "outgoing_signature_b64",
                           "incoming_signature_b64"))
EVENT_FIELDS = frozenset((
    "schema", "epoch", "previous_record_sha256", "authority_id", "receiver_id",
    "state", "active_public_key_b64", "retired_public_key_sha256",
    "minimum_sequence", "minimum_generation", "transition",
    "release", "production"
))
ACTIONS = ("status", "inspect", "verify", "restore", "resume")


def strict_b64(value: object, limit: int, code: str) -> bytes:
    require(type(value) is str and 1 <= len(value) <= (limit + 2) // 3 * 4,
            code)
    try:
        raw = base64.b64decode(value, validate=True)
    except (ValueError, binascii.Error):
        raise Denied(code) from None
    require(0 < len(raw) <= limit and base64.b64encode(raw).decode("ascii") == value,
            code)
    return raw


def key_bytes(value: object) -> tuple[bytes, str]:
    content = strict_b64(value, signed.MAX_PUBLIC_KEY,
                         "AUTHORITY_ACTIVE_KEY_ENCODING_INVALID")
    # APP-020 strict Ed25519 parsing and canonical public-key validation is
    # reused; the returned hash is over the exact canonical key-file bytes.
    with tempfile.TemporaryDirectory(prefix="nordoi-authority-public-") as tmp:
        key = Path(tmp) / "public.key"
        key.write_bytes(content)
        pin = anchor.digest(content)
        signed.check_pubkey(key, pin)
    return content, pin


def verify_sshsig(message: bytes, signature_field: object,
                  key: bytes, authority: str):
    sig = strict_b64(signature_field, signed.MAX_SIGNATURE,
                     "AUTHORITY_SIGNATURE_ENCODING_INVALID")
    require(sig.startswith(b"-----BEGIN SSH SIGNATURE-----\n")
            and sig.rstrip().endswith(b"-----END SSH SIGNATURE-----"),
            "AUTHORITY_SIGNATURE_FORMAT_INVALID")
    binary = shutil.which("ssh-keygen")
    require(binary is not None, "AUTHORITY_VERIFIER_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-authority-verify-") as tmp:
        root = Path(tmp)
        allowed = root / "signers"
        signature = root / "statement.sig"
        allowed.write_bytes(authority.encode("ascii") + b" " + key)
        signature.write_bytes(sig)
        try:
            checked = subprocess.run(
                [binary, "-Y", "verify", "-f", str(allowed),
                 "-I", authority, "-n", NAMESPACE, "-s", str(signature)],
                input=message, capture_output=True, timeout=20)
        except (OSError, subprocess.TimeoutExpired):
            raise Denied("AUTHORITY_VERIFIER_UNAVAILABLE") from None
        require(checked.returncode == 0,
                "AUTHORITY_SIGNATURE_VERIFICATION_FAILED")


def chain_file(args, root: Path) -> tuple[dict, dict, bytes | None, str | None]:
    require(type(args.expected_chain_sha256) is str
            and pinned.PIN.fullmatch(args.expected_chain_sha256) is not None,
            "AUTHORITY_CHAIN_EXTERNAL_PIN_INVALID")
    require(type(args.expected_root_key_sha256) is str
            and pinned.PIN.fullmatch(args.expected_root_key_sha256) is not None,
            "AUTHORITY_ROOT_EXTERNAL_PIN_INVALID")
    require(signed.valid_id(args.authority_id)
            and signed.valid_id(args.receiver_id),
            "AUTHORITY_RECEIVER_IDENTITIES_INVALID")
    require(type(args.minimum_epoch) is int and 1 <= args.minimum_epoch <= MAX_RECORDS
            and type(args.minimum_sequence) is int
            and 1 <= args.minimum_sequence <= anchor.MAX_ENTRIES
            and type(args.minimum_generation) is int
            and 1 <= args.minimum_generation <= anchor.MAX_ROSTER_GENERATION,
            "AUTHORITY_RECEIVER_FLOOR_INVALID")
    require(args.authority_chain_file is not None,
            "AUTHORITY_CHAIN_EXTERNAL_FILE_REQUIRED")
    chain_path = Path(args.authority_chain_file)
    package = Path(args.portable_dir) if args.portable_dir else None
    native = Path(args.restore_root) if args.restore_root else None
    signed.nonoverlap(root, package, native, [chain_path])
    raw = audit.bounded_regular_file(
        chain_path, MAX_CHAIN_BYTES, "AUTHORITY_CHAIN_NOT_REGULAR_BOUNDED")
    require(anchor.digest(raw) == args.expected_chain_sha256,
            "AUTHORITY_CHAIN_EXTERNAL_PIN_MISMATCH")
    content = anchor.decode_strict(raw, "AUTHORITY_CHAIN_NOT_CANONICAL")
    require(set(content) == CHAIN_FIELDS and content.get("schema") == CHAIN_SCHEMA
            and content.get("release") == "RESEARCH_ONLY"
            and content.get("production") == "NOT_AUTHORIZED"
            and type(content.get("records")) is list
            and 1 <= len(content["records"]) <= MAX_RECORDS,
            "AUTHORITY_CHAIN_SCHEMA_INVALID")
    previous_record = None
    previous_key = None
    previous_pin = None
    retired = set()
    latest_key = None
    latest_pin = None
    last_event = None
    for number, record in enumerate(content["records"], 1):
        require(type(record) is dict and set(record) == RECORD_FIELDS
                and type(record.get("event")) is dict
                and set(record["event"]) == EVENT_FIELDS,
                "AUTHORITY_RECORD_SCHEMA_INVALID")
        event = record["event"]
        require(event.get("schema") == EVENT_SCHEMA
                and event.get("release") == "RESEARCH_ONLY"
                and event.get("production") == "NOT_AUTHORIZED"
                and event.get("authority_id") == args.authority_id
                and event.get("receiver_id") == args.receiver_id
                and type(event.get("epoch")) is int
                and event["epoch"] == number,
                "AUTHORITY_EVENT_IDENTITY_OR_EPOCH_INVALID")
        require(type(event.get("minimum_sequence")) is int
                and 1 <= event["minimum_sequence"] <= anchor.MAX_ENTRIES
                and type(event.get("minimum_generation")) is int
                and 1 <= event["minimum_generation"] <= anchor.MAX_ROSTER_GENERATION,
                "AUTHORITY_EVENT_FLOOR_INVALID")
        retired_list = event["retired_public_key_sha256"]
        require(type(retired_list) is list
                and len(retired_list) <= MAX_RECORDS
                and retired_list == sorted(set(retired_list))
                and all(type(x) is str and pinned.PIN.fullmatch(x) is not None
                        for x in retired_list),
                "AUTHORITY_RETIRED_KEYS_INVALID")
        expected_prev = (anchor.digest(anchor.canonical(previous_record))
                         if previous_record is not None else None)
        require(event["previous_record_sha256"] == expected_prev,
                "AUTHORITY_CHAIN_SHA256_LINK_INVALID")
        require(event["state"] in ("ACTIVE", "FROZEN")
                and type(event["state"]) is str,
                "AUTHORITY_STATE_INVALID")
        is_active = event["state"] == "ACTIVE"
        if is_active:
            key, key_pin = key_bytes(event["active_public_key_b64"])
        else:
            require(event["active_public_key_b64"] is None,
                    "AUTHORITY_FROZEN_MUST_HAVE_NO_ACTIVE_KEY")
            key, key_pin = None, None

        if number == 1:
            require(event["transition"] == "BOOTSTRAP" and is_active
                    and retired_list == []
                    and record["incoming_signature_b64"] is None
                    and key_pin == args.expected_root_key_sha256,
                    "AUTHORITY_BOOTSTRAP_EXTERNAL_ROOT_INVALID")
            verify_sshsig(anchor.canonical(event),
                          record["outgoing_signature_b64"],
                          key, args.authority_id)
        else:
            require(previous_key is not None and previous_pin is not None,
                    "AUTHORITY_FROZEN_CHAIN_CANNOT_ADVANCE")
            expected_retired = retired | {previous_pin}
            require(retired_list == sorted(expected_retired),
                    "AUTHORITY_REVOKED_KEY_HISTORY_MISMATCH")
            require(event["minimum_sequence"] >= last_event["minimum_sequence"]
                    and event["minimum_generation"] >= last_event["minimum_generation"]
                    and (event["minimum_sequence"] > last_event["minimum_sequence"]
                         or event["minimum_generation"] > last_event["minimum_generation"]),
                    "AUTHORITY_NONMONOTONE_RECEIVER_FLOOR")
            require(event["transition"] == ("ROTATE" if is_active else "FREEZE"),
                    "AUTHORITY_TRANSITION_INVALID")
            verify_sshsig(anchor.canonical(event),
                          record["outgoing_signature_b64"],
                          previous_key, args.authority_id)
            if is_active:
                require(key_pin not in expected_retired,
                        "AUTHORITY_REVOKED_KEY_READMITTED")
                require(record["incoming_signature_b64"] is not None,
                        "AUTHORITY_INCOMING_POSSESSION_PROOF_REQUIRED")
                verify_sshsig(anchor.canonical(event),
                              record["incoming_signature_b64"],
                              key, args.authority_id)
            else:
                require(record["incoming_signature_b64"] is None,
                        "AUTHORITY_FROZEN_CANNOT_COSIGN_NEW_KEY")
        retired = set(retired_list)
        latest_key, latest_pin = key, key_pin
        previous_record, previous_key, previous_pin = record, key, key_pin
        last_event = event
    require(last_event["epoch"] >= args.minimum_epoch,
            "AUTHORITY_EPOCH_BELOW_RECEIVER_FLOOR")
    require(last_event["minimum_sequence"] >= args.minimum_sequence
            and last_event["minimum_generation"] >= args.minimum_generation,
            "AUTHORITY_CHAIN_BELOW_RECEIVER_FLOOR")
    return content, last_event, latest_key, latest_pin


def status_output(status: str, current: dict, event: dict,
                  args, key_pin: str | None, **extra) -> dict:
    return {
        "schema": RESULT_SCHEMA, "status": status,
        "authority_epoch": event["epoch"],
        "receiver_id": args.receiver_id,
        "authority_id": args.authority_id,
        "active_key_sha256": key_pin,
        "retired_authorities": len(event["retired_public_key_sha256"]),
        "minimum_sequence": event["minimum_sequence"],
        "minimum_generation": event["minimum_generation"],
        "journal_sequence": current["sequence"],
        "roster_generation": current["roster_generation"],
        "authority_chain": "SHA256_LINKED_SSHSIG_VERIFIED",
        "key_rotation": "OUTGOING_AND_INCOMING_SSHSIG_REQUIRED",
        "independent_root_custody": "NOT_PROVEN",
        "externally_trusted_chain_pin": "RECEIVER_SUPPLIED_NOT_ATTESTED",
        "global_anti_rollback": "NOT_PROVIDED",
        "external_security_certification": "NOT_CLAIMED",
        "production": "NOT_AUTHORIZED",
        **extra,
    }


def call_signed(args, root: Path, key: bytes, key_pin: str,
                floor_seq: int, floor_gen: int):
    # An APP-021 caller cannot supply or swap the APP-020 public key.
    with tempfile.TemporaryDirectory(prefix="nordoi-authority-active-") as temp:
        key_file = Path(temp) / "receiver-active-public.key"
        key_file.write_bytes(key)
        previous_args = SimpleNamespace(**vars(args))
        previous_args.trusted_public_key = str(key_file)
        previous_args.expected_public_key_sha256 = key_pin
        previous_args.minimum_sequence = floor_seq
        previous_args.minimum_generation = floor_gen
        head, payload = signed.verified_signed_head(previous_args, root)
        if args.action == "status":
            anchor.emit(status_output(
                head["roster_state"], head,
                args._authority_event, args, key_pin,
                crypto_signature="SSHSIG_ED25519_VERIFIED",
                native_engine="NOT_EXECUTED"))
            return 0
        require(head["roster_state"] == "ACTIVE",
                "AUTHORITY_FROZEN_NATIVE_ADMISSION_DENIED")
        if args.action in ("inspect", "verify"):
            return signed.native(previous_args, root, head, payload)
        # For effects, APP-020 does not know the APP-021 authority chain;
        # therefore we MUST revalidate it here after actual Rust replay.
        require(args.receiver_roster is not None
                and args.receiver_policy is not None
                and args.portable_dir is not None,
                "AUTHORITY_NATIVE_INPUTS_REQUIRED")
        require(args.restore_root is not None,
                "AUTHORITY_NATIVE_DESTINATION_REQUIRED")
        require(args.engine is not None and args.source_root is not None,
                "AUTHORITY_ENGINE_SOURCE_REQUIRED")
        roster_file = Path(args.receiver_roster)
        policy_file = Path(args.receiver_policy)
        package = Path(args.portable_dir)
        native_root = Path(args.restore_root)
        before, old_roster, old_policy, _ = anchor.active_locked(
            root, roster_file, args.action, policy_file, package, native_root)
        require(before == head, "AUTHORITY_HEAD_CHANGED_BEFORE_REPLAY")
        engine, source = Path(args.engine), Path(args.source_root)
        roster.previous.admitted_replay(old_policy, package, engine, source)
        again, event, public, again_sha = chain_file(args, root)
        require(again == args._authority_chain
                and event == args._authority_event
                and public == key and again_sha == key_pin,
                "AUTHORITY_ROTATED_DURING_NATIVE_REPLAY")
        new_head, new_payload = signed.verified_signed_head(previous_args, root)
        after, new_roster, new_policy, _ = anchor.active_locked(
            root, roster_file, args.action, policy_file, package, native_root)
        require(new_head == head and new_payload == payload
                and after == head and new_roster == old_roster
                and new_policy == old_policy,
                "AUTHORITY_POLICY_OR_SIGNATURE_CHANGED_BEFORE_EFFECT")
        if args.action == "restore":
            receipt = portable.restore(
                package, old_policy["portable_sha256"], native_root)
            receipt["authority_epoch"] = event["epoch"]
            receipt["authority_chain"] = "SSHSIG_VERIFIED_BEFORE_EFFECT"
            receipt["independent_root_custody"] = "NOT_PROVEN"
            receipt["production"] = "NOT_AUTHORIZED"
            anchor.emit(receipt)
            return 0
        return portable.resume_native(
            package, old_policy["portable_sha256"], native_root,
            engine, source, old_policy["engine_sha256"],
            old_policy["source_tree_sha256"])


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=ACTIONS)
    parser.add_argument("--anchor-root", required=True)
    parser.add_argument("--authority-chain-file", required=True)
    parser.add_argument("--expected-chain-sha256", required=True)
    parser.add_argument("--expected-root-key-sha256", required=True)
    parser.add_argument("--authority-id", required=True)
    parser.add_argument("--receiver-id", required=True)
    parser.add_argument("--minimum-epoch", type=int, required=True)
    parser.add_argument("--minimum-sequence", type=int, required=True)
    parser.add_argument("--minimum-generation", type=int, required=True)
    parser.add_argument("--signed-statement")
    parser.add_argument("--detached-signature")
    parser.add_argument("--receiver-roster")
    parser.add_argument("--receiver-policy")
    parser.add_argument("--portable-dir")
    parser.add_argument("--engine")
    parser.add_argument("--source-root")
    parser.add_argument("--restore-root")
    args = parser.parse_args(argv)
    root = Path(args.anchor_root)
    try:
        with anchor.locked(root):
            chain, event, key, key_pin = chain_file(args, root)
            head, _ = anchor.read_locked(root)
            require(head["sequence"] >= max(event["minimum_sequence"],
                                           args.minimum_sequence)
                    and head["roster_generation"] >= max(
                        event["minimum_generation"], args.minimum_generation),
                    "AUTHORITY_JOURNAL_BELOW_CURRENT_FLOOR")
            if event["state"] == "FROZEN":
                require(head["roster_state"] == "FROZEN",
                        "AUTHORITY_FROZEN_ROSTER_MISMATCH")
                if args.action != "status":
                    raise Denied("AUTHORITY_FROZEN_DENIES_NATIVE_ADMISSION")
                anchor.emit(status_output("FROZEN", head, event, args, None,
                                          crypto_signature="AUTHORITY_FREEZE_SSHSIG_VERIFIED",
                                          native_engine="NOT_EXECUTED"))
                return 0
            require(key is not None and key_pin is not None,
                    "AUTHORITY_ACTIVE_KEY_MISSING")
            args._authority_chain = chain
            args._authority_event = event
            return call_signed(
                args, root, key, key_pin,
                max(event["minimum_sequence"], args.minimum_sequence),
                max(event["minimum_generation"], args.minimum_generation))
    except (Unknown, roster.previous.replay.Uncertain,
            portable.Unknown) as exc:
        anchor.emit({"schema": RESULT_SCHEMA, "status": "INCONCLUSIVE",
                     "error_code": exc.code, "publication": "UNKNOWN",
                     "production": "NOT_AUTHORIZED"})
        return 3
    except (Denied, roster.Denied, roster.previous.Denied,
            roster.previous.replay.Reject, portable.Refused,
            portable.inner.Refused, pinned.Denied, audit.AuditRejected,
            ValueError, OSError) as exc:
        anchor.emit({"schema": RESULT_SCHEMA, "status": "REJECTED",
                     "error_code": getattr(exc, "code", "AUTHORITY_INPUT_REJECTED"),
                     "publication": "NOT_STARTED",
                     "production": "NOT_AUTHORIZED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
