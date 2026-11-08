# NORDOI G0.3 R4B — Cooperative native checkpoint, cancellation and owner-only commit

**RESEARCH LAB ONLY — NO PUBLIC NORDOI TASK EXECUTOR — NO NATIVE ADMISSION — NO PRODUCTION**

## Parent and boundary

R4B is stacked on the existing **draft, unmerged R4A PR #56**, exact commit `5b500e5c5cb2c67910ea56a34e81967995b73320`. Baseline `main`: `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6` after the authorized R3-R13 kernel optimization merge. R4A supplied 10/10 real fixed-operation Rust scoped-thread tests on owner Mac Intel, and 3/3 hosted OS CI jobs + 5/5 general Test Gate. R4B does not merge its parent or write to `main`.

NORDOI already has an **in-process pure reference** model at `src/resource_task_r02.rs`, and 12 native-admission gaps in `governance/r11_native_readiness_gates_v1.tsv`. Even R0.23's finality cases remain mock-local and research-only. No witness in that series proves a runtime executor or nonforgeable host-native capability.

## What R4B implements, precisely

`tests/g03_r4b_native_cancel_authority_lab.rs` is additive and test-only; it imports the **unchanged R0.2 model** and links the already-certified **unchanged K1.18 NAM kernel**. No new public runtime module, syntax, command, NAIR instruction, process privilege, native device handle, network/filesystem I/O or grant is added. The lab runs a **fixed** program enum (checked integer add/multiply, one synchronization checkpoint, explicit failure, synthetic panic); it does NOT execute arbitrary guest `.noi` or arbitrary untrusted Rust.

- Parent validates the plan first: max **four** native workers, max **one** checkpoint, and cancellation can refer only to that checkpoint. An invalid plan fails **before spawning**.
- Each worker is a **real Rust scoped OS thread**; parent owns the R0.2 model, host permit and K1.18 NAM kernel. No kernel, NAM pointer/AtomId or model/host grant crosses to any worker.
- A zero-capacity channel establishes that one worker has **already reached a blocking checkpoint**. Parent applies `RequestCancel` in the R0.2 model, sets a shared `AtomicBool` with **Release** ordering, and opens a `Barrier`. Worker checks with **Acquire** and returns `Cancelled`; parent applies `AcknowledgeCancel` and `Join`. Without cancel, the worker proceeds to checked pure computation.
- Every native worker handle is joined before model terminal events and canonical `Close` are recorded. The immutable R0.2 replay checker is used as a reference consistency witness. Completion reports are sorted by model task identity, not host scheduling order.
- A **separate parent-only commit** accepts only a fully successful, replay-checked scope; rejects arity/duplicate targets; uses the existing NAM atomic transaction for all targets and retains its normal root-owner validation. Any cancellation, failure, panic, wrong arity, duplicate target or ownership failure => no partial observable NAM mutation.
- Empty input creates **zero worker threads** and **zero NAM transaction work**.
- An independent R0.2-only modeled grant test validates explicit declaration and host-grant/revocation semantics. It never executes real external I/O or passes model grant material into threads.

The 15 targeted Rust tests cover these bounds, including a **live** worker checkpoint and cancel acknowledgement, plus an error path ensuring the parent never releases its scope before children join.

## Evidence required

1. `cargo test --offline --release --test g03_r4b_native_cancel_authority_lab`: 15 real Rust integration tests PASS on Ubuntu, macOS, Windows, including true thread cooperation.
2. `cargo fmt --all -- --check` and `cargo clippy --offline --all-targets -- -D warnings` (general Test Gate covers the latter).
3. Full existing NORDOI GitHub Test Gate 5/5 PASS on exact R4B commit.
4. Owner Mac Intel isolated clone of exact commit; command checks pinned Git SHA + test file Git blob, then runs the real 15 tests.
5. No user-language/production semantics changes; no benchmark-based performance claim, no merge without explicit owner direction.

## Future-Native Gate FNG1–FNG6

- FNG1 constitutional: C1 C2 C3 C5 C6 C7 C9 C11 C12 C13 C14 C17 C18 C19 C20; implementation evidence gaps in real concurrency/ownership.
- FNG2: advances canonical task semantics and real synchronization rather than copying legacy thread APIs.
- FNG3: user-facing future NORDOI language stays extremely simple; no task ID/join/cancellation flag leaks to public syntax.
- FNG4: default deny, explicit modeled grants, parent-only NAM, closed scope, no partial commit.
- FNG5: pure Rust std thread primitives, platform-independent model, cross-OS CI; no vendor-specific scheduler assumptions.
- FNG6: reference source, CLI, compiler/NAIR, NAM kernel and all certified production boundaries remain unchanged.

## R4B limits — NOT established by 15 green tests

Cooperative cancellation **only at one known checkpoint**, not preemptive or forced termination. No general task scheduler, fairness, arbitrary untrusted code, deadline, real OS resource ownership/handle revocation, native authenticated nonforgeable authority, memory-unsafe FFI, external effect enforcement, crash/restart recovery, formal proof of liveness, universal runtime performance, or distributed execution. The host model's `HostPermit` is **not** a security boundary against malicious Rust code.

The R0.2 model is still an experimental oracle; agreement in this lab is not proof of production runtime security. **All 12 native admission gates R11-G01..R11-G12 remain UNMET**, including independent external authorization and any production release. No third-party human evaluator/signature requirement is imposed on the current research lab.

## Next gate, if R4B passes

Do not repeat endless mock transcript loops. R4C, if opened separately, must validate one truly **narrow** operational native boundary (explicit host adapters and cancellation cleanup under adversarial races) *without* exposing a new `.noi` surface before semantics and authority are proven.

```text
G03_R4B=RESEARCH_ONLY
PRODUCTION=NOT_AUTHORIZED
NATIVE_ADMISSION_GATES=12_UNMET
PARENT_PR56=DRAFT_UNMERGED
MERGE_AUTHORIZATION=NONE
```
