# NORDOI K1.0 — First Closed Atomic Runtime

K1.0 is the first major NORDOI kernel milestone. It closes the bootstrap execution
loop built across K0.1–K0.9 without freezing the future source language.

## Certified architecture target

```text
Authorized host input
        ↓
Canonical InputBatch
        ↓
      NAIR 0.3
        ↓
        NAM
        ↓
Atomic Render Core
        ↓
Quiescent closed activation
        ↓
RuntimeReport + deterministic replay key
```

## What K1.0 adds

- `AtomicRuntime` as the first closed execution capsule.
- `run_closed()` convenience entry point.
- canonicalization of publicly supplied `InputBatch` values.
- strict monotonic input-sequence validation.
- deterministic canonical input bytes.
- `RuntimeReplayKey` derived from canonical NAIR + canonical input.
- final atom snapshots keyed by semantic `AtomSlot`.
- mandatory zero-residual-work success condition.
- closed-runtime error isolation: failed activations return no partial report.
- zero new external Rust dependencies.

## Important replay-key rule

The K1.0 replay key is a deterministic engineering fingerprint, **not** a
cryptographic hash or signature. It must never be used as authorization or proof of
authenticity.

## Scope

K1.0 closes one complete isolated activation. Persistent multi-frame application
sessions, final NORDOI source syntax, compiler frontend, privileged external effects
and platform backends remain future layers.

This preserves the architectural order:

```text
semantics → NAM → NAIR → closed runtime → compiler/frontend
```

rather than freezing syntax before the execution model is proven.

## Test corpus

K1.0 adds **12 runtime tests** to the 96 inherited K0.9 tests, for a total of
**108 tests**.

New tests cover:

- complete input → NAM → render execution through `AtomicRuntime`;
- deterministic repeated execution;
- replay-key sensitivity to program/input changes;
- canonical negative-zero replay identity;
- rejection of non-monotonic public input batches;
- rejection of non-finite public input batches;
- XR pose canonicalization at the runtime boundary;
- quiescent state-only execution;
- deterministic empty-input identity;
- semantic slot ordering in final atom snapshots;
- isolation of failed closed activations.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must then repeat validation on Linux, macOS and Windows before the
`k1.0` tag can be created.

## Key specifications

- `docs/ATOMIC_RUNTIME_SPEC_1_0.md`
- `docs/NAIR_SPEC_0_3.md`
- `docs/NAIR_NATIVE_INPUT_SPEC_0_1.md`
- `docs/INPUT_CORE_SPEC_0_1.md`
- `docs/NAIR_NATIVE_RENDER_SPEC_0_1.md`
- `docs/RENDER_CORE_SPEC_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
