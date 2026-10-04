# NORDOI Types & Effects Intelligence Record 0.4

Status: research record for **L0.4 candidate**  
Date: 2026-10-04

## 1. Question

What is the smallest type/effect foundation NORDOI can introduce now without importing historical
syntax debt, confusing effects with authority, or prematurely committing to handlers, function
syntax, runtime representation or a specific implementation strategy?

The L0.4 answer is intentionally narrower than a full type/effect system:

```text
opaque nominal type identities
named effect identities
bounded canonical effect sets
validate-before-publication
no authority in source semantics
```

## 2. Koka

Official reference: <https://koka-lang.github.io/koka/doc/kokaspec.html>

Koka is a useful reference because effects participate directly in function types and effect
polymorphism can express that higher-order functions inherit effects from their arguments. Its
specification shows the value of making effects statically visible rather than hidden runtime facts.

Strengths relevant to NORDOI:

- effect information is type-level information;
- an absence of effects can be represented explicitly;
- effect polymorphism avoids hard-coding every higher-order composition;
- handlers are a separate mechanism from merely naming an effect.

Risks if copied too early:

- NORDOI has not yet defined functions or function types;
- adopting row syntax now would freeze surface and inference behavior before vertical slices exist;
- handlers introduce control-flow and runtime consequences that L0.4 does not need.

L0.4 conclusion: establish effect identity and canonical bounded effect sets first; defer function
effect rows/inference/handlers.

## 3. Unison abilities

Official reference: <https://www.unison-lang.org/docs/language-reference/abilities-and-ability-handlers/>

Unison exposes ability requirements in function types and makes the empty ability set meaningful.
Its documentation emphasizes that a function cannot secretly use an ability absent from its type
requirements.

Strengths relevant to NORDOI:

- effect requirements are statically inspectable;
- empty requirements can mean no abilities are permitted;
- ability handling is distinct from ordinary function application;
- effect vocabulary can be abstracted independently from the implementation performing it.

L0.4 conclusion: `SemanticEffectSet::empty()` is explicitly pure/no-required-effects, while actual
source attachment to functions is deferred until NORDOI has a function law.

## 4. OCaml 5 effect handlers

Official reference: <https://ocaml.org/manual/5.4/effects.html>

OCaml demonstrates the expressive runtime/control-flow side of effect handlers: handlers can model
resumable control, fibers, generators and user-level concurrency. The mechanism has evolved across
OCaml 5 releases, including syntax evolution.

Strengths relevant to NORDOI:

- effect operations and handlers can be powerful modular control abstractions;
- handler implementation has concrete stack/runtime consequences;
- effect identity, effect performance and effect handling are separable concerns.

Risk for NORDOI now:

- choosing resumable handler semantics in L0.4 would prematurely bind language law to continuation
  behavior, stack strategy and control-flow semantics.

L0.4 conclusion: do not introduce `perform`, handlers or continuations yet.

## 5. Rust type/ownership lessons

Official reference: <https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html>

Rust is not used here as an effect-system template. Its relevant lesson is architectural: a static
type discipline can make large classes of invalid memory behavior impossible without requiring a
tracing garbage collector.

Strength relevant to NORDOI:

- encode safety constraints in types/semantics where practical rather than relying only on runtime
  checks or programmer discipline.

Risk if copied literally:

- Rust's syntax, lifetime surface and compatibility constraints are Rust-specific historical choices;
- NORDOI should preserve the invariant-first goal without assuming Rust's surface is optimal for
  humans, AI or the Atomic Machine.

L0.4 conclusion: opaque nominal types are introduced as identity only. Ownership, borrowing,
lifetimes, layout and representation require their own evidence and law.

## 6. NORDOI constitutional constraints

The NORDOI Master Law already requires:

```text
Impossible States First
No Work Without Effect
safe by omission
undeclared effects should become impossible where practical
authority != intent
host authority stays outside program bytes
```

This rules out a design where `effect Network;` silently grants network permission.

The L0.4 effect declaration is only semantic vocabulary. Future executable effect use must still
satisfy a later static effect-use rule and an independent host capability/policy boundary.

## 7. Chosen L0.4 design

Surface:

```noi
type UserId;
effect Network;
```

Properties:

- contextual only in a leading top-level prelude;
- lexical keywords are not globally frozen;
- types are nominal and opaque;
- effects are nominal and non-authorizing;
- same-kind duplicates are invalid;
- type/effect namespaces are distinct at L0.4;
- declaration count is bounded;
- semantic effect sets are bounded, sorted and duplicate-free;
- source order, comments, whitespace, file names, IDs and spans are excluded from canonical semantic
  identity;
- residual body remains `UNLOWERED`.

## 8. Rejected L0.4 alternatives

### Full algebraic effects now

Rejected because it would require operations, handlers, continuation semantics, function types and
runtime consequences before NORDOI has certified expression/function foundations.

### Effect declarations that imply capabilities

Rejected constitutionally. Declared intent is not host authority.

### Structural type definitions now

Rejected because records, variants, layout, generic parameters and representation need separate
research and vertical slices.

### Global lexical `type` / `effect` keywords

Rejected for now. Contextual recognition preserves surface-language flexibility.

### Declaration order as semantic identity

Rejected for L0.4 opaque declarations because they have no dependencies or initialization behavior.
Canonical L0.4 identity sorts by kind/name.

## 9. Future evidence required

Before attaching effects to executable functions or operations, later milestones should study:

- function/value semantics;
- type inference vs explicit type contracts;
- effect inference and effect polymorphism;
- effect subtyping/rows/sets;
- capability binding and least-authority policy;
- error effects vs result types;
- state/transaction effects;
- async/concurrency effects;
- handlers vs direct runtime lowering;
- optimization consequences of explicit purity;
- AI-generation ambiguity and canonical formatting.

L0.4 deliberately leaves these questions open.
