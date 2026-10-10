# NORDOI R0.13 — Evidence Integrity & Trust Chain Specification

- Frozen baseline `r0.12`, SHA `851d978c36f4c12fc16c115e89a7774e33ce361b`; **TEST-ONLY**; no changes to frozen source or production interfaces.
- Research `governance/r13_integrity_chain_v1.tsv` contains 12 canonical `ABSENT` entries. Their native readiness gates stay `UNMET`, and R0.12 evidence entries remain `ABSENT`, all in `RESEARCH_ONLY` scope.
- The 34 direct test witnesses and two inherited FIPS tests exercise **SHA-256** byte integrity. A bounded synthetic chain carries domain-separated, length-prefixed transcript commitments for item/gate/evidence/area/baseline/status/scope/payload hash/previous link/producer/reviewer/signature/trust root.
- The chain is sequential (1–12), verifies per-artifact digest and previous commitment, checks duplicate IDs and duplicate payload digests, and fails closed for mutations. Synthetic revocation blocks an otherwise reviewable proposal. An unknown/duplicate revocation fails closed.
- Mock producer/reviewer strings are **NOT** authenticated identities or independent reviews. No signature verifier or production trust root is supplied: only `NONE` accepted. Digital signatures, trust store, key rotation/revocation and authentic real-world attestation are deferred.
- A complete 12-item synthetic chain, mock CI and mock approval can yield only `RESEARCH_REVIEW_ONLY`; otherwise `Blocked` or `Invalid`. No native authorization decision type exists.
- Two existing SHA-256 FIPS vector tests are imported via an unchanged `#[path]` module, so **36 executed tests** are expected: 34 R0.13 tests + 2 certified hash tests. This tests the integrity algorithm only, not artifacts in the absence registry.
- No OS threads, scheduling, network activity, dynamic imports, unsafe blocks, runtime authority, compiler work or new NAIR instructions.
- The R0.13 validator verifies seven additive paths and certified Git boundary against r0.12, not cryptographic source authenticity. Certify only after Mac Release Gate PASS and 5/5 GitHub jobs on the exact SHA.
