# Research decision — R0.16

Baseline: `r0.15` / `3158ee20a894461df5402335b7db5509db9abd02`. `TEST-ONLY`, `RESEARCH_ONLY` — **NOT** production.

Decision: use deterministic checked-in binary input for offline parsing and SHA-256 consistency exercises rather than introducing a live external trust assertion. The manifest contains 12 mock fixtures, not 12 independent proofs. R0.11 readiness remains 12/12 `UNMET`; R0.12–R0.15 admissions remain `ABSENT`, as do all 12 R0.16 real evidence admissions.

The SHA-256 digest `f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393` is a static public reference. It cannot establish creator identity, independent verification or signature authenticity, and never authorizes native execution.

**Next boundary:** real external evidence may be introduced only with a separate explicit authorization and verified identity/policy design; this candidate does not connect any production module.
