# NORDOI G0.3 R4T — Owner-pinned guardian enrollment and high-water custody protocol lab

**Research only. No real separately administered custodian. Production NOT_AUTHORIZED.**

## Parent and immutable project boundaries

Direct successor of owner-replayed R4S candidate PR #83 at exact SHA \`ba88eb3a447fb1a02d370c32a5e8b10630b55b7d\` (owner macOS 14.5 Intel: 182/182 regular R4A–R4S PASS, 3 intentionally ignored, 19/19 test blob pins, Rustfmt strict and Clippy \`-D warnings\` PASS, script exit zero). GitHub CI R4S dedicated 4/4 and general Test Gate 5/5 SUCCESS. Previous research PRs remain draft, unmerged.

R4T adds only a test-only Rust lab, this research report, and one CI workflow, stacked as a **draft PR**. No public \`.noi\` grammar, kernel, compiler, user repo, runtime, host I/O admission, original tests or GitHub \`main\` modifications. NORDOI's external-I/O reference permission remains default-denied. All **12 native admission gates UNMET**.

## 1. Explicitly separate three propositions

1. **Public key enrollment:** A verification system receives a public key *and a previously approved SHA256 fingerprint from a separate input*. A candidate signer cannot self-approve by supplying a new fingerprint or replacing the approved public root. A detached Ed25519 SSHSIG in a dedicated owner-enrollment namespace then authorizes a named guardian public key.
2. **Signed latest-head receipt:** A guardian-custodian issues a canonical, signed checkpoint high-water receipt binding an \`epoch\`, \`sequence\`, 64-hex SHA256 head, specifically endorsed publisher public key and a revoked key. Admission fails without an available authenticated receipt, a preapproved owner root, an exact checkpoint match, and a nonrevoked, approved publisher.
3. **Actual administration and monotonicity:** A **different real administrator and protected append-only head** must continue to exist *outside* the code, developer, original repository, user Mac and compromised owner account. R4T does **NOT** create such a custodian, independently custody any secret, secure a checkpoint log, prove real administrator ownership, or protect a verifier from being replaced. Test role separation is not administrative separation.

## 2. Actual signed lab experiment

The Mac test executes native \`/usr/bin/ssh-keygen\` Ed25519 key generation, signing and \`ssh-keygen -Y verify\`. It reuses **unchanged R4S checkpoint lineage and cryptographic tests as copied standalone test definitions** (11 Mac regular controls including the inherited default-denial reference test), preserving the actual R4P signed GitHub epoch2 evidence verification as a live *same-owner* network anchor.

Six additional native Mac R4T controls cover:
- **Enrollment positive and default denial:** A fixture owner root public key's full SHA256 is supplied as a distinct expected pin before verification. A signed, canonically encoded guardian enrollment record verifies only under that key and owner-specific SSHSIG namespace; no pin or mismatched pin is rejected. This *models* preapproval; in the lab **the same process generated BOTH root and expected pin**. It is NOT genuine owner out-of-band enrollment.
- **Enrollment substitution and tamper:** A signed enrollment for guardian B cannot authorize different guardian C; signed message mutation and cross-namespace SSHSIG replay fail.
- **One-sided whole-history restore:** With an intact newer signed high-water receipt, an old fully restored R4S checkpoint head is rejected. This is a conditional result under the assumption that the signed custodian receipt is more recent and not itself restored.
- **Publisher key revocation:** A signed newer head names the current publisher and explicitly revokes a previous key. Even an authentic older publisher is denied once the intact newer receipt is required.
- **Missing witness and root:** Without signed guardian receipt, without expected owner pin, or with cross-domain signature, admission is denied.
- **Required counterexample:** Both the client head and the custodian's **old, authentic, signed high-water receipt** are restored to an earlier epoch. The old checkpoint **is accepted**. This demonstrates a concrete replay vulnerability; a PASS for this counterexample is **not** a monotonicity guarantee.

Test-only private keys are generated and erased with disposable fixture directories. Keys are never uploaded. Owner enrollment and high-water receipts are signed with a *same-test-owner fixture key*, even if separate Ed25519 cryptographic identities are used.

### Important security limitations in the prototype

The two new admission checks (owner enrollment and current-head receipts) are exercised as laboratory APIs; they are not wired into NORDOI's production runtime, installer, OS trust store, or a third-party service. The receipt is presented as a signed input and its current freshness is **not** independently authenticated or persisted in monotonic storage. An attacker can replay an older legitimately signed receipt; the last R4T test deliberately proves it. A claimed separate administrator in a JSON string, directory name, second GitHub branch, or second software-generated key is not evidence of real independent operation.

The R4P GitHub signed static V2 anchor is **off-device but controlled by the same repository administrator**. Fetching it verifies that one historical checkpoint, not a dynamic R4T high-water service or a new guardian administrator. R4T's owner pin is generated by the test driver; no owner-approved out-of-band fingerprint is established for a real application.

## 3. Deployment acceptance contract (NOT MET, no production)

A future user-authorized independent custodian must provide all of the following verifiable *external* evidence:
- Explicit external operator identity and authorized ownership, distinct from the primary NORDOI code/release administrator, with independently maintained credential and signing material.
- Owner-reviewed guardian key fingerprint captured through an authenticated out-of-band channel before any remote key acceptance. Never enroll a root solely from bytes returned by the same candidate endpoint.
- Externally committed append-only checkpoint or independently witnessed monotonic high-water head, with a durable way to detect restoration of **both** locally held state and the custodian's latest receipt. Where global anti-equivocation is required, use independently operated witnesses/gossip with fork detection.
- Signed endpoint response to a fresh challenge, bounded validity and downtime denial. Implement a verifiable state-persistence and crash-recovery contract; **do not** silently reset or skip epochs after outage.
- Explicit key rotation with old and new guardian proof (from R4S), revocation and protected recovery authority, with denial of old keys after rotation.
- Transparent and auditable receipts for fork, signer compromise, timing, outage, network partition, revoked publisher, and malicious repository-owner deployment of a substituted verifier.
- Actual independent Mac/Windows/Linux security testing before a platform-wide native access claim.

**No such external custodian was enrolled or contacted in R4T.** Do not report an independent-admin PASS, permanent anti-rollback, protected production signing or NORDOI native execution authorization.

## 4. CI and owner replay

- macOS 15 Intel/Apple Silicon: real native Ed25519 SSHSIG and R4S signed checkpoint lineage including inherited negative whole-history-replay test; R4T enrollment/high-water/revocation/combined-replay tests; real pinned R4P GitHub anchor; plus R4S predecessor regression. Strict Rustfmt and offline \`cargo clippy --all-targets -- -D warnings\`.
- Linux/Windows: portable non-claim and reference-model host-I/O default denial only.
- Owner Mac Intel 14.5: independent pinned temp Git checkout, exact **20 source Git blob IDs**, expected cumulative **199 regular tests PASS, 0 fail, 3 deliberately ignored auxiliary cases**, strict Rustfmt and Clippy, clean worktree; original owner checkout untouched.
- Actual owner replay is distinct from CI; no completion claim until full local Terminal transcript passes.

\`\`\`ini
G03_R4S_OWNER_MAC_INTEL=PASS_WITH_CONTROLS
G03_R4T=RESEARCH_CANDIDATE
SIMULATED_OWNER_KEY_ENROLLMENT=CI_PENDING
SIGNED_HIGHWATER_RECEIPTS=CI_PENDING
PUBLISHER_REVOCATION=CI_PENDING
REPLAY_OLD_SIGNED_RECEIPT=KNOWN_VULNERABILITY
REAL_UNRELATED_ADMIN_CUSTODIAN=NOT_ENROLLED
INDEPENDENT_APPEND_ONLY_MONOTONIC_LOG=NOT_IMPLEMENTED
PROTECTED_VERIFIER_INSTALLATION=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
\`\`\`

## Primary reference

OpenSSH SSHSIG protocol / Ed25519 signature verification: https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig

Git reference and fetch: https://git-scm.com/docs/git-ls-remote
