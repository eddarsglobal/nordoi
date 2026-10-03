# NORDOI Constitutional Principles — K0.2

NORDOI is designed as a universal programming architecture for humans, AI/SI and machines.

## C1 — Universal Execution
NORDOI must never be confined to the Web. The Web is one target among others.

## C2 — Atomic Speed
The system should perform only the minimum necessary work for each observable effect.

## C3 — Atomic Security
Authority is denied by default and granted explicitly through narrow capabilities.

## C4 — Zero Legacy Debt
NORDOI must not preserve a weakness merely because older languages historically contain it.

## C5 — Impossible States First
When a dangerous program state can reasonably be made unrepresentable, NORDOI should make it unrepresentable rather than merely document it.

## C6 — No Work Without Effect
A computation with no useful state change or observable effect should create no downstream work.

## C7 — What You Do Not Use Must Cost Nothing
Unused subsystems should contribute no meaningful runtime cost.

## C8 — AI/SI Native, Not AI Dependent
AI/SI may generate, analyze, optimize and verify NORDOI programs, but ordinary compiled execution must not require an AI model.

## C9 — Simple Surface, Deep Engine
Complexity that can safely be absorbed by compiler/runtime/tooling should not be imposed on the developer.

## C10 — Canonical Semantics
Human source, AI generation, visual tooling and future frontends should converge on the same semantic model and NAIR.

## C11 — Declared Effects
A unit of code may not perform an effect it did not declare.

## C12 — Authority Is Not Intent
Possessing a capability does not authorize undeclared behavior. Both declaration and authority are required.

## C13 — Least Authority
Capabilities must be narrow, explicit and scoped as precisely as practical.

## C14 — Evidence Before Evolution
Changes to core semantics require tests, counterexamples, performance evidence and security analysis.

## C15 — Community Evolvable, Constitutionally Governed
The community may propose and build freely; core invariants may only evolve through evidence-backed governance.

## C16 — Master Law Supremacy
`laws/LAW_0001_NORDOI_MASTER_LAW.md` is a foundational design law. Kernel, NAIR, NAM, compiler, runtime, tooling and future language proposals must be evaluated against it.

## C17 — Verified Performance Supremacy
The founder's declaration — **"NORDOI est le plus leger langage au monde et le plus vite dans l'univer"** — is adopted as a permanent engineering mission. Claims of superiority must be demonstrated with reproducible evidence; correctness and security may not be sacrificed for benchmark results.

## C18 — Atomic Transactions
A multi-state operation must not expose partially committed state. Transaction failure before commit must leave unrelated staged state unchanged.

## C19 — Explicit Ownership Boundaries
Mutable state belongs to an explicit ownership domain. Mutation from another domain is forbidden unless ownership or authority is explicitly transferred/delegated by a future verified mechanism.

## C20 — Testing & Release Law
No NORDOI version is complete until its automated regression and invariant tests pass in the repository's GitHub CI test gate. A subsequent version must not be treated as started before the preceding version is green.

## C21 — IR Before Surface Lock-In
NORDOI must stabilize semantic invariants and NAIR contracts before freezing convenient surface syntax. Syntax serves semantics; semantics do not serve syntax.

## C22 — Validate Before Execute
Invalid NAIR must be rejected before it can mutate NAM state.

## C23 — Safe By Omission
A privileged operation that has not yet received a complete effect, authority, validation and execution contract must not be representable in canonical NAIR.

## C24 — Canonical Machine Meaning
The same valid NAIR program and initial NAM state must have one defined machine meaning. Canonical encoding must not depend on incidental map order, platform syntax or frontend origin.
