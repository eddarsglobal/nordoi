# R0.18 Differential Fault Injection — TEST-ONLY Specification

Frozen input `r0.17` `c5f76bc2e393f95f38552778f870fac4be8206c3`. **RESEARCH_ONLY**; all twelve native readiness gates remain `UNMET`; external real-world evidence and third-party verifiers remain `ABSENT`.

## Fault grammar

60 individual mutations against fresh copies of existing public fixtures: 39 packet cases, 15 transcript cases, 3 manifest cases, 3 registry cases. `XOR` applies `byte[position] ^= argument`; `ZERO` sets the byte to zero; `CUT` keeps bytes before the position; `ADD` appends a single byte `argument`; `SWAP` exchanges bytes at `position` and `argument`. Position and argument are canonical decimal, bounded and checked.

A: independently walks length-prefixed R0.16 packet. B: independently indexes R0.16 offsets from the normalized and pinned manifest. Outcomes `PASS`/`REJECT` describe **structural readability, not trustworthy admission**. A mutation of the manifest can cause A to pass and B to reject; divergence is recorded rather than silently reconciled. All mutations must produce `INVALID` at the pinned content gate, even after recalculating each mutated payload digest.

## Cross-runtime result transcript: 2,307 bytes

- 9-byte magic `NDR18FLT1`
- 40 ASCII bytes of frozen R0.17 commit `c5f76bc2e393f95f38552778f870fac4be8206c3`
- 2-byte LE count `60`
- 32 raw SHA-256 bytes of R0.16 pack root `f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393`
- 32 raw SHA-256 bytes of R0.17 transcript root `1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab`
- 60 records × 36 bytes (`index: u16le`, `domain: u8`, `structural-result mask: u8`, `SHA-256(mutated object): [u8; 32]`)
- 32 bytes of `SHA-256(prefix)` trailer

The entire transcript is independently anchored at `9512d8f24a84bccdbd87ade72ea8c8ab18f9873864a859a11a3abd5612f86293`. Any self-reseal remains rejected by immutable expected constants. Python `hashlib` recomputes it independently of the Rust frozen SHA-256 implementation; no signatures or externally authenticated trust roots are inferred.

## Cross-platform normalization

TSV inputs are normalized by replacing only valid CRLF delimiters with LF before hashing or mutation. Embedded CR, invalid UTF-8 and corrupted metadata remain disallowed. Both POSIX and Windows checkouts must produce identical canonical results; the new R0.18 source uses no Windows-only paths, network, host permissions or runtime API.

**No production admission**, executable code authorization, native installation, security guarantee, verified external evidence or real independent audit. The only non-invalid state for unchanged fixtures is `ResearchReviewOnly`, conditional on synthetic CI and reviewer flags.
