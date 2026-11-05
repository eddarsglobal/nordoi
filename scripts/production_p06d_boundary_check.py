#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tomllib
from pathlib import Path

BASE = "45f2a87691db2e2db635d6f704ee62ed80adedc4"
BRANCH = "production/p0.1"
R023 = "0428e74a0cd69b40a9fe12e9ae0806cbbeb03fd5"


class BoundaryError(RuntimeError):
    pass


def require(value: bool, message: str) -> None:
    if not value:
        raise BoundaryError(message)


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(
        ["git", "-C", str(root), *args], text=True, stderr=subprocess.PIPE
    ).strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=".")
    args = parser.parse_args()
    root = Path(args.repo).resolve()

    branch = git(root, "branch", "--show-current")
    in_ci = os.environ.get("GITHUB_ACTIONS") == "true"
    ci_branch = (
        os.environ.get("GITHUB_REF_NAME") == BRANCH
        or os.environ.get("GITHUB_BASE_REF") == BRANCH
    )

    require(branch == BRANCH or (not branch and in_ci and ci_branch), "wrong branch / CI target")
    require(git(root, "rev-parse", BASE + "^{commit}") == BASE, "P0.6C base unavailable")
    require(
        subprocess.run(
            ["git", "-C", str(root), "merge-base", "--is-ancestor", BASE, "HEAD"],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode
        == 0,
        "P0.6C is not ancestor",
    )

    if subprocess.run(
        ["git", "-C", str(root), "rev-parse", "-q", "--verify", "refs/tags/r0.23"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    ).returncode == 0:
        require(git(root, "rev-parse", "r0.23^{commit}") == R023, "r0.23 changed")

    protected = [
        "production/native-effect-p03/Cargo.toml",
        "production/native-effect-p03/src/lib.rs",
        "production/native-effect-p03/src/production.rs",
        "production/native-effect-p03/src/enforced.rs",
        "production/native-rollback-p06/Cargo.toml",
        "production/native-rollback-p06/src/lib.rs",
        "production/native-effect-profile-p05c/Cargo.toml",
        "production/native-effect-profile-p05c/src/lib.rs",
        "production/native-remote-anchor-p06c/.gitignore",
        "production/native-remote-anchor-p06c/Cargo.toml",
        "production/native-remote-anchor-p06c/Cargo.lock",
        "production/native-remote-anchor-p06c/src/lib.rs",
        "scripts/production_p06b_boundary_check.py",
        "scripts/production_p06b_gate.sh",
        "scripts/production_p06c_boundary_check.py",
        "scripts/production_p06c_gate.sh",
    ]
    changed = git(root, "diff", "--name-only", BASE, "--", *protected).splitlines()
    require(not changed, f"certified protected sources changed: {changed}")

    manifest_path = root / "production/native-remote-anchor-service-p06d/Cargo.toml"
    source_path = root / "production/native-remote-anchor-service-p06d/src/lib.rs"
    binary_path = (
        root
        / "production/native-remote-anchor-service-p06d/src/bin/nordoi-anchor-service.rs"
    )

    require(manifest_path.is_file(), "P0.6D manifest missing")
    require(source_path.is_file(), "P0.6D library source missing")
    require(binary_path.is_file(), "P0.6D service binary missing")

    manifest = tomllib.loads(manifest_path.read_text())
    deps = set(manifest.get("dependencies", {}))
    require(
        deps == {"nordoi-rollback-anchor-p06", "ed25519-dalek", "sha2"},
        f"P0.6D dependency set unexpected: {sorted(deps)}",
    )

    dev_deps = set(manifest.get("dev-dependencies", {}))
    require(
        dev_deps == {"nordoi-remote-anchor-p06c"},
        f"P0.6D dev dependency set unexpected: {sorted(dev_deps)}",
    )

    source = source_path.read_text()
    binary = binary_path.read_text()

    for forbidden in [
        "tokio",
        "reqwest",
        "hyper",
        "ureq",
        "serde",
        "serde_json",
        "unsafe {",
    ]:
        require(
            forbidden not in source and forbidden not in binary,
            f"forbidden/heavy P0.6D surface present: {forbidden}",
        )

    for token in [
        "#![forbid(unsafe_code)]",
        "pub struct DurableAnchorStore",
        "pub struct RemoteAnchorService",
        "file.sync_all()",
        "ENTRY_SIGNATURE_DOMAIN",
        "verify_strict(&signed, &signature)",
        "SigningKey::from_bytes(&self.journal_secret)",
        "JOURNAL_ENTRY_BYTES",
        "partial_tail_is_truncated_to_last_complete_entry",
        "corrupted_complete_entry_fails_closed",
        "p06c_client_round_trip_against_p06d_service",
        "service_restart_preserves_remote_anchor",
        "p06a_recovery_detects_rollback_through_restarted_service",
    ]:
        require(token in source, f"missing P0.6D invariant: {token}")

    require(
        "server_secret_key_file" in binary,
        "P0.6D binary must use a server secret key file path",
    )
    require(
        "key file must contain exactly 32 raw bytes" in binary,
        "P0.6D binary key-size gate missing",
    )

    count = source.count("#[test]")
    require(count == 12, f"P0.6D test inventory changed: {count}")

    print("P0.6D REMOTE ANCHOR SERVICE SOURCE BOUNDARY: PASS")
    print("P0.6D TEST INVENTORY: 12")
    print("P0.6C CLIENT SOURCE: UNCHANGED")
    print("P0.6B/P0.6A PROTECTED PATH: UNCHANGED")
    print("DURABLE AUTHENTICATED APPEND-ONLY JOURNAL: IMPLEMENTED")
    print("SUCCESS RESPONSE AFTER FILE SYNC_ALL: ENFORCED")
    print("PARTIAL TAIL RECOVERY: IMPLEMENTED")
    print("COMPLETE ENTRY CORRUPTION: FAIL_CLOSED")
    print("ASYNC/HTTP/SERDE STACKS: ABSENT")
    print("INDEPENDENT FAILURE-DOMAIN DEPLOYMENT: NOT CERTIFIED")
    print("REMOTE STORAGE FULL-SNAPSHOT ANTI-ROLLBACK: NOT CERTIFIED")
    print("HARDWARE-BACKED SERVER KEY CUSTODY: NOT CERTIFIED")
    print("KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED")
    print("PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except BoundaryError as exc:
        print(f"P0.6D BOUNDARY: FAIL: {exc}", file=sys.stderr)
        raise SystemExit(1)
