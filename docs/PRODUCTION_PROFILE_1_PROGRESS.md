# NORDOI Production Profile 1 Progress

## Certified baseline entering V1.4

V1.3 — Real Modules & Imports is the certified baseline.

Estimated Production Profile 1 completion entering V1.4 = **89%**, remaining = **11%**.

## V1.4 candidate closure target

V1.4 converts the multi-file language into a reproducible project/build boundary.

Candidate proof targets:

- strict `NORDOI.toml` manifest;
- portable project name/version/source-root validation;
- deterministic source-root module discovery through certified V1.3;
- no new NAIR minor;
- deterministic `.npkg` artifact;
- canonical `NORDOI.lock`;
- byte-identical build output for equal canonical inputs;
- `--locked` rejects source/module/manifest drift;
- package validation/inspection works without source access;
- corrupted or noncanonical packages fail closed;
- no registry or dependency network;
- no build scripts or dynamic loader;
- runtime filesystem authority remains NONE;
- public certified version boundary remains unchanged.

If and only if V1.4 passes the complete local Release Gate, 20/20 focused tests, build/rebuild/locked/package smoke proofs, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **92% complete / 8% remaining**.

The remaining Production Profile 1 work should then concentrate on developer-grade cross-file diagnostics/source maps, capability-based production I/O composition, final hardening/fuzzing and reproducibility checks, reference application coverage, and final release/distribution certification.
