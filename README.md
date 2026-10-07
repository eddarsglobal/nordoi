# NORDOI V1.0 — Bounded Runtime Function Calls

V1.0 is additive over certified V0.9. It introduces direct runtime function calls while keeping the Atomic Machine deliberately bounded and non-VM-like.

Minimal V1.0 example:

```noi
fn add_bias(x) returns x + 100;

input key_code;

entry main returns add_bias(key_code);
```

Run it with:

```bash
cargo run --quiet --bin nordoi -- call-run program.noi 41
```

Expected semantic result: `INT(141)` with `runtime-calls=1`, `runtime-branches=0`, `call-body-instructions=3`, `max-call-depth=1`, `nair-minor=0.12`, `authority=NONE`, and `input-boundary=CANONICAL`.

NAIR `0.12` adds structured `CALL_EVAL`. A call carries a canonical function identity, an explicitly bounded argument register list, and a bounded pure call-expression body. Parameters are positional and V1.0 parameters are INT-only.

V1.0 deliberately does **not** add a general call stack. Function bodies cannot call functions, so runtime call depth is certified at exactly `<= 1`. Recursion, indirect calls, function values, arbitrary jump targets, host callbacks, effects inside call bodies, and hidden authority remain forbidden.

Static calls are folded before runtime and remain base NAIR `0.6 CONST + HALT`. Earlier NAIR `0.7` through `0.11` programs preserve their existing encodings and semantics.

The public tooling version string remains intentionally frozen:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

See `docs/NOI_BOUNDED_RUNTIME_FUNCTION_CALLS_SPEC_1_0.md`, `docs/NAIR_SPEC_0_12.md`, and `docs/PRODUCTION_PROFILE_1_PROGRESS.md`.
