# NORDOI Production Release Candidate Intelligence 1.6

## Decision

Production release hardening should verify an existing deterministic build rather than silently create a new trusted artifact during the release step.

That decision keeps authority explicit: V1.4 owns canonical project construction, V1.5 owns developer diagnostics, and V1.6 owns release verification and publication.

## Why exact package equality matters

A package can be syntactically valid while still being the wrong valid package. Therefore package decoding alone is insufficient. V1.6 compares the existing package bytes to the bytes recalculated from the current source graph. This detects both corruption and substitution.

## Why the lock is independently required

The lock is a human- and CI-auditable statement of the build graph. Requiring exact current lock equality catches drift before distribution and prevents the release command from silently refreshing the build state.

## Why provenance excludes timestamps

Timestamps are useful operational metadata but destructive to reproducibility. NORDOI release provenance is semantic build provenance, not deployment telemetry. Time, host and path metadata can be recorded externally without contaminating the canonical release identity.

## SHA-256 scope

SHA-256 binds bytes to an identifier. It does not by itself identify the publisher. V1.6 deliberately avoids conflating checksums with signatures. A future signing layer can attest the canonical V1.6 provenance without changing package semantics.

## Minimal distribution set

The smallest practical deterministic distribution set is:

1. the exact `.npkg`;
2. a conventional SHA-256 checksum file;
3. a canonical provenance record.

This is intentionally smaller and easier to audit than introducing archives, registries, installers or network package managers before the language reaches its first production profile.

## Production profile effect

If V1.6 passes the complete certification gate, NORDOI should be considered a release-candidate-grade language/tooling slice, not yet a claim that every long-term target backend or effect domain is complete. The remaining production work should be limited to final reference-application/adversarial qualification and final Production Profile 1 release certification.
