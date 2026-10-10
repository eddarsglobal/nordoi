# R0.10 Research Decision — Consolidate before expanding

**Certified parent:** `r0.9` / `860d09774c881cf23f2a1233877109ed49c21a01`. **Disposition:** TEST-ONLY candidate; no production adoption.

R0.3–R0.9 have added seven distinct kinds of finite model evidence. Creating ever-larger action alphabets would increase numeric coverage without guaranteeing new correctness insights. We choose a cross-version **117-witness traceability audit** and **18 new test-only witness checks**. These test the link between governance records and the previously certified Rust test declarations, the declared finite arithmetic and independent-oracle boundary, and a few direct R0.2 negative-transition smoke cases.

The independent source of truth for historical code identity is the immutable Git tree at `r0.9`, not a generated candidate ZIP. The Python validator refuses source modifications when it has that tree available. Its static-only PASS, if run outside Git, must be labeled accordingly and **must not claim provenance certification**. GitHub CI and local release gates continue to own compilation and behavior evidence.

**Nonclaim:** an in-process fingerprint is not cryptographic; a witness mapping is not a proof of witness validity; repeated tests do not model true parallel scheduling, I/O or security of a running system. An R0.10 tag is prohibited until the exact-SHA CI 5/5 succeeds. No frozen source or kernel API may be changed as a shortcut.
