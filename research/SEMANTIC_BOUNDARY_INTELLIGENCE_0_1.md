# C0.1 Semantic Boundary Intelligence 0.1

## Decision

NORDOI needs a compiler-owned semantic layer before it can safely connect `.noi` surface syntax to
certified NAIR semantics. Direct AST → NAIR lowering would couple an unstable language surface to a
stable execution contract and would make future syntax/type/effect evolution unnecessarily risky.

C0.1 therefore inserts two explicit stages:

```text
HIR  = source-aligned compiler representation
NSIR = validated semantic-boundary representation
```

Neither stage is executable in C0.1.

## Why the body is `Unlowered`

At L0.3, NORDOI knows source text, tokens, balanced groups and an optional contextual module header.
It does not yet know what a declaration, expression, operator, type or effect means. Treating the
remaining AST as semantic content would manufacture language law from punctuation.

`Unlowered` is therefore information, not a missing implementation detail: it is an explicit proof
that C0.1 has not assigned semantics that have not been designed and certified.

## Source identity versus semantic identity

Compiler diagnostics need exact spans. Reproducible semantic identity must not depend on where the
file lives, what SourceId a compiler session assigns, or how comments are spaced.

C0.1 stores spans in `NsirOrigin` but excludes them from canonical semantic module identity bytes.
This establishes a reusable pattern for later compiler stages:

```text
diagnostic provenance ≠ semantic meaning
```

## Why canonical bytes are domain-separated

Even before hashing is required, domain separation prevents one byte encoding from being casually
reused as another semantic concept. `NORDOI-C0.1-MODULE-ID\0` makes the witness self-scoped.

C0.1 does not call the encoding a globally unique module identifier. Packages, dependency graphs and
multi-unit resolution do not exist yet.

## Why compiler limits are repeated

L0.3 already bounds names and module segments. C0.1 validates its own owned semantic objects because
later frontends may evolve. A semantic boundary should not depend on an upstream caller forever
remaining perfectly constrained.

## Why no NAIR dependency

The semantic boundary is useful only if it can evolve independently from the certified execution
format. C0.1 therefore has no NAIR lowering and no runtime dependency. Later lowering must target
published semantic contracts, not private Rust layouts.

## Future pressure points

Likely next compiler work includes:

- item/declaration HIR;
- namespace/symbol identities;
- multi-unit assembly and duplicate-module detection;
- name resolution;
- source-level type/effect representation;
- typed semantic validation;
- canonical NSIR program identity;
- only then an explicit NSIR → NAIR lowering boundary.

Each should be introduced only when a vertical source-to-semantic slice justifies it.
