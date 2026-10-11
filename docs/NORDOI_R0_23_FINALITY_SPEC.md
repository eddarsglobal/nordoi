# R0.23 — Research Finality and Equivocation Contract

Certified frozen baseline `r0.22`: `9d0bbdbb5153e5d38ff0f387ee381ad806c790d9`. Direct parent transcript: `bfb3dd4b39d4bd6a0d756e8d0dff9b6f25bd94705e2e3883aa90f5863233d1da`.

## Scope and semantics

- Each of 12 research domains maintains distinct A and B lanes, with epoch, root, monotonic height, canonical head, revocation, finalized height and immutable finalized head.
- A successful `FINALIZE_A` or `FINALIZE_B` appends a *synthetic* new witness and freezes `(finalized_height, finalized_head)`; later valid descendants may extend the head but may not rewrite the sealed value.
- Any same-height contradictory witness, ancestor replay, forged parent, cross-root transplantation, unexpected producer, forged verifier, native authority escalation or conflicting history is blocked; malformed certified-parent lineage is invalid.
- Revoking A is a separate event; it does not replace A's canonical head or sealed head. B can progress without altering A after revocation.
- The 18 canonical scenarios run in order on each of 12 independent gate groups. Each group returns 1 INVALID, 8 BLOCKED, 9 RESEARCH_REVIEW_ONLY.
- LF is canonical. CRLF is normalized. Bare carriage returns are rejected. Binary format `NDR23FN1` + count LE16 + parent transcript SHA256 + parent commit bytes + 216 packed records + trailing SHA256 digest.
- Canonical TSV policy SHA-256: `6da4eb212e7a8e8ed1c297808ad6c7beaae14e6f2cb665b12c0d5d9b6c56dba8`; transcript SHA-256: `26fd97b314f458458b4f89cbdde5b55d41403c5dc5ea89c061a057b5ecf494b7`.
- This is controlled by a local mock producer. A local SHA256 digest is **not** authentication. No independent producer or external attester is present.

## Production exit criteria — NOT SATISFIED

Production requires engineering and independent evidence *outside these simulations*, including: (1) a concrete supported threat model and production runtime boundary; (2) real trusted identities and cryptographic signatures with independently implemented verification, key provisioning and revocation; (3) explicit integration into native authority checks that currently remain **12/12 UNMET**; (4) independent validation evidence for all currently **12/12 ABSENT** verification gates; (5) durable crash-safe audit/recovery and rollback protections, across operating systems; (6) adversarial fuzzing, supply-chain reproducibility and external security review; (7) deployment and incident response procedures; (8) precise claims with no implication of absolute unhackability.

**R0.23 cannot grant production admission, and no automatic promotion to production may occur from synthetic successes.** Do not reinterpret `RESEARCH_REVIEW_ONLY` as authorization. TEST-ONLY / RESEARCH_ONLY.
