# R0.11 Research Decision — Do not promote test evidence to production

**Baseline:** r0.10 / `16de3cf71b4be39118eb2bb4b963c64ceeb5a65a`. **Decision:** create a `TEST-ONLY` native readiness register and negative policy tests; retain the certified source unchanged.

The preceding 117 witnesses and 18 R0.10 checks are useful for bounded reference-model confidence, but their nature cannot warrant adding new runtime/NAIR/compiler authority. All 12 production requirements remain **UNMET**.

**Rejected alternatives:** (1) automatically authorize native integration when the finite oracle agrees; (2) treat green CI or annotated tags as security certification; (3) label mock governance approval as real authorization; (4) silently edit frozen Rust source or Cargo dependencies. All four would cross the research/production boundary without evidence.

**Allowed next work:** independently design and test a native prototype on a separate governed track, generate concrete evidence, commission review and seek explicit approval. This decision does not implement that track and is **not** itself a production approval.
