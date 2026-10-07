# NORDOI V1.4 — Project Build & Package System

V1.4 is additive over certified V1.3 and turns real multi-file `.noi` programs into deterministic, lockable, distributable project artifacts without introducing a package network, build scripts, runtime module loading, or a new NAIR minor.

## Project layout

```text
demo/
├── NORDOI.toml
└── src/
    └── app/
        ├── math.noi
        └── main.noi
```

`NORDOI.toml`:

```toml
[project]
name = "demo"
version = "0.1.0"
entry = "app.main"
source-root = "src"
```

`src/app/math.noi`:

```noi
module app.math;
fn add_100(x) returns x + 100;
```

`src/app/main.noi`:

```noi
module app.main;
import app.math;
input key_code;
entry main returns math.add_100(key_code);
```

Build:

```bash
cargo run --quiet --bin nordoi -- build demo
```

V1.4 produces:

```text
demo/
├── NORDOI.toml
├── NORDOI.lock
├── build/
│   └── demo-0.1.0.npkg
└── src/...
```

A successful build reports `reproducible=true`, `dependency-network=NONE`, `runtime-fs=NONE`, and `authority=NONE`.

## Locked reproducibility

After the first build:

```bash
cargo run --quiet --bin nordoi -- build demo --locked
```

`--locked` recompiles the current canonical manifest/module graph and compares the generated lock state byte-for-byte with `NORDOI.lock`. Any semantic source change, import-graph change, manifest change, or lowering change is rejected before package publication.

## Package inspection

```bash
cargo run --quiet --bin nordoi -- package-info demo/build/demo-0.1.0.npkg
```

The package inspector validates V1.4 package framing and embedded canonical NAIR without reading project sources.

## No new runtime machinery

V1.4 is a compiler/tooling/build milestone. Imports remain erased before NAIR. Existing certified formats remain exact:

- static project -> NAIR 0.6;
- simple dynamic imported call -> NAIR 0.12;
- transitive acyclic call graph -> NAIR 0.13;
- structured runtime control -> NAIR 0.14.

There is no package registry, dependency download, arbitrary build command, plugin hook, post-install script, dynamic module loader, or runtime filesystem authority in V1.4.

## Manifest law

The V1.4 manifest parser intentionally accepts only one `[project]` section with exactly four required keys: `name`, `version`, `entry`, and `source-root`. Unknown/duplicate keys and unsafe paths fail closed. Source roots are relative, portable and cannot contain traversal segments.

## Reproducibility identity

The build witness commits to canonical project metadata, canonical V1.3 module order/import graph, V1.3 semantic/lowering witnesses, and canonical NAIR bytes. Equal canonical inputs produce byte-identical lock/package output independent of absolute host path or source-array order.

## Certification boundary

V1.4 does **not** change the public certified version string. Until a separately governed release boundary changes it, this remains:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

See:

- `docs/NOI_PROJECT_BUILD_PACKAGE_SPEC_1_4.md`
- `research/PROJECT_BUILD_PACKAGE_INTELLIGENCE_1_4.md`
- `docs/PRODUCTION_PROFILE_1_PROGRESS.md`
