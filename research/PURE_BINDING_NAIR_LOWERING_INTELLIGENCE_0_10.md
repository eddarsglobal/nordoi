# C0.10 Research Note — Pure Binding Erasure into NAIR

`CONSTITUTION.md` remains the supreme NORDOI authority. This note records the design reasoning for C0.10; it does not create a competing constitutional rule.

## Question

What should an immutable compile-time binding cost at runtime?

For the L0.8 model, the answer is: no cost beyond the computation that would already exist if the programmer had written the bound literal directly.

## Rejected design: runtime binding load

A naive compiler could introduce something like:

```text
LOAD_BINDING r0, #1
```

That would require some combination of runtime name/ID storage, lookup machinery, initialization, memory traffic or new NAIR semantics. None of that is necessary for an immutable integer value already certified at compile time.

C0.10 rejects this design.

## Selected design: compile-time erasure

`BINDING(id)` remains meaningful through L0.8 and C0.9. At C0.10 lowering, the compiler resolves the ID against the certified canonical registry and emits the same `Const` as the equivalent literal.

Thus:

```noi
const x = 20;
entry main returns x + 22;
```

and:

```noi
entry main returns 20 + 22;
```

have the same operational NAIR structure.

The binding version still has a distinct C0.10 compiler witness because that witness contains the exact C0.9 plan.

## Why not fold the entire expression to one constant?

All current L0.8 bindings are compile-time constants, so `x + y` could theoretically become only `CONST 42; HALT`.

C0.10 deliberately does not add that optimization yet. Certified C0.8 established faithful postfix-to-SSA lowering, and C0.9 preserves expression structure. Replacing the whole expression with its result would introduce a broader optimization policy rather than merely proving that naming has zero additional runtime cost.

C0.10 therefore chooses the narrower invariant:

> Binding names cost nothing extra compared with the equivalent literal expression.

A future optimizer may prove stronger elimination under an explicit optimization law and benchmark gate without changing source semantics.

## Unused bindings

Unused bindings are stronger: because they do not participate in the postfix expression, they produce no instruction whatsoever. This directly tests the zero-unused-cost property.

## Compatibility

No NAIR format increment is needed. Literal-only/binding-only results retain 0.6. Arithmetic additions automatically require 0.7 through the existing N0.7 opcode.

No runtime code changes are needed.

## Security

The lowering does not grant authority, does not read ambient state and does not introduce runtime indirection. Binding ID resolution occurs against the already-validated C0.9-owned registry. Any impossible/mismatched internal binding ID fails closed rather than becoming ambient lookup behavior.

## Future

The natural next vertical milestone after C0.10 certification is execution of the produced program through the existing observed closed runtime, with validation of every final SSA register and proof that there is still zero binding-specific runtime state.
