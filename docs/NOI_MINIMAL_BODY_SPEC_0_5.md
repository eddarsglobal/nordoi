# NORDOI NOI Minimal Body Specification 0.5

Status: **L0.5 candidate**  
Scope: first fully understood source-body form above certified L0.4 and C0.2.  
Kernel / NAIR impact: **none**.

## 1. Purpose

L0.5 ends the phase in which every post-prelude source body is necessarily opaque. It introduces one
small, closed semantic surface that can be fully validated without freezing a general function,
expression, statement, call, control-flow, ABI or runtime model.

The accepted residual body is exactly one of:

```text
<only trivia / empty>
```

or:

```noi
entry Name;
```

`entry` is contextual. At the lexical layer it remains an ordinary `Identifier` token.

## 2. Pipeline

```text
SourceText
  -> L0.1 lexer
  -> L0.2 structural AST
  -> L0.3 module analysis
  -> L0.4 type/effect prelude
  -> L0.5 minimal body analysis
  -> C0.1/L0.4/C0.2 semantic validation and registry
  -> L0.5 validated body wrapper
```

L0.5 does not lower to NAIR and does not execute runtime work.

## 3. Grammar

Ignoring trivia between significant elements:

```text
minimal-body := empty | entry-declaration
entry-declaration := "entry" identifier ";"
```

Only one entry declaration is permitted. Any additional significant element is an error.

## 4. Entry semantics

`entry Name;` declares one named program entry identity with all of the following L0.5 properties:

- zero parameters;
- no return-type syntax;
- no executable statement list;
- no calls;
- no values or literals;
- no control flow;
- no effect operations;
- an explicitly empty required-effect set;
- zero host authority;
- zero runtime work.

This is a semantic declaration, not yet a function ABI.

## 5. Effect and authority law

An L0.5 entry is pure by construction:

```text
required_effects(entry) = {}
```

Declared L0.4 effects remain available in the C0.2 semantic registry but are not implicitly attached
to the entry.

```text
effect Network;
entry main;
```

does **not** mean `main` can use Network and does **not** grant network authority.

The existing constitutional separation remains:

```text
declared effect != resolved effect != host authority
```

## 6. Fail-closed body publication

The L0.5 boundary publishes a body only when it understands the entire significant residual body.
These examples fail:

```noi
future_body
entry main
entry 42;
entry (main);
entry main; other
entry main; entry other;
```

The older C0.2 boundary remains available and continues to describe such residual source as
`UNLOWERED`; L0.5 does not rewrite that certified contract.

## 7. Source identity and diagnostics

`SurfaceEntry`, `HirEntryPoint` and `NsirEntryPoint` retain exact source spans for diagnostics.
Source IDs, spans, comments and whitespace do not participate in the L0.5 canonical body witness.

## 8. Canonical witness

L0.5 adds:

```text
NsirBodyUnit::canonical_l05_bytes()
```

The witness is domain separated by:

```text
NORDOI-L0.5-BODY\0
```

and binds:

1. the existing C0.2 canonical semantic witness;
2. body variant (`EMPTY` or `ENTRY`);
3. entry name when present;
4. the resolved effect-ID list, which is empty in L0.5.

C0.1, L0.4 and C0.2 witness methods remain unchanged.

## 9. Public surface

Frontend additions:

```text
BodyError
BodyResult
SurfaceEntry
MinimalBodyForm
MinimalBodyUnit
MinimalBodyAnalyzer
analyze_minimal_body_unit(...)
```

Compiler additions:

```text
HirEntryPoint
HirMinimalBody
HirBodyUnit
NsirEntryPoint
NsirMinimalBody
NsirBodyUnit
lower_minimal_body_unit_to_hir(...)
validate_body_hir(...)
compile_minimal_body_boundary(...)
```

Tooling extension:

```text
nordoi body <path|->
```

The certified `nordoi semantic` C0.2 inspection path remains unchanged.

## 10. Explicit non-goals

L0.5 does not define:

- general `fn` / function syntax;
- parameters or return values;
- function types;
- statements;
- expressions;
- literals;
- calls;
- operators or precedence;
- local variables;
- branching or loops;
- handlers;
- effect invocation syntax;
- imports;
- visibility;
- ABI or calling convention;
- NAIR instructions;
- NSIR -> NAIR lowering;
- runtime execution.

## 11. Security properties

L0.5 is bounded by existing source/name limits and introduces no unbounded nested parser. It fails
closed on unsupported body syntax, inherits all lexical/structural security failures, performs no host
I/O from program semantics and serializes no authority into source or semantic identity.

## 12. Release law

L0.5 is not certified until all of the following are proven for the exact commit:

1. `cargo fmt --all -- --check`;
2. `cargo clippy --all-targets -- -D warnings`;
3. `cargo check --all-targets`;
4. `cargo test --all-targets`;
5. GitHub CI green on all required jobs for the exact SHA;
6. annotated tag `l0.5` pushed.

## 13. Next boundary

After L0.5, the next compiler milestone should lower only the fully understood empty/entry body into a
minimal compiler-owned executable semantic plan. NAIR lowering should occur only after that plan has
its own validation law; no source syntax should bind directly to private runtime layout.
