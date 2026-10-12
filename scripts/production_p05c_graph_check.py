#!/usr/bin/env python3
"""P0.5C closed build-graph boundary check.

This certifies only the selected Cargo import boundary. It does not claim that
historical kernel APIs have disappeared or that deployment is authorized.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

POLICY_PATH = "production/p05c_build_policy.json"


class PolicyError(Exception):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PolicyError(message)


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(
        ["git", "-C", str(repo), *args], text=True, stderr=subprocess.PIPE
    ).strip()


def code_tokens(source: str) -> str:
    """Remove Rust comments and string contents for conservative token checks."""
    require(re.search(r'r#+"', source) is None, "raw Rust string requires review")
    out: list[str] = []
    i = 0
    depth = 0
    while i < len(source):
        if depth:
            if source.startswith("/*", i):
                depth += 1
                i += 2
            elif source.startswith("*/", i):
                depth -= 1
                i += 2
            else:
                i += 1
            continue
        if source.startswith("//", i):
            pos = source.find("\n", i)
            i = len(source) if pos < 0 else pos
            continue
        if source.startswith("/*", i):
            depth = 1
            i += 2
            continue
        if source[i] == '"':
            out.append('""')
            i += 1
            closed = False
            while i < len(source):
                if source[i] == "\\":
                    i += 2
                elif source[i] == '"':
                    i += 1
                    closed = True
                    break
                else:
                    i += 1
            require(closed, "unterminated Rust string")
            continue
        out.append(source[i])
        i += 1
    require(depth == 0, "unterminated Rust block comment")
    return "".join(out)


def validate_manifest_data(manifest: dict, policy: dict) -> None:
    package = manifest.get("package", {})
    require(package.get("name") == policy["profile_package_name"], "wrong profile package name")
    require(package.get("version") == policy["profile_version"], "wrong profile version")
    require(package.get("edition") == "2021", "profile edition changed")
    require(package.get("publish") is False, "profile crate must remain non-publishable")

    deps = manifest.get("dependencies", {})
    required = policy["required_direct_dependency"]
    require(set(deps) == {required["name"]}, f"direct dependency set widened: {sorted(deps)}")
    spec = deps[required["name"]]
    require(isinstance(spec, dict), "profile dependency must use explicit table form")
    require(spec.get("path") == required["path"], "P0.3 dependency path changed")
    require(set(spec) == {"path"}, "P0.3 dependency gained unreviewed options")
    require(not manifest.get("dev-dependencies"), "profile dev-dependencies are forbidden")
    require(not manifest.get("build-dependencies"), "profile build-dependencies are forbidden")
    require(not manifest.get("features"), "profile Cargo features are forbidden at P0.5C")
    require("workspace" not in manifest, "profile must not redefine a workspace")


def validate_profile_source(source: str, policy: dict) -> None:
    code = code_tokens(source)
    require("#![forbid(unsafe_code)]" in source, "unsafe code is not forbidden")
    require(code.count("pub use ") == 1, "profile must expose exactly one pub use")
    require(policy["required_public_reexport"] in source, "fenced facade re-export changed")
    require(re.search(r"\bpub\s+(?:struct|enum|trait|mod)\b", code) is None, "new public type/module exposed")
    require(re.search(r"\bpub\s+fn\b", code) is None, "runtime public function added")
    require(re.search(r"\bpub\s+const\s+PROFILE_ID\b", code) is not None, "PROFILE_ID missing")
    require(re.search(r"\bpub\s+const\s+fn\s+profile_id\b", code) is not None, "profile_id missing")
    for token in policy["forbidden_profile_tokens"]:
        require(token not in code, f"forbidden raw token exposed in profile source: {token}")
    require("unsafe" not in code.replace("forbid(unsafe_code)", ""), "unsafe token introduced")


def metadata(repo: Path, manifest_path: Path) -> dict:
    raw = subprocess.check_output(
        [
            "cargo",
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
            str(manifest_path),
        ],
        cwd=repo,
        text=True,
    )
    return json.loads(raw)


def validate_metadata(data: dict, policy: dict, crate_dir: Path) -> None:
    package = next(
        (p for p in data["packages"] if p["name"] == policy["profile_package_name"]),
        None,
    )
    require(package is not None, "profile package absent from cargo metadata")
    require(Path(package["manifest_path"]).resolve() == (crate_dir / "Cargo.toml").resolve(), "metadata selected wrong profile manifest")

    deps = package.get("dependencies", [])
    normal = [d for d in deps if d.get("kind") is None]
    require(len(normal) == 1, f"metadata direct normal dependencies changed: {len(normal)}")
    required = policy["required_direct_dependency"]["name"]
    require(normal[0]["name"] == required, f"metadata direct dependency is not {required}")
    require(not [d for d in deps if d.get("kind") in {"dev", "build"}], "metadata contains dev/build dependencies")

    resolve = data.get("resolve") or {}
    node = next((n for n in resolve.get("nodes", []) if n["id"] == package["id"]), None)
    require(node is not None, "profile resolve node missing")
    package_names_by_id = {item["id"]: item["name"] for item in data["packages"]}
    direct_packages = {
        package_names_by_id.get(dep["pkg"], f"<unknown:{dep['pkg']}>")
        for dep in node.get("deps", [])
    }
    require(
        direct_packages == {required},
        f"resolved direct package graph widened: {sorted(direct_packages)}",
    )


def check_repo(repo: Path) -> None:
    policy = json.loads((repo / POLICY_PATH).read_text(encoding="utf-8"))
    require(policy["schema"] == "nordoi-production-p05c-build-policy-v1", "unsupported P0.5C policy schema")
    require(not policy["kernel_wide_mandatory_enforcement_certified"], "global enforcement claim forbidden")
    require(not policy["production_deployment_authorized"], "deployment authorization forbidden")

    branch = git(repo, "branch", "--show-current")
    in_ci = os.environ.get("GITHUB_ACTIONS") == "true"
    ci_branch = (
        os.environ.get("GITHUB_REF_NAME") == policy["branch"]
        or os.environ.get("GITHUB_BASE_REF") == policy["branch"]
    )
    require(branch == policy["branch"] or (not branch and in_ci and ci_branch), "wrong branch / CI target")

    base = policy["certified_base_sha"]
    require(git(repo, "rev-parse", base + "^{commit}") == base, "P0.5B baseline unavailable")
    ancestor = subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", base, "HEAD"],
        check=False,
    )
    require(ancestor.returncode == 0, "P0.5B is not ancestor of HEAD")

    tag = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "-q", "--verify", "refs/tags/r0.23"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if tag.returncode == 0:
        require(git(repo, "rev-parse", "r0.23^{commit}") == policy["research_tag_sha"], "r0.23 tag changed")

    crate_dir = repo / policy["profile_crate_path"]
    manifest_path = crate_dir / "Cargo.toml"
    lock_path = crate_dir / "Cargo.lock"
    source_path = crate_dir / "src/lib.rs"
    require(manifest_path.is_file(), "P0.5C profile Cargo.toml missing")
    require(lock_path.is_file(), "P0.5C Cargo.lock missing: run cargo generate-lockfile first")
    require(source_path.is_file(), "P0.5C profile lib.rs missing")

    with manifest_path.open("rb") as handle:
        manifest = tomllib.load(handle)
    validate_manifest_data(manifest, policy)
    validate_profile_source(source_path.read_text(encoding="utf-8"), policy)
    validate_metadata(metadata(repo, manifest_path), policy, crate_dir)

    # No pre-existing manifest outside P0.3 may acquire a direct raw production dependency
    # as part of this additive candidate. New future consumers require explicit review.
    forbidden_direct = {"nordoi_kernel", "nordoi-native-trust-p01"}
    for path in repo.rglob("Cargo.toml"):
        if "target" in path.parts or path == manifest_path:
            continue
        with path.open("rb") as handle:
            data = tomllib.load(handle)
        package_name = data.get("package", {}).get("name")
        deps = data.get("dependencies", {})
        if package_name != "nordoi-native-effect-p03":
            require(
                not (set(deps) & forbidden_direct),
                f"new direct raw production dependency outside P0.3: {path.relative_to(repo)}",
            )

    print("P0.5C CLOSED BUILD-GRAPH BOUNDARY: PASS")
    print("DIRECT CONSUMER RAW-CRATE IMPORTS: DENIED BY CARGO DEPENDENCY GRAPH")
    print("HISTORICAL KERNEL APIS: STILL PRESENT OUTSIDE THIS SELECTED BOUNDARY")
    print("KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED")
    print("PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        check_repo(args.repo.resolve())
        return 0
    except (PolicyError, OSError, ValueError, KeyError, subprocess.CalledProcessError, json.JSONDecodeError, tomllib.TOMLDecodeError) as exc:
        print(f"P0.5C BUILD GRAPH: FAIL: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
