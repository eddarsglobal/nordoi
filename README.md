# NORDOI V0.9 — Dynamic Branch Bodies & Selective Runtime Evaluation

V0.9 is additive over certified V0.8. It allows a dynamic `if/else` to carry bounded pure computation inside its branches while proving that only the selected branch body performs runtime expression work.

Minimal V0.9 example:

```noi
input key_code;

entry main returns if key_code > 40 {
    key_code + 100
} else {
    key_code + 200
};
```

Run it with:

```bash
cargo run --quiet --bin nordoi -- branch-body-run program.noi 41
```

Expected semantic result: `INT(141)` with `runtime-calls=0`, `runtime-branches=1`, `selected-branch-instructions=3`, `discarded-branch-instructions=0`, `nair-minor=0.11`, `authority=NONE`, and `input-boundary=CANONICAL`.

With key-code `39`, only the else expression is evaluated and the result is `INT(239)`.

NAIR `0.11` adds structured `BRANCH_EVAL`. Its branch bodies are bounded pure expression trees over canonical values and already-defined registers. There is still no arbitrary jump target, instruction pointer API, runtime function stack, recursion, implicit host call, or hidden authority.

V0.8 `BRANCH_VALUE` remains unchanged. If a dynamic condition chooses only between two compile-time values, V0.9 preserves the smaller NAIR `0.10` representation instead of forcing `0.11`.

Static conditions continue to erase before runtime. Fully static programs still collapse to base NAIR `0.6` `CONST + HALT`.

See `docs/NOI_DYNAMIC_BRANCH_BODY_VERTICAL_SLICE_SPEC_0_9.md`, `docs/NAIR_SPEC_0_11.md`, and `docs/PRODUCTION_PROFILE_1_PROGRESS.md`.
