# NORDOI Kernel Semantic Stability Map 1.0

**Milestone:** K1.18  
**Status:** Candidate until Release Gate + CI + annotated tag  
**NAIR:** 0.6 unchanged  
**Runtime checkpoint:** `NDRTSM01` 1.1 unchanged

K1.18 is a consolidation milestone. It does not add a new runtime capability. It defines a
machine-readable contract describing which already-certified NORDOI surfaces are stable semantic
law, which remain experimental developer-facing surfaces, which are host authority, and which are
operational or durable metadata rather than program replay meaning.

## Three independent axes

Every classified surface has:

1. `StabilityClass`
   - `Stable`
   - `Experimental`
   - `Internal`
2. `SemanticKind`
   - `ProgramSemantic`
   - `HostAuthority`
   - `OperationalMetadata`
   - `DurableEncoding`
   - `DeveloperSurface`
3. `ChangePolicy`
   - `AdditiveOnly`
   - `VersionedEvolution`
   - `InternalOnly`

These axes are intentionally independent. A durable binary format can be stable without becoming
program replay meaning. A host authority interface can be stable without being serializable by the
program. A developer-facing Rust API can remain experimental while the semantic law underneath it
is stable.

## Canonical manifest

The kernel exports:

```rust
kernel_semantic_stability_manifest()
kernel_semantic_stability_manifest_bytes()
kernel_semantic_stability_manifest_hash()
```

The canonical domain is:

```text
NORDOI-KERNEL-SEMANTIC-STABILITY-MAP-1.0
```

The hash is SHA-256 over the canonical manifest bytes and is intended for tooling, compiler gates,
release auditing and machine inspection. It is not a signature and does not replace K1.11
attestation.

## Stable semantic boundary

K1.18 classifies the certified semantics for transactions, ownership, capabilities, input, logical
time, reactions, rendering semantics, effect intent/completion semantics, runtime execution,
checkpoint recovery, program upgrade, native/dynamic timer upgrade and NAIR 0.6 as stable.

`Stable` means future work must preserve the law or introduce explicit versioned evolution. It does
not mean every Rust symbol name is permanently frozen.

## Host authority remains external

Host-side effect dispatch authority, effect backends and render backends are classified separately
from program semantics. Program bytes shall not serialize ambient authority.

## Operational metadata remains non-semantic

Effect fencing, retry scheduling, audit and attestation remain operational trust/delivery machinery.
They can protect execution without automatically changing program replay identity.

## Developer surfaces remain intentionally unfrozen

Two important surfaces are intentionally not stable in K1.18:

```text
host.rust_public_api      EXPERIMENTAL
language.noi_surface      EXPERIMENTAL
```

The compiler frontend is classified `INTERNAL` because L0/C0 have not yet been certified.

This distinction is the main reason K1.18 exists: the project may now begin L0.1/C0.1 without
confusing experimental syntax/API design with already-certified semantic law.

## Compatibility rule

Future milestones must not silently reclassify a `Stable` surface as experimental/internal or change
its semantic role. A breaking semantic change requires an explicit versioned protocol and new laws.

## What K1.18 does not do

K1.18 does not:

- increment NAIR;
- change `NDRTSM01`;
- add runtime work;
- freeze `.noi` syntax;
- freeze all Rust API names;
- introduce HIR or a parser;
- claim ABI stability for arbitrary host binaries.

Its job is to create the stable semantic floor on which those future layers can be built.
