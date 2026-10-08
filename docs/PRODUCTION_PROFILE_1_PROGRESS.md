# NORDOI Production Profile 1 Progress

## Certified baseline entering V1.5

V1.4 — Project Build & Package System is the certified baseline.

Estimated Production Profile 1 completion entering V1.5 = **92%**, remaining = **8%**.

## V1.5 candidate closure target

V1.5 closes the developer-grade diagnostic boundary required for practical CI/editor use.

Candidate proof targets:

- stable versioned `NDX` diagnostic code families;
- `nordoi check <project-root>` with zero package/lock publication;
- schema-versioned `--json` output;
- exact source file/line/column when a certified span exists;
- no invented source locations;
- deterministic multi-file import traces;
- dedicated missing-import and import-cycle identities;
- deterministic success JSON independent of source-array order;
- V1.4 project/NAIR semantics reused unchanged;
- no new NAIR minor;
- no runtime filesystem authority;
- no dependency network;
- frozen public version string.

If and only if V1.5 passes the complete local Release Gate, 20/20 focused tests, text/JSON/missing-import/cycle smoke proofs, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **95% complete / 5% remaining**.

The remaining Production Profile 1 work should then concentrate on capability-based production I/O composition, final adversarial hardening/fuzzing, a complete reference application, distribution artifacts and final release-candidate certification.
