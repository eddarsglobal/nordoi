# NORDOI V1.5 — Production Diagnostics & Developer Experience

V1.5 is additive over certified V1.4. It makes real NORDOI projects diagnosable by humans, CI systems and future IDE integrations without changing kernel semantics, NAIR, runtime authority or package execution.

## Project check

```bash
cargo run --quiet --bin nordoi -- check demo
```

A clean project reports a deterministic summary and writes no `NORDOI.lock` or `.npkg` output:

```text
check project="demo" version="0.1.0" entry-module="app.main" modules=2 imports=1 nair-minor=0.12 nair-instructions=3 diagnostics=0 status=PASS dependency-network=NONE runtime-fs=NONE authority=NONE
```

Machine-readable mode:

```bash
cargo run --quiet --bin nordoi -- check demo --json
```

The output uses the stable schema `nordoi.diagnostic.v1`.

## Stable diagnostic codes

V1.5 separates machine identity from human wording. Initial stable families are:

- `NDX1001` project manifest;
- `NDX2001` project/source I/O;
- `NDX2002` source/lexical boundary;
- `NDX2003` module declaration;
- `NDX2004` missing imported module;
- `NDX2005` cyclic import graph;
- `NDX2101` module-graph resolution;
- `NDX2201` language semantics;
- `NDX3001` lowering/NAIR validation;
- `NDX4001` package validation;

Codes are the compatibility surface. Human wording may improve later without changing the code meaning.

## Multi-file import traces

When project loading reaches an unavailable imported module, V1.5 records the deterministic path that led to it:

```text
error[NDX2004] source-io: cannot load module 'app.math': ...
 --> demo/src/app/math.noi
  = import-trace: app.main -> app.mid -> app.math
```

Import cycles are similarly reported as a closed deterministic trace.

## Source locations

Frontend errors that already carry certified `SourceSpan` information are projected as `file:line:column` with a source excerpt and caret. V1.5 does not invent locations for errors that do not own a source span.

## JSON contract

Error example:

```json
{"schema":"nordoi.diagnostic.v1","status":"error","code":"NDX2004","stage":"source-io","message":"...","location":{"file":"demo/src/app/math.noi","line":0,"column":0,"excerpt":""},"importTrace":["app.main","app.math"]}
```

A clean check produces a single success object with `status="pass"` and an empty `diagnostics` array.

## Zero runtime change

V1.5 is compiler/tooling-only. Existing certified lowering remains exact:

- static project -> NAIR 0.6;
- simple dynamic imported call -> NAIR 0.12;
- transitive acyclic runtime call graph -> NAIR 0.13;
- structured runtime function control -> NAIR 0.14.

There is no dependency network, dynamic module loader, runtime filesystem grant or new host authority.

## Certification boundary

V1.5 does **not** change the frozen public version string:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

See:

- `docs/NOI_PRODUCTION_DIAGNOSTICS_SPEC_1_5.md`
- `research/PRODUCTION_DIAGNOSTICS_INTELLIGENCE_1_5.md`
- `docs/PRODUCTION_PROFILE_1_PROGRESS.md`
