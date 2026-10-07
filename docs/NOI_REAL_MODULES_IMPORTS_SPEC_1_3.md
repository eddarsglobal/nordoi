# NOI Real Modules & Imports Specification 1.3

## Status

Candidate V1.3 specification. Certification requires the complete local Release Gate, focused V1.3 tests, positive and negative smoke proofs, exact-SHA multi-platform CI, and an immutable annotated tag.

## Objective

V1.3 introduces real multi-file NOI modules and statically resolved imports without introducing runtime module loading or a new NAIR format.

## Canonical source-root mapping

A canonical module path `a.b.c` maps to:

```text
<source-root>/a/b/c.noi
```

The loaded file MUST declare:

```noi
module a.b.c;
```

A declaration mismatch fails closed.

## Import form

V1.3 accepts:

```noi
import app.math;
```

The short qualifier is the final module segment. `app.math` is referenced as `math`:

```noi
entry main returns math.add_100(key_code);
```

Two imports with the same final segment are ambiguous and MUST be rejected.

## Static graph law

Before lowering, the compiler MUST know the complete reachable module graph from the entry module. The graph MUST satisfy:

- all imported modules exist;
- every canonical module identity is unique;
- import aliases are unambiguous;
- recursive/cyclic imports are rejected;
- module and edge ordering is canonical and deterministic;
- unreachable modules do not affect the reachable graph witness.

Certified bounds:

- maximum reachable modules: 64;
- maximum imports per module: 32;
- maximum reachable import edges: 256.

## V1.3 library-module boundary

Imported modules are library-only in the V1.3 vertical slice. They MAY define pure functions and further imports. They MUST NOT define `input` or `entry`. Imported-module `const` declarations are deferred beyond V1.3.

The entry module owns the canonical type/effect prelude, input, constants, and entry expression. Imported modules remain pure-function libraries in this first slice.

## Name resolution

Local pure function calls bind to the current module. Qualified calls of the form `alias.function(...)` bind to the module imported by `alias`.

The compiler rewrites resolved functions into collision-free internal canonical names before invoking the certified V1.2 function/control compiler.

Dynamic target discovery, function values, wildcard imports, re-exports, reflection, and explicit `as` aliases are outside V1.3.

## NAIR boundary

V1.3 MUST introduce no module/import runtime opcode and no new NAIR minor. Imports are erased before NAIR.

Existing requirements remain:

- static result: NAIR 0.6;
- simple dynamic call: NAIR 0.12;
- acyclic nested call graph: NAIR 0.13;
- structured runtime function control: NAIR 0.14.

## Authority boundary

The tooling/compiler MAY read source files from the explicitly supplied source root. After compilation, runtime module resolution and runtime filesystem access are forbidden in V1.3.

Expected runtime/tooling evidence:

```text
import-resolution=STATIC
runtime-fs=NONE
authority=NONE
```

## Failure law

All of the following fail before runtime publication:

- missing imported module;
- path/declaration module mismatch;
- duplicate module identity;
- ambiguous short import alias;
- cyclic import graph;
- unknown qualified imported function;
- imported module declaring input or entry;
- imported-module const state in this V1.3 slice.
