# C0.11 Intelligence Record — Pure Condition Execution Plan

## Question

What is the smallest plan layer that prepares NORDOI for control flow without prematurely
freezing branch syntax, NAIR opcodes or runtime behavior?

## Decision

Preserve the exact L0.9 semantic condition and its deterministic truth value in a zero-work,
zero-storage plan.

The plan does not replace a condition with only `true` or `false`. Doing so would erase semantic
history needed for canonical auditability and future lowering choices.

The plan also does not invent a branch instruction. Branching is a later contract because branch
representation affects NAIR validation, canonical machine meaning, optimization and execution.

## Constitutional fit

- **Atomic Speed / No Work Without Effect:** planning creates no runtime work.
- **What You Do Not Use Must Cost Nothing:** no branch/runtime subsystem is activated merely because
  a condition exists semantically.
- **Canonical Semantics:** distinct condition forms remain distinct even when truth is equal.
- **IR Before Surface Lock-In:** the boolean planning contract is stabilized before `if` syntax or
  branch opcodes are frozen.
- **Validate Before Execute:** no execution is introduced in this milestone.
- **Safe By Omission:** branch behavior remains unrepresentable until a complete later contract exists.

## Rejected alternatives

### Lower comparisons directly to a branch now

Rejected. It would merge semantic planning, NAIR design and control-flow execution into one milestone.

### Constant-fold every condition to one boolean identity

Rejected. It would make `true` and `1 < 2` indistinguishable at the semantic-plan layer.
Operational equivalence may be exploited later without deleting semantic identity.

### Introduce runtime boolean storage

Rejected. C0.11 has no need for state or storage.

## Next frontier

After C0.11 certification, the next decision should be whether to introduce a minimal pure
conditional source form or first add NAIR boolean/comparison representation. That decision must be
made under `CONSTITUTION.md`, especially Atomic Speed, Canonical Machine Meaning and IR Before
Surface Lock-In.
