# NORDOI K0.9 — NAIR 0.3 Native Interaction Semantics

NORDOI K0.9 makes K0.8's Atomic Input Core directly expressible in canonical
NAIR while keeping raw device acquisition outside the language IR.

## Architecture

```text
Keyboard / Mouse / Touch / Pen / Gamepad / XR
                    ↓
          authorized host adapter
                    ↓
           Atomic Input Core
                    ↓
                InputBatch
                    ↓
              NAIR 0.3
  CREATE_INPUT_BRIDGE / BIND_INPUT_ATOM
                    ↓
              APPLY_INPUT
                    ↓
          one AtomicTransaction
                    ↓
                    NAM
                    ↓
           single causal frontier
                    ↓
          Atomic Render Core
```

## What K0.9 adds

- NAIR format 0.3 with backward decoding of valid 0.1 and 0.2 programs.
- `InputBridgeSlot` as a canonical single-assignment semantic identity.
- `InputTargetRef` using NAIR render slots instead of serialized runtime IDs.
- `CREATE_INPUT_BRIDGE` (`0x40`).
- `BIND_INPUT_ATOM` (`0x41`).
- `APPLY_INPUT` (`0x42`).
- Explicit source, device, target and signal filters.
- Explicit state-only, render-only, input-only and combined execution paths.
- Missing render/input context rejection before execution.
- Native input-to-NAM atomic application.
- Native `InputBatch → NAM → RenderBatch` integration without a second dependency engine.
- Zero new external Rust dependencies.

## Four explicit execution surfaces

```text
execute_nair(...)
execute_nair_with_render(...)
execute_nair_with_input(...)
execute_nair_with_render_and_input(...)
```

Programs cannot silently obtain an input or render context they were not given.

## Input remains external data, not executable code

NAIR does not serialize live hardware events into the program. The host supplies a
normalized `InputBatch`; NAIR serializes only the deterministic policy that maps
semantic input to owned NAM atoms.

This preserves the authorized adapter boundary and keeps global capture, raw
hardware access and OS permissions safe by omission.

## Tests

K0.9 adds **16 native interaction tests** on top of the 80 inherited tests, for a
total of **96 tests**.

They cover:

- NAIR 0.3 / 0.2 compatibility,
- rejection of 0.3 input opcodes under a 0.2 header,
- bridge-slot single assignment and use-before-definition,
- atom/render-target validation,
- canonical byte-stable interaction encoding,
- missing input-context rejection,
- source/device filtering,
- zero work for unmatched input,
- final-snapshot transaction collapse,
- ownership enforcement,
- full `InputBatch → NAM → render` propagation.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must then repeat validation on Linux, macOS and Windows before K0.9 can
be tagged as certified.

## Specifications

- `docs/NAIR_SPEC_0_3.md`
- `docs/NAIR_NATIVE_INPUT_SPEC_0_1.md`
- `docs/INPUT_CORE_SPEC_0_1.md`
- `docs/NAIR_SPEC_0_2.md`
- `docs/NAIR_NATIVE_RENDER_SPEC_0_1.md`
- `docs/RENDER_CORE_SPEC_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
