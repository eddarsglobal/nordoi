# NORDOI R0.21 — Epoch Replay & Cross-Root Isolation Specification

**TEST-ONLY / RESEARCH_ONLY — NOT CERTIFIED.** Frozen base `r0.20`, commit `bd54ef97523fc9f415ce0d6eaa41830b7105b7bf`. Nothing in this specification grants trust or native authority.

## Scope and lineages

Each of 12 independent research-only areas maintains two unrelated mock trust-root lineages, A and B, each storing epoch (u8), synthetic identity string and revoked flag. Both start at epoch 0, identity `NONE`, unrevoked. Synthetic IDs are scoped to one area and lane, e.g. `MOCK-G01-A-E1`. Neither these strings nor a hash are a cryptographic identity attestation.

The 13 scenarios execute in the fixed order: REGISTER_A; REGISTER_B; ROTATE_A; CROSS_ROOT_WRITE; STALE_A; ROTATE_B; EPOCH_COLLISION; REVOKE_A; RESURRECT_A; REPLAY_A; B_ROTATES_AFTER_A_REVOKED; TAMPER_LINEAGE; NATIVE_ESCALATION. A and B each register at epoch 1 and rotate independently. A revokes irreversibly at epoch 2. B is allowed to rotate to epoch 3 *within the research model only* after A is revoked. No action can reset A to unrevoked. The denied scenarios preserve both lane states, even where their claims have a superficially valid epoch or recomputed checksum.

## Fail-closed decision ordering

1. Reject mismatched certified baseline SHA, R0.20 fixture SHA, and malformed/overflowing claimed epochs as `INVALID`.
2. Block any non-mock producer, unapproved scope, non-`NONE` signature/verifier, missing synthetic CI/review, or `native != NO` as `BLOCKED`.
3. Block any actor/target mismatch, cross-root write, unknown target or revoked target.
4. Admit only six specific mock lifecycle transitions per domain to `RESEARCH_REVIEW_ONLY`: register A, register B, rotate A, rotate B, revoke A, rotate B after A was revoked.
5. All remaining scenarios produce `BLOCKED`, with no simulated state mutation. No path yields native authorization or `PRODUCTION_ADMITTED`.

This is 156 cases: 12 INVALID, 72 BLOCKED and 72 RESEARCH_REVIEW_ONLY. Outcome distribution reflects deliberately constructed fixtures, not a real-world acceptance or attack-success rate.

## Canonical binary replay

Registry: `governance/r21_cross_root_epochs_v1.tsv`, 21 tab-separated columns, normalized CRLF/LF with embedded isolated CR forbidden. SHA-256 `179b9e332d9562d90df7bdaec8c46297b9a467cfd04b728c47750b0aec7a9a57`.

Transcript `research/fixtures/r21_cross_root_transcript_v1.bin`:

- Eight ASCII bytes `NDR21XC1`; two-byte little-endian record count `156`.
- SHA-256 of frozen R0.20 transcript (32 bytes): `2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817`.
- Exact certified `r0.20` commit bytes (20 bytes).
- 156 fixed 42-byte records: u16 LE index, u8 scenario ordinal, u8 decision code (0/1/2), u8 before/after epochs of A and B, u8 post-transition revoked flag for A and B, SHA-256 of normalized canonical TSV row including trailing LF.
- Final SHA-256 of all preceding transcript bytes (32 bytes). Total size: **6646 bytes**. Golden transcript SHA-256: `cc08cac8ec0f196563ded1bebbd7c07633c47815a7a0eb3db49064079018c2ba`.

Python and Rust independently reconstruct the exact bytes. Rust has 48 direct test witnesses + 2 inherited SHA-256 vectors (50 expected tests), including a Windows CRLF roundtrip and rejection of embedded CR. All six accepted synthetic transitions remain `RESEARCH_REVIEW_ONLY`. A checksum is not a signature, and the fixture does NOT prove cryptographic trust.

## Immutable certification boundary

The static validator runs historical R0.20 static contracts, rechecks the R0.20 fixture, 12/12 native `UNMET`, 12/12 independent verifiers `ABSENT`, and Git tag/working-tree constraints. Exactly eight R0.21 files may be additive. No historical certified file is rewritten. Fail-close on unknown changes. Require full local Release Gate and exact-commit CI 5/5 (including Windows) before tagging `r0.21`. Real signatures, independent external verifier identities, public trust-anchor distribution, and native execution authority are NOT implemented here.
