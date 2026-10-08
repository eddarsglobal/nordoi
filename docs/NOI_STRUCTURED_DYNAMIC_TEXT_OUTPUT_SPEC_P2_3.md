# NOI Structured Dynamic Text Output Specification — P2.3

## Status

Candidate specification for NORDOI P2.3.

## Certified baseline

P2.3 is additive above:

- Production Profile 1, immutable tag `v1.7`;
- P2.1 Capability-Secured Observable I/O, immutable tag `p2.1`;
- P2.2 Dynamic Capability-Secured Observable Output, immutable tag `p2.2`.

P2.3 MUST NOT alter the semantics or durable encodings certified by those milestones.

## Surface

Canonical P2.3 entry form:

```noi
entry <name> emits <quoted-prefix> + <dynamic-expression> [ + <quoted-suffix> ];
```

Rules:

1. `<quoted-prefix>` is mandatory.
2. Exactly one runtime expression is allowed.
3. `<quoted-suffix>` is optional.
4. Quoted text inside the runtime segment is forbidden.
5. The runtime segment MUST depend on explicit V0.7 runtime input.
6. Runtime result kind MUST be `Int` or `Bool`.
7. Numeric addition remains V0.7 arithmetic, not a general string operator.

## Canonical rendering

`Int` uses canonical base-10 signed rendering with no decoration.

`Bool` uses exactly `true` or `false`.

Final output is:

```text
prefix || canonical(runtime-result) || suffix
```

where `||` denotes P2.3 structural composition, not a new NOI runtime operator.

## Bounds

`MAX_P23_OUTPUT_BYTES = 4096`.

Before any authority check or runtime execution, compilation MUST prove:

```text
prefix_bytes + maximum_runtime_render_bytes + suffix_bytes <= 4096
```

Maximum runtime render bytes are 20 for `i64` and 5 for `Bool`.

## Authority

The only effect is `ConsoleWrite`.

The only authority capable of authorizing output is explicit host `Capability::ConsoleWrite`.

Missing, unrelated, or revoked authority MUST fail closed before runtime computation and before any stdout program byte is materialized.

## Runtime reuse

P2.3 MUST derive its runtime segment from the certified V0.7 dynamic-input computation vertical. It MUST NOT introduce a second expression evaluator or new NAIR opcodes.

## Deterministic evidence

The P2.3 plan identity commits to:

- V0.7 canonical semantic bytes;
- result kind;
- decoded prefix;
- decoded suffix;
- `ConsoleWrite` boundary;
- output byte bound.

The P2.3 execution receipt commits to:

- P2.3 plan SHA-256;
- V0.7 runtime receipt SHA-256;
- exact runtime key-code;
- module, entry, and input identities;
- runtime result kind and canonical value;
- final structured output;
- explicit authority mode and zero ambient authority.

Source paths, source IDs, hostnames, operating systems, wall-clock time, and usernames MUST NOT affect plan or receipt identities.

## Non-goals

P2.3 does not certify:

- arbitrary string variables;
- multiple runtime interpolation slots;
- unrestricted string concatenation;
- dynamic heap/string APIs;
- file, network, process, camera, microphone, location, GPU, or XR I/O;
- new kernel or NAIR semantics.
