# NOI Production Release Candidate & Distribution Hardening Specification 1.6

## Status

Candidate specification for NORDOI V1.6. V1.5 remains the certified baseline until the complete V1.6 release gate, focused tests, smoke proofs, exact-SHA CI and annotated tag succeed.

## Purpose

V1.6 defines the first bounded release-candidate boundary for real NORDOI projects. It does not expand NOI syntax or runtime semantics. It proves that an already-built project can be distributed reproducibly without hidden rebuilds, implicit dependency resolution, runtime filesystem authority or host-specific provenance.

## Commands

### `nordoi release-check <project-root> [--json]`

The command is side-effect free. It MUST:

1. parse the canonical V1.4 project manifest;
2. discover the reachable V1.3 module graph;
3. compile the current sources through the existing V1.4 project build;
4. require an existing `NORDOI.lock`;
5. require exact textual equality between that lock and the recalculated canonical lock;
6. require the existing `build/<name>-<version>.npkg`;
7. validate that package canonically through the V1.4 package decoder;
8. require byte-for-byte equality between the existing package and the recalculated package;
9. require decoded package metadata to agree with the current canonical build;
10. compute deterministic package and witness SHA-256 identities;
11. derive canonical V1.6 provenance;
12. publish PASS only after all checks succeed.

A failed check MUST NOT update source files, the lock, the build package or `dist/`.

### `nordoi release <project-root> [--json]`

The command MUST first satisfy the entire `release-check` boundary. Only then may it atomically publish:

- `dist/<name>-<version>.npkg`;
- `dist/<name>-<version>.npkg.sha256`;
- `dist/<name>-<version>.provenance`.

The distributed package MUST be byte-identical to the already verified build package.

## Schemas

Release report schema:

```text
nordoi.release.v1
```

Provenance schema:

```text
nordoi.release.provenance.v1
```

## Canonical provenance

Provenance MUST be deterministic and platform-neutral. It MUST NOT contain wall-clock time, hostname, username, absolute project path or operating-system identity.

It commits to project identity, version, entry, package filename, package SHA-256, build-witness SHA-256, package format, existing NAIR minor/instruction count, module/import counts and the explicit zero-authority properties of this release boundary.

## Cryptographic scope

V1.6 uses SHA-256 as an identity/checksum primitive. This is integrity/provenance metadata, not a claim of publisher authenticity. Signing and external trust roots remain outside this milestone unless explicitly introduced by a later governed specification.

## Security properties

- fail closed on lock drift;
- fail closed on package corruption;
- fail closed on valid-but-different package substitution;
- no network dependency resolution;
- no dynamic import;
- no runtime filesystem grant;
- no host authority serialized into the package or provenance;
- no implicit rebuild during release verification;
- atomic publication of individual distribution files.

## Compatibility

V1.6 introduces no new NAIR format. Existing programs continue to lower through the already certified 0.6, 0.12, 0.13 and 0.14 paths.

The public version string remains:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
