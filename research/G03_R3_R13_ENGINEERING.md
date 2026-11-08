# NORDOI G0.3 R3-R13 — Single-lookup atomic-kernel candidate and paired engine proof

**ENGINEERING ONLY · DRAFT · NO HUMAN-STUDY GATE · PRODUCTION NOT AUTHORIZED.**

## Why this candidate?

G0.3 R3-R11 measured the whole NORDOI CLI process (~5.7–6.0 ms median). R3-R12 measured direct Rust API boundaries on the owner's Mac Intel: identical atomic write 57 ns/op, changed write + flush 283 ns/op, no-op transaction 245 ns/op, NAIR CONST+HALT with validation 498 ns/op. These figures have different work/semantics and cannot establish comparative superiority.

In baseline `main` `98da03b2ea1efcf665d205189284aa8b66fda5d3`, the private `AtomicKernel::apply_single` starts with `self.ensure_exists(id)?` and immediately calls `self.get(id)?`, which itself rejects `UnknownAtom` if no entry exists. The prior public `set_owned` already verifies ownership and atom identity. The candidate removes exactly the redundant lookup, keeping `self.get(id)?` and all ownership, capability, transaction and dependency guards unchanged. No new source syntax, NAIR semantic feature, runtime authority or release.

**This is an optimization hypothesis, not a speed gain.** Compiler optimization might already eliminate some redundant work; the candidate can be faster, equal or slower.

## Engineering gate

- Pinned baseline SHA `98da03b2ea1efcf665d205189284aa8b66fda5d3` and candidate exact HEAD; fail closed on branch drift.
- Python `verify_delta` requires candidate `src/kernel.rs` bytes to equal the baseline with *only* that exact replacement. It rejects other changes to baseline-tracked files and unlisted new files. No original user repo edits.
- Three dedicated Rust regression tests: no-op writes/versions, missing atom error and denied host capability; changed write + deduplicated dependency work; ownership denial and no-op transaction.
- Identical external Rust probe is compiled with the **same Rust flags**, separately linked against baseline/candidate offline release `nordoi_kernel` libraries. Each probe must PASS before sampling.
- Cases: `noop_set`, `changed_set_flush`, `dependency_set_flush`, `noop_transaction`, `denied_capability`, `nair_validate` and `nair_execute_with_validation`. The final case *includes validation again*. Do not subtract medians to infer an execution-only cost.
- Seven default rotated paired Mac blocks, 30,000 operations per sample/arm/case; A→B and B→A interleaved. Report ns/op medians, paired-delta medians, positive vs negative blocks and raw timing samples. Host-load noise and process differences remain limitations; the Rust timed loop excludes process startup. Run independent sessions before selecting the candidate.
- JSON receipt created only once, private `0600`, local to Mac; CI runs correctness smoke **without** publishing any performance claim.
- Require GitHub 3-OS CI and existing NORDOI Test Gate; do not merge or authorize production based on a synthetic selftest or unreplicated microbenchmark.

## Running on owner Mac

A single, pinned Python runner `research/g03_r3_r13_paired_mac.py` offers `selftest`, `smoke`, and `measure --candidate-sha <exact SHA> --out <new private JSON> --iterations 30000 --blocks 7`. The shell wrapper must verify the downloaded Git blob before executing. Baseline and candidate are cloned into separate disposable folders and compiled offline, then verified. Keep private reports off GitHub.

Decision standard: only repeatable paired improvement with unchanged correctness/invariants justifies merging this **one-line** code change. If the difference is within noise, revert/reject; do not cherry-pick samples. Engineering review/AI testing is allowed; external volunteers and signatures are **not** required. C2/C3/C6/C7/C14/C17/C18/C20 preserved.

`PERFORMANCE_GAIN=NOT_ESTABLISHED`. `CROSS_LANGUAGE_SUPERIORITY=NOT_ESTABLISHED`. `PRODUCTION=NOT_AUTHORIZED`.
