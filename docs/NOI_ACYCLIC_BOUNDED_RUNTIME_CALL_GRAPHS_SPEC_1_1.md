# NOI V1.1 — Acyclic Bounded Runtime Call Graphs

## Purpose

V1.1 extends V1.0 pure runtime calls so a pure function may directly call another declared pure function when the complete graph is statically known, acyclic, and bounded.

## Source form

```noi
fn add_100(x) returns x + 100;
fn add_200(x) returns add_100(x) + 100;
input key_code;
entry main returns add_200(key_code);
```

For `key_code = 41`, the result is `INT(241)`.

## Constitutional restrictions

V1.1 forbids:

- direct recursion;
- mutual or longer recursion cycles;
- indirect dispatch;
- function pointers / function values;
- runtime call-target discovery;
- effectful call bodies;
- unbounded call depth;
- general VM call stacks.

Parameters remain INT-only in this vertical slice, with at most 8 parameters per function. The complete graph is limited to the existing function bound and a certified maximum runtime depth of 8.

## Compilation

The compiler must:

1. parse all pure function declarations;
2. resolve every direct callee and arity;
3. build the complete direct call graph;
4. reject any cycle before lowering;
5. compute a canonical maximum call depth;
6. infer result kinds through the acyclic graph;
7. statically erase fully static calls;
8. lower genuinely dynamic nested calls to NAIR 0.13.

## Runtime proof

For the canonical two-level example, the expected observable proof is:

```text
result=INT(241)
runtime-computed=true
runtime-calls=2
runtime-branches=0
max-call-depth=2
nair-minor=0.13
authority=NONE
input-boundary=CANONICAL
```

The public certified `--version` output remains unchanged.
