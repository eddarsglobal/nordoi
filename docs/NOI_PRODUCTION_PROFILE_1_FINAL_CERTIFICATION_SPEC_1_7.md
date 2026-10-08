# NORDOI Production Profile 1 Final Certification Specification 1.7

## 1. Scope

V1.7 closes Production Profile 1 by certifying the already-existing project/build/release chain. It MUST NOT introduce new language semantics, NAIR instructions, runtime authority, dependency-network access, dynamic module loading or ambient filesystem capability.

## 2. Final command

```text
nordoi profile1-certify <project-root> [--json]
```

The command is verification-only. It MUST NOT create, rebuild, refresh, repair, replace or publish project artifacts.

## 3. Required artifacts

The project MUST already contain:

```text
NORDOI.toml
NORDOI.lock
build/<name>-<version>.npkg
dist/<name>-<version>.npkg
dist/<name>-<version>.npkg.sha256
dist/<name>-<version>.provenance
```

## 4. Certification chain

The command MUST prove, in order:

1. current source graph compiles under the existing V1.4/V1.3/V1.2 pipeline;
2. current canonical lock bytes equal the existing `NORDOI.lock`;
3. current canonical package bytes equal the existing build package;
4. build package passes the V1.4 canonical package validator;
5. distributed package is byte-identical to the build package;
6. checksum text equals the canonical V1.6 checksum text exactly;
7. provenance text equals the canonical V1.6 provenance text exactly;
8. all existing zero-authority invariants remain true.

Any failed proof MUST reject certification.

## 5. Final certificate schema

Machine-readable success uses:

```text
nordoi.production-profile-1.v1
```

Success status is `certified` and commits to:

- project, version and entry identity;
- module/import counts;
- existing NAIR minor and instruction count;
- package filename;
- exact source-lock/build-package/dist-package/checksum/provenance status;
- package SHA-256;
- V1.4 build-witness SHA-256;
- V1.6 provenance SHA-256;
- final certification SHA-256;
- reproducible/platform-neutral/zero-authority invariants.

## 6. Canonical certification identity

The final certification hash MUST be derived only from canonical project/release evidence. It MUST NOT contain:

- timestamps;
- absolute paths;
- hostnames;
- usernames;
- operating-system labels;
- CI run identifiers;
- mutable environment metadata.

This keeps certification identity reproducible across eligible hosts.

## 7. Bounds

- checksum companion: at most 512 bytes;
- canonical final certificate: at most 16 KiB;
- provenance remains bounded by the V1.6 certified provenance bound;
- package reading remains bounded by the existing project package/tool input bounds.

## 8. Fail-closed adversarial behavior

Certification MUST reject:

- source/lock drift;
- source/build-package drift;
- invalid/corrupted build package;
- substituted distributed package;
- checksum modifications, including semantically equivalent but noncanonical text;
- provenance modifications, including added host/time metadata;
- missing required artifacts.

No rejection path may repair or regenerate a missing or invalid artifact.

## 9. Reference application

`examples/profile1_reference/` is the final reference project. It uses:

- a real module import;
- canonical runtime input;
- structured function control;
- existing NAIR 0.14 lowering.

The tooling rehearsal MUST copy it to a temporary location and execute the complete production chain through `profile1-certify`.

## 10. Cross-platform qualification

The final reference rehearsal MUST run through the existing CI test matrix on:

- Ubuntu;
- macOS;
- Windows.

No platform-specific semantic or release output is permitted.

## 11. Frozen boundaries

V1.7 adds no NAIR minor and does not change the public version string:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

## 12. Certification condition

V1.7 may be called certified only after:

- local Release Gate PASS;
- 20/20 focused V1.7 tests PASS;
- complete reference-app rehearsal PASS;
- adversarial smoke proofs PASS;
- exact-SHA GitHub CI PASS on all five required jobs;
- annotated immutable `v1.7` tag.

Only then may Production Profile 1 be declared 100% certified.
