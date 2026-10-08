# Structured Dynamic Text Output Intelligence — P2.3

## Decision

P2.3 should not introduce a general-purpose string `+` operator. Doing so would blur numeric addition, string coercion, allocation behavior, type rules, and future optimizer semantics in one milestone.

The smallest production-useful step is a structured template with one runtime slot:

```noi
"prefix" + runtime_expression + "suffix"
```

The source uses familiar `+` punctuation, but the P2.3 compiler recognizes a bounded structured-output form. The runtime expression itself remains V0.7 arithmetic/comparison semantics.

## Why one runtime slot

One slot is sufficient to prove the important new boundary:

- static UTF-8 decoding;
- runtime value computation;
- deterministic canonical conversion;
- bounded composition;
- explicit authority;
- receipt binding across static and runtime evidence.

Multiple slots can later be added as an additive template model after the one-slot evidence is certified.

## Security reasoning

Authority is checked before runtime computation. This prevents an unauthorized observable command from using the output path as a hidden computation oracle and preserves P2.2's deny-before-materialization property.

Quota is proven before authority using worst-case render widths. This prevents runtime-dependent output size from becoming an unbounded or late-failing resource decision.

## Compatibility reasoning

P2.3 lives in `observable_io_p23.rs` and adds a new CLI command. P2.1 and P2.2 remain callable and semantically unchanged. No old durable format receives new variants.

## Future direction

A later milestone may generalize structured output to multiple typed interpolation slots, but it should preserve:

- explicit type conversion;
- fixed quotas;
- deterministic canonical rendering;
- capability-scoped sinks;
- receipt identities that commit to every rendered slot.
