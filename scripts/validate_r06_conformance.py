#!/usr/bin/env python3
"""R0.6 static contract only. Does NOT run Rust or certify runtime security."""
from __future__ import annotations

import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = "6cebd4051304c81901669303265c0ad7456b7e12"
TEST = ROOT / "tests/resource_task_r06.rs"
DOC = ROOT / "docs/NORDOI_R0_6_INDEPENDENT_REFERENCE_CONFORMANCE_SPEC.md"
RESEARCH = ROOT / "research/R0_6_INDEPENDENT_ORACLE_DECISION.md"
README = ROOT / "README_R0_6.md"
MATRIX = ROOT / "governance/r06_conformance_witnesses_v1.tsv"
EXPECTED = 15
CLASSIFICATIONS = {
    "GENERATED_DIFFERENTIAL", "FINITE_EXHAUSTIVE", "BOUNDED_DIFFERENTIAL",
    "DETERMINISM_EXAMPLE", "SYNTHETIC_MUTATION", "STATIC_BOUNDARY",
}


def validate() -> None:
    sources = [TEST, DOC, RESEARCH, README, MATRIX]
    for path in sources:
        if not path.is_file():
            raise ValueError(f"Missing R0.6 file: {path.relative_to(ROOT)}")
    rust = TEST.read_text(encoding="utf-8")
    readme = README.read_text(encoding="utf-8")
    spec = DOC.read_text(encoding="utf-8")
    decision = RESEARCH.read_text(encoding="utf-8")
    with MATRIX.open(encoding="utf-8", newline="") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    names = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", rust)
    if len(names) != EXPECTED or len(set(names)) != EXPECTED:
        raise ValueError(f"Expected {EXPECTED} unique Rust test witnesses, got {len(names)}")
    if len(rows) != EXPECTED:
        raise ValueError(f"Expected {EXPECTED} governance rows, got {len(rows)}")
    if [row["id"] for row in rows] != [f"R06-C{i:02}" for i in range(1, EXPECTED + 1)]:
        raise ValueError("Noncanonical conformance witness identities")
    if names != [row["witness"] for row in rows]:
        raise ValueError("Test/governance witness mapping or ordering changed")
    for row in rows:
        if row["classification"] not in CLASSIFICATIONS:
            raise ValueError(f"Unknown evidence class: {row['id']}")
        if len(row["limitation"].strip()) < 45:
            raise ValueError(f"Unclear limitation: {row['id']}")
    if rows[1]["classification"] != "FINITE_EXHAUSTIVE":
        raise ValueError("Finite exhaustive bound not classified")
    for text, label in [(readme, "README"), (spec, "specification"), (decision, "decision")]:
        if "r0.5" not in text or BASE not in text:
            raise ValueError(f"Certified reference baseline not anchored in {label}")
        if "not" not in text.lower() or "TEST-ONLY" not in text:
            raise ValueError(f"Necessary nonproduction limitation missing from {label}")
    essentials = [
        '#[path = "../src/resource_task_r02.rs"]',
        "struct Oracle", "impl Oracle", "struct Trial", "fn foreign_fixture(",
        "fn generated(", "fn finite_five_action_sequences_match_independent_oracle(",
        "const SEED_COUNT: usize = 32", "const STEPS_PER_SEED: usize = 96",
        "const FINITE_ALPHABET: usize = 5", "const FINITE_DEPTH: usize = 5",
        "R06_WITNESS version=1", "synthetic_oracle_mutation_is_detected_not_a_real_bug",
        'include_str!("../src/lib.rs")', "self.model.replay_checked()",
    ]
    for token in essentials:
        if token not in rust:
            raise ValueError(f"Missing static evidence mechanism: {token}")
    for expression in [r"assert_eq!\s*\(\s*actual\s*,\s*predicted",
                       r"assert_eq!\s*\(\s*self\.model\s*,\s*before",
                       r"assert_eq!\s*\(\s*self\.oracle\s*,\s*oracle_before"]:
        if not re.search(expression, rust):
            raise ValueError(f"Independent oracle / rejection state check missing: {expression}")
    for forbidden in [
        "std::thread::spawn", "tokio::spawn", "unsafe {", "std::process::Command",
        "self.model.phase(", "self.model.outcome(", "self.model.report(",
    ]:
        if forbidden in rust:
            raise ValueError(f"Forbidden authority/oracle coupling: {forbidden}")
    for tag in ["FNG1", "FNG2", "FNG3", "FNG4", "FNG5", "FNG6", "5^5 = 3,125"]:
        if tag not in spec:
            raise ValueError(f"Missing governance or finite-profile caveat: {tag}")
    lib = ROOT / "src/lib.rs"
    if lib.exists() and "resource_task_r06" in lib.read_text(encoding="utf-8"):
        raise ValueError("R0.6 accidentally exported in frozen Rust library")
    print("R0.6 static contract: PASS")
    print(f"Certified input baseline: r0.5 / {BASE}")
    print("Governance witnesses: 15/15; generated comparisons: 32 x 96 = 3072")
    print("Finite-word profile: 5^5 = 3125; scope: 1 root / <=1 task / <=1 resource")
    print("Isolation: TEST-ONLY; independent oracle does not export runtime authority")
    print("Rust tests, real differential results, release gate and CI: NOT RUN by this validator")


if __name__ == "__main__":
    validate()
