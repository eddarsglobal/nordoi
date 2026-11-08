# NORDOI G0.3 R3-R12 — In-process atomic engineering baseline

**Research-only, unmerged candidate.** Tracking issue #37, engineering mission #35 and R3-R11 PR #36. The owner closed all human-study gates: no volunteers, consent attestations or signatures are required for development.

## Why

Owner Mac Intel R3-R11 measured whole-process CLI medians ~5.7–6.05 ms, release binary 2,949,456 bytes, offline release build 28.9254 s. CLI timing includes process creation, initialization and output handling: it is not the core engine's execution time. The V0.6 pure computation sample was constant-folded into two NAIR instructions, not a dynamic arithmetic benchmark.

## Six in-process measurements

`research/g03_r3_r12_kernel_probe.rs` links directly to the unchanged `nordoi_kernel` release library using `rustc --extern`; it lives under `research/`, not `src/`. No public language or runtime change.

| Case | Timed API path | Contract |
|---|---|---|
| `noop_set` | Same-value `AtomicKernel::set` | no change, no version or scheduled work |
| `changed_set_flush` | Alternating set + flush | one frontier atom, correct version |
| `dependency_set_flush` | Alternating set + flush with one dependent | two frontier atoms, deduplicated |
| `noop_transaction` | Begin, stage same value, commit | no changed or scheduled atoms |
| `denied_capability` | `require(ConsoleWrite)` without grant | always refused, no mutation |
| `nair_const_halt` | `execute_nair` on `CONST 42; HALT` | two instructions, no observable state work; **includes repeated NAIR validation** |

Fresh kernel initialization is outside timed sections. The Rust probe rotates case order across blocks and reports per-block whole-loop wall time divided by iterations as integer ns/op; this includes API validation, per-call assertions, black-box and loop overhead. **It is neither a precise kernel instruction counter nor a fair speedup comparison between cases that do different work.** Selftest checks invalid duplicate-register NAIR fails before mutation.

Proposed Mac Intel: 7 blocks × 10,000 iterations, per-case raw series. Sample CI uses 3 blocks × 100 iterations **only as functionality smoke**, not a real speed benchmark.

## Engineering safeguards

Use exact source commit and source Git blob. Build the original library offline in a disposable clone, link and execute this research-only probe, record the report under `~/Downloads` with private permissions. Do not modify the original repository. Run the existing Rust regression gate in CI. No new dependencies, permissions or shipped binary. Do not merge the draft PR or authorize production until technical review and owner Mac evidence.

Future R3-R13: choose a concrete bottleneck from repeatable in-process measurements, instrument allocations and work frontier where relevant, then implement a minimal guarded optimization with before/after matched workloads. Cross-language C/C++/Rust/Zig comparisons require equal semantics, authority and build conditions. No ranking before data. Human-study gates remain closed.
