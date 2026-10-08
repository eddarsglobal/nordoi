# NORDOI Production Profile 1 Progress

## Certified baseline entering V1.7

V1.6 — Production Release Candidate & Distribution Hardening is the certified baseline.

Certified commit: `ae765c405e8650327ea81a86c550b96160eaef23`.

Certified exact-SHA CI run: `37730271259`.

Certified tag: `v1.6`.

Estimated Production Profile 1 completion entering V1.7 = **98%**, remaining = **2%**.

## V1.7 final closure target

V1.7 is the final planned Production Profile 1 qualification milestone.

Candidate proof targets:

- side-effect-free `nordoi profile1-certify <project-root>`;
- schema-versioned `nordoi.production-profile-1.v1` JSON output;
- exact source -> lock proof;
- exact source -> build-package proof;
- exact build-package -> dist-package proof;
- exact canonical checksum proof;
- exact canonical V1.6 provenance proof;
- final deterministic certification SHA-256;
- no host, timestamp, OS, username or absolute-path contribution to certification identity;
- included `examples/profile1_reference/` application;
- complete reference rehearsal: check -> build -> build --locked -> release-check -> release -> profile1-certify;
- adversarial substitution/drift rejection at every artifact boundary;
- regression proof across existing NAIR 0.6/0.12/0.13/0.14 project shapes;
- zero new runtime/network/filesystem authority;
- zero new NAIR minor;
- frozen public version string;
- exact-SHA multi-platform CI through the existing Ubuntu/macOS/Windows matrix.

If and only if V1.7 passes the complete local Release Gate, 20/20 focused tests, final reference/adversarial smoke proofs, exact-SHA CI and immutable annotated `v1.7` tag, **Production Profile 1 becomes 100% certified / 0% remaining**.

Any subsequent semantic, runtime, capability, target-platform or language expansion belongs to a later production profile and MUST NOT retroactively change the frozen Profile 1 certification evidence.
