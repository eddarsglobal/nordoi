# Production Profile 1 Final Certification Intelligence 1.7

## Decision

The final production milestone should not grow the language. The remaining risk after V1.6 is evidence fragmentation: source validation, deterministic build, release provenance and distribution integrity are individually proven but still need one fail-closed end-to-end qualification boundary.

V1.7 therefore adds a verifier, not an executor.

## Why no new package format

V1.6 already defines deterministic `.npkg`, checksum and provenance artifacts. Introducing a new archive or installer at the final milestone would create another format requiring its own stability cycle. V1.7 instead certifies the exact V1.6 artifacts.

## Why certification is side-effect free

A command that repairs lock/package/distribution drift while claiming to verify it collapses the distinction between evidence and mutation. `profile1-certify` must observe only. Explicit V1.4/V1.6 commands remain the only build/publication paths.

## Final certification hash

The final SHA-256 is a compact commitment to canonical release evidence, including package, build witness and provenance hashes plus exactness flags. Host/time metadata is intentionally excluded so identical evidence produces identical certification identity on every supported platform.

## Reference application strategy

The reference app uses the highest currently certified vertical needed for a meaningful real program—module import + runtime input + structured runtime control—without inventing a new semantic feature. The same project is rehearsed on the existing CI operating-system matrix through integration tests.

## Adversarial closure

The remaining qualification concentrates on substitution and drift attacks at every seam:

- source vs lock;
- source vs build package;
- build package vs dist package;
- dist package vs checksum;
- canonical release vs provenance.

Every seam is exact-byte or cryptographic-identity checked and fails closed.

## Production Profile 1 completion

If V1.7 passes local gates, focused tests, reference rehearsal, adversarial proofs, exact-SHA CI and immutable tag certification, Production Profile 1 has no remaining planned closure work and may be recorded as 100% certified. Future work belongs to a new profile/version rather than extending Profile 1 indefinitely.
