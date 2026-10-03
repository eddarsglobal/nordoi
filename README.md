# NORDOI K0.8 — Atomic Input & Interaction Core 0.1

K0.8 closes the next part of NORDOI's interactive execution loop by introducing a
single backend-independent input model for keyboard, mouse, touch, pen, gamepad and
XR interaction.

```text
OS / device / XR adapter
          ↓
validated normalized event
          ↓
 Atomic Input Core 0.1
          ↓
deterministic InputBatch
          ↓
   InputAtomBridge
          ↓
 one atomic NAM transaction
          ↓
 canonical NAM frontier
          ↓
NAIR / Atomic Render Core
```

## What K0.8 adds

- `AtomicInputCore` with deterministic monotonic event sequencing.
- Unified `InputSource` classes for Keyboard, Mouse, Touch, Pen, Gamepad,
  XR Controller and XR Hand.
- Immutable routing through `InputTarget::Global` or
  `InputTarget::RenderNode(RenderNodeId)`.
- Focus routing for keyboard/gamepad without retroactively retargeting queued events.
- Normalized and validated pointer, axis, scroll and XR pose data.
- Unit-quaternion normalization for XR pose orientation.
- Safe coalescing of replaceable state samples only.
- Lossless key/button transitions and scroll boundaries.
- `InputAtomBridge` from selected input signals into NAM atoms.
- One atomic transaction per input batch.
- Ownership validation before an input binding may mutate state.
- Zero work for unmatched bindings or identical resulting state.
- Zero external Rust dependencies remain.

## Safe coalescing law

```text
pointer move → pointer move → pointer move
               same route/control
                       ↓
                latest position + accumulated delta
                / latest axis/pose state

key down → key up
button down → button up
scroll A → scroll B
                       ↓
                 never discarded
```

Optimization may collapse replaceable state. It may not erase an observable
transition.

## Atomic input-to-state law

```text
InputBatch
   ↓
match selectors
   ↓
stage all resulting atom writes
   ↓
one AtomicTransaction
   ↓
all-or-nothing NAM commit
```

Multiple events that map to the same atom collapse inside the transaction to the
last deterministic **state snapshot**. This bridge is intentionally a state projection,
not an event-handler system: the complete ordered transitions remain available in
`InputBatch` for future canonical interaction/action semantics. NAM then preserves the
existing `No Work Without Effect` law.

## Security boundary

K0.8 does **not** poll raw hardware or request operating-system permissions. The core
receives semantic events from an already-authorized host adapter. Global key capture,
raw device access, IME/text composition, haptics and other privileged acquisition
mechanisms remain unrepresentable until their effect/capability contracts are
specified.

## Tests

K0.8 adds **18 Atomic Input tests** on top of the 62 inherited tests, for a total of
**80 tests**.

The mandatory release gate remains:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI then repeats validation on Linux, macOS and Windows.

## Specifications

- `docs/INPUT_CORE_SPEC_0_1.md`
- `docs/NAIR_SPEC_0_1.md`
- `docs/NAIR_SPEC_0_2.md`
- `docs/RENDER_CORE_SPEC_0_1.md`
- `docs/NAIR_RENDER_BRIDGE_SPEC_0_1.md`
- `docs/NAIR_NATIVE_RENDER_SPEC_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
