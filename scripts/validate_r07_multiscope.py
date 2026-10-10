#!/usr/bin/env python3
"""R0.7 static contract only; no Rust execution, proof or runtime certification."""
from __future__ import annotations

import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = "9c12bee933c13eff313aa11e49c02d56d86e91f3"
ARTIFACTS = [
    "README_R0_7.md",
    "docs/NORDOI_R0_7_TWO_SCOPE_CONFORMANCE_SPEC.md",
    "governance/r07_multiscope_witnesses_v1.tsv",
    "research/R0_7_TWO_SCOPE_ORACLE_DECISION.md",
    "scripts/validate_r07_multiscope.py",
    "tests/resource_task_r07.rs",
]
EXPECTED = 18
CATEGORIES = {
    "GENERATED_DIFFERENTIAL", "FINITE_EXHAUSTIVE", "BOUNDED_DIFFERENTIAL",
    "FINITE_INTERLEAVINGS", "DETERMINISM_EXAMPLE", "SYNTHETIC_MUTATION", "STATIC_BOUNDARY",
}


def ensure_tokens(source: str, tokens: list[str], name: str) -> None:
    for token in tokens:
        if token not in source:
            raise ValueError(f"{name}: missing required evidence text: {token!r}")


def validate() -> None:
    for rel in ARTIFACTS:
        path = ROOT / rel
        if not path.is_file() or path.is_symlink():
            raise ValueError(f"Missing/unsafe R0.7 artifact: {rel}")

    rust = (ROOT / ARTIFACTS[-1]).read_text(encoding="utf-8")
    spec = (ROOT / ARTIFACTS[1]).read_text(encoding="utf-8")
    research = (ROOT / ARTIFACTS[3]).read_text(encoding="utf-8")
    readme = (ROOT / ARTIFACTS[0]).read_text(encoding="utf-8")
    with (ROOT / ARTIFACTS[2]).open(encoding="utf-8", newline="") as fp:
        rows = list(csv.DictReader(fp, delimiter="\t"))

    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", rust)
    if len(tests) != EXPECTED or len(set(tests)) != EXPECTED:
        raise ValueError(f"Expected {EXPECTED} unique Rust witnesses, found {len(tests)}")
    if len(rows) != EXPECTED or [r.get("id") for r in rows] != [
        f"R07-C{i:02}" for i in range(1, EXPECTED + 1)
    ]:
        raise ValueError("Expected 18 canonical governance IDs")
    if tests != [r.get("witness") for r in rows]:
        raise ValueError("Test names and 1:1 governance witness order differ")
    for row in rows:
        if row.get("classification") not in CATEGORIES:
            raise ValueError(f"Unknown witness classification: {row['id']}")
        if len(row.get("limitation", "").strip()) < 48:
            raise ValueError(f"Insufficient bounded limitation for {row['id']}")
    for n, classification in [
        (0, "GENERATED_DIFFERENTIAL"), (1, "FINITE_EXHAUSTIVE"),
        (13, "FINITE_INTERLEAVINGS"), (15, "DETERMINISM_EXAMPLE"),
        (16, "SYNTHETIC_MUTATION"), (17, "STATIC_BOUNDARY"),
    ]:
        if rows[n]["classification"] != classification:
            raise ValueError(f"Misclassified evidence: {rows[n]['id']}")

    for name, content in [("README", readme), ("SPEC", spec), ("DECISION", research)]:
        ensure_tokens(content, ["r0.6", BASE, "TEST-ONLY"], name)
        if not re.search(r"\bnot\b", content, re.I):
            raise ValueError(f"{name}: missing explicit non-claim")

    ensure_tokens(spec, [
        "MS1", "MS2", "MS3", "MS4", "MS5", "MS6", "MS7", "MS8", "MS9", "MS10",
        "4,096", "5^5 = 3,125", "all 6", "No grandchild",
    ], "SPEC")
    ensure_tokens(rust, [
        '#[path = "../src/resource_task_r02.rs"]',
        "struct Oracle", "impl Oracle", "struct Trial", "impl Trial",
        "const SEEDS: usize = 32", "const STEPS: usize = 128",
        "const WORD_ALPHABET: usize = 5", "const WORD_DEPTH: usize = 5",
        "fn generated(", "R07_WITNESS version=1", "self.model.replay_checked()",
        'include_str!("../src/lib.rs")', 'include_str!("../Cargo.toml")',
        "synthetic_oracle_mutation_is_detected_but_not_reported_as_model_defect",
    ], "RUST")
    for expr in [
        r"assert_eq!\s*\(\s*actual\s*,\s*expected",
        r"assert_eq!\s*\(\s*self\.model\s*,\s*before",
        r"assert_eq!\s*\(\s*self\.oracle\s*,\s*oracle_before",
    ]:
        if re.search(expr, rust) is None:
            raise ValueError(f"Missing oracle or atomic rejection comparison: {expr}")
    for forbidden in [
        "std::thread::spawn", "tokio::spawn", "unsafe {", "std::process::Command",
        "self.model.phase(", "self.model.outcome(", "self.model.report(",
    ]:
        if forbidden in rust:
            raise ValueError(f"Forbidden oracle coupling / authority: {forbidden}")
    frozen_lib = ROOT / "src/lib.rs"
    if frozen_lib.is_file() and any(key in frozen_lib.read_text(encoding="utf-8")
                                    for key in ["resource_task_r07", "resource_task_r02"]):
        raise ValueError("Experiment inadvertently exported by frozen lib.rs")

    print("R0.7 static contract: PASS")
    print(f"Certified input baseline: r0.6 / {BASE}")
    print("Governance witnesses: 18/18; two-scope oracle: 32 x 128 = 4096 generated comparisons")
    print("Finite words: 5^5 = 3125; valid cross-scope completion/join orders: 6/6 (finite only)")
    print("TEST-ONLY: <=2 scopes, <=2 tasks and <=2 resources per scope")
    print("Rust execution, differential results, full release gate and GitHub CI: NOT RUN by this validator")


if __name__ == "__main__":
    validate()
