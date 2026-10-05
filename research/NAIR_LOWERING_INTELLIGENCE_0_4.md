# NORDOI C0.4 — NAIR Lowering Intelligence Record

## Decision

The first source-to-NAIR bridge must be smaller than the first executable language feature.
C0.3 already proved that `EMPTY` and pure `ENTRY(name)` mean zero semantic work. C0.4 therefore
lowers only that proven semantic floor.

## Why `[Halt]` instead of an empty instruction vector?

Existing NAIR 0.6 validation requires a terminal `Halt`. An empty `NairProgram` is invalid because
it has no halt. The minimal valid zero-work NAIR program is therefore exactly `[Instruction::Halt]`.

No NAIR change is necessary and no new opcode is justified.

## Why entry names are not embedded in NAIR

At this stage an entry name does not alter runtime behavior. Serializing it into NAIR would add
bytes with no operational effect and would violate the NORDOI principle that unused information
should cost nothing.

The compiler still needs a verifiable provenance relationship, so C0.4 introduces a separate
lowering witness binding the C0.3 plan identity to the canonical NAIR bytes.

This deliberately separates:

```text
semantic provenance  → C0.4 witness
runtime behavior      → NAIR bytes
host authority        → external host policy
```

## Security consequence

No source declaration can smuggle authority into the lowered NAIR program. `effect Network;` still
changes only semantic registry identity; because the current entry requires zero effects, C0.4
emits only `Halt`.

## Future direction

The next executable source feature must first acquire explicit source semantics and effect typing.
Only then should a later compiler milestone add corresponding NAIR instructions. C0.4 must not be
expanded into speculative lowering for syntax that the language has not yet defined.
