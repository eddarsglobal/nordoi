# NORDOI V1.7 — Production Profile 1 Final Certification

V1.7 is the final additive qualification layer over certified V1.6. It does not add language syntax, kernel semantics, NAIR instructions, runtime authority, dependency resolution or ambient I/O. Its job is to prove that one already-built and already-released project remains exact across the complete production chain.

## Final certification command

After the V1.6 workflow has created a canonical lock, build package and verified distribution, run:

```bash
cargo run --quiet --bin nordoi -- profile1-certify demo
```

`profile1-certify` recalculates the current project from source and requires all of the following to agree exactly:

- current source graph -> canonical `NORDOI.lock`;
- current source graph -> canonical build `.npkg`;
- build `.npkg` -> distributed `.npkg` byte identity;
- canonical `.sha256` companion -> exact package SHA-256;
- canonical `.provenance` companion -> exact V1.6 provenance;
- package metadata, module/import graph, build witness and NAIR metadata -> current build;
- zero dependency network, zero runtime filesystem authority and zero ambient authority.

A successful text result contains:

```text
profile1-certify project="demo" ... source-lock=EXACT build-package=EXACT dist-package=EXACT checksum=EXACT provenance=EXACT ... status=CERTIFIED profile="Production Profile 1" reproducible=true platform-neutral=true dependency-network=NONE runtime-fs=NONE authority=NONE
```

Machine-readable mode uses schema `nordoi.production-profile-1.v1`:

```bash
cargo run --quiet --bin nordoi -- profile1-certify demo --json
```

The JSON certificate includes the package SHA-256, build-witness SHA-256, provenance SHA-256 and a final deterministic `certificationSha256` for the complete Production Profile 1 proof.

## Reference application and cross-platform rehearsal

The repository includes:

```text
examples/profile1_reference/
├── NORDOI.toml
└── src/app/
    ├── main.noi
    └── rules.noi
```

The reference application exercises a real imported module, canonical input and structured runtime function control through the existing NAIR 0.14 boundary.

The V1.7 tooling tests copy this application into an isolated temporary project and execute the entire production rehearsal:

```text
check
  -> build
  -> build --locked
  -> release-check
  -> release
  -> profile1-certify
```

Because these tooling tests run under the existing GitHub matrix, the same rehearsal is exercised on Ubuntu, macOS and Windows without adding provider-specific release logic.

## Adversarial final qualification

V1.7 fails closed when any production artifact is substituted or drifts. Focused tests cover:

- stale `NORDOI.lock` after source modification;
- corrupted or substituted build package;
- substituted distributed package;
- forged checksum companion;
- modified provenance companion;
- oversized checksum/provenance metadata;
- source order determinism;
- project metadata identity;
- exact preservation of existing NAIR 0.6, 0.12, 0.13 and 0.14 project shapes.

`profile1-certify` never repairs, rebuilds or republishes an artifact. Certification is verification-only.

## Canonical final certificate

The internal canonical certificate uses schema:

```text
nordoi.production-profile-1.v1
```

It deliberately contains no timestamp, username, hostname, operating-system name or absolute path. Therefore identical certified inputs produce an identical final certification hash.

## Frozen public boundary

V1.7 does **not** change the certified public version string:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

V1.7 is final production qualification around the existing certified semantics, not a semantic version promotion.

See:

- `docs/NOI_PRODUCTION_PROFILE_1_FINAL_CERTIFICATION_SPEC_1_7.md`
- `research/PRODUCTION_PROFILE_1_FINAL_CERTIFICATION_INTELLIGENCE_1_7.md`
- `docs/PRODUCTION_PROFILE_1_PROGRESS.md`
- `examples/profile1_reference/`
