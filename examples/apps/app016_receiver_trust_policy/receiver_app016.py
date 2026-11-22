#!/usr/bin/env python3
"""APP-016: receiver-anchored, fail-closed native checkpoint admission policy.

Unlike sender-provided "trusted" pins, the RECEIVER supplies a separate
allowlist policy file and its expected SHA-256 via a distinct trusted channel.
Its exact engine, .noi source tree, original plan and portable checkpoint
hashes govern all admission decisions; no pin comes from a portable package.
A coherent alternate source+history cannot self-authorize via new matching
pins. The sender cannot expand policy permissions through its checkpoint.

inspect: read-only policy+portable evidence check; no native engine run.
verify: full APP-015 independent pinned Rust replay (private scratch only).
restore: replay and recheck policy BEFORE APP-014 exclusive native restoration.
resume: replay and recheck policy BEFORE APP-014/011 real native continuation.

No signatures, independent trust authority, OS sandbox, TOCTOU immunity,
all-or-nothing filesystem transaction, in-flight crash recovery or production.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app015_independent_native_replay"))
import replay_app015 as replay

portable = replay.portable
pinned = replay.pinned
audit = portable.audit

SCHEMA = "NORDOI_AGENT_RECEIVER_POLICY_V1"
RESULT = "NORDOI_AGENT_RECEIVER_ADMISSION_V1"
MAX_POLICY_BYTES = 4096
ALL_ACTIONS = ("inspect", "restore", "resume", "verify")
PIN_KEYS = ("engine_sha256", "source_tree_sha256", "portable_sha256",
            "plan_sha256")
POLICY_KEYS = frozenset((
    "schema", "policy_id", *PIN_KEYS, "allowed_actions",
    "release", "pin_origin", "signature", "production",
))


class Denied(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


def require(ok: bool, code: str) -> None:
    if not ok:
        raise Denied(code)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(data: dict) -> bytes:
    return json.dumps(data, ensure_ascii=True, sort_keys=True,
                      separators=(",", ":")).encode("ascii") + b"\n"


def emit(row: dict) -> None:
    sys.stdout.buffer.write(canonical_json(row))
    sys.stdout.buffer.flush()


def load_policy(policy_file: Path, expected_policy_sha: str) -> dict:
    require(isinstance(expected_policy_sha, str)
            and pinned.PIN.fullmatch(expected_policy_sha) is not None,
            "RECEIVER_POLICY_EXTERNAL_PIN_INVALID")
    raw = audit.bounded_regular_file(
        policy_file, MAX_POLICY_BYTES, "RECEIVER_POLICY_NOT_REGULAR_BOUNDED")
    require(digest(raw) == expected_policy_sha,
            "RECEIVER_POLICY_EXTERNAL_PIN_MISMATCH")
    content = portable.independent.decode_json(
        raw, "RECEIVER_POLICY_NOT_VALID_JSON")
    require(type(content) is dict and set(content) == POLICY_KEYS
            and content.get("schema") == SCHEMA,
            "RECEIVER_POLICY_SCHEMA_INVALID")
    require(type(content["policy_id"]) is str
            and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}",
                             content["policy_id"]) is not None,
            "RECEIVER_POLICY_ID_INVALID")
    for key in PIN_KEYS:
        require(type(content[key]) is str
                and pinned.PIN.fullmatch(content[key]) is not None,
                f"RECEIVER_POLICY_{key.upper()}_INVALID")
    allowed = content["allowed_actions"]
    require(type(allowed) is list
            and 1 <= len(allowed) <= len(ALL_ACTIONS)
            and all(type(action) is str for action in allowed)
            and allowed == sorted(set(allowed))
            and set(allowed).issubset(ALL_ACTIONS),
            "RECEIVER_POLICY_ACTIONS_INVALID")
    require(content["release"] == "RESEARCH_ONLY"
            and content["pin_origin"] == "RECEIVER_SUPPLIED_NOT_EXTERNALLY_ATTESTED"
            and content["signature"] == "NOT_PRESENT"
            and content["production"] == "NOT_AUTHORIZED",
            "RECEIVER_POLICY_AUTHORITY_CLAIM_INVALID")
    require(canonical_json(content) == raw,
            "RECEIVER_POLICY_NOT_CANONICAL")
    return content


def source_separated(policy_file: Path, package: Path,
                     restored: Path | None) -> None:
    # The receiving agent must bring the policy from OUTSIDE the package.
    # Normal path/symlink checks aren't an OS sandbox or race-proof isolation.
    p = policy_file.resolve(strict=True)
    q = package.resolve(strict=True)
    require(p != q and q not in p.parents,
            "RECEIVER_POLICY_INSIDE_UNTRUSTED_PACKAGE")
    if restored is not None and restored.exists():
        r = restored.resolve(strict=True)
        require(p != r and r not in p.parents,
                "RECEIVER_POLICY_INSIDE_NATIVE_DESTINATION")


def authorize(policy_file: Path, policy_sha: str, action: str,
              package: Path, restored: Path | None = None
              ) -> tuple[dict, dict]:
    policy = load_policy(policy_file, policy_sha)
    source_separated(policy_file, package, restored)
    require(action in policy["allowed_actions"],
            "RECEIVER_POLICY_ACTION_DENIED")
    # APP-014 validates ALL nested native receipts and real P2.7 files.
    # It accepts externally pinned content; the receiver's independently
    # supplied policy now fixes that pin, not a sender-controlled argument.
    manifest, _, _ = portable.verify(package, policy["portable_sha256"])
    require(manifest["plan_sha256"] == policy["plan_sha256"],
            "RECEIVER_POLICY_PLAN_NOT_AUTHORIZED")
    return policy, manifest


def recheck_before_effect(policy_file: Path, policy_sha: str,
                          original: dict, action: str,
                          package: Path, restored: Path | None) -> None:
    checked, _ = authorize(policy_file, policy_sha, action, package, restored)
    require(checked == original,
            "RECEIVER_POLICY_CHANGED_DURING_REPLAY")


def admitted_replay(policy: dict, package: Path, engine: Path,
                    source: Path) -> dict:
    # The engine/source pins are ALWAYS sourced from receiver policy,
    # never from the package or flags claiming to supersede that policy.
    return replay.replay(
        package, policy["portable_sha256"], engine, source,
        policy["engine_sha256"], policy["source_tree_sha256"])


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=ALL_ACTIONS)
    parser.add_argument("--receiver-policy", required=True)
    parser.add_argument("--expected-policy-sha256", required=True)
    parser.add_argument("--portable-dir", required=True)
    parser.add_argument("--engine")
    parser.add_argument("--source-root")
    parser.add_argument("--restore-root")
    args = parser.parse_args(argv)
    policy_file, package = Path(args.receiver_policy), Path(args.portable_dir)
    restored = Path(args.restore_root) if args.restore_root else None
    action = args.action
    try:
        if action in ("restore", "resume"):
            require(restored is not None, "RECEIVER_RESTORE_ROOT_REQUIRED")
        policy, manifest = authorize(
            policy_file, args.expected_policy_sha256,
            action, package, restored)
        if action == "inspect":
            emit({
                "schema": RESULT, "status": "POLICY_MATCH",
                "policy_id": policy["policy_id"],
                "receiver_policy_sha256": args.expected_policy_sha256,
                "portable_sha256": policy["portable_sha256"],
                "plan_sha256": policy["plan_sha256"],
                "native_jobs_held": manifest["committed"],
                "native_replay": "NOT_PERFORMED",
                "external_policy_authentication": "NOT_PERFORMED",
                "release": "RESEARCH_ONLY",
                "production": "NOT_AUTHORIZED",
            })
            return 0

        require(bool(args.engine and args.source_root),
                "RECEIVER_ENGINE_SOURCE_REQUIRED")
        engine, source = Path(args.engine), Path(args.source_root)
        proof = admitted_replay(policy, package, engine, source)
        if action == "verify":
            emit({
                "schema": RESULT, "status": "PASS",
                "policy_id": policy["policy_id"],
                "receiver_policy_sha256": args.expected_policy_sha256,
                "portable_sha256": policy["portable_sha256"],
                "native_jobs_recomputed": proof["native_jobs_recomputed"],
                "native_files_bytewise_equal": proof["native_files_bytewise_equal"],
                "native_receipts_exactly_equal": proof["native_receipts_exactly_equal"],
                "pin_origin": "RECEIVER_SUPPLIED_NOT_EXTERNALLY_ATTESTED",
                "engine_sandbox": "NOT_PROVIDED",
                "signature": "NOT_PRESENT",
                "release": "RESEARCH_ONLY",
                "production": "NOT_AUTHORIZED",
            })
            return 0

        recheck_before_effect(policy_file, args.expected_policy_sha256,
                              policy, action, package, restored)
        if action == "restore":
            result = portable.restore(package, policy["portable_sha256"], restored)
            result["policy_id"] = policy["policy_id"]
            result["policy_sha256"] = args.expected_policy_sha256
            result["replay_qualification"] = "PASS_EXACT_PINNED_NATIVE_RUST_REPLAY"
            emit(result)
            return 0
        # Emit ONLY genuine native APP011 suffix, no extra positive receipt.
        return portable.resume_native(
            package, policy["portable_sha256"], restored, engine, source,
            policy["engine_sha256"], policy["source_tree_sha256"])
    except (replay.Uncertain, portable.Unknown) as e:
        emit({"schema": RESULT, "status": "INCONCLUSIVE",
              "error_code": e.code, "publication": "UNKNOWN"})
        return 3
    except (Denied, replay.Reject, portable.Refused,
            portable.inner.Refused, pinned.Denied, audit.AuditRejected,
            OSError, ValueError) as e:
        emit({"schema": RESULT, "status": "REJECTED",
              "error_code": getattr(e, "code", "RECEIVER_INPUT_REJECTED"),
              "publication": "NOT_STARTED"})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
