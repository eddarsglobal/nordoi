# NORDOI P2.2 — Dynamic Capability-Secured Observable Output Specification

## Status

Candidate. This specification is additive above immutable Profile 1 tag `v1.7` and immutable P2.1 tag `p2.1`.

## Goal

Permit one runtime-computed scalar result to cross the observable console boundary without introducing ambient authority or changing certified Profile 1 encodings.

## Source form

P2.2 accepts:

```noi
effect ConsoleWrite;
input key_code;
entry main emits <dynamic-expression>;
```

The dynamic expression is translated into the already-certified V0.7 `returns` computation boundary. It must remain dynamically dependent on the explicit runtime input.

Quoted output literals remain P2.1. Static scalar expressions are rejected from P2.2.

## Result domain

Only V0.7 result kinds are admitted:

- `Int` → canonical base-10 signed integer text;
- `Bool` → `true` or `false`.

Maximum rendered output is 64 bytes.

## Authority

Observable materialization requires exact `Capability::ConsoleWrite`. The grant is host-issued and is never inferred from the source effect declaration. Missing, unrelated, or revoked authority fails closed.

The authority check occurs before dynamic runtime execution in the P2.2 execution boundary.

## Deterministic identity

The P2.2 plan SHA-256 commits to:

- canonical V0.7 dynamic semantics;
- `ConsoleWrite`;
- scalar result kind;
- output bound.

The P2.2 receipt SHA-256 commits to:

- P2.2 plan SHA-256;
- V0.7 runtime receipt SHA-256;
- exact key-code input;
- module, entry, and input names;
- scalar result kind and canonical result;
- rendered output;
- `EXPLICIT` authority;
- `NONE` ambient authority.

No source path, host, OS, process id, or wall-clock timestamp enters the canonical identity.

## Non-goals

P2.2 does not add string concatenation, interpolation, multiple writes, stderr effects, filesystem I/O, network I/O, arbitrary host calls, new NAIR instructions, or Profile 1 serialization changes.

## Certification gate

P2.2 requires repository Release Gate PASS, 20/20 focused tests, deterministic positive/negative smokes, exact-SHA CI success on Linux/macOS/Windows, then immutable annotated tag `p2.2`.
