# R0.15 Research Decision — Verification ≠ Permission

The baseline `r0.14` / `86fc90a79f5d0cef83c730bcd7c02175307fffd3` is immutable.

- **Decision:** TEST-ONLY, RESEARCH_ONLY. External independent evidence verification: **12/12 ABSENT**.
- **Native readiness:** 12/12 `UNMET`. R0.12 evidence / R0.13 integrity / R0.14 real authentication: 12/12 `ABSENT` each.
- **Witnesses:** 39/39 synthetic policy tests; 2 inherited FIPS vectors; 41 Rust tests expected.
- A SHA-256 match and a double synthetic approval **do NOT** authenticate any individual or constitute external verification.
- The only permissive experimental decision is `ResearchReviewOnly`; it is **NOT** a native runtime, compiler, NAIR, or production deployment authorization.
- No manifests, historical source, or certified files are modified. A future release must create a separately governed and authenticated evidence path.
