# NORDOI G0.3 R3-R7 — Beginner error guidance (research candidate)

**NON-NORMATIVE / NOT ADOPTED / NO CLI OR LANGUAGE MODIFICATION.** Follow-up to issue #25, draft PR #26 and issue #27.

Source `main`: `98da03b2ea1efcf665d205189284aa8b66fda5d3`. Frozen production: `e2898bf7c511e53636ff28c9fd3efc93165a0215`.

## Evidence boundary

- R3-R6 owner Mac Intel technical audit: 10/10 PASS on a **private** JSON report; never upload it to GitHub.
- R3-R6 exact-head GitHub workflows: 5/5 Rust + 3/3 Python PASS on `13c6523f151e521d74a727c01692929bdb4bf7f8`.
- Human comprehension **NOT MEASURED**. Earlier R2 A=1/B=0 and independence unverified. Technical pass cannot prove novice understanding.
- No authority, compiler, syntax, runtime, receipt or diagnostic behavior change is permitted by this research.

## First successful program — unchanged

```noi
module app.main;

effect ConsoleWrite;

entry main emits "Hello from NORDOI P2.1!\n";
```

```sh
cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console
```

The source `effect ConsoleWrite;` declares intent; the separate host `--grant-console` grants authority. One does not imply the other. Program stdout and stderr authority receipts are distinct.

## Candidate teaching format (DOCUMENTATION ONLY)

Use three short lines: **Problem → Why → Next safe action**. Keep raw stderr and exit code unchanged. Never grant permissions automatically. Full `nordoi --help` stays available on demand.

Example, source effect missing: **Problem:** output effect was not declared. **Why:** a host grant is not a source declaration. **Next:** add `effect ConsoleWrite;` only if intended, and deliberately review any grant.

| Observed R3-R6 case | Existing exit / stage | Proposed novice explanation / safe action |
|---|---|---|
| valid_run | 0 / success | Program succeeded; stdout is output, stderr is the receipt. |
| missing_grant_console_run | 5 / console-run | Console authority refused; review whether explicit grant is wanted. |
| undeclared_effect | 4 / console-run | Source lacks `effect ConsoleWrite;`; grant and declaration differ. |
| misspelled_emits | 4 / console-run | Correct the `emits` spelling; don't enlarge permissions. |
| invalid_option | 2 / usage | Check accepted flags; ask for full `nordoi --help` if needed. |
| missing_argument | 2 / usage | Supply source path or `-` for stdin. |
| unknown_command | 2 / usage | Correct `rnu` to `run`. |
| missing_source_file | 3 / io | Check file path relative to working directory. |
| invalid_utf8 | 3 / input | Save source as UTF-8; this is not a capability error. |
| legacy_plain_run | 4 / run | This is the old HALT-only route, not the `console-run` denial. |

`run ... --grant-console` delegates to an old handler, so its errors may say `error[console-run]`; usage failures currently emit ~12 KB of help. Neither observation establishes actual novice confusion.

## Safe technical verification

The additive `research/g03_r3_r7_beginner_error_review.py` is Python standard library. It validates exactly ten real R3-R6 recorded cases, code and stage, byte counts, silent failed stdout, separate legacy/permission path, exact source pins, and prints **only static candidate advice**, never raw report data. It does not invoke NORDOI or write output.

```sh
python3 -B research/g03_r3_r7_beginner_error_review.py selftest
python3 -B research/g03_r3_r7_beginner_error_review.py review --input "$HOME/Downloads/NORDOI_G03_R3_R6_20261107T150311Z_90154.json"
```

The real report remains on the owner's Mac. CI runs only synthetic selftests; **CI never uses owner private data**. Inaccessible/mismatched report is BLOCKED.

## Adoption gate

Independent true beginners, preregistered A/B comprehension questions and safe-repair criteria required before claims of superiority. Preserve source correctness, permissions, zero runtime overhead and low cognitive burden simultaneously. Any future CLI help integration needs separate compatibility/security review and authorization. Keep PR #26 untouched, PR R3-R7 draft/unmerged and production NOT_AUTHORIZED.
