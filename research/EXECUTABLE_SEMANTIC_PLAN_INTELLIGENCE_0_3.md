# C0.3 Executable Semantic Plan — Intelligence Record

C0.3 exists to create a semantic seam between "the source body is fully understood" and "the
compiler is allowed to lower anything into executable machine/runtime IR".

The key design choice is that the first plan has **zero operations**. This may look intentionally
small, but it proves several architectural properties without inventing language semantics:

- a program can nominate an entry point without implying function syntax;
- a compiler plan can be canonical without depending on Rust object layout;
- a plan can be validated before publication;
- semantic intent remains separate from host authority;
- source formatting and diagnostic spans remain outside semantic identity;
- NAIR can stay completely unchanged until an explicit lowering law exists.

The C0.3 plan therefore carries only one semantic distinction: `EMPTY` versus `ENTRY(name)`. Both
are pure and zero-work.

## Why not lower to NAIR yet?

A zero-work entry can technically be represented by an empty NAIR program, but doing so in C0.3
would prematurely equate source-level entry semantics with a particular NAIR representation. NORDOI
instead certifies the compiler-owned plan first. A future lowering milestone can then state and test
the exact mapping explicitly.

## Security consequence

An execution plan is not permission. C0.3 never reads host capability state and never serializes
host authority. An impure manually constructed entry is rejected by plan validation even though the
normal L0.5 path cannot currently create one.

## Compatibility consequence

C0.3 adds a new witness rather than modifying C0.1/L0.4/C0.2/L0.5 witnesses. This preserves the
ability to compare semantic layers independently and avoids retroactive reinterpretation of
certified identities.
