# NORDOI V1.6 — Production Release Candidate & Distribution Hardening

V1.6 is additive over certified V1.5. It turns the deterministic V1.4 project package and V1.5 diagnostics into a fail-closed release-candidate workflow without changing kernel semantics, NAIR, runtime authority or the frozen public version boundary.

## Release candidate verification

A release candidate must already have a canonical `NORDOI.lock` and deterministic `.npkg` produced by `nordoi build`.

```bash
cargo run --quiet --bin nordoi -- release-check demo
```

`release-check` recalculates the project from source and proves all of the following before returning PASS:

- current sources reproduce the exact `NORDOI.lock`;
- current sources reproduce the exact existing `.npkg` bytes;
- the package decodes canonically through the V1.4 package validator;
- package metadata, module order, import edges, build witness and NAIR metadata match the current canonical build;
- a deterministic SHA-256 identifies the exact package;
- a deterministic, platform-neutral provenance record can be regenerated exactly.

A successful human-readable result contains:

```text
release-check project="demo" version="0.1.0" ... lock=EXACT package-match=EXACT ... status=PASS reproducible=true platform-neutral=true dependency-network=NONE runtime-fs=NONE authority=NONE
```

Machine-readable mode uses schema `nordoi.release.v1`:

```bash
cargo run --quiet --bin nordoi -- release-check demo --json
```

## Distribution publication

Only a verified candidate may be published:

```bash
cargo run --quiet --bin nordoi -- release demo
```

V1.6 writes exactly three deterministic files under `dist/`:

```text
dist/demo-0.1.0.npkg
dist/demo-0.1.0.npkg.sha256
dist/demo-0.1.0.provenance
```

The distributed `.npkg` is byte-identical to the verified build artifact. The checksum file is canonical SHA-256 text. The provenance record uses schema `nordoi.release.provenance.v1`.

## Reproducible provenance

The provenance deliberately contains no timestamp, absolute filesystem path, hostname, username or operating-system name. Those values would make otherwise identical builds produce different release metadata.

It commits to:

- project and version identity;
- entry module;
- package filename and package SHA-256;
- canonical V1.4 build-witness SHA-256;
- package format;
- existing NAIR minor and instruction count;
- module and import counts;
- `reproducible = true`;
- `platform-neutral = true`;
- `dependency-network = "NONE"`;
- `runtime-fs = "NONE"`;
- `authority = "NONE"`.

## Fail-closed drift handling

V1.6 never silently rebuilds, refreshes a lock, or replaces an invalid package during release verification. Any source/lock/package divergence must be resolved through the explicit V1.4 build workflow first.

Typical workflow:

```bash
nordoi check demo
nordoi build demo
nordoi build demo --locked
nordoi release-check demo --json
nordoi release demo
```

## Zero language/runtime change

V1.6 is release tooling only. Existing certified lowering remains exact:

- static project -> NAIR 0.6;
- simple dynamic imported call -> NAIR 0.12;
- transitive acyclic runtime call graph -> NAIR 0.13;
- structured runtime function control -> NAIR 0.14.

No new NAIR minor, runtime filesystem grant, network dependency resolver, dynamic loader or host authority is introduced.

## Certification boundary

V1.6 does **not** change the frozen public version string:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

See:

- `docs/NOI_PRODUCTION_RELEASE_CANDIDATE_SPEC_1_6.md`
- `research/PRODUCTION_RELEASE_CANDIDATE_INTELLIGENCE_1_6.md`
- `docs/PRODUCTION_PROFILE_1_PROGRESS.md`
