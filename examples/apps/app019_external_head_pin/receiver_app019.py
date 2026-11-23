#!/usr/bin/env python3
"""NORDOI APP-019 — receiver-supplied HEAD pin gates APP-018 native admission.

A receiver retains the SHA-256 of a verified APP-018 HEAD.json OUTSIDE the
candidate's control and supplies it separately for EACH admission. Under the
APP-018 local lock, compare that exact pin, verify the complete append-only
journal, then authorize the selected receiver roster/policy before native Rust.

capture/bootstrap/advance emit UNAUTHENTICATED PIN CANDIDATES to be anchored
separately. A local file, a CLI argument, or two agreeing hashes do NOT
authenticate the source of the pin. If the attacker controls BOTH the journal
and the supplied pin, a coherent rollback remains possible. No signature,
TPM, remote witness, OS sandbox, hostile-host defense, crash certification,
global anti-rollback, native privilege, external attestation, or production.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app018_local_freshness"))
import freshness_app018 as anchor

roster = anchor.roster
portable = anchor.portable
pinned = anchor.pinned
audit = anchor.audit
Denied = anchor.Denied
Unknown = anchor.Unknown
require = anchor.require

SCHEMA = "NORDOI_RECEIVER_EXTERNAL_HEAD_PIN_V1"
ACTIONS = ("bootstrap", "capture", "advance", "status", "inspect",
           "verify", "restore", "resume")
PIN_CHANNEL = "RECEIVER_SUPPLIED_NOT_AUTHENTICATED_BY_NORDOI"


def receipt(status: str, head: dict, head_sha: str, **additional) -> dict:
    return {
        "schema": SCHEMA, "status": status,
        "head_sha256": head_sha,
        "roster_sha256": head["roster_sha256"],
        "roster_generation": head["roster_generation"],
        "roster_state": head["roster_state"],
        "journal_sequence": head["sequence"],
        "pin_channel": PIN_CHANNEL,
        "external_pin_authentication": "NOT_PERFORMED",
        "external_security_certification": "NOT_CLAIMED",
        "global_anti_rollback": "NOT_PROVIDED",
        "host_adversary_protection": "NOT_PROVIDED",
        "power_loss_certification": "NOT_PROVIDED",
        "signature": "NOT_PRESENT",
        "production": "NOT_AUTHORIZED",
        **additional,
    }


def head_bytes_locked(root: Path) -> bytes:
    return audit.bounded_regular_file(
        root / "HEAD.json", anchor.MAX_HEAD_BYTES,
        "HEAD_PIN_SOURCE_NOT_REGULAR_BOUNDED")


def read_trusted_locked(root: Path, expected_pin: str) -> tuple[dict, list[dict]]:
    require(type(expected_pin) is str
            and pinned.PIN.fullmatch(expected_pin) is not None,
            "HEAD_PIN_INVALID")
    # Full APP-018 integrity validation remains mandatory, not only HEAD hash.
    head, history = anchor.read_locked(root)
    require(anchor.digest(head_bytes_locked(root)) == expected_pin,
            "HEAD_PIN_MISMATCH")
    return head, history


def head_sha_locked(root: Path) -> str:
    return anchor.digest(head_bytes_locked(root))


def capture(root: Path) -> dict:
    # This operation deliberately does NOT assert independent authentication.
    with anchor.locked(root):
        head, _ = anchor.read_locked(root)
        return receipt("UNAUTHENTICATED_CANDIDATE", head,
                       head_sha_locked(root),
                       native_engine="NOT_EXECUTED",
                       admission="NOT_AUTHORIZED_BY_CAPTURE")


def bootstrap(root: Path, roster_file: Path, roster_sha: str,
              minimum: int) -> dict:
    head = anchor.bootstrap(root, roster_file, roster_sha, minimum)
    # The returned hash is only a candidate for trusted out-of-band enrollment.
    return receipt("BOOTSTRAPPED_PIN_CANDIDATE", head,
                   anchor.digest(anchor.canonical(head)),
                   native_engine="NOT_EXECUTED",
                   next_step="EXTERNAL_TRUST_ENROLLMENT_REQUIRED")


def advance_locked(root: Path, events: list[dict], current: dict,
                   roster_file: Path, roster_sha: str,
                   minimum: int) -> dict:
    anchor.external_root(root, roster_file)
    minimum = max(minimum, current["roster_generation"] + 1)
    new_roster = anchor.load_new_roster(roster_file, roster_sha, minimum)
    entry = anchor.record(new_roster, roster_sha, len(events) + 1,
                          current["latest_entry_sha256"])
    retired = {item for old in events
               for item in old["revoked_policy_sha256"]}
    anchor.verify_transition(events[-1], entry, retired)
    anchor.append_locked(root, entry)
    latest, _ = anchor.read_locked(root)
    return receipt("ADVANCED_PIN_CANDIDATE", latest,
                   head_sha_locked(root),
                   native_engine="NOT_EXECUTED",
                   next_step="EXTERNAL_TRUST_REPIN_REQUIRED")


def native_operation(args, root: Path, head: dict) -> int:
    require(args.receiver_roster is not None
            and args.receiver_policy is not None
            and args.portable_dir is not None, "HEAD_PIN_ADMISSION_INPUTS_REQUIRED")
    roster_file = Path(args.receiver_roster)
    policy_file = Path(args.receiver_policy)
    package = Path(args.portable_dir)
    native = Path(args.restore_root) if args.restore_root else None
    if args.action in ("restore", "resume"):
        require(native is not None, "HEAD_PIN_NATIVE_ROOT_REQUIRED")

    checked, current_roster, policy, manifest = anchor.active_locked(
        root, roster_file, args.action, policy_file, package, native)
    require(checked == head, "HEAD_PIN_JOURNAL_CHANGED_DURING_CHECK")
    if args.action == "inspect":
        anchor.emit(receipt("POLICY_MATCH", head, args.expected_head_sha256,
                            policy_id=policy["policy_id"],
                            native_jobs_held=manifest["committed"],
                            native_engine="NOT_EXECUTED"))
        return 0

    require(args.engine is not None and args.source_root is not None,
            "HEAD_PIN_ENGINE_SOURCE_REQUIRED")
    engine, source = Path(args.engine), Path(args.source_root)
    proof = roster.previous.admitted_replay(policy, package, engine, source)
    if args.action == "verify":
        anchor.emit(receipt(
            "PASS", head, args.expected_head_sha256,
            native_jobs_recomputed=proof["native_jobs_recomputed"],
            native_files_bytewise_equal=proof["native_files_bytewise_equal"],
            native_receipts_exactly_equal=proof["native_receipts_exactly_equal"],
            native_replay="PASS_RECEIVER_PINNED_RUST"))
        return 0

    # The local lock is held throughout replay, second verification and effect.
    # Recheck both the external HEAD pin and the APP018 roster/policy boundary.
    rechecked, _ = read_trusted_locked(root, args.expected_head_sha256)
    again, next_roster, next_policy, _ = anchor.active_locked(
        root, roster_file, args.action, policy_file, package, native)
    require(rechecked == head and again == head
            and next_roster == current_roster and next_policy == policy,
            "HEAD_PIN_OR_POLICY_CHANGED_BEFORE_EFFECT")
    if args.action == "restore":
        output = portable.restore(package, policy["portable_sha256"], native)
        output["external_head_sha256"] = args.expected_head_sha256
        output["head_pin"] = "RECEIVER_SUPPLIED_MATCH_NOT_AUTHENTICATED"
        output["production"] = "NOT_AUTHORIZED"
        anchor.emit(output)
        return 0
    # APP-011 native JSONL suffix only; do not append non-native receipts.
    return portable.resume_native(
        package, policy["portable_sha256"], native, engine, source,
        policy["engine_sha256"], policy["source_tree_sha256"])


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=ACTIONS)
    parser.add_argument("--anchor-root", required=True)
    parser.add_argument("--expected-head-sha256")
    parser.add_argument("--receiver-roster")
    parser.add_argument("--expected-roster-sha256")
    parser.add_argument("--receiver-minimum-generation", type=int, default=1)
    parser.add_argument("--receiver-policy")
    parser.add_argument("--portable-dir")
    parser.add_argument("--engine")
    parser.add_argument("--source-root")
    parser.add_argument("--restore-root")
    args = parser.parse_args(argv)
    root = Path(args.anchor_root)
    try:
        if args.action in ("bootstrap", "capture"):
            require(args.expected_head_sha256 is None,
                    "HEAD_PIN_UNEXPECTED_FOR_UNAUTHENTICATED_CAPTURE")
        else:
            require(args.expected_head_sha256 is not None,
                    "HEAD_PIN_EXTERNAL_REFERENCE_REQUIRED")
        if args.action in ("bootstrap", "advance"):
            require(args.receiver_roster is not None
                    and args.expected_roster_sha256 is not None,
                    "HEAD_PIN_NEW_ROSTER_EXTERNAL_REFERENCE_REQUIRED")
        else:
            require(args.expected_roster_sha256 is None,
                    "HEAD_PIN_UNEXPECTED_ROSTER_PIN")
        if args.action == "bootstrap":
            anchor.emit(bootstrap(
                root, Path(args.receiver_roster),
                args.expected_roster_sha256,
                args.receiver_minimum_generation))
            return 0
        if args.action == "capture":
            anchor.emit(capture(root))
            return 0
        with anchor.locked(root):
            head, history = read_trusted_locked(root, args.expected_head_sha256)
            if args.action == "status":
                anchor.emit(receipt(
                    head["roster_state"], head, args.expected_head_sha256,
                    local_journal="SHA256_CHAIN_VALID",
                    native_engine="NOT_EXECUTED"))
                return 0
            if args.action == "advance":
                anchor.emit(advance_locked(
                    root, history, head,
                    Path(args.receiver_roster),
                    args.expected_roster_sha256,
                    args.receiver_minimum_generation))
                return 0
            return native_operation(args, root, head)
    except (Unknown, roster.previous.replay.Uncertain,
            portable.Unknown) as exc:
        anchor.emit({
            "schema": SCHEMA, "status": "INCONCLUSIVE",
            "error_code": exc.code, "publication": "UNKNOWN",
            "production": "NOT_AUTHORIZED"})
        return 3
    except (Denied, roster.Denied, roster.previous.Denied,
            roster.previous.replay.Reject, portable.Refused,
            portable.inner.Refused, pinned.Denied,
            audit.AuditRejected, ValueError, OSError) as exc:
        anchor.emit({
            "schema": SCHEMA, "status": "REJECTED",
            "error_code": getattr(exc, "code", "HEAD_PIN_INPUT_REJECTED"),
            "publication": "NOT_STARTED", "production": "NOT_AUTHORIZED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
