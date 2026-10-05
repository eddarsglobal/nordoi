# NORDOI C0.5 — Pure Result Execution Plan Intelligence Record

## Decision

The first source-level result must enter execution planning as a compiler semantic value before NORDOI
attempts to represent that value in NAIR or expose it from the runtime.

C0.5 therefore introduces a new additive plan rather than widening the certified C0.3 plan in place.
This keeps old witnesses and old acceptance boundaries stable.

## Why the result is not NAIR yet

NAIR 0.6 can represent the already-certified zero-work program as `[Halt]`, but it has no certified
contract for carrying a source-language return value. Treating `returns 42` as the same raw `[Halt]`
without a result channel would erase the program's observable semantic result. Inventing an opcode in
C0.5 would collapse planning and execution-IR design into one milestone.

The safer sequence is:

```text
L0.6 source result
  -> C0.5 semantic result plan
  -> future explicit result-plan -> NAIR design
  -> future runtime result observation
```

## Zero-work meaning

`returns 42` is currently a literal already known at compile time. C0.5 records that value but creates
no operational work item. This respects the architecture rule that unused mechanisms should cost
nothing and prevents a semantic value from being confused with an external effect.

## Authority separation

The result plan carries no host authority. Declaring `effect Network;` beside a pure result changes the
module registry witness but does not authorize networking or add a required effect to the entry.

## Compatibility strategy

C0.3, C0.4 and V0.1 are deliberately left untouched. Their source boundaries continue to reject the
new result-bearing body. C0.5 has a dedicated API and CLI command so acceptance cannot expand by
accident.

## Future hinge

The next compiler milestone should decide the smallest safe operational representation for a pure
result. It should not broaden the source grammar at the same time. The likely design question is how
to carry an `i64` result through NAIR while preserving canonical bytes, deterministic execution,
zero authority and the existing HALT semantics.
