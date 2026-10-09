# NORDOI P2.4 — Multi-Segment Structured Output

Production Profile 1 and P2.1/P2.2/P2.3 are immutable certified baselines. P2.4 adds one narrow capability above them: a single observable output may contain **2 to 8 ordered runtime `Int`/`Bool` segments** separated by bounded static UTF-8 text.

P2.4 does not change kernel K1.18, NAIR, runtime semantics, package/release formats, or any already-certified P2.x command.

## Source surface

```noi
module app.main;

effect ConsoleWrite;

input key_code;
const bias = 1;

entry main emits
    "input=" + key_code +
    ", next=" + (key_code + bias) +
    ", accepted=" + (key_code >= 40);
```

P2.4 requires at least two runtime segments. One segment remains P2.3.

Each dynamic slot reuses the certified P2.3 -> V0.7 computation path. Numeric arithmetic is still numeric arithmetic; P2.4 does not introduce unrestricted String `+`.

## Explicit authority

Without authority:

```bash
cargo run --quiet --bin nordoi -- \
  multi-console-run examples/p2_4_multi/value.noi 40
```

must fail closed with zero stdout program bytes.

Authorized execution:

```bash
cargo run --quiet --bin nordoi -- \
  multi-console-run examples/p2_4_multi/value.noi 40 --grant-console
```

emits exactly:

```text
input=40, next=41, accepted=true
```

Receipt metadata is written to stderr and commits to ordered P2.3 segment receipts, values, final output, and explicit authority.

## Bounds

- minimum runtime segments: 2;
- maximum runtime segments: 8;
- maximum final UTF-8 output: 4096 bytes;
- worst-case size is proven before authority and runtime execution.

## Tests

```bash
cargo test --test observable_io_p24 -- --nocapture
cargo test --test security_observable_io_p24 -- --nocapture
cargo test --test tooling_observable_io_p24 -- --nocapture
```

Expected focused totals: 8 + 8 + 4.

The normal Release Gate remains mandatory:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

## Frozen public boundary

The public version string intentionally remains:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
