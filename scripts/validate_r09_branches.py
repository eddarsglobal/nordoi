#!/usr/bin/env python3
"""R0.9 static source/governance audit. Does not run or certify Rust."""
from __future__ import annotations
import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE = "35fc2487ce180ff1fdc1a0f2fa8df3213127865d"
FILES = [
    "README_R0_9.md",
    "docs/NORDOI_R0_9_BRANCH_BOUNDARY_CONFORMANCE_SPEC.md",
    "governance/r09_branch_boundary_witnesses_v1.tsv",
    "research/R0_9_BRANCH_BOUNDARY_ORACLE_DECISION.md",
    "scripts/validate_r09_branches.py",
    "tests/resource_task_r09.rs",
]
COUNT = 23
CATEGORIES = {"GENERATED_DIFFERENTIAL", "FINITE_EXHAUSTIVE", "BOUNDED_DIFFERENTIAL",
              "FINITE_INTERLEAVINGS", "DETERMINISM_EXAMPLE", "SYNTHETIC_MUTATION", "STATIC_BOUNDARY"}


def check() -> None:
    for file in FILES:
        p = ROOT / file
        if not p.is_file() or p.is_symlink():
            raise RuntimeError(f"Missing or unsafe R0.9 file: {file}")
    readme = (ROOT / FILES[0]).read_text(encoding="utf8")
    spec = (ROOT / FILES[1]).read_text(encoding="utf8")
    research = (ROOT / FILES[3]).read_text(encoding="utf8")
    rust = (ROOT / FILES[5]).read_text(encoding="utf8")
    with (ROOT / FILES[2]).open(newline="", encoding="utf8") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", rust)
    if len(tests) != COUNT or len(set(tests)) != COUNT:
        raise RuntimeError(f"Expected {COUNT} unique tests, got {len(tests)}")
    if len(rows) != COUNT or [r.get("id") for r in rows] != [f"R09-B{i:02}" for i in range(1, COUNT + 1)]:
        raise RuntimeError("Governance witness IDs are incomplete or out of order")
    if tests != [r.get("witness") for r in rows]:
        raise RuntimeError("Governance and test order/name mismatch")
    for r in rows:
        if r.get("classification") not in CATEGORIES or len(r.get("limitation", "")) < 70:
            raise RuntimeError(f"Missing/invalid bounded limitation for {r.get('id')}")
    expected_categories = {0: "GENERATED_DIFFERENTIAL", 1: "FINITE_EXHAUSTIVE",
                           19: "FINITE_INTERLEAVINGS", 20: "DETERMINISM_EXAMPLE",
                           21: "SYNTHETIC_MUTATION", 22: "STATIC_BOUNDARY"}
    for index, kind in expected_categories.items():
        if rows[index]["classification"] != kind:
            raise RuntimeError(f"Incorrect class at R09-B{index+1:02}")
    for name, text in (("README", readme), ("SPEC", spec), ("DECISION", research)):
        for term in ("r0.8", BASE, "TEST-ONLY"):
            if term not in text:
                raise RuntimeError(f"{name} missing required {term}")
        if "not" not in text.lower() and "no " not in text.lower():
            raise RuntimeError(f"{name} missing non-claims")
    for term in [
        '#[path = "../src/resource_task_r02.rs"]', "struct Oracle", "impl Oracle",
        "struct Trial", "impl Trial", "const SEEDS: usize = 40",
        "const STEPS: usize = 160", "const LIMIT: usize = 144",
        "const WORD_ALPHABET: usize = 7", "const WORD_DEPTH: usize = 5",
        "fn four_pair_orders()", "R09_WITNESS version=1",
        "self.oracle.step(act)", "self.model.replay_checked()",
        "all_four_task_finish_join_orders_have_canonical_reports",
        "cousin_resources_and_grants_are_scope_exact",
        "root_child_reports_are_sorted_even_when_close_order_is_reversed",
        'include_str!("../src/lib.rs")', 'include_str!("../Cargo.toml")',
    ]:
        if term not in rust:
            raise RuntimeError(f"R0.9 Rust missing required {term}")
    for pattern in (r"assert_eq!\s*\(\s*actual\s*,\s*expected",
                    r"assert_eq!\s*\(\s*self\.model\s*,\s*before",
                    r"assert_eq!\s*\(\s*self\.oracle\s*,\s*oracle_before"):
        if not re.search(pattern, rust):
            raise RuntimeError(f"Independent comparison / atomicity pattern missing: {pattern}")
    for bad in ("std::thread::spawn", "tokio::spawn", "unsafe {", "std::process::Command",
                "self.model.phase(", "self.model.report(", "self.model.outcome("):
        if bad in rust:
            raise RuntimeError(f"Unexpected runtime, unsafe or oracle coupling: {bad}")
    lib = ROOT / "src/lib.rs"
    if lib.exists() and ("resource_task_r09" in lib.read_text(encoding="utf8")
                         or "resource_task_r02" in lib.read_text(encoding="utf8")):
        raise RuntimeError("R0.9/R0.2 exported from frozen library")
    for phrase in ("6,400", "16,807", "2,520", "sibling", "scope"):
        if phrase not in spec:
            raise RuntimeError(f"Missing conformance scope in specification: {phrase}")
    print("R0.9 static contract: PASS")
    print(f"Certified input baseline: r0.8 / {BASE}")
    print("Governance witnesses: 23/23; fixed generated profile: 40 x 160 = 6400 attempted comparisons")
    print("Finite words: 7^5 = 16807; four-task finish/join orders: 2520 (finite only)")
    print("Isolation: TEST-ONLY; at most five scopes in two sibling branches, two tasks/resources per scope")
    print("Rust execution, behavioral results, full release gate, proof and CI: NOT RUN by validator")

if __name__ == "__main__":
    check()
