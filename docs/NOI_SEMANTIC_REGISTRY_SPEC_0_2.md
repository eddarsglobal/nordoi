# NORDOI C0.2 — Semantic Registry & Resolved IDs Specification

Status: candidate until local Release Gate, exact-commit GitHub CI and annotated `c0.2` tag are green.

## 1. Purpose

C0.2 gives L0.4 opaque type/effect declarations compiler-owned typed identities without defining
runtime representation or executable semantics.

## 2. Input boundary

C0.2 consumes only a fully analyzed and validated L0.4 semantic unit. Frontend or HIR validation
failure prevents NSIR and registry publication.

## 3. Separate namespaces

Type declarations produce `SemanticTypeId`. Effect declarations produce `SemanticEffectId`.
The namespaces are independent. Equal spelling across namespaces is valid.

## 4. Canonical ID assignment

Within each namespace, declarations are sorted by canonical ASCII semantic name. IDs are assigned
starting at 1 in that order. Source order, trivia, comments, source IDs and diagnostic spans cannot
change an ID.

Zero is never published as a semantic symbol ID.

## 5. Registry

`SemanticRegistry` contains sorted `NsirTypeSymbol` and `NsirEffectSymbol` tables. Every symbol keeps
its source origin span for diagnostics, but spans do not enter canonical registry bytes.

Resolution is exact and case-sensitive under the current ASCII identifier profile.

## 6. Effect requirement resolution

`resolve_effect_set(registry, requirements)` converts a bounded `SemanticEffectSet` of declared names
into a `ResolvedEffectSet` of `SemanticEffectId` values. Missing declarations fail closed with
`UnknownEffectRequirement`.

A resolved effect is semantic intent only. It grants no capability, host authority, I/O access,
backend access or runtime work.

## 7. Compatibility witnesses

C0.1 `canonical_identity_bytes()` remains unchanged. L0.4 `canonical_semantic_bytes()` remains
unchanged. C0.2 adds `canonical_c02_bytes()` with its own domain separator.

## 8. Body state

The residual body remains exactly `UNLOWERED`. C0.2 does not define functions, declarations beyond
the L0.4 prelude, expressions, calls, handlers, control flow, type layouts, ABI, generics or imports.

## 9. NAIR/runtime boundary

C0.2 performs no NSIR → NAIR lowering. NAIR remains 0.6 and the certified K1.18 runtime/checkpoint
surfaces are unchanged.

## 10. Bounds

C0.2 inherits the certified candidate bounds from L0.4: at most 1024 semantic declarations, 128-byte
semantic names, and at most 256 semantic effect requirements.

## 11. Publication law

No `SemanticRegistry` is published from malformed frontend input, invalid HIR, duplicate same-kind
declarations, or out-of-bounds declarations. Validation precedes registry construction.

## 12. Next boundary

The next language/compiler milestone may attach resolved type/effect IDs to actual body constructs.
That future work must not reinterpret an effect ID as host authority.
