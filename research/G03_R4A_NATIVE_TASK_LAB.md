# NORDOI G0.3 R4A — Scoped native execution against the existing resource/task model

**LAB_ONLY · RESEARCH_ONLY · NEVER PUBLIC EXECUTOR · PRODUCTION_NOT_AUTHORIZED**

## Why R4A follows the merged R3 work

The R3-R13 atomic write improvement is now merged in main at commit 277d9a1af9aeed4dbd4afb1306e2931bc4439cb6 and the post-merge NORDOI Test Gate succeeded. Further R3 optimizations are not the next priority. The G0.1 delivery planning model marks structured concurrency / memory / ownership at 45% (a **planning estimate**, not certification). Its guidance favors general-purpose resources/tasks before historical filesystem feature parity.

NORDOI already has a sophisticated pure, bounded R0.2 reference model in src/resource_task_r02.rs, exercised by tests/resource_task_r02.rs and later R0.23 synthetic research. The model is NOT exported from lib.rs; it does NOT run threads, host I/O, native resources or user programs. The research/R0_23_FINALITY_BOUNDARY_DECISION.md states synthetic trust is not external authentication or native execution and all 12 native admission criteria remain unmet.

R4A therefore does not create a second task semantics system, and does not claim the existing research is a native runtime.

## Exact test-only experiment

tests/g03_r4a_structured_native_lab.rs imports the existing R0.2 model using a source-relative test path. It invokes REAL Rust scoped OS threads via std::thread::scope for a tiny fixed, deterministic, pure instruction enum (checked add, checked multiply, yield-then-add, synthetic fail, synthetic panic). It is **not a general-purpose callback executor** and is not a sandbox or production primitive.

A bounded number (at most four) of child threads is declared/admitted/started under the R0.2 model before spawning. All threads finish or panic and are joined before their results are mapped to model terminal and Join events. Successful tasks produce a canonical scope report and a vector of values in semantic task order; any task failure discards all candidate outputs. The unmodified kernel is subsequently updated, if and only if the scope was fully successful, through its existing atomic transaction path, using the parent root domain. No child sees the kernel, any host capability, the model or a native resource handle. Empty work spawns zero threads and makes zero NAM writes. This constrains the **test harness** only; it does NOT authorize arbitrary untrusted NORDOI code.

Unit witnesses exercise:
1. Empty scope: no workers, no NAM work.
2. Two actual threads, deterministic joined report, one owner-side atomic transaction.
3. A child failure blocks all writes.
4. A child panic is joined and blocks all writes.
5. More than four tasks rejected before any thread creation.
6. Checked integer overflow fails.
7. Wrong number of output targets fails before any mutation.
8. Ownership denial on a multi-output transaction blocks even the valid first target.
9. Runtime ConsoleWrite remains default-denied after the lab.
10. Equivalent real executions give the same canonical reference outcome.

CI runs the targeted experiment on macOS, Ubuntu and Windows, and the regular NORDOI Test Gate ensures compatibility with the complete repository.

## Future-Native Gate FNG1–FNG6

- FNG1: motivated by C1 C2 C3 C5 C6 C7 C9 C14 C17 C18 C19 C20 and the explicitly incomplete resource/task architecture.
- FNG2: not implementing threads for legacy feature parity; demonstrating a bounded causal boundary between certified atomic NAM and existing modelled structured tasks.
- FNG3: users are never exposed to join loops, task IDs, cancellation bookkeeping, or grant bookkeeping; in a future verified implementation this complexity belongs below the simplest possible language surface.
- FNG4: no user privilege added, no kernel mutation in workers, all outputs gated through an existing atomic ownership-validating transaction, failures cannot partially commit.
- FNG5: std Rust threads and canonical model only, tested on all three available CI OS families. No vendor-specific backend API.
- FNG6: no source/NAIR/compiler/kernel/authority or public language changes; experiment is additive test-only.

## Explicit limitations / NOT claimed

- Native executor admission gates R11-G01 through R11-G12 **remain UNMET**, including cancellation cleanup, lifetime-unsafe host resource handles, nonforgeable host grants, OS IO, crash-safe replay, external review or production.
- No fair scheduler, no async cancellation delivery, no task starvation bounds, no host-resource transfer, no I/O, no GPU or distributed execution, no standard library API, no new .noi syntax, no NAIR opcode, no production sandbox.
- Thread workers run only five hardcoded test operations. The test doesn't prove safe execution of arbitrary user-defined Rust functions or NORDOI programs.
- Rust thread panic is captured by join; the test doesn't prove crash safety or native process isolation.
- Results are not benchmarked. No new speed/footprint or cross-language superiority claim.
- The NORDOI Test Gate checks test-only behavior, **not** universal production readiness.

## Next bounded gate

On successful three-OS lab tests, R4B may independently design the explicit compiler-to-runtime semantic interface, a real nonforgeable authority contract and cancellation semantics, **without** weakening the 12/12 unmet native admission boundary. Avoid another twenty rounds of mock-only approval.

MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
