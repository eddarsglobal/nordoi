# NORDOI G0.3 R3-R6 — First-program error audit (research candidate)

**Status: NON-NORMATIVE TECHNICAL AUDIT.** Do not treat this inventory as a human comprehension test, an adopted language/error-format standard, or permission to modify runtime behavior.

Source pin: `main` `98da03b2ea1efcf665d205189284aa8b66fda5d3`. Frozen production: `production/p0.1` `e2898bf7c511e53636ff28c9fd3efc93165a0215`.

## Verified source contracts

The official first-use command remains:

```sh
cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console
```

The source `effect ConsoleWrite;` **declares intent**, while `--grant-console` separately **grants execution authority**. Removing the CLI grant must not produce unauthorized console output. The historical `console-run` missing-grant denial and ungranted `run` HALT-only frontend remain *distinct contracts*.

The pinned CLI source (`src/bin/nordoi.rs`, Git blob `e126a3f8ffe5a45ac33c23323ef3de2ccdfbc35d`) defines these exit categories: `0` success, `2` usage, `3` load/I/O, `4` frontend/semantic, `5` runtime/authority. This is a useful **internal diagnostic taxonomy**, not a guarantee that different operations sharing a code are interchangeable.

## Error examples and safe, explanatory next action

| User situation | Existing behavior to verify with binary | Safe beginner explanation / next action |
|---|---|---|
| Correct P2.1 program with grant | `0`, exact program stdout, separate explicit-authority receipt on stderr | Successful run; receipt is not program output |
| `console-run` without grant | `5`, no stdout, `capability denied` | Permission was not granted; decide whether to explicitly authorize the intended console effect |
| `run --grant-console` with undeclared `ConsoleWrite` effect | `4`, no stdout, source policy diagnostic | Granting a capability is **not** the same as declaring an effect; confirm the intended source declaration |
| Typo `emtis` for `emits` | `4`, no stdout, expected `emits` error | Check the spelling in the program; no need for broader permissions |
| Invalid flag, unknown command, or missing path argument | `2`, `error[usage]`; general CLI help printed on stderr | Check command spelling/position/flag; do not modify source permissions |
| Missing source file | `3`, `error[io]` | Check path relative to repository root and file existence |
| Invalid UTF-8 stdin | `3`, `error[input]` | Supply a valid UTF-8 `.noi` source; this is not a capability denial |
| Plain `run` without grant on P2.1 source | Historical HALT-only frontend failure; not necessarily exit 5 | It is a **different execution route**, not a successful or implicit grant of `ConsoleWrite` |

**Important UX observation:** `run ... --grant-console` is delegated to the P2.1 `console-run` implementation, so a new user's error may be prefixed **`error[console-run]`** even though they typed **`run`**. The CLI currently appends its long general `--help` text for usage errors. Both are plausible sources of confusion, **not findings from a real novice study**. Changing these lines directly would risk existing error contracts, so this milestone makes **no diagnostic-emission change**.

## Audit method and evidence levels

1. **Static evidence:** inspection of pinned CLI source, `src/observable_io_p21.rs`, source frontend error implementation and existing P2.1 CLI tests.
2. **Executable CI evidence:** new additive `tests/tooling_first_error_audit_g03.rs` executes actual existing NORDOI binary on GitHub macOS, Windows and Ubuntu to guard exit codes, program-output silence, error stages and authority separation. CI PASS validates observable technical contracts, not cognitive clarity.
3. **Mac local evidence:** run the pinned read-only technical audit script in a separate isolated shallow clone with the exact-source pin, offline release build, synthetic stdin fixtures and a private 0600 report. No human study files are read.
4. **Human evidence:** incomplete R2 study has A=1/B=0 with human independence unverified; no pedagogical-success or learning claim is authorized.

## Pre-adoption ideas — no implementation authorization

- Provide a compact **documentation-only troubleshooting map** with the safe next action for each diagnostic class; do not auto-insert `--grant-console`.
- For any future machine-generated hint, distinguish what the user *typed* from the internal command handler, preserve original error code and underlying stderr, and never rewrite evidence receipts.
- For usage errors, consider a targeted "valid example" plus a link to full help, but only after technical regression review and independent novice evaluation.
- Keep failed program stdout empty; do not expose private source or inferred permissions in diagnostic UI.

## R3-R6 exit criteria

Rust executable regression tests PASS on exact candidate CI, explicit Mac report succeeds, and the final technical inventory is reproducible. Human usability remains **UNMEASURED** and production **NOT_AUTHORIZED**. No changes to `.noi` syntax, original quickstart, original README, compiler, runtime, kernel, NAIR/NAM, effects, grants, R2 study or production.
