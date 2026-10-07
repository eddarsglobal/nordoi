# NAIR 0.13 — Acyclic Bounded Runtime Call Graphs

## Status

V1.1 candidate specification. Additive over NAIR 0.12.

## New semantic capability

NAIR 0.13 permits a `CALL_EVAL` body to contain a nested direct-call expression:

```text
DirectCall {
  function_id,
  args,
  body
}
```

This is not an indirect call instruction and does not create a general-purpose VM stack. The nested call target and its pure body are canonically embedded before runtime.

## Compatibility

- base static NAIR remains 0.6;
- arithmetic remains 0.7;
- comparison remains 0.8;
- canonical input register remains 0.9;
- dynamic value branch remains 0.10;
- selective branch body remains 0.11;
- direct bounded runtime call remains 0.12;
- nested acyclic bounded call expression requires 0.13.

A 0.12 `CALL_EVAL` with no nested direct call remains byte-compatible and continues to require 0.12.

## Validation

Validation is fail-closed and enforces:

- all outer argument registers exist;
- nested argument count is bounded;
- parameters are in range;
- call-expression node/depth bounds remain enforced;
- nested call depth is at most 8;
- integer operations remain checked;
- only pure INT/BOOL call-expression values are supported.

Source-level recursion/cycle detection belongs to the V1.1 language/compiler vertical slice. NAIR itself stores only a finite embedded expression tree and therefore cannot encode an actual cyclic object graph.

## Runtime observation

Runtime reports include:

- `runtime_calls` counting outer and nested calls;
- `call_body_instructions` counting evaluated call-expression nodes;
- `max_call_depth` measuring the deepest active direct-call chain.

No call grants authority. No host callback is introduced.
