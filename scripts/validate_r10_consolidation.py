#!/usr/bin/env python3
"""R0.10 TEST-ONLY static traceability and optional r0.9 Git tree boundary audit."""
from __future__ import annotations
import csv
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE_SHA = "860d09774c881cf23f2a1233877109ed49c21a01"
NEW = {
    "README_R0_10.md",
    "docs/NORDOI_R0_10_VERIFICATION_CONSOLIDATION_SPEC.md",
    "governance/r10_consolidation_witnesses_v1.tsv",
    "research/R0_10_CONSOLIDATION_DECISION.md",
    "scripts/validate_r10_consolidation.py",
    "tests/resource_task_r10.rs",
}
SUITES = [
    ("03", "r03_state_space_obligations_v1.tsv", 9, "R03-V"),
    ("04", "r04_adversarial_witnesses_v1.tsv", 12, "R04-A"),
    ("05", "r05_hardening_witnesses_v1.tsv", 14, "R05-H"),
    ("06", "r06_conformance_witnesses_v1.tsv", 15, "R06-C"),
    ("07", "r07_multiscope_witnesses_v1.tsv", 18, "R07-C"),
    ("08", "r08_hierarchy_witnesses_v1.tsv", 26, "R08-H"),
    ("09", "r09_branch_boundary_witnesses_v1.tsv", 23, "R09-B"),
 ]


def rows(path: Path):
    with path.open(encoding="utf8", newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def run_git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)


def static_check() -> None:
    for name in NEW:
        path = ROOT / name
        if not path.is_file() or path.is_symlink():
            raise RuntimeError(f"R0.10 file absent/unsafe: {name}")
    rust = (ROOT / "tests/resource_task_r10.rs").read_text(encoding="utf8")
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", rust)
    matrix = rows(ROOT / "governance/r10_consolidation_witnesses_v1.tsv")
    if len(tests) != 18 or len(set(tests)) != 18 or len(matrix) != 18:
        raise RuntimeError("R0.10 must contain exactly 18 unique Rust tests/witnesses")
    if tests != [record["witness"] for record in matrix]:
        raise RuntimeError("R0.10 matrix-to-test bijection/order mismatch")
    valid = {"STATIC_CONSOLIDATION", "FINITE_ARITHMETIC", "DETERMINISM_EXAMPLE",
             "SYNTHETIC_MUTATION", "DIRECT_REFERENCE_SMOKE"}
    for i, record in enumerate(matrix, 1):
        if record["id"] != f"R10-C{i:02}" or record["classification"] not in valid:
            raise RuntimeError(f"Invalid R0.10 matrix row: {i}")
        if len(record["limitation"].strip()) < 70:
            raise RuntimeError(f"R0.10 limitation too short: {i}")
    if len(SUITES) != 7 or sum(count for _, _, count, _ in SUITES) != 117:
        raise RuntimeError("Historical count drift")
    for ver, file, count, prefix in SUITES:
        source = ROOT / f"tests/resource_task_r{ver}.rs"
        tsv = ROOT / "governance" / file
        if not source.is_file() or not tsv.is_file():
            raise RuntimeError(f"Missing historical source/TSV: R{ver}")
        names = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", source.read_text(encoding="utf8"))
        data = rows(tsv)
        if len(names) != count or len(data) != count:
            raise RuntimeError(f"Historical count drift R{ver}")
        if names != [r["witness"] for r in data] or \
           [r["id"] for r in data] != [f"{prefix}{i:02}" for i in range(1, count + 1)]:
            raise RuntimeError(f"Historical matrix-to-test drift R{ver}")
    text = "\n".join((ROOT / f).read_text(encoding="utf8") for f in NEW if f.endswith(".md"))
    for required in ("r0.9", BASE_SHA, "TEST-ONLY", "117", "18", "not", "CI"):
        if required not in text:
            raise RuntimeError(f"Missing governance nonclaim/identity: {required}")
    for forbidden in ("std::thread::spawn(", "tokio::spawn(", "std::process::Command::new("):
        if forbidden in rust:
            raise RuntimeError(f"Unexpected runtime/unsafe behavior in new Rust tests: {forbidden}")
    for required in ("#[path = \"../src/resource_task_r02.rs\"]",
                     "fn parse_matrix(", "fn material(", "fn digest(",
                     "117", "R10_WITNESS version=1", "replay_checked()"):
        if required not in rust:
            raise RuntimeError(f"R0.10 Rust evidence missing: {required}")
    lib = ROOT / "src/lib.rs"
    if lib.is_file() and "resource_task_r10" in lib.read_text(encoding="utf8"):
        raise RuntimeError("R0.10 leaked into certified library")
    print("R0.10 static contract: PASS")
    print(f"Certified input baseline: r0.9 / {BASE_SHA}")
    print("Historical traceability: R0.3-R0.9 = 117/117 witnesses; R0.10 = 18/18 planned tests")
    print("Scope: TEST-ONLY governance and finite-profile arithmetic, not a security/concurrency proof")


def git_boundary() -> None:
    check = run_git("rev-parse", "--show-toplevel")
    if check.returncode != 0 or Path(check.stdout.strip()).resolve() != ROOT.resolve():
        print("R0.9 Git tree boundary: NOT CHECKED (outside complete repository)")
        return
    baseline = run_git("rev-parse", "r0.9^{commit}")
    if baseline.returncode != 0 or baseline.stdout.strip() != BASE_SHA:
        raise RuntimeError("Certified r0.9 annotated tag missing or unexpected target")
    # Both staged and unstaged tracked changes relative to the certified tree are inspected.
    change = run_git("diff", "--name-only", "r0.9", "--", ".")
    if change.returncode != 0:
        raise RuntimeError(f"Cannot compare certified Git tree: {change.stderr}")
    changed = set(change.stdout.splitlines())
    if changed - NEW:
        raise RuntimeError(f"Frozen tracked file changes since r0.9: {sorted(changed - NEW)}")
    # Baseline files missing from the working tree must also fail even if unstaged.
    tree = run_git("ls-tree", "-r", "--name-only", "r0.9")
    if tree.returncode != 0:
        raise RuntimeError("Could not inspect certified Git tree")
    reused = NEW.intersection(tree.stdout.splitlines())
    if reused:
        raise RuntimeError(f"New paths collide with certified baseline: {sorted(reused)}")
    for path in tree.stdout.splitlines():
        if not (ROOT / path).is_file():
            raise RuntimeError(f"Missing certified file: {path}")
    other = run_git("ls-files", "--others", "--exclude-standard")
    if other.returncode != 0:
        raise RuntimeError("Cannot inspect untracked files")
    if set(other.stdout.splitlines()) - NEW:
        raise RuntimeError("Unexpected untracked paths; Git boundary not clean")
    print("R0.9 Git tree boundary: PASS (zero frozen changes; six additive paths only)")


if __name__ == "__main__":
    static_check()
    git_boundary()
    print("Rust tests, Clippy, complete release gate and GitHub CI: NOT RUN by validator")
