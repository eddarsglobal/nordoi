# R0.16 TEST-ONLY Offline Bundle Specification

Baseline `r0.15` / `3158ee20a894461df5402335b7db5509db9abd02`. Eight additive paths. The previous five research phases remain frozen.

## Wire encoding and bounds

- Magic: `NDR16PK1` (8 ASCII bytes).
- Count: little-endian unsigned 16-bit integer, exactly 12.
- Then 12 segments, each encoded as a little-endian unsigned 16-bit length followed by raw payload octets.
- Each payload length is 1–256 bytes. No trailing or missing bytes are allowed.
- The manifest requires exactly 12 ordered rows. Each row binds R0.11 gate, R0.12 evidence, R0.13 integrity, R0.14 authentication and R0.15 verifier IDs plus a fixed `r0.15` SHA, byte offset, byte length and SHA-256.
- Canonical root fixture digest: `f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393`; a modified binary is rejected, even if an untrusted manifest digest is rewritten.
- Every row's **admitted** evidence status remains `ABSENT`; the payload is `TEST-ONLY` public synthetic data.

## Decision and threat boundary

- Malformed or corrupt input: `Invalid`.
- Valid byte fixture without synthetic CI or synthetic reviewer approval: `Blocked`.
- Valid mock fixture with both synthetic flags: `ResearchReviewOnly`.
- Real third-party authenticity, revocation checking, independent review and trust roots are **NOT** implemented. No real signature, network dependency, native task authority, production scheduler or deployment permission.
- SHA-256 equality means only matching bytes under an anchored test digest; it does **NOT** establish authorship or semantic correctness.

## Planned tests

44 explicit Rust integration tests with 44 one-to-one witness rows, plus the two inherited SHA-256 FIPS tests. The release gate and cross-platform GitHub CI remain to be run on the user's machine.
