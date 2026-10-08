# NORDOI P2.2 — Dynamic Capability-Secured Observable Output

Production Profile 1 is frozen and certified at tag `v1.7`. P2.1 is certified at immutable tag `p2.1`. P2.2 adds one narrow capability above that certified base: a runtime-computed `Int` or `Bool` may be emitted to stdout only after explicit `ConsoleWrite` authorization.

P2.2 does **not** modify the certified kernel K1.18, the Profile 1 runtime, NAIR encodings, package/release formats, or P2.1 static console semantics.

## Source surface

P2.1 remains unchanged:

```noi
module app.main;
effect ConsoleWrite;
entry main emits "Hello";
```

P2.2 adds dynamic observable output:

```noi
module app.main;

effect ConsoleWrite;

input key_code;
const bias = 2;

entry main emits key_code + bias;
```

The expression is compiled through the already-certified V0.7 dynamic computation vertical. P2.2 requires runtime dependence; static numeric/boolean expressions are rejected and quoted text remains the P2.1 surface.

## Explicit authority

Compilation grants no authority. The new command is additive:

```bash
cargo run --quiet --bin nordoi --   dynamic-console-run examples/p2_2_dynamic/value.noi 40
```

Without `--grant-console`, execution fails closed with zero program bytes on stdout.

Authorized execution:

```bash
cargo run --quiet --bin nordoi --   dynamic-console-run examples/p2_2_dynamic/value.noi 40 --grant-console
```

The program emits exactly:

```text
42
```

Receipt metadata is written to stderr and includes the P2.2 plan identity, the certified V0.7 runtime receipt identity, the explicit authority state, and the final P2.2 receipt identity.

## Security boundary

P2.2 guarantees for this slice:

- source must explicitly declare `effect ConsoleWrite;`;
- host must explicitly grant `Capability::ConsoleWrite`;
- no ambient console authority exists;
- unrelated or revoked capabilities fail closed;
- P2.2 accepts only runtime-computed `Int`/`Bool` output;
- canonical rendering is decimal integer or lowercase `true`/`false`;
- rendered output is bounded to 64 UTF-8 bytes;
- runtime computation reuses the certified V0.7 NAIR path;
- the P2.2 receipt commits to the P2.2 plan, exact runtime input, V0.7 runtime receipt hash, rendered result, and authority mode;
- source path, source ID, hostname, OS, and wall-clock time do not enter the receipt identity;
- no filesystem, network, process, camera, microphone, location, GPU, or XR authority is introduced.

P2.2 deliberately does not add string concatenation or arbitrary formatting. That remains a later milestone.

## Tests

P2.2 adds 20 focused tests:

```bash
cargo test --test observable_io_p22 -- --nocapture
cargo test --test security_observable_io_p22 -- --nocapture
cargo test --test tooling_observable_io_p22 -- --nocapture
```

Expected focused totals are 8 + 8 + 4.

The normal Release Gate remains mandatory:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

## Frozen public boundary

P2.2 remains additive. The public version string stays:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

Production Profile 1 remains immutable at `v1.7`, and P2.1 remains immutable at `p2.1`.
