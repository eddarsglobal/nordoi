# K1.18 Research — Kernel Consolidation Intelligence 0.1

The K1.0→K1.17 sequence accumulated a strong semantic runtime. The next risk is architectural drift:
a compiler/frontend may accidentally depend on implementation details, or a future kernel patch may
silently change a law that earlier milestones treated as certified.

K1.18 therefore introduces a semantic stability map before the language frontend is born.

## Research conclusions

- Semantic stability and Rust API stability are not the same thing.
- Host authority and program meaning must stay distinct even when both APIs are stable.
- Durable transport/checkpoint formats may be stable while remaining outside replay semantics.
- Experimental `.noi` syntax should be allowed to evolve against stable lower laws.
- Compiler/HIR work should target the stability manifest rather than internal module layout.
- A canonical manifest hash gives CI/tooling a cheap way to detect unreviewed boundary changes.

## Recommended next development track

After K1.18 certification, development should begin in parallel:

```text
L0.1  source text + spans + lexer
C0.1  frontend semantic boundary / HIR skeleton
T0.1  nordoi CLI shell
```

Kernel milestones should continue only for blockers discovered by those vertical-slice efforts.
