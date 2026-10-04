# NORDOI K1.18 — Kernel Consolidation & Semantic Stability Map

K1.18 is the first deliberate **consolidation milestone** after the K1.0→K1.17 runtime build-out.
It adds no new runtime capability. Instead, it makes the certified semantic boundary explicit and
machine-readable so the upcoming language/compiler work can evolve without silently changing laws
that the kernel already guarantees.

```text
K1.0 → K1.17
certified semantic machinery
        ↓
K1.18
classify + hash + test the semantic boundary
        ↓
L0.1 / C0.1 / T0.1
experimental language + compiler + CLI above stable law
```

## Why K1.18 exists

NORDOI now has certified semantics for atomic state, ownership, capabilities, deterministic input,
logical time, reactions, rendering, governed effects, persistence/recovery and program/timer upgrade.
The next architectural risk is no longer missing runtime machinery. It is **boundary confusion**:

- treating a Rust API name as if it were permanent semantic law;
- accidentally turning host authority into program-serializable privilege;
- letting delivery/audit metadata leak into replay meaning;
- freezing `.noi` syntax before real source programs exist;
- allowing a future compiler to depend on private runtime implementation details.

K1.18 creates an explicit stability map to prevent those mistakes.

## Machine-readable stability manifest

The kernel exports:

```rust
kernel_semantic_stability_manifest()
kernel_semantic_stability_manifest_bytes()
kernel_semantic_stability_manifest_hash()
```

The canonical manifest domain is:

```text
NORDOI-KERNEL-SEMANTIC-STABILITY-MAP-1.0
```

K1.18 candidate manifest SHA-256:

```text
0cab1dcaf93b82d1974701058741b271dcb550374effa4d047c27adb92efdb29
```

This hash is a deterministic release/tooling identity. It is **not a signature** and does not replace
K1.11 effect-audit attestation.

## Three independent classification axes

Every surface is classified by:

```text
StabilityClass
    Stable
    Experimental
    Internal

SemanticKind
    ProgramSemantic
    HostAuthority
    OperationalMetadata
    DurableEncoding
    DeveloperSurface

ChangePolicy
    AdditiveOnly
    VersionedEvolution
    InternalOnly
```

A surface can therefore be a stable host-authority boundary without becoming program replay meaning,
or a stable durable encoding without becoming a semantic event.

## Stable lower laws

The stability map marks the already-certified semantics for these areas as `Stable`:

```text
atomic transaction / ownership / capability law
input semantics
logical time
reaction semantics
render semantics
effect intent + completion semantics
runtime closed execution + event loop
runtime recovery checkpoint
program upgrade
native timer upgrade
dynamic timer upgrade
NAIR 0.6
```

`Stable` means future milestones must preserve the law or evolve it through an explicit versioned
protocol. It does **not** mean every current Rust symbol name is frozen forever.

## Explicitly unfrozen surfaces

K1.18 intentionally classifies:

```text
host.rust_public_api    EXPERIMENTAL
language.noi_surface    EXPERIMENTAL
tooling.compiler_frontend INTERNAL
```

This is a feature, not a deficiency. It lets NORDOI begin L0/C0 work while the lower semantics remain
protected.

## Host authority stays outside program bytes

Host authority surfaces remain separately classified. The manifest asserts that program bytes do not
serialize ambient authority. This preserves the security model already established by effects,
completion bindings, fencing and governed program upgrade.

## Operational metadata stays outside replay meaning

Fencing, retry scheduling, audit and attestation protect delivery/trust history but remain
operational metadata rather than program replay meaning unless a future explicit semantic promotion
says otherwise.

## Binary compatibility preserved

K1.18 does not change:

```text
NAIR                     0.6
runtime checkpoint       NDRTSM01 / 1.1
effect journal families  inherited unchanged
```

There is no new opcode and no checkpoint field.

## Test gate

K1.18 adds tests that enforce:

- unique/sorted manifest IDs;
- stable surfaces have real specification files;
- stable surfaces never use internal-only evolution policy;
- host authority is never program-serialized;
- operational metadata is not replay semantic;
- stable program semantics are replay relevant;
- `.noi` and Rust public API remain explicitly unfrozen;
- NAIR remains 0.6;
- runtime checkpoint remains `NDRTSM01/1.1`;
- canonical manifest bytes have a golden SHA-256 identity.

## What comes after certification

After K1.18 is certified, NORDOI should begin parallel tracks instead of continuing runtime-only
feature accumulation:

```text
L0.1 — source text, spans and lexer
C0.1 — compiler semantic/HIR boundary
T0.1 — nordoi CLI shell
```

Kernel milestones should then be driven mainly by blockers discovered by those vertical-slice tracks.

## Release Gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

K1.18 is certified only after local gate success, GitHub CI success on all required platforms and an
annotated `k1.18` tag.
