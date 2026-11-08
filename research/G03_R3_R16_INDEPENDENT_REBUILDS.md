# NORDOI G0.3 R3-R16 — Independent rebuild controls and denied-capability diagnosis

**Engineering research only; no automatic merge; production NOT AUTHORIZED.**

## Motivation from Mac Intel data
- R3-R14: 3 balanced sessions, 24/24 noop_set blocks favorable, paired session changes +22.422%, +18.813%, +19.257%. No software proof of independent host conditions.
- R3-R15: noop_set AB +21.9846% (12/12 positive), changed_set_flush AB +4.4561% (10/12 positive), denied_capability AB -1.4296% (3/12 positive), AA same-binary placebo denied_capability -0.2037%.
- Baseline source apply_single calls ensure_exists before get, which also verifies atom existence. Candidate removes only the duplicate check; the capability check require() is unchanged. A systematic ~0.1 ns difference on a ~6.9 ns permission check is measurable but not evidence of lost security or source-level causal regression.

## Procedure
Exact pinned baseline main SHA: 98da03b2ea1efcf665d205189284aa8b66fda5d3.
Exact pinned R3-R13 candidate SHA: a1ae15326eaa837f8994a75fa7cfbe2eccc44976.
R3-R13 Python tool Git blob: dac0109959e10a15764ff54a4da91bf190deff59.
Rust R3-R13 probe SHA256: be6bbd0b74ca8ff12035ee2e8529961285767251f26074d2ce1b4663d723204a.
Rust R3-R16 security probe Git blob: 4080350aab5c3b2f377f7c22bca54d49c50e6df7.

The owner Mac Intel performs 4 pinned, independent source clones and 4 optimized/offline Rust builds, one per build directory. The tool checks that both baseline clones have exactly the same tracked content, both candidate clones have exactly the same tracked content and the only baseline/candidate code change is the exact redundant-existence removal. It compiles the same verified external R3-R13 probe for all builds. It compiles and runs a separate security-contract probe on **all four builds**, checking default-denied ConsoleWrite, explicit ConsoleWrite allow, no authority leakage across kernels, ownership denial, unknown-atom rejection, no-op writes/version, transaction conflict and all-or-nothing state.

Three cases: noop_set, changed_set_flush, denied_capability.
Three comparisons in each case: AA = two independently built baseline binaries; BB = two independently built candidate binaries; AB = baseline A versus candidate A. 12 ordered blocks per case by default, 30,000 repetitions per executable arm per block, balanced 6 left-first/6 right-first in every pair. Pair group order is rotated. Rust times only the internal call loop, not process startup; all calls include correctness checks.

The report records exact per-block nanosecond totals, paired deltas, build hashes and sizes, rustc/cargo versions, exact SHA of code section via otool when available, and the safety tests. Different executable hashes or Mach-O text-section hashes show binary-level differences but do NOT identify the require() function's exact machine instructions, nor prove that any observed timing delta was caused by binary layout. Equal binary hashes do not prove all execution conditions equal.

## Decision
If AA and BB independently built controls show timing differences comparable to AB, binary-build/layout/timing noise is a plausible explanation, not proof. If the denied-capability AB slowdown materially exceeds both controls and persists, investigate at assembly and host noise level before fusion. Interpret absolute ns/op, not only relative percentages. Report only as ENGINEERING_REVIEW_ONLY and MERGE_AUTHORIZATION=NONE. Never auto-merge based on timings or synthetic tests. One Intel Mac and 12 blocks cannot establish language superiority.

NORDOI's human-simplicity and speed principles remain required; no new syntax, public API, effect authority, parser, compiler or production change in this R3-R16 PR. No volunteer A/B or human signature gate. No production authorization.

## Local command
Download the R3-R16 Python runner and Rust security probe from their **exact GitHub commit** and verify both Git blob IDs before execution; reuse the Git-pinned R3-R13 Python runner under Downloads. Example:
python3 -B "$R16" selftest
python3 -B "$R16" measure --r13-tool "$R13" --security-rust "$SAFETY" --out "$PRIVATE_JSON" --iterations 30000 --blocks 12

The CI only runs synthetic selftest and a Rust real security contract smoke on GitHub-hosted macOS/Ubuntu/Windows. CI is **not a real owner-Mac benchmark**.

FINAL: HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED; MERGE_AUTHORIZATION=NONE; PRODUCTION=NOT_AUTHORIZED.

## R3-R16 FIX1 — protocol correction (2026-11-08)

The original R3-R16 Mac report successfully checked four independent build identities and four security contracts. Its AA and BB independent-build controls are structurally valid, **but the group placement order is confounded**. The initial scheduler rotated by `(block + offset) % 3` while choosing `case = CASES[(block + offset) % 3]`, so each individual case always had one fixed group position sequence across all 12 blocks. Specifically, the observed order was: `noop_set: AA,BB,AB`; `changed_set_flush: BB,AB,AA`; `denied_capability: AB,AA,BB`, repeated in every block. Arm ordering within each pair *was* 6/6 balanced, but **group placement was not balanced**. Consequently, the recorded -1.4288% denied-capability AB slowdown cannot be causally attributed to source code or layout until the group order issue is corrected. The R3-R16 original receipt is retained unchanged as exploratory evidence.

FIX1 changes only the measurement driver; the baseline/candidate source and Rust safety probe stay identical. For each case it now rotates group placement using `group_permutation(block, per_case_phase)` and checks that each of AA, BB and AB appears exactly `blocks/3` times at each of first, middle, last position. Valid block counts are 12, 18 or 24. The new synthetic negative test rejects any fixed-position confound. Result schema `NORDOI-G03-R3-R16-FIX1-INDEPENDENT-CONTROLS-v1` distinguishes this from the earlier report. A fresh Mac FIX1 report is needed, not a retroactive relabeling of earlier data. One extreme prior denied-capability A/B block (49,977,445 ns over 30,000 ops) must remain in archived raw data; a robust median masks its influence, but it indicates host timing jitter.

No automatic merge. A verified owner Mac FIX1 report still requires engineering assessment of the absolute 0.1–0.2 ns/op change and any actual assembly/code-layout difference; test security correctness independently. No human-evaluation gate and no production approval.
