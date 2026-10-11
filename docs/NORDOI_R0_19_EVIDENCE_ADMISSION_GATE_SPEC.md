# R0.19 — Admission Decision Boundary (Research Only)

## Trust model

Certified additive parent: `r0.18` `2839c377e78e3c0e58693bc5aa8e668d8e0e436f`. Every public fixture is controlled synthetic data. None proves external identity; no signature verification API is invoked. The R0.11 native gates remain 12/12 **UNMET**, while evidence and verifier status remain 12/12 **ABSENT**. **TEST-ONLY / RESEARCH_ONLY**.

## Canonical attempt record

Exactly 96 rows, sorted by gate `R11-G01` through `R11-G12` and scenario ordering: CLEAN_MOCK, TAMPER_ROOT, FORGED_SIGNATURE, FORGED_VERIFIER, REVOKED_ROOT, MISSING_CI, MISSING_REVIEW, NATIVE_ESCALATION. Input bytes remain the certified R0.16 bundle, R0.17 replay, R0.18 fault transcript. Explicit baseline `2839c377e78e3c0e58693bc5aa8e668d8e0e436f`.

## State precedence

1. Incorrect fixed SHA-256 roots or structurally malformed candidate -> `INVALID` (not an authorization failure).
2. Any purported signature/verifier, revocation flag, unsafe scope, failed mock CI/review or native escalation -> `BLOCKED`. These are local policy simulations, **NOT** real cryptographic verification.
3. Exact public fixture roots and mock CI+review, with no provenance claims and no native authority -> `RESEARCH_REVIEW_ONLY`. This is not evidence admission.

No state such as ADMITTED, APPROVED or AUTHORIZED is expressible in this test-only evaluator. Authentic external attestations and real revocation validation are outside its contract.

## Binary transcript

Magic `NDR19AD1` (8 bytes), u16 LE row count 96, then 96 records of u16 LE ordinal + u8 scenario + u8 decision + SHA-256 of canonical TSV row including LF, and final SHA-256 prefix checksum. Length 3498 bytes, root `ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33`. Registry SHA-256 `7e7f3eccc8e06f6acc0bbb811ddd82b7813654b2b7a676764a5341a81643a3e2`. The Python hashlib reference is reconstructed independently in Rust using the certified SHA implementation.

## Compatibility and limitations

Normalize only CRLF line endings before text hashing; isolated CR remains invalid. Existing certified code, Rust kernel, NAIR, compiler, and every prior source file remain untouched. No external verification, real evidence admission or native authorization is performed. All outputs **RESEARCH_ONLY / TEST-ONLY**; real production readiness remains **NOT** established; **ABSENT** remains the factual evidence state.
