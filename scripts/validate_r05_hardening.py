#!/usr/bin/env python3
"""Static R0.5 governance check. Does NOT compile or run Rust or prove security."""
from __future__ import annotations

import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "3ce87f1943c8c97ee304cebad9eff236ae646fa6"
TEST = ROOT / "tests/resource_task_r05.rs"
DOC = ROOT / "docs/NORDOI_R0_5_RESOURCE_TASK_HARDENING_SPEC.md"
RESEARCH = ROOT / "research/R0_5_HARDENING_DECISION.md"
README = ROOT / "README_R0_5.md"
MATRIX = ROOT / "governance/r05_hardening_witnesses_v1.tsv"
EXPECTED_TESTS = 14
VALID = {
    "EXHAUSTIVE_WITHIN_PROFILE",
    "BOUNDED_ADVERSARIAL",
    "SYNTHETIC_MUTATION",
    "NEGATIVE_TRACE_SHRINK",
    "DETERMINISM_EXAMPLE",
    "STATIC_CHECK_ONLY",
}


def validate() -> None:
    source = TEST.read_text(encoding="utf-8")
    spec = DOC.read_text(encoding="utf-8")
    decision = RESEARCH.read_text(encoding="utf-8")
    readme = README.read_text(encoding="utf-8")
    with MATRIX.open(encoding="utf-8", newline="") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))
    if len(rows) != EXPECTED_TESTS:
        raise ValueError(f"Expected {EXPECTED_TESTS} governance witnesses, got {len(rows)}")
    if [r["id"] for r in rows] != [f"R05-H{i:02}" for i in range(1, EXPECTED_TESTS + 1)]:
        raise ValueError("Noncanonical witness identities or order")
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", source)
    if len(tests) != EXPECTED_TESTS or len(set(tests)) != EXPECTED_TESTS:
        raise ValueError(f"Expected {EXPECTED_TESTS} unique declared Rust tests, got {tests}")
    if [r["witness"] for r in rows] != tests:
        raise ValueError("Matrix-to-test witness bijection/order drift")
    for row in rows:
        if row["classification"] not in VALID:
            raise ValueError(f"Unknown evidence classification: {row['id']}")
        if len(row["limitation"].strip()) < 35:
            raise ValueError(f"Missing meaningful coverage limitation: {row['id']}")
    if rows[0]["classification"] != "EXHAUSTIVE_WITHIN_PROFILE":
        raise ValueError("Finite-profile exhaustive qualifier missing")
    required = [
        '#[path = "../src/resource_task_r02.rs"]',
        "const EXPECTED_INTERLEAVINGS: usize = 90",
        "fn walk_interleavings(",
        "fn rejected(",
        "fn shrink_invalid_trace(",
        "has_rejection(&reduced)",
        "synthetic_oracle_mutation_is_detected_without_claiming_real_defect",
        "replay_checked()",
        'include_str!("../src/lib.rs")',
        'include_str!("../Cargo.toml")',
        "R05_WITNESS version=1",
    ]
    for token in required:
        if token not in source:
            raise ValueError(f"Missing test-only evidence mechanism: {token}")
    if not re.search(r"assert_eq!\s*\(\s*\*model\s*,\s*before\b", source):
        raise ValueError("Atomic rejection equality check missing")
    if not re.search(r"assert_eq!\s*\(\s*model\.event_count\(\)\s*,\s*count\b", source):
        raise ValueError("Atomic rejection accepted-log check missing")
    if not re.search(r"assert_eq!\s*\(\s*count\s*,\s*EXPECTED_INTERLEAVINGS", source):
        raise ValueError("Finite-profile exhaustive count check missing")
    if "6! / 2^3 = 90" not in spec:
        raise ValueError("Bounded schedule domain derivation missing")
    for token in ["FNG1", "FNG2", "FNG3", "FNG4", "FNG5", "FNG6", "not", "TEST-ONLY"]:
        if token not in spec:
            raise ValueError(f"Scope gate/disclaimer missing: {token}")
    for text, label in [(readme, "readme"), (spec, "spec"), (decision, "decision")]:
        if BASELINE not in text or "r0.4" not in text:
            raise ValueError(f"Frozen certified baseline identity absent: {label}")
    if "expected rejection" not in readme.lower() or "not" not in decision.lower():
        raise ValueError("Synthetic/negative evidence limitation absent")
    for forbidden in ["std::thread::spawn", "tokio::spawn", "unsafe {", "async_std::task::spawn", "std::process::Command"]:
        if forbidden in source:
            raise ValueError(f"Out-of-scope runtime operation: {forbidden}")
    lib = ROOT / "src/lib.rs"
    if lib.exists() and "resource_task_r05" in lib.read_text(encoding="utf-8"):
        raise ValueError("R0.5 test witness exported into certified library")
    print("R0.5 static contract: PASS")
    print(f"Certified input baseline: r0.4 / {BASELINE}")
    print("Governance witnesses: 14/14; three-task admissible interleavings: 90 (finite profile only)")
    print("Isolation: TEST-ONLY; no new runtime/NAIR/compiler/authority export")
    print("Rust tests, measured state coverage, security proof and CI: NOT RUN by this validator")


if __name__ == "__main__":
    validate()
