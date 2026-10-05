# V0.3 Intelligence Record — First Executed Pure Expression

## Decision

Execute C0.8 pure-expression NAIR with the already-certified closed runtime observation path and validate the complete final SSA register map against the C0.7 postfix plan.

## Why validate every register?

Checking only the final register could prove the final number but would not strongly bind runtime evidence to the certified calculation structure. C0.8 intentionally preserved postfix structure rather than constant-folding it. V0.3 therefore validates every node/register pair so execution evidence preserves that decision.

For `20 + 22`:

```text
r0 = 20
r1 = 22
r2 = checked_add(r0, r1) = 42
```

For `1 + (2 + 3)`:

```text
r0 = 1
r1 = 2
r2 = 3
r3 = checked_add(r1, r2) = 5
r4 = checked_add(r0, r3) = 6
```

This also makes different parenthesizations operationally distinguishable even when their final value is equal.

## No new runtime mechanism

V0.2 already introduced transient register observation without persistence or authority. V0.3 reuses that same mechanism. It does not add another interpreter, copy results into atoms, alter checkpoints, or create a host output channel.

## Compatibility

Minimal-required-minor encoding remains a compiler/NAIR property. V0.3 accepts the C0.8 artifact exactly as produced: 0.6 for no arithmetic, 0.7 when `ADD_INT_CHECKED` is used.

## Safety posture

- checked arithmetic only,
- validate before publishing execution evidence,
- canonical empty input,
- zero effects and authority,
- zero persistent state,
- fail closed on any forged/missing/extra register,
- receipt includes exact C0.8 witness and observed register map.

## Deferred

V0.3 does not add variables, subtraction, multiplication, division, calls, control flow, effects, I/O, or optimization. Those require separate semantic and IR laws.
