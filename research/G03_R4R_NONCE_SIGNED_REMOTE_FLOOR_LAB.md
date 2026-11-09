# NORDOI G0.3 R4R — Live signed remote floor + nonce-bound second-authority freshness

**Laboratory-only / no independent administrative witness / NO PRODUCTION**

## Exact parent and custody

Additive stacked research on R4Q owner-accepted-with-controls `5ff90df53e979a5d27e83db07bddce52c79cbd79`, which passed **156/156 regular R4A–R4Q Mac Intel 14.5 tests**, 3 intentionally ignored, Rustfmt/Clippy strict, 17/17 source pins and `SCRIPT_EXIT=0`. GitHub dedicated CI R4Q 4/4 success and general gate 5/5 success. R4R adds only a new research Rust test, this report and a dedicated workflow; no changes to prior tests, public `.noi` grammar, compiler, production runtime, any owner's local checkout or `main`. All predecessor PRs remain draft/unmerged.

## R4Q's demonstrated residual attack

R4Q correctly showed that **A** (primary repository signer) with two stale writable local cursors is insufficient to authorize a rollback if **B** (second signer) still endorses a newer checkpoint. But **B's current-head file was itself writable in the same test account**. An attacker who restores a previously authentic B-signed old checkpoint, together with A's older signature and both local cursors, can pass the R4Q admission test. This was explicitly reproduced and remains true without a separate high-water reference.

## Incremental R4R design

R4R reuses the native OpenSSH SSHSIG two-signer negative controls from R4Q and adds two independent conditions to admission:

1. **Off-device signed high-water floor:** On macOS, a native test fetches the official public evidence branch `refs/heads/evidence/g03-r4p-external-signed-checkpoint-20261109` via actual GitHub HTTPS. `git ls-remote` must return *exactly* pinned commit `ff8abe01cc1a527dd84b463dec1736d20a5391a4`; a fresh temporary `git fetch` must return the *same* `FETCH_HEAD`. The verifier reads the **actual public root and V2 signed checkpoint bytes** from that commit, requires the root to equal the independently compiled-in Ed25519 public key, requires canonical `NORDOI-R4P-EXTERNAL-WITNESS-V1 / epoch=2 / sequence=2`, and verifies real `ssh-keygen -Y verify` against the pinned principal and namespace. A valid B checkpoint at epoch 1 may remain cryptographically authentic, but cannot pass against this authenticated V2 high-water mark. No untrusted caller may bypass the external-floor requirement for the new admission entrypoint.

2. **Nonce-bound current B acknowledgment:** A native 32-byte nonce is read from `/dev/urandom` for each challenge and hex-encoded canonically. In the laboratory B signs `NORDOI-R4R-FRESHNESS-V1` over the exact epoch/sequence and the challenge nonce, using a distinct SSHSIG namespace `nordoi-r4r-freshness@eddarsglobal`. The verifier checks B's **pre-enrolled public key**, detached signature, dedicated namespace and exact nonce/state bytes. Replayed signatures for a different challenge, mutated messages and cross-namespace signatures fail.

## Central adversarial experiment

The native test initially validates R4Q's legitimate A+B epoch 2 release. It then installs an *authentic, older B-signed epoch 1 checkpoint* and presents valid A epoch 1 with BOTH writable local states restored to epoch 1. **R4Q's original admission method still returns OK**, documenting the antecedent vulnerability. The attacker can additionally obtain a genuinely **fresh** B SSHSIG acknowledgment for epoch 1 in the laboratory. R4R performs **a brand-new GitHub HTTPS lookup, Git fetch and SSHSIG verification after the restore**, sees independently anchored epoch 2, and denies the old release with `REMOTE_HIGH_WATER_MISMATCH`. No provisional local marker or process cache is sufficient.

Other native negative controls: missing/offline remote floor denies by default, new nonce rejects an authentic old challenge response, tampering and wrong namespace reject B acknowledgments, plus all R4Q original dual-signer forged/checkpoint rotation tests.

## Non-claims: strict security boundary

- The evidence GitHub branch and main NORDOI repository are administered by **the same GitHub account**. It is an *off-device* witness, NOT an administratively independent third-party authority. GitHub administrators with source deployment rights can change the verifier/pins. The pinned commit prevents silent remote-replacement only while the original trusted verifier binary/pin remains intact.
- This is a **static epoch 2 floor**, not an append-only independent monotonic store or authorized future release update path; future epoch 3 admission requires new owner-governed evidence and pin changes. A malicious signer who can control B plus the verifier/pin or independently governed floor can defeat the test. Signature freshness is relative to the nonce; it is not third-party trusted time, independent online B custody, anti-equivocation consensus, or hardware monotonicity.
- **Both A and B ephemeral Ed25519 private keys are generated in the same disposable research process/account**. The B "fresh response" is signed by an in-process test harness, not an unrelated organizational administrator or independent remote service.
- A few tests intentionally use a static **in-memory fake floor snapshot** to isolate signature tampering/nonce behavior. They DO NOT claim live remote freshness; only the explicitly named tests perform authenticated GitHub remote operations.
- The remote check is inside disposable native **research tests only**. The NORDOI reference model still refuses `ResourceEffect::ExternalIo`. No production host I/O grant, signed installation, key enrollment, real HSM, independent certificate, process isolation, secure replay log, offline grace policy, dynamic rotation or authorization under the twelve native admission gates.

## Gate targets

- GitHub CI macOS Intel 15 and Apple Silicon 15 must execute the 15 regular macOS R4R tests (10 R4Q-inherited + five new), including a fresh network verification **after** the malicious restore, plus preceding R4Q 10/10 regression, strict Rustfmt/Clippy.
- Portable Linux/Windows must execute only reference model default denial and platform non-claim (no real Mac SSHSIG or GitHub security assertions).
- Owner macOS **14.5 Intel** must replay exactly R4A–R4R in an isolated pinned temporary Git checkout, **18/18 Git source blobs**, cumulative **171 regular PASS, zero fail, 3 intentionally ignored**, Rustfmt and Clippy strict and clean checkout. Mac execution remains separate from CI.
- PR DRAFT_UNMERGED, main untouched, **NATIVE_ADMISSION_GATES=12_UNMET**, production NOT_AUTHORIZED.

```ini
G03_R4R=RESEARCH_CANDIDATE
BASE_R4Q=5ff90df53e979a5d27e83db07bddce52c79cbd79
LIVE_REMOTE_SIGNED_FLOOR=CI_PENDING
CHALLENGE_NONCE_SSHSIG=CI_PENDING
RESTORED_B_FRESHLY_SIGNED_OLD_STATE=CI_PENDING
REAL_THIRD_PARTY_ADMINISTRATION=NOT_IMPLEMENTED
INDEPENDENT_APPEND_ONLY_CHECKPOINT=NOT_IMPLEMENTED
HARDWARE_MONOTONICITY=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

## Primary references

- https://man.openbsd.org/ssh-keygen.1
- https://git-scm.com/docs/git-ls-remote
- https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches
