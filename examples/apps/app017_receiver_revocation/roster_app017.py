#!/usr/bin/env python3
"""APP-017 — receiver-pinned, versioned offline policy revocation roster.

The receiver supplies an externally pinned roster of EXACTLY ONE currently
admissible APP-016 policy. The roster can be ACTIVE or FROZEN. Its immutable
SHA-256 anchor is distinct from the untrusted portable checkpoint; it may
revoke prior policy hashes. A stale but valid policy loses authorization when
a newer externally trusted roster selects another policy.

status: read-only check of roster; no source, policy or native execution.
inspect: validate roster, selected policy and native portable evidence.
verify: replay with independently pinned real Rust via APP-016/015.
restore/resume: fail closed on roster and policy, replay, recheck both
immediately before delegated APP-014/011 native effects.

Generation is advisory unless the receiver securely anchors its latest
roster hash and minimum generation OUTSIDE the candidate's control. There
is NO independently authenticated pin distribution, durable anti-rollback,
signature, OS sandbox, transaction, hostile-host concurrency guarantee,
or production admission. Frozen state refuses ALL execution/effects.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app016_receiver_trust_policy"))
import receiver_app016 as previous

portable = previous.portable
pinned = previous.pinned
audit = previous.audit

SCHEMA = "NORDOI_AGENT_RECEIVER_REVOCATION_ROSTER_V1"
RESULT = "NORDOI_AGENT_RECEIVER_ROSTER_RESULT_V1"
MAX_ROSTER_BYTES = 8192
MAX_REVOKED = 32
ROSTER_FIELDS = frozenset((
    "schema", "generation", "state", "active_policy_sha256",
    "revoked_policy_sha256", "pin_origin", "signature", "release",
    "production", "durable_anti_rollback",
))
ALL_ACTIONS = ("status", "inspect", "verify", "restore", "resume")


class Denied(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


def require(ok: bool, code: str) -> None:
    if not ok:
        raise Denied(code)


def canonical(data: dict) -> bytes:
    return json.dumps(data, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=True).encode("ascii") + b"\n"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def emit(value: dict) -> None:
    sys.stdout.buffer.write(canonical(value))
    sys.stdout.buffer.flush()


def check_pin(value: object, code: str) -> None:
    require(type(value) is str and pinned.PIN.fullmatch(value) is not None,
            code)


def load_roster(path: Path, expected_sha: str,
                minimum_generation: int) -> dict:
    check_pin(expected_sha, "ROSTER_EXTERNAL_PIN_INVALID")
    require(type(minimum_generation) is int
            and 1 <= minimum_generation <= 1_000_000,
            "ROSTER_RECEIVER_MINIMUM_GENERATION_INVALID")
    raw = audit.bounded_regular_file(
        path, MAX_ROSTER_BYTES, "ROSTER_FILE_NOT_REGULAR_BOUNDED")
    require(digest(raw) == expected_sha, "ROSTER_EXTERNAL_PIN_MISMATCH")
    obj = portable.independent.decode_json(raw, "ROSTER_NOT_VALID_JSON")
    require(type(obj) is dict and set(obj) == ROSTER_FIELDS
            and obj.get("schema") == SCHEMA,
            "ROSTER_SCHEMA_INVALID")
    generation = obj["generation"]
    require(type(generation) is int
            and 1 <= generation <= 1_000_000,
            "ROSTER_GENERATION_INVALID")
    require(generation >= minimum_generation,
            "ROSTER_BELOW_RECEIVER_MINIMUM_GENERATION")
    require(obj["state"] in ("ACTIVE", "FROZEN")
            and type(obj["state"]) is str,
            "ROSTER_STATE_INVALID")
    active = obj["active_policy_sha256"]
    if obj["state"] == "ACTIVE":
        check_pin(active, "ROSTER_ACTIVE_POLICY_SHA_INVALID")
    else:
        require(active is None, "ROSTER_FROZEN_MUST_HAVE_NO_ACTIVE_POLICY")
    revoked = obj["revoked_policy_sha256"]
    require(type(revoked) is list and len(revoked) <= MAX_REVOKED
            and all(type(x) is str
                    and pinned.PIN.fullmatch(x) is not None for x in revoked)
            and revoked == sorted(set(revoked)),
            "ROSTER_REVOKED_LIST_INVALID")
    require(active is None or active not in revoked,
            "ROSTER_ACTIVE_POLICY_IS_REVOKED")
    require(obj["pin_origin"] == "RECEIVER_SUPPLIED_NOT_EXTERNALLY_ATTESTED"
            and obj["signature"] == "NOT_PRESENT"
            and obj["release"] == "RESEARCH_ONLY"
            and obj["production"] == "NOT_AUTHORIZED"
            and obj["durable_anti_rollback"] == "NOT_PROVIDED",
            "ROSTER_AUTHORITY_CLAIM_INVALID")
    require(canonical(obj) == raw, "ROSTER_NOT_CANONICAL")
    return obj


def path_boundaries(roster_file: Path, policy_file: Path,
                    package: Path, native_root: Path | None) -> None:
    # Not OS-level symlink/race isolation: originals must remain intact.
    anchor = roster_file.resolve(strict=True)
    policy = policy_file.resolve(strict=True)
    package_real = package.resolve(strict=True)
    require(anchor != policy and anchor != package_real
            and package_real not in anchor.parents
            and package_real not in policy.parents,
            "ROSTER_OR_POLICY_INSIDE_UNTRUSTED_PACKAGE")
    if native_root is not None and native_root.exists():
        native = native_root.resolve(strict=True)
        require(anchor != native and policy != native
                and native not in anchor.parents and native not in policy.parents,
                "ROSTER_OR_POLICY_INSIDE_NATIVE_ROOT")


def authorize(roster_file: Path, expected_roster_sha: str,
              minimum_generation: int, policy_file: Path,
              package: Path, action: str,
              native_root: Path | None) -> tuple[dict, dict, dict]:
    roster = load_roster(roster_file, expected_roster_sha,
                         minimum_generation)
    require(roster["state"] == "ACTIVE", "ROSTER_FROZEN_DENIES_ADMISSION")
    require(roster["active_policy_sha256"] not in roster["revoked_policy_sha256"],
            "ROSTER_POLICY_REVOKED")
    path_boundaries(roster_file, policy_file, package, native_root)
    previous_action = "inspect" if action == "inspect" else action
    policy, manifest = previous.authorize(
        policy_file, roster["active_policy_sha256"], previous_action,
        package, native_root)
    return roster, policy, manifest


def recheck(roster_file: Path, anchor: str, generation: int,
            policy_file: Path, package: Path, action: str,
            native_root: Path, expected_roster: dict,
            expected_policy: dict) -> None:
    new_roster, new_policy, _ = authorize(
        roster_file, anchor, generation,
        policy_file, package, action, native_root)
    require(new_roster == expected_roster and new_policy == expected_policy,
            "ROSTER_OR_POLICY_CHANGED_DURING_REPLAY")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=ALL_ACTIONS)
    parser.add_argument("--receiver-roster", required=True)
    parser.add_argument("--expected-roster-sha256", required=True)
    parser.add_argument("--receiver-minimum-generation",
                        type=int, required=True)
    parser.add_argument("--receiver-policy")
    parser.add_argument("--portable-dir")
    parser.add_argument("--engine")
    parser.add_argument("--source-root")
    parser.add_argument("--restore-root")
    args = parser.parse_args(argv)
    roster_file = Path(args.receiver_roster)
    try:
        roster = load_roster(roster_file, args.expected_roster_sha256,
                             args.receiver_minimum_generation)
        if args.action == "status":
            emit({
                "schema": RESULT, "status": roster["state"],
                "generation": roster["generation"],
                "roster_sha256": args.expected_roster_sha256,
                "active_policy_sha256": roster["active_policy_sha256"],
                "revoked_count": len(roster["revoked_policy_sha256"]),
                "native_engine": "NOT_EXECUTED",
                "durable_anti_rollback": "NOT_PROVIDED",
                "external_pin_authentication": "NOT_PERFORMED",
                "production": "NOT_AUTHORIZED",
            })
            return 0
        require(args.receiver_policy is not None
                and args.portable_dir is not None,
                "ROSTER_RECEIVER_POLICY_AND_PACKAGE_REQUIRED")
        policy_file = Path(args.receiver_policy)
        package = Path(args.portable_dir)
        native = Path(args.restore_root) if args.restore_root is not None else None
        if args.action in ("restore", "resume"):
            require(native is not None, "ROSTER_NATIVE_DESTINATION_REQUIRED")
        current_roster, policy, manifest = authorize(
            roster_file, args.expected_roster_sha256,
            args.receiver_minimum_generation, policy_file,
            package, args.action, native)
        if args.action == "inspect":
            emit({
                "schema": RESULT, "status": "POLICY_MATCH",
                "roster_generation": current_roster["generation"],
                "roster_sha256": args.expected_roster_sha256,
                "active_policy_sha256": current_roster["active_policy_sha256"],
                "policy_id": policy["policy_id"],
                "portable_sha256": policy["portable_sha256"],
                "native_jobs_held": manifest["committed"],
                "native_engine": "NOT_EXECUTED",
                "signature": "NOT_PRESENT",
                "production": "NOT_AUTHORIZED",
            })
            return 0
        require(args.engine is not None and args.source_root is not None,
                "ROSTER_ENGINE_AND_SOURCE_REQUIRED")
        engine = Path(args.engine)
        source = Path(args.source_root)
        proof = previous.admitted_replay(policy, package, engine, source)
        if args.action == "verify":
            emit({
                "schema": RESULT, "status": "PASS",
                "roster_sha256": args.expected_roster_sha256,
                "roster_generation": current_roster["generation"],
                "active_policy_sha256": current_roster["active_policy_sha256"],
                "native_jobs_recomputed": proof["native_jobs_recomputed"],
                "native_files_bytewise_equal": proof["native_files_bytewise_equal"],
                "native_receipts_exactly_equal": proof["native_receipts_exactly_equal"],
                "external_pin_authentication": "NOT_PERFORMED",
                "durable_anti_rollback": "NOT_PROVIDED",
                "signature": "NOT_PRESENT",
                "production": "NOT_AUTHORIZED",
            })
            return 0

        recheck(roster_file, args.expected_roster_sha256,
                args.receiver_minimum_generation, policy_file, package,
                args.action, native, current_roster, policy)
        if args.action == "restore":
            restored = portable.restore(
                package, policy["portable_sha256"], native)
            restored["roster_generation"] = current_roster["generation"]
            restored["roster_sha256"] = args.expected_roster_sha256
            restored["native_replay"] = "PASS_RECEIVER_ANCHORED_NATIVE_RUST"
            emit(restored)
            return 0
        # APP-011 outputs real suffix JSONL; do not append fabricated receipt.
        return portable.resume_native(
            package, policy["portable_sha256"], native, engine, source,
            policy["engine_sha256"], policy["source_tree_sha256"])
    except (previous.replay.Uncertain, portable.Unknown) as exc:
        emit({"schema": RESULT, "status": "INCONCLUSIVE",
              "error_code": exc.code, "publication": "UNKNOWN"})
        return 3
    except (Denied, previous.Denied, previous.replay.Reject,
            portable.Refused, portable.inner.Refused, pinned.Denied,
            audit.AuditRejected, OSError, ValueError) as exc:
        emit({"schema": RESULT, "status": "REJECTED",
              "error_code": getattr(exc, "code", "ROSTER_INPUT_REJECTED"),
              "publication": "NOT_STARTED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
