# Semantic Registry Intelligence — C0.2

## Why typed IDs now

L0.4 established names but no compiler-owned references. Future body semantics should not depend on
string comparison everywhere. C0.2 therefore introduces deterministic typed IDs before expression or
function syntax is frozen.

## Why IDs are sorted by name

Source-order IDs would make harmless declaration reordering change semantic references. Canonical
name ordering makes symbol identity stable across formatting and source reordering while retaining
origin spans separately for diagnostics.

## Why type/effect IDs are different Rust types

A numeric ID alone permits accidental namespace confusion. `SemanticTypeId` and `SemanticEffectId`
make cross-namespace misuse harder by construction.

## Why IDs start at one

Zero remains an impossible published symbol identity. This leaves an obvious sentinel outside the
valid semantic domain without exposing sentinel-based logic in normal APIs.

## Why keep source spans

Diagnostics still need exact source locations. C0.2 stores origin spans in symbols but excludes them
from canonical identity, separating developer ergonomics from reproducible semantics.

## Why registry resolution is not authority

An effect name answers “what semantic effect does this code claim to require?” Host authority answers
“what may this execution actually do?”. Combining those questions would violate NORDOI's authority ≠
intent law. C0.2 resolves only the first.

## Why no NAIR lowering

NAIR 0.6 is a certified execution contract. The experimental source language is not yet rich enough
to lower useful bodies without prematurely freezing calls, values, control flow or effect operations.
A canonical semantic registry is the safer prerequisite.

## Compatibility strategy

C0.1 and L0.4 canonical witnesses remain byte-for-byte defined by their previous domains. C0.2 adds a
new domain-separated witness rather than mutating old meaning.

## Rejected alternatives

- Source-order symbol IDs: rejected because reordering would perturb identity.
- One shared type/effect namespace: rejected because it creates needless collisions.
- Hash-only symbol IDs: rejected for now because compact deterministic indexes are simpler and bounded.
- Inferring authority from declared effects: constitutionally rejected.
- Lowering opaque bodies to NAIR placeholders: rejected because it would claim semantics not defined.
