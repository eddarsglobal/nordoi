# Dynamic Input & Runtime Computation Intelligence 0.7

V0.6 proved that closed pure computation should disappear before runtime whenever all operands are known. V0.7 asks the complementary question: what is the smallest legitimate reason for computation to remain alive at runtime?

The answer is canonical external data, not implicit host authority.

The V0.7 design therefore introduces a single SSA-producing input read over the already-certified `InputBatch`. It deliberately does not invent a second event system, VM, call stack, callback surface, environment reader, or device API.

The important compiler/runtime distinction is now explicit: a register can have a known semantic type while not having a compile-time value. NAIR validation tracks both facts separately. Static integer operands can still be pre-evaluated and checked for overflow; dynamic integer operands are type-validated ahead of execution and overflow-checked when executed.

This creates the first proof-preserving mixed program shape in NORDOI:

```text
static region -> erased/folded
runtime input -> retained
runtime arithmetic/comparison -> retained only where data-dependent
```

That boundary prepares the language for V0.8 dynamic control flow without prematurely introducing branching semantics in V0.7.
