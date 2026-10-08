# NORDOI P2.1 — Capability-Secured Observable I/O

Production Profile 1 is frozen and certified at tag `v1.7`. P2.1 opens Production Profile 2 with one deliberately narrow observable capability: bounded UTF-8 console output guarded by an explicit source effect declaration and an explicit host capability grant.

P2.1 does **not** modify the certified kernel, runtime, NAIR instruction set, package format, release format, or Production Profile 1 certification chain.

## Source surface

The first observable slice is intentionally small:

```noi
module app.main;

effect ConsoleWrite;

entry main emits "Hello from NORDOI P2.1!\n";
```

P2.1 accepts exactly one `entry <name> emits "<text>";` body and one declared `effect ConsoleWrite;`. Output is bounded to 4096 decoded UTF-8 bytes. Supported escapes are `\\`, `\"`, `\n`, `\r`, and `\t`.

## Explicit authority

Compilation alone grants no authority. Running without a host grant fails closed and emits zero program bytes:

```bash
cargo run --quiet --bin nordoi -- \
  console-run examples/p2_1_console/hello.noi
```

The observable effect executes only with the exact capability:

```bash
cargo run --quiet --bin nordoi -- \
  console-run examples/p2_1_console/hello.noi --grant-console
```

Program output is written exactly to stdout. Deterministic receipt metadata is written to stderr so tooling metadata never changes the program's output bytes.

## Security boundary

P2.1 guarantees for this slice:

- source must explicitly declare `effect ConsoleWrite;`;
- host must explicitly grant `Capability::ConsoleWrite`;
- unrelated capabilities never authorize console output;
- revoke removes authority immediately;
- no ambient authority exists;
- no filesystem, network, process, camera, microphone, location, GPU, or XR authority is introduced;
- output is bounded before authority is exercised;
- receipt identity is deterministic and independent of source file path, source ID, host name, OS, or wall-clock time;
- denial happens before stdout receives program bytes.

`ConsoleWrite` is a P2.1-local observable effect. It is intentionally **not** added to the frozen Profile 1 `Effect` serialization because doing so would mutate certified NAIR/audit/persistence encodings. P2.1 reuses the existing `CapabilitySet` authority model while preserving those byte-level boundaries unchanged.

## Receipt

A successful execution creates a deterministic canonical receipt and SHA-256 identity. Tooling reports fields including:

```text
status=EMITTED
authority=EXPLICIT
grant=ConsoleWrite
ambient-authority=NONE
plan-sha256=...
receipt-sha256=...
```

The library also exposes schema `nordoi.observable-io.p2.1` through `P21ObservableOutputReceipt::render_json()`.

## Tests

P2.1 adds 20 focused tests:

```bash
cargo test --test observable_io_p21 -- --nocapture
cargo test --test security_observable_io_p21 -- --nocapture
cargo test --test tooling_observable_io_p21 -- --nocapture
```

Expected focused totals are 8 + 8 + 4.

The normal repository Release Gate remains mandatory:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

## Frozen public boundary

P2.1 is additive and does not promote the historical public version string. It remains:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

Production Profile 1 remains immutable at `v1.7`; P2.1 is the first candidate milestone of Production Profile 2.
