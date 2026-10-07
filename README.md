# NORDOI V0.8 — Dynamic Control Flow Vertical Slice

V0.8 is additive over certified V0.7. It introduces the first governed runtime control-flow decision while preserving NORDOI's rule that static work disappears before runtime.

Minimal V0.8 example:

```noi
input key_code;

entry main returns if key_code > 40 {
    100
} else {
    200
};
```

Run it with:

```bash
cargo run --quiet --bin nordoi -- branch-run program.noi 41
```

Expected semantic result: `INT(100)` with `runtime-calls=0`, `runtime-branches=1`, `nair-minor=0.10`, `authority=NONE`, and `input-boundary=CANONICAL`.

The V0.8 branch primitive is deliberately structured rather than a general jump machine. A dynamic condition may choose between two statically reducible branch values. This creates real runtime path selection without introducing arbitrary instruction pointers, a runtime call stack, recursion, or a conventional VM.

Static conditions continue to erase before NAIR. For example:

```noi
entry main returns if 2 > 1 { 42 } else { 7 };
```

still lowers to base NAIR `0.6`:

```text
CONST r0 INT(42)
HALT
```

Certified V0.7 `dynamic-run` remains unchanged and continues to use NAIR `0.9` for dynamic input, arithmetic, and comparison without runtime branching.

See `docs/NOI_DYNAMIC_CONTROL_FLOW_VERTICAL_SLICE_SPEC_0_8.md`, `docs/NAIR_SPEC_0_10.md`, and `docs/PRODUCTION_PROFILE_1_PROGRESS.md`.
