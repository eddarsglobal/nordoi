# NOI Multi-Segment Structured Output Specification — P2.4

## Status

Candidate specification for NORDOI P2.4.

## Certified baseline

P2.4 is additive above immutable certified milestones:

- Production Profile 1 — `v1.7`;
- P2.1 Capability-Secured Observable I/O — `p2.1`;
- P2.2 Dynamic Capability-Secured Observable Output — `p2.2`;
- P2.3 Structured Dynamic Text Output — `p2.3`.

P2.4 MUST NOT alter those semantics, durable encodings, commands, receipts, kernel behavior, or NAIR formats.

## Surface

Canonical P2.4 form is an ordered alternation of quoted static UTF-8 and runtime segments:

```noi
entry main emits
    "input=" + key_code +
    ", next=" + (key_code + 1) +
    ", accepted=" + (key_code >= 40);
```

Rules:

1. Output MUST begin with quoted static UTF-8 text.
2. P2.4 requires at least 2 and at most 8 runtime segments.
3. A one-runtime-segment program remains P2.3 and MUST be rejected by the P2.4 command.
4. Static quoted segments separate runtime segments; a final quoted suffix is optional.
5. Every runtime segment MUST depend on explicit V0.7 runtime input.
6. Each runtime result MUST independently be `Int` or `Bool`.
7. Numeric `+` inside a runtime segment remains V0.7 arithmetic; P2.4 does not add unrestricted string concatenation.
8. Runtime segment order is source order and is canonical.

## Certified-substrate reuse

Each runtime segment is transformed into an internal P2.3 single-segment plan with empty static text. Therefore the proof chain is:

```text
P2.4 ordered composition
  -> P2.3 per-segment plan/receipt
     -> V0.7 runtime computation/receipt
```

P2.4 MUST NOT add a second expression evaluator or new NAIR opcodes.

## Bounds

`MIN_P24_RUNTIME_SEGMENTS = 2`.

`MAX_P24_RUNTIME_SEGMENTS = 8`.

`MAX_P24_OUTPUT_BYTES = 4096`.

Before authority is checked, compilation MUST prove:

```text
sum(static UTF-8 bytes)
+ sum(maximum canonical render bytes for every runtime segment)
<= 4096
```

Maximum canonical render bytes are 20 for an `i64` and 5 for a `Bool`.

## Authority

The only observable effect is `ConsoleWrite`.

The host MUST explicitly grant `Capability::ConsoleWrite` before any runtime segment is evaluated or any program byte is materialized.

Missing, unrelated, or revoked authority MUST fail closed with zero stdout program bytes.

## Canonical output

For static segments `S0..Sn` and runtime results `R0..R(n-1)`:

```text
S0 || canonical(R0) || S1 || canonical(R1) || ... || canonical(Rn-1) || Sn
```

`Int` rendering is canonical signed base-10 decimal. `Bool` rendering is exactly `true` or `false`.

## Deterministic evidence

The P2.4 plan commits to:

- `ConsoleWrite` boundary;
- exact ordered static segments;
- exact ordered P2.3 canonical segment plans;
- each segment result kind;
- runtime-segment and byte bounds.

The P2.4 receipt commits to:

- P2.4 plan SHA-256;
- exact runtime input;
- module, entry and input identities;
- ordered P2.3 segment receipt SHA-256 identities;
- ordered result kinds and canonical values;
- exact final output;
- explicit authority and zero ambient authority.

Source path, source ID, host, operating system, wall-clock time and username MUST NOT affect canonical identities.

## Non-goals

P2.4 does not certify arbitrary strings, string variables, loops over output segments, dynamic segment counts, heap/string APIs, filesystem/network/process output, or any new kernel/NAIR authority.
