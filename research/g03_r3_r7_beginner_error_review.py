#!/usr/bin/env python3
"""R3-R7 research-only verifier: no writes, no NORDOI invocation, no raw stderr print."""
import argparse
import json
from pathlib import Path
import sys

SCHEMA = "NORDOI-G03-R3-R6-FIRST-ERROR-AUDIT-v1"
MAIN = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
PRODUCTION = "e2898bf7c511e53636ff28c9fd3efc93165a0215"
# id, exit code, observed stage, immutable diagnostic marker, proposed safe action
CASES = (
    ("valid_run", 0, None, "authority=EXPLICIT", "Success: stdout is program text; stderr is an explicit authority receipt."),
    ("missing_grant_console_run", 5, "console-run", "capability denied: ConsoleWrite", "Permission refused. Review the intended console effect; grant only by deliberate choice."),
    ("undeclared_effect", 4, "console-run", "requires explicit 'effect ConsoleWrite;'", "Add the missing source declaration only if console output is intended. A grant is not a declaration."),
    ("misspelled_emits", 4, "console-run", "expected contextual 'emits'", "Correct the source spelling to 'emits'; do not widen permissions."),
    ("invalid_option", 2, "usage", "error[usage]", "Check supported flags; consult 'nordoi --help' if needed."),
    ("missing_argument", 2, "usage", "error[usage]", "Provide a source path or '-' for standard input."),
    ("unknown_command", 2, "usage", "unknown command 'rnu'", "Correct 'rnu' to 'run'."),
    ("missing_source_file", 3, "io", "error[io]", "Check the path and working directory."),
    ("invalid_utf8", 3, "input", "not valid UTF-8", "Save source as UTF-8; this is not a permission problem."),
    ("legacy_plain_run", 4, "run", "error[run]", "Plain 'run' is the legacy route; do not mistake it for the console grant refusal."),
)

def validate(report):
    expected = {
        "schema": SCHEMA, "source_sha": MAIN, "production_sha": PRODUCTION,
        "case_count": 10, "passed": 10, "source_modifications": "NONE",
        "diagnostic_rewrites": "NONE", "production": "NOT_AUTHORIZED",
    }
    if not isinstance(report, dict) or any(report.get(k) != v for k,v in expected.items()):
        raise ValueError("report metadata mismatch")
    events = report.get("events")
    if not isinstance(events, list) or len(events) != len(CASES):
        raise ValueError("wrong event count")
    by_id = {}
    for item in events:
        if not isinstance(item, dict) or not isinstance(item.get("id"), str) or item["id"] in by_id:
            raise ValueError("duplicate or invalid event")
        by_id[item["id"]] = item
    if set(by_id) != {c[0] for c in CASES}:
        raise ValueError("event IDs mismatch")
    for name, code, stage, marker, _ in CASES:
        item = by_id[name]
        if item.get("exit_code") != code or item.get("error_stage") != stage:
            raise ValueError("unexpected exit code or stage")
        out, err = item.get("stdout_text"), item.get("stderr_text")
        if not isinstance(out, str) or not isinstance(err, str) or marker not in err:
            raise ValueError("missing or invalid output")
        if item.get("stdout_bytes") != len(out.encode()) or item.get("stderr_bytes") != len(err.encode()):
            raise ValueError("transcript byte lengths mismatch")
        if name == "valid_run":
            if out != "Hello from NORDOI P2.1!\n" or "ambient-authority=NONE" not in err:
                raise ValueError("incorrect successful output or authority receipt")
        elif out:
            raise ValueError("failed program emitted output")
    if by_id["missing_grant_console_run"].get("args") != ["console-run", "-"]:
        raise ValueError("not an actual console-run permission refusal")
    if by_id["legacy_plain_run"].get("args") != ["run", "-"]:
        raise ValueError("not an actual plain-run legacy case")
    if "capability denied" in by_id["legacy_plain_run"]["stderr_text"]:
        raise ValueError("legacy route conflated with permission refusal")
    return True

def selftest():
    marks = {
        "valid_run": "authority=EXPLICIT ambient-authority=NONE",
        "missing_grant_console_run": "capability denied: ConsoleWrite",
        "undeclared_effect": "requires explicit 'effect ConsoleWrite;'",
        "misspelled_emits": "expected contextual 'emits'",
        "invalid_option": "error[usage]", "missing_argument": "error[usage]",
        "unknown_command": "unknown command 'rnu'",
        "missing_source_file": "error[io]", "invalid_utf8": "not valid UTF-8",
        "legacy_plain_run": "error[run]",
    }
    events = []
    for name, code, stage, _, _ in CASES:
        out = "Hello from NORDOI P2.1!\n" if name == "valid_run" else ""
        err = marks[name]
        events.append({"id":name,"exit_code":code,"error_stage":stage,"stdout_text":out,
                       "stderr_text":err,"stdout_bytes":len(out.encode()),"stderr_bytes":len(err.encode()),
                       "args":["console-run","-"] if name=="missing_grant_console_run" else ["run","-"]})
    fixture = {"schema": SCHEMA, "source_sha":MAIN,"production_sha":PRODUCTION,
               "case_count":10,"passed":10,"source_modifications":"NONE",
               "diagnostic_rewrites":"NONE","production":"NOT_AUTHORIZED","events":events}
    assert validate(fixture)
    for field, value in (("passed",9),("production","AUTHORIZED")):
        bad = {**fixture, field:value}
        try:
            validate(bad)
        except ValueError:
            continue
        raise AssertionError("invalid report unexpectedly accepted")
    print("G03_R3_R7_SELFTEST=PASS")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("selftest")
    review = commands.add_parser("review")
    review.add_argument("--input", required=True)
    args = parser.parse_args()
    if args.command == "selftest":
        selftest()
        return 0
    try:
        p = Path(args.input)
        if p.is_symlink() or not p.is_file() or p.stat().st_size > 524288:
            raise ValueError("invalid report file")
        with p.open("rb") as fh:
            raw = fh.read(524289)
        if len(raw) > 524288:
            raise ValueError("size limit")
        validate(json.loads(raw.decode("utf-8")))
    except (ValueError, OSError, UnicodeError):
        print("G03_R3_R7_REVIEW=BLOCKED", file=sys.stderr)
        return 2
    print("G03_R3_R7_READONLY_REVIEW=PASS")
    print("R3_R6_EVIDENCE=10/10_VERIFIED")
    print("PEDAGOGICAL_HINTS=RESEARCH_ONLY_NOT_EMITTED_BY_NORDOI")
    for name, _, _, _, suggestion in CASES:
        print(name + ": " + suggestion)
    print("HUMAN_COMPREHENSION=NOT_MEASURED")
    print("PRODUCTION=NOT_AUTHORIZED")
    return 0

if __name__ == "__main__":
    sys.exit(main())
