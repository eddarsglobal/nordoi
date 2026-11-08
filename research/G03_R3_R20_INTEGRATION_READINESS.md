# NORDOI G0.3 R3-R20 — Governed integration-readiness review

**Research-only validation. No merge, no production, no new public syntax.**

## Frozen sources and exact scope

Baseline main SHA: 98da03b2ea1efcf665d205189284aa8b66fda5d3.
Candidate PR #40 SHA: a1ae15326eaa837f8994a75fa7cfbe2eccc44976.
Baseline kernel SHA256: dcb0689dc86cd2646a5f4ffb3c8b0c5eaa72c59e0cdea1307aeba2e86f8de7e2.
Candidate kernel SHA256: 35194549eee9d6798de2c9afb091a668c2d491a40c548693ee6e87a3aa29aba0.
External Rust security witness Git blob: 4080350aab5c3b2f377f7c22bca54d49c50e6df7.

Only source modification: removal of a redundant ensure_exists lookup inside private AtomicKernel::apply_single. All other shared tracked files must have identical bytes, and exactly five candidate-only paths are allowed. No delete or other source delta permitted.

## Historical evidence — not rerun at R3-R20

Mac Intel R3-R16 FIX1 (balanced AA/BB/AB positions): noop_set A/B +23.1812% (12/12 positive), changed_set_flush +3.9061% (10/12), denied_capability A/B -1.3758% (2/12 faster), ~0.17 ns/op slower. Four security probes passed. R3-R17 FIX1: require 41/41 and no-inline witness 47/47 normalized assembly instructions match, two default-deny/grant witnesses passed. R3-R18/R3-R19: CapabilitySet::contains has 94/94 visible instructions; remaining text difference is a relocated call to the same memcmp stub. This does not explain timing causally.

**Explicit unresolved performance exception:** DENIED_CAPABILITY_PERFORMANCE_EXCEPTION=OPEN. This is a real observed microbenchmark cost, not a security violation, and remains a risk to weigh if eventually deciding to merge. The prior samples do not establish universal performance effects on all machines.

## Actual R3-R20 integration gate — both pinned variants, isolated

1. Immutable Git branch SHA and kernel SHA256 plus byte-for-byte exact only-one-lookup source delta verification; common tracked files identical and five candidate-only tracked files allowed.
2. Run cargo fmt --all -- --check.
3. Run cargo test --offline --release --all-targets; parse and verify every test harness reports zero failures and retain passed counts and stdout SHA256.
4. Run cargo clippy --offline --all-targets -- -D warnings.
5. Build offline release Rust library and link an independent pinned external security program. Real tested invariants: capability deny by default, explicit allow, fresh-instance deny, unknown atom rejection, ownership denial, no-op zero work, transaction conflict leaves state atomic. Require exact stdout PASS contract.
6. Report toolchain and operating system, exact source identities, tests and witness verification status. When requested, create a brand-new immutable private 0600 JSON receipt and refuse to overwrite any existing path.

The Python synthetic selftest deliberately checks invalid patches and added tracked files are rejected, and validates private output protection. The synthetic selftest is not a substitute for Rust tests and is not a benchmark.

GitHub CI performs the real two-revision gate on Ubuntu, macOS and Windows. The CI does not claim owner-Mac performance or language-wide superiority.

## Decision rules

All actual checks pass => READY_FOR_GOVERNED_MERGE_DECISION_WITH_PERFORMANCE_EXCEPTION, **not merger approval**. Tests fail or frozen pin drifts => BLOCK. Historical -1.3758% denied_capability must be explicitly disclosed during any separate merge decision. The owner has not authorized merging PR #40, updating main or deployment. No outside human evaluator, volunteer/signature or human-study gate is required.

No changes to public NORDOI grammar, parser, NAIR, scheduler, security policy or runtime. Preserve the constitutional simplicity-for-humans and maximal speed/lightness goals.

MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
