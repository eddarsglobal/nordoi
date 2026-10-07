# NORDOI V1.1 — Acyclic Bounded Runtime Call Graphs

V1.1 is additive over certified V1.0. It extends bounded direct runtime calls into statically known, acyclic, bounded pure call graphs without introducing a general VM stack, indirect dispatch, recursion, or hidden authority.

Minimal V1.1 example:

```noi
fn add_100(x) returns x + 100;
fn add_200(x) returns add_100(x) + 100;

input key_code;

entry main returns add_200(key_code);
```

Run it with:

```bash
cargo run --quiet --bin nordoi -- call-graph-run program.noi 41
```

Expected result: `INT(241)` with `runtime-calls=2`, `max-call-depth=2`, `runtime-branches=0`, authority `NONE`, and canonical input boundary `CANONICAL`.

## NAIR 0.13

NAIR 0.13 extends the existing 0.12 `CALL_EVAL` body expression with a canonical nested direct-call expression. The callee identity, argument expressions, and callee body are embedded deterministically. The runtime therefore does not discover call targets, resolve function pointers, or operate an unbounded program counter/call stack.

A V1.0-style `CALL_EVAL` with no nested direct call remains NAIR 0.12. Only a body that actually contains an acyclic nested direct call requires 0.13.

## Call-graph law

Before lowering, V1.1 constructs the complete direct call graph and rejects:

- direct recursion;
- indirect recursion and cycles such as `A -> B -> A`;
- unknown callees;
- arity mismatch;
- call depth above the certified bound;
- indirect calls and function values;
- effects or host authority in function bodies.

The certified V1.1 maximum call depth is 8 and the parameter bound remains 8. Runtime work is observed explicitly through `runtime-calls`, `call-body-instructions`, and `max-call-depth`.

## Static erasure remains mandatory

A fully static acyclic call graph is evaluated before runtime and still collapses to base NAIR 0.6 `CONST + HALT`. V1.1 pays runtime cost only for genuinely dynamic work.

The public certified tool boundary intentionally remains:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
