# R0.22 — Witness Fork & Monotonic Replay Isolation Specification

Scope **TEST-ONLY / RESEARCH_ONLY**. Certified baseline `a34bc552b31661570f270c6c75700849b261084c` pinned to annotated tag `r0.21`; the source tree through R0.21 is frozen. No modifications to the runtime or certified files are permitted.

## Data contract

Registry `governance/r22_witness_forks_v1.tsv`: 24 fixed fields, 180 ordered rows, 15 per each of 12 gates. Canonical LF SHA-256 `31a2f2ef5fbadf9d3a026e570db45e9da0c0d990ffafb52a3577bc267b14155f`. The R0.21 frozen binary SHA-256 is `cc08cac8ec0f196563ded1bebbd7c07633c47815a7a0eb3db49064079018c2ba`.

Simulated lanes A and B independently retain `(epoch, root, height, head_sha256, revoked)`; first parent is exactly sixty-four ASCII zeroes. The synthetic witness for an event is:

`SHA256(UTF8("R22-WITNESS|<gate:02>|<lane>|<epoch>|<height>|<parent>|<root>|<action>"))`

This unkeyed hash is only for deterministic trace identity, **NOT** a signature, trusted identity or proof of authenticity.

## Fail-closed predicates

1. Baseline commit and certified R0.21 evidence hash must equal the fixed anchors; mutation => INVALID.
2. Only a mock producer and RESEARCH_ONLY scope are permitted. Any non-NONE signature/verifier, missing CI/review, or native request => BLOCKED.
3. The acting lane must equal the target lane; cross-root updates are BLOCKED.
4. Every successful transition requires `height == prior_height+1`, an exact current parent head and canonical witness hash.
5. Registered lanes increment epochs on advances; no alternate fork at an occupied height may replace the canonical head.
6. Revocation of A is a one-way simulated transition; no A state mutation is allowed thereafter, while B may continue.
7. Every rejected transition leaves *both* lane states unchanged, without creating production authority.
8. Registry row IDs/order, before/after snapshots, decisions, transcript roots and trailing digest must agree in Python and Rust.

Expected histogram: **12 INVALID**, **84 BLOCKED**, **84 RESEARCH_REVIEW_ONLY**. Binary format `NDR22FK1` + LE-u16 count + 32-byte certified R0.21 SHA + 20-byte baseline Git SHA + 180 fixed-size 42-byte records (LE-u16 row ID, eight 1-byte fields, SHA256 row) + 32-byte SHA-256 trailing checksum. Transcript **7654 bytes**, SHA-256 `bfb3dd4b39d4bd6a0d756e8d0dff9b6f25bd94705e2e3883aa90f5863233d1da`.

Windows: the test intentionally synthesizes CRLF from canonical LF, never from an already-CRLF embedded fixture; isolated embedded carriage returns are refused.

## Constitutional boundary

12/12 native gates **UNMET**. 12/12 external verifier attestations **ABSENT**. R0.22 is TEST-ONLY / RESEARCH_ONLY; real signatures, independently authenticated trust roots, native admission and external audit were **NOT** performed. The labels `RESEARCH_REVIEW_ONLY` do not authorize a client, machine or native module.
