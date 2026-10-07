# NORDOI V1.4 — Project Build & Package System Specification

## Status

Candidate Production Profile 1 vertical slice. V1.4 is additive over certified V1.3 Real Modules & Imports.

## Project layout

A V1.4 project has one strict manifest at the project root:

```text
project/
├── NORDOI.toml
└── src/
    └── app/
        ├── math.noi
        └── main.noi
```

Minimal manifest:

```toml
[project]
name = "demo"
version = "0.1.0"
entry = "app.main"
source-root = "src"
```

The V1.4 parser intentionally accepts only this `[project]` section and the four required keys. Unknown sections, unknown keys, duplicate keys, missing keys, unsafe names, invalid module identities, absolute source roots, traversal segments, backslashes, and non-portable source-root segments fail closed.

## Build command

```text
nordoi build <project-root> [--locked]
```

The build pipeline is:

```text
NORDOI.toml
  -> canonical manifest
  -> V1.3 static module graph discovery
  -> V1.3 module compile plan
  -> existing V1.2/V1.1/V1.0 semantic lowering
  -> existing canonical NAIR 0.6..0.14
  -> V1.4 build witness
  -> NORDOI.lock + deterministic .npkg package
```

V1.4 adds no NAIR opcode and no NAIR minor.

## Canonical lock

A normal build publishes `NORDOI.lock`. The lock commits to:

- canonical project identity;
- version;
- entry module;
- source-root identity;
- canonical reachable module order;
- canonical import edges;
- resulting NAIR minor;
- V1.4 build witness.

The lock is generated text and is intended to be committed with the project.

`--locked` requires the existing lock to match the newly derived canonical lock exactly. Drift in source semantics, module graph, imports, manifest metadata, or resulting lowering therefore fails closed before package publication.

## Package

Build output:

```text
build/<name>-<version>.npkg
```

Package format V1.0 uses magic `NPKG` and contains canonical project metadata, canonical module graph metadata, the V1.4 build witness, and canonical NAIR bytes.

The package does not contain runtime filesystem resolution instructions. Module imports have already been erased before NAIR.

`nordoi package-info <package.npkg>` validates the package framing and embedded canonical NAIR, then reports package metadata without access to source files.

## Determinism

Equal canonical manifest + equal reachable source semantics must produce byte-identical:

- V1.4 build witness;
- `NORDOI.lock`;
- `.npkg` artifact.

Input source-array order and project absolute path are not build identity.

## Authority and dependency law

V1.4 has no registry, package download, dependency resolver network, post-build script, plugin execution, dynamic loader, or runtime filesystem authority.

```text
dependency-network=NONE
runtime-fs=NONE
authority=NONE
```

Compiler/tooling filesystem reads are explicit host-side inputs and are complete before the package/runtime boundary.

## Certified bounds

- manifest bytes: 16 KiB maximum;
- package bytes: 4 MiB maximum;
- project name: 64 bytes maximum;
- project version: 64 bytes maximum;
- source-root: 256 bytes maximum;
- module/import graph bounds are inherited unchanged from V1.3.

## Compatibility

V1.4 preserves all prior certified NAIR boundaries:

- static project -> 0.6;
- simple dynamic call -> 0.12;
- acyclic nested call graph -> 0.13;
- structured runtime function control -> 0.14.

The public version boundary remains frozen until an explicit separately certified public release boundary changes it.
