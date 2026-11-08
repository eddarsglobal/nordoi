# NORDOI G0.3 R3-R11 — Atomic engineering performance baseline

**Owner decision (2026-11-08):** no mandatory human participant studies, no human A/B recruitment, no novice-consent attestation gate. These were retired and their earlier open PRs/issues closed without merging. The research archive remains accessible; the compiler, language, NAIR, NAM, authority and production remain untouched. **Engineer-led development proceeds immediately.**

Source: pinned `main` `98da03b2ea1efcf665d205189284aa8b66fda5d3`. Production frozen: `e2898bf7c511e53636ff28c9fd3efc93165a0215`. Engineering parent issue #35. See `CONSTITUTION.md` C2, C3, C6, C7, C9, C14, C17, C18, C20, C21 and `laws/LAW_0001_NORDOI_MASTER_LAW.md`.

## What the engineering team decides

Engineers lead selection of the baseline competitors (C, Rust, Zig, C++, native assembly, interpreter or runtime baselines where functionally appropriate), design workload parity, identify high-impact bottlenecks and propose optimized NAIR/NAM/compiler/runtime changes. Do not treat a competitor as universally fastest. Choose baselines **per metric and matched supported semantics**. Claims of superiority require correctness/security parity and reproducible measurements, not an impressive isolated number.

## First measurement: R3-R11 isolated, pinned, NORDOI-only

Script `research/g03_r3_r11_engineering_baseline.py` is a Python 3 standard-library executable:

- `selftest`: synthetic-only contract checks, no network, no NORDOI invocation, no real measurements.
- `measure`: clone `main` into a disposable directory; refuse to proceed if the commit is not **exactly** the pinned SHA; build `cargo --offline --release` in isolated `CARGO_TARGET_DIR` (use `--locked` when a lockfile exists); execute the freshly built binary, check stdout/stderr and exit codes on every sample; write one exclusive `0600` local-only JSON report, never modify the owner's original repo.
- Four **whole-process** scenarios: `--version` startup; `core-run -` with proven pure function result `INT(42)`, compile-time-folded to minimal NAIR; `run - --grant-console` with explicit source effect and exactly one observable output; `console-run -` *without* grant, denied with exit 5 and zero program stdout. In particular, the explicit-grant and denied cases are **not interchangeable**.
- Five default warmups and 30 timed iterations *per mode*, interleaved, build duration measured separately. Report exact binary bytes and executable SHA256, host/architecture/compiler versions, per-mode median/p95/min/max, and raw nanosecond samples in a local private report.
- This is **not** a kernel throughput result or cross-language performance comparison; source parsing, whole-process launch and CLI rendering are included. Never infer fastest/smallest from this first diagnostic. A prior R3-R2/R3-R3 study on Cargo vs direct binary is a **different** overhead measurement and remains valuable for context.
- `--output` must be a new file path and is not overwritten. Failure to clone pinned SHA, compile offline or reproduce exact outputs fails closed. No automatic authority expansion.

### Mac Intel commands

After downloading the pinned script from a verified GitHub blob, use:

```sh
python3 -B "$SCRIPT" selftest
python3 -B "$SCRIPT" measure --out "$HOME/Downloads/NORDOI_G03_R3_R11_PRIVATE_$(date -u +%Y%m%dT%H%M%SZ).json" --samples 30 --warmups 5
```

No code in the original checkout is changed; compilation takes place in a throwaway clone. The private JSON remains on Mac. On machines where `cargo` or `rustc` is missing, measurement is BLOCKED; do not invent substitutes. The optional one-shot CLI outputs may be used by engineers as baseline evidence but not as certification of the entire NORDOI language.

## Next engineering decision: R3-R12 optimization proposal

Only after actual Mac baseline, engineers identify hot paths and build a **workload-equivalent** benchmark comparing appropriate competitor languages, with explicit notes on compilation stage, warm/cold runs, number of instructions, I/O differences and safety behavior. Candidate axes: startup latency, binary size, memory footprint, steady-state computation, throughput under dynamic input, number of unavoidable semantic steps per observable effect, transactional rollback and unused-feature zero-cost. The team selects priorities using evidence.

**No external human testing gate.** Technical CI/tests and security/invariant gates remain fully binding under C14/C20. No new `.noi` grammar or experimental runtime optimization is authorized by this research branch alone. Production remains NOT_AUTHORIZED.
