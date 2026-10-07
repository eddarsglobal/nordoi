# NORDOI V0.6 — Core Computation & Functions Vertical Slice

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
