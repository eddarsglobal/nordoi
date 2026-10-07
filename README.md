# NORDOI V1.3 — Real Modules & Imports

V1.3 is additive over certified V1.2 and shifts the Production Profile toward a usable multi-file language. NORDOI source can now be organized as real `.noi` modules with statically resolved imports while preserving deterministic compilation, zero hidden runtime authority, acyclic bounded execution, and the certified NAIR/runtime boundaries.

Minimal V1.3 source tree:

```text
source-root/
└── app/
    ├── math.noi
    └── main.noi
```

`app/math.noi`:

```noi
module app.math;

fn add_100(x) returns x + 100;
```

`app/main.noi`:

```noi
module app.main;

import app.math;

input key_code;
entry main returns math.add_100(key_code);
```

Run the graph with:

```bash
cargo run --quiet --bin nordoi -- \
  module-graph-run source-root app.main 41
```

Expected proof includes `modules=2`, `imports=1`, `result=INT(141)`, `runtime-calls=1`, `import-resolution=STATIC`, `runtime-fs=NONE`, and `authority=NONE`.

## Module resolution law

V1.3 uses canonical dotted module identities. CLI source-root resolution maps:

```text
app.main -> <source-root>/app/main.noi
app.math -> <source-root>/app/math.noi
```

Every loaded file must declare the exact module identity expected for its path. Imports are resolved before lowering, missing modules fail closed, duplicate canonical module identities fail closed, ambiguous short aliases fail closed, and recursive/cyclic import graphs fail closed.

The short qualifier is the final module segment:

```noi
import app.math;
entry main returns math.add_100(key_code);
```

Explicit import aliases are deliberately deferred beyond V1.3.

## Library-only imported modules

The V1.3 vertical slice keeps imported modules deliberately narrow: imported modules may define pure functions and imports, but may not declare `input`, `entry`, or imported-module `const` state. The entry module owns the canonical type/effect prelude, input, constants, and entry boundary.

This restriction keeps the first real module system easy to verify while still supporting transitive pure function graphs and all previously certified runtime composition.

## Imports disappear before NAIR

V1.3 introduces **no new NAIR minor**. Module and import structure is compile-time language/compiler metadata and is erased before NAIR publication.

Therefore existing certified NAIR minors remain exact:

- fully static imported program -> NAIR 0.6;
- one direct dynamic imported call -> NAIR 0.12;
- transitive acyclic imported call graph -> NAIR 0.13;
- imported structured function control -> NAIR 0.14.

No runtime module lookup, runtime filesystem access, dynamic import, package-network access, reflection, indirect dispatch, or general VM loader is introduced.

## Determinism and authority

Reachable modules and import edges are canonicalized independently of caller/source-array order. The V1.3 witness commits to the entry module, canonical reachable module set, canonical import graph, and the already-certified V1.2 semantic plan.

Filesystem reading belongs only to the tooling/compiler input phase. The runtime receives already-lowered NAIR and retains:

```text
authority=NONE
runtime-fs=NONE
import-resolution=STATIC
```

The public certified tool boundary intentionally remains:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
