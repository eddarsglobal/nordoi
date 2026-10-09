#!/usr/bin/env python3
"""Non-certifying R0.2 static package/obligation validator (stdlib only)."""
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = "5a95dad7c7e95498b8bf04319007635bd4048152"


def rows(path):
    with (ROOT / path).open(encoding="utf-8", newline="") as stream:
        return list(csv.DictReader(stream, delimiter="\t"))


def check():
    r01 = rows("governance/r01_resource_task_invariants_v1.tsv")
    r02 = rows("governance/r02_reference_obligations_v1.tsv")
    expected = [f"R01-I{index:02d}" for index in range(1, 17)]
    assert [row["id"] for row in r01] == expected
    assert [row["id"] for row in r02] == expected
    assert all(row["status"] != "CERTIFIED" for row in r01 + r02)
    assert all(row["model_witness"] and row["limitation"] for row in r02)
    source = (ROOT / "src/resource_task_r02.rs").read_text(encoding="utf-8")
    tests = (ROOT / "tests/resource_task_r02.rs").read_text(encoding="utf-8")
    names = [line.removeprefix("fn ").split("(", 1)[0]
             for line in tests.splitlines() if line.startswith("fn ")]
    assert len(names) == len(set(names))
    assert all(row["model_witness"] in names for row in r02)
    test_count = tests.count("#[test]")
    assert test_count == 27, test_count
    assert source.startswith("//! R0.2:")
    assert "pub fn replay_checked" in source
    assert "ExternalIo" in source
    assert "HostPermit" in source
    assert "pub fn apply" in source
    docs = (ROOT / "docs/NORDOI_R0_2_DETERMINISTIC_RESOURCE_TASK_REFERENCE_SPEC.md").read_text(encoding="utf-8")
    assert BASELINE in docs and "NOT RUNTIME CERTIFIED" in docs
    assert "intentionally NOT exported from lib.rs" in source
    print("R0.2 static integrity: PASS")
    print("R0.1 baseline: " + BASELINE)
    print("R01 obligations mapped: 16/16 (MODEL_TESTED or STATIC_CHECKED_ONLY)")
    print(f"Dedicated Rust test cases declared: {test_count} (NOT RUN by this validator)")
    print("Certified source boundaries: MUST VERIFY via git diff and Rust release gate")


if __name__ == "__main__":
    check()
