# NORDOI V0.7 — Dynamic Input & Runtime Computation Vertical Slice

V0.7 is additive over certified V0.6. It introduces the first value intentionally unknown at compile time: an explicit canonical input binding consumed through `InputBatch` and lowered to a NAIR 0.9 input-register instruction. Static V0.6 computation remains frozen and fully foldable.

Minimal V0.7 example:

```noi
input key_code;
const bias = 2;
entry main returns key_code + bias;
```

Run it with:

```bash
cargo run --quiet --bin nordoi -- dynamic-run program.noi 40
```

Expected semantic result: `INT(42)` with `runtime-calls=0`, `runtime-branches=0`, and `nair-minor=0.9`.


> Production-batch candidate. `CONSTITUTION.md` remains the supreme project authority.

V0.6 accelerates NORDOI by certifying a practical closed pure-computation core in one vertical batch instead of releasing one operator or compiler layer at a time.

New command:

```text
nordoi core-run <path|->
```

Example:

```noi
const base = 20;

fn scale(x, factor) {
    const product = x * factor;
    if product >= 40 {
        product + 2
    } else {
        0
    }
}

entry main returns scale(base, 2);
```

V0.6 supports checked `+ - * /`, comparisons, `&& || !`, immutable globals/locals, pure functions, parameters, nested calls and nested static `if/else` expressions.

Because the V0.6 surface is closed and pure, the complete computation is proven before runtime and optimized to `CONST result; HALT`. Function calls and branches remain present in the canonical semantic witness but cost zero runtime frames and zero runtime branches.

See `docs/NOI_CORE_COMPUTATION_FUNCTIONS_VERTICAL_SLICE_SPEC_0_6.md` and `docs/PRODUCTION_PROFILE_1_PROGRESS.md`.
