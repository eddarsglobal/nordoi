#!/usr/bin/env python3
"""R0.8 source/governance integrity only; never executes Rust or certifies CI."""
from __future__ import annotations
import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = "8452ce32a27ea93c4eb343dec7165ae4b6be704a"
ARTIFACTS = [
    "README_R0_8.md",
    "docs/NORDOI_R0_8_HIERARCHICAL_CONFORMANCE_SPEC.md",
    "governance/r08_hierarchy_witnesses_v1.tsv",
    "research/R0_8_HIERARCHICAL_ORACLE_DECISION.md",
    "scripts/validate_r08_hierarchy.py",
    "tests/resource_task_r08.rs",
]
EXPECTED = 26
CATEGORIES = {"GENERATED_DIFFERENTIAL", "FINITE_EXHAUSTIVE", "BOUNDED_DIFFERENTIAL",
              "FINITE_INTERLEAVINGS", "DETERMINISM_EXAMPLE", "SYNTHETIC_MUTATION", "STATIC_BOUNDARY"}


def require_tokens(content: str, needles: list[str], path: str) -> None:
    for needle in needles:
        if needle not in content:
            raise ValueError(f"{path}: required literal missing: {needle!r}")


def validate() -> None:
    for rel in ARTIFACTS:
        path = ROOT / rel
        if not path.is_file() or path.is_symlink():
            raise ValueError(f"Missing or unsafe R0.8 artifact: {rel}")
    rust = (ROOT / ARTIFACTS[-1]).read_text(encoding="utf-8")
    spec = (ROOT / ARTIFACTS[1]).read_text(encoding="utf-8")
    research = (ROOT / ARTIFACTS[3]).read_text(encoding="utf-8")
    readme = (ROOT / ARTIFACTS[0]).read_text(encoding="utf-8")
    with (ROOT / ARTIFACTS[2]).open(encoding="utf-8", newline="") as fp:
        rows = list(csv.DictReader(fp, delimiter="\t"))
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", rust)
    if len(tests) != EXPECTED or len(set(tests)) != EXPECTED:
        raise ValueError(f"Expected {EXPECTED} distinct Rust tests; got {len(tests)}")
    if len(rows) != EXPECTED or [r.get("id") for r in rows] != [f"R08-H{i:02}" for i in range(1, EXPECTED + 1)]:
        raise ValueError("R0.8 witness IDs are not complete, ordered or unique")
    if tests != [r.get("witness") for r in rows]:
        raise ValueError("R0.8 test function names and governance order do not agree")
    for row in rows:
        if row.get("classification") not in CATEGORIES:
            raise ValueError(f"Unknown category in {row['id']}")
        if len(row.get("limitation", "").strip()) < 60:
            raise ValueError(f"Insufficient witness-specific limitation in {row['id']}")
    for pos, classification in [(0,"GENERATED_DIFFERENTIAL"), (1,"FINITE_EXHAUSTIVE"),
                                (13,"FINITE_INTERLEAVINGS"), (15,"DETERMINISM_EXAMPLE"),
                                (16,"SYNTHETIC_MUTATION"), (17,"STATIC_BOUNDARY"),
                                (23,"FINITE_INTERLEAVINGS")]:
        if rows[pos]["classification"] != classification:
            raise ValueError(f"Witness incorrectly classified: {rows[pos]['id']}")
    for name, content in [("README",readme), ("SPEC",spec), ("DECISION",research)]:
        require_tokens(content, ["r0.7", BASE, "TEST-ONLY"], name)
        if not re.search(r"\bnot\b", content, re.I):
            raise ValueError(f"Missing negative claim in {name}")
    require_tokens(spec, [f"H{i} " for i in range(1,13)] + ["5,120", "7,776", "90", "grandchild"], "SPEC")
    require_tokens(rust, [
        '#[path = "../src/resource_task_r02.rs"]',
        "struct Oracle", "impl Oracle", "struct Trial", "impl Trial",
        "const SEEDS: usize = 32", "const STEPS: usize = 160", "const LIMIT: usize = 128",
        "const WORD_ALPHABET: usize = 6", "const WORD_DEPTH: usize = 5",
        "fn enumerate_three_generation_orders()", "R08_WITNESS version=1",
        "self.model.replay_checked()", "self.oracle.step(act)",
        'include_str!("../src/lib.rs")', 'include_str!("../Cargo.toml")',
        "grandchild_failure_propagates_to_root", "grandchild_grants_are_not_inherited_from_ancestors",
        "ninety_three_generation_interleavings_are_canonical",
        "synthetic_oracle_mutation_is_detected_but_not_reported_as_model_defect",
    ], "RUST")
    for pattern in [r"assert_eq!\s*\(\s*actual\s*,\s*expected",
                    r"assert_eq!\s*\(\s*self\.model\s*,\s*before",
                    r"assert_eq!\s*\(\s*self\.oracle\s*,\s*oracle_before"]:
        if re.search(pattern, rust) is None:
            raise ValueError(f"R0.8 missing oracle or atomicity comparison: {pattern}")
    for forbidden in ["std::thread::spawn", "tokio::spawn", "unsafe {", "std::process::Command",
                      "self.model.phase(", "self.model.outcome(", "self.model.report("]:
        if forbidden in rust:
            raise ValueError(f"Forbidden real concurrency/oracle coupling: {forbidden}")
    library = ROOT / "src/lib.rs"
    if library.exists() and ("resource_task_r08" in library.read_text(encoding="utf-8")
                             or "resource_task_r02" in library.read_text(encoding="utf-8")):
        raise ValueError("R0.8 or R0.2 would be exported from frozen library")
    print("R0.8 static contract: PASS")
    print(f"Certified input baseline: r0.7 / {BASE}")
    print("Governance witnesses: 26/26; generated profile: 32 x 160 = 5120 attempted comparisons")
    print("Finite words: 6^5 = 7776; three-generation finish/join orders: 90/90 (finite only)")
    print("Isolation: TEST-ONLY; at most three scopes, two tasks/resources per scope")
    print("Rust compilation, behavioral results, release gate, security proof and CI: NOT RUN by this script")

if __name__ == "__main__":
    validate()
