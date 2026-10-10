# R0.9 Research Decision — Forked Scope Boundary Conformance

**Certified parent:** `r0.8` / `35fc2487ce180ff1fdc1a0f2fa8df3213127865d`. **Disposition:** TEST-ONLY candidate. No production integration approved.

## Why this increment

R0.6 covered one root. R0.7 covered a root and one child. R0.8 covered a linear three-generation chain. These versions were bounded, and passing their tests cannot establish isolation between **two sibling branches**. R0.9 independently predicts the behavior of a deliberately small fork: root -> two children -> one grandchild per child, at most five scopes.

The new evidence targets negative capability boundaries, root closure with a still-open sibling, cousin resource misuse, independent grants and revocations, cancellation/failure aggregation, quotas and canonical reports. The fixture uses a separately maintained oracle, not a view into R0.2's state. There are 6,400 generated attempts, 16,807 five-step words, and 2,520 valid four-task finish/join orders, each bounded to its declared action set.

## Stop condition

No R0.2 SUT modifications to make a new test pass. Unexpected differences must be investigated as either oracle defects, fixture defects, specification inconsistencies or potential SUT defects; do not silently alter frozen semantics. The package is approved for review only after local Rust compile/test/Clippy and the complete release gate succeed. Only exact-SHA five-job GitHub certification permits an `r0.9` tag.

**Non-claim:** these finite sequential tests do not simulate real OS concurrency or certify security of a live runtime. Kernel K1.18, the compiler, NAIR 0.6 and existing public host authority remain untouched.
