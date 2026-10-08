# NORDOI P2.3 — Structured Dynamic Text Output

Production Profile 1 is frozen and certified at immutable tag `v1.7`. P2.1 and P2.2 are certified at immutable tags `p2.1` and `p2.2`. P2.3 adds one narrow structured-text capability above that base: bounded static UTF-8 text may surround exactly one runtime-computed `Int` or `Bool`, and the final output may reach stdout only after explicit `ConsoleWrite` authorization.

P2.3 does **not** modify kernel K1.18, Profile 1 runtime semantics, NAIR encodings, package/release formats, P2.1 static console behavior, or P2.2 dynamic-value behavior.

## Source surface

P2.3 accepts forms such as:

```noi
module app.main;

effect ConsoleWrite;

input key_code;
const bias = 2;

entry main emits "value=[" + (key_code + bias) + "]";
```

and:

```noi
entry main emits "accepted=" + (key_code >= 40);
```

The first quoted UTF-8 segment is required. One optional quoted suffix is allowed. Exactly one runtime segment sits between them. Numeric `+` remains inside the V0.7 runtime expression, so NORDOI does not introduce an unrestricted string-addition operator.

Examples:

```noi
entry main emits "value=" + key_code;
entry main emits "value=" + (key_code + bias);
entry main emits "[" + key_code + "]";
entry main emits "accepted=" + (key_code >= 40);
```

The runtime segment must depend on explicit runtime input. A template such as `"value=" + 42` is rejected by P2.3.

## Explicit authority

Compilation grants no authority. The additive command is:

```bash
cargo run --quiet --bin nordoi -- \
  structured-console-run examples/p2_3_structured/value.noi 40
```

Without `--grant-console`, execution fails closed with zero program bytes on stdout.

Authorized execution:

```bash
cargo run --quiet --bin nordoi -- \
  structured-console-run examples/p2_3_structured/value.noi 40 --grant-console
```

The program emits exactly:

```text
value=[42]
```

Receipt metadata is written to stderr. It commits to the P2.3 template plan, the exact V0.7 runtime receipt identity, the runtime input, the canonical rendered value, the final structured output, and the explicit authority state.

## Security boundary

P2.3 guarantees for this slice:

- source must explicitly declare `effect ConsoleWrite;`;
- host must explicitly grant `Capability::ConsoleWrite`;
- no ambient console authority exists;
- unrelated or revoked capabilities fail closed;
- exactly one runtime `Int`/`Bool` segment is permitted;
- static prefix is required and one static suffix is optional;
- runtime computation reuses the certified V0.7 path;
- integer rendering is canonical decimal and boolean rendering is lowercase `true`/`false`;
- total structured output is bounded to 4096 UTF-8 bytes;
- worst-case output size is proven before authority or runtime execution;
- P2.3 plan and receipt identities are deterministic and source-path/host/time independent;
- no filesystem, network, process, camera, microphone, location, GPU, or XR authority is introduced.

P2.3 deliberately does not add arbitrary string variables, multiple runtime interpolation slots, dynamic allocation APIs, filesystem output, or network output. Those remain later milestones.

## Tests

P2.3 adds 20 focused tests:

```bash
cargo test --test observable_io_p23 -- --nocapture
cargo test --test security_observable_io_p23 -- --nocapture
cargo test --test tooling_observable_io_p23 -- --nocapture
```

Expected focused totals are 8 + 8 + 4.

The normal Release Gate remains mandatory:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

## Frozen public boundary

P2.3 remains additive. The public version string stays:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

Production Profile 1 remains immutable at `v1.7`; P2.1 remains immutable at `p2.1`; P2.2 remains immutable at `p2.2`.
