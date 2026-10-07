# NOI Bounded Runtime Function Calls — Vertical Slice 1.0

## Status

Candidate specification for NORDOI V1.0. Certification requires the complete local Release Gate, focused V1.0 tests, exact-SHA multi-platform CI, and an immutable annotated `v1.0` tag.

## Objective

V1.0 introduces the first runtime function-call boundary without turning NAIR into a conventional bytecode VM.

The certified source shape is:

```noi
fn add_bias(x) returns x + 100;
input key_code;
entry main returns add_bias(key_code);
```

For input key-code `41`, the required result is `INT(141)`.

## Source rules

- Direct named pure functions only.
- At most 32 functions.
- At most 8 parameters per function.
- Parameters are INT-only in V1.0.
- Function bodies may contain integer literals, names, checked `+`, and integer comparisons.
- Function bodies cannot call functions.
- Entry expressions may contain direct function calls.
- Function-call arguments cannot themselves contain calls.
- At most 32 runtime calls may survive lowering.
- Constants remain compile-time only.
- Dynamic input remains the canonical keyboard key-code input introduced in V0.7.

## Runtime rules

A dynamic direct call lowers to NAIR `CALL_EVAL`.

The runtime:

1. resolves the already-defined argument registers;
2. opens one conceptual bounded call frame;
3. evaluates the certified pure call-expression body;
4. writes exactly one result register;
5. closes the conceptual frame.

V1.0 does not expose a mutable call stack, instruction pointer, arbitrary return address, indirect target, closure, or host callback.

`max-call-depth <= 1` is a semantic invariant.

## Static erasure

When every argument is static, the function call is evaluated before NAIR emission and the program remains base NAIR `0.6 CONST + HALT`.

## Security and authority

`CALL_EVAL` grants no capability and no host authority. Call bodies are pure. No filesystem, network, process, camera, microphone, location, GPU, XR, or other authority can be reached through V1.0 call semantics.

## Required proof

For the canonical smoke source with key-code `41`:

```text
result=INT(141)
runtime-computed=true
runtime-calls=1
runtime-branches=0
call-body-instructions=3
max-call-depth=1
nair-minor=0.12
authority=NONE
input-boundary=CANONICAL
```

The public `--version` boundary remains unchanged.
