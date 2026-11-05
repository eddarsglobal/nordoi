#!/usr/bin/env python3
from __future__ import annotations
import argparse
import os
import subprocess
import sys
import tomllib
from pathlib import Path

BASE = "31d13af3d620f8d961c777ad7fdfb0bb29073181"
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
    require(git(root, "rev-parse", BASE + "^{commit}") == BASE, "P0.6B base unavailable")
    require(
        subprocess.run(
            ["git", "-C", str(root), "merge-base", "--is-ancestor", BASE, "HEAD"],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode == 0,
        "P0.6B is not ancestor",
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
        "scripts/production_p06b_boundary_check.py",
        "scripts/production_p06b_gate.sh",
    ]
    changed = git(root, "diff", "--name-only", BASE, "--", *protected).splitlines()
    require(not changed, f"P0.6B protected sources changed: {changed}")

    manifest_path = root / "production/native-remote-anchor-p06c/Cargo.toml"
    source_path = root / "production/native-remote-anchor-p06c/src/lib.rs"
    require(manifest_path.is_file(), "P0.6C manifest missing")
    require(source_path.is_file(), "P0.6C source missing")
    manifest = tomllib.loads(manifest_path.read_text())
    deps = set(manifest.get("dependencies", {}))
    require(
        deps == {"nordoi-rollback-anchor-p06", "ed25519-dalek", "getrandom"},
        f"P0.6C dependency set unexpected: {sorted(deps)}",
    )
    require(not manifest.get("dev-dependencies"), "P0.6C unexpected dev dependencies")

    source = source_path.read_text()
    for forbidden in ["tokio", "reqwest", "hyper", "ureq", "serde", "serde_json", "unsafe {"]:
        require(forbidden not in source, f"forbidden/heavy P0.6C surface present: {forbidden}")
    for token in [
        "#![forbid(unsafe_code)]",
        "pub struct RemoteAnchorClient",
        "impl MonotonicAnchor for RemoteAnchorClient",
        "TcpStream::connect_timeout",
        "set_read_timeout",
        "set_write_timeout",
        "set_nodelay(true)",
        "getrandom::fill(&mut request_id)",
        "self.client_key.sign(&request[..REQUEST_SIGNED_BYTES])",
        "verify_strict(&response[..RESPONSE_SIGNED_BYTES], &signature)",
        "self.stream = None;",
        "OP_COMPARE_AND_ADVANCE",
    ]:
        require(token in source, f"missing P0.6C invariant: {token}")
    require(source.count("#[test]") == 10, f"P0.6C test inventory changed: {source.count('#[test]')}")
    for name in [
        "remote_read_empty_round_trip",
        "remote_compare_and_advance_initializes_and_reads_back",
        "remote_conflicting_expected_record_fails_closed",
        "remote_non_monotonic_generation_is_rejected",
        "wrong_client_key_is_rejected_without_state_change",
        "tampered_server_signature_is_rejected",
        "response_for_another_request_id_is_rejected",
        "unreachable_remote_service_is_unavailable",
        "p06_recovery_contract_operates_over_remote_backend",
        "remote_anchor_detects_older_local_trust_state",
    ]:
        require(name in source, f"missing P0.6C test: {name}")

    print("P0.6C REMOTE ANCHOR SOURCE BOUNDARY: PASS")
    print("P0.6C TEST INVENTORY: 10")
    print("P0.6B PROTECTED PATH: UNCHANGED")
    print("P0.6A CONTRACT NORMAL DEPENDENCIES: PRESERVED")
    print("REMOTE MONOTONIC ANCHOR CLIENT BACKEND: IMPLEMENTED")
    print("ASYNC/HTTP/SERDE STACKS: ABSENT")
    print("INDEPENDENT FAILURE-DOMAIN DEPLOYMENT: NOT CERTIFIED")
    print("REMOTE SERVICE ROLLBACK-RESISTANT STORAGE: NOT CERTIFIED")
    print("KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED")
    print("PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED")
    return 0

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except BoundaryError as exc:
        print(f"P0.6C BOUNDARY: FAIL: {exc}", file=sys.stderr)
        raise SystemExit(1)
