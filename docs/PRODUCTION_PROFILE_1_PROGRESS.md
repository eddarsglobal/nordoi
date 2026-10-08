# NORDOI Production Profile 1 Progress

## Certified baseline entering V1.6

V1.5 — Production Diagnostics & Developer Experience is the certified baseline.

Estimated Production Profile 1 completion entering V1.6 = **95%**, remaining = **5%**.

## V1.6 candidate closure target

V1.6 closes the deterministic release-candidate and distribution boundary required before final Production Profile 1 certification.

Candidate proof targets:

- side-effect-free `nordoi release-check <project-root>`;
- schema-versioned `nordoi.release.v1` JSON output;
- exact source -> lock agreement;
- exact source -> package byte agreement;
- canonical V1.4 package validation before distribution;
- SHA-256 package identity;
- SHA-256 canonical build-witness identity;
- deterministic `nordoi.release.provenance.v1` record;
- provenance with no timestamp, host, user, OS or absolute path;
- `nordoi release` publication only after the complete release gate passes;
- byte-identical package copy into `dist/`;
- deterministic `.sha256` and `.provenance` companions;
- no dependency network;
- no runtime filesystem authority;
- no new NAIR minor;
- frozen public version string;
- regression proof across NAIR 0.6/0.12/0.13/0.14 project shapes.

If and only if V1.6 passes the complete local Release Gate, 20/20 focused tests, release-check/publication/drift/reproducibility smoke proofs, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **98% complete / 2% remaining**.

The final **2%** should be reserved for final adversarial qualification, one complete reference application/release rehearsal, documentation/reproducibility closure and the final Production Profile 1 release certification. It should not reopen experimental runtime layering.
