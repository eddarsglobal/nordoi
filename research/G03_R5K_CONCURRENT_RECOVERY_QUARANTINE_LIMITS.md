# NORDOI G0.3 R5K — Actual concurrent signed recovery intents and time-of-check quarantine

**Research-only, fail-closed; no real independent custodian or authority granted.**

## Immutable scientific base

R5K is a direct **DRAFT, UNMERGED** child of R5J PR #101 head `b348fac65dbe91dec958878f868fe47118a68f44`. The owner executed the complete R5J Mac Intel 14.5 replay: **535 regular tests PASS**, 3 intentionally ignored Rust, 83/83 exact Git blob pins, Rustfmt/Clippy strict PASS, `SCRIPT_EXIT=0`. R5J dedicated GitHub CI was 4/4 SUCCESS and NORDOI general Test Gate 5/5 SUCCESS. No main or production authority granted.

## Actual R5K experimental question

Do two apparently valid, concurrently presented, cryptographically signed *local* recovery intents authenticate independent decision makers and permit recovery? Does reading two SQLite states twice prevent a same-administrator rollback between verification and decision?

**No.** R5K deliberately returns `BLOCK / NOT_AUTHORIZED` for every path. The experiment qualifies *detection of some contradictory presented histories*, **not** an independent custody, atomic snapshot, or global monotonic recovery guarantee.

R5K binds two genuine R5J `RECOVERY_INTENT_DENY_ONLY` SSHSIG packages in a bounded, canonical JSON envelope with roles `candidate_a` and `candidate_b`, the exact immutable R5J head, per-packet SHA-256 hashes, and canonical base64 encoding. It insists on exactly two records and checks:
- the original R5J packet syntax, R5J namespace-bound **real OpenSSH Ed25519 signature verification**, expected pinned **temporary fixture** public keys and validity window;
- actual R5H signed-revocation and R5I checkpoint SQLite states via the existing R5J read-only observation path (no new databases substituted);
- the observed state before and after a bounded experiment callback, with all states invalid, missing, rolled back or divergent refused;
- whether the two cryptographically valid intents are identical, are signed by the same fixture key, reuse the same signed nonce, or claim different simultaneous recovery decisions. **Every case is BLOCK**.

To create a deterministic **real time-of-check/time-of-use (TOCTOU) experiment**, R5K temporarily snapshots genuine disposable revocation and checkpoint SQLite databases and supplies a callback to its verification function. The callback runs a **second actual Python OS process** that restores the older primary state after the first state observation. The second observation detects a preserved-floor mismatch and denies recovery. Another experiment restores **both** old SQLite databases between the first and second observations. The old pair is mutually consistent, but it is pre-revocation and incompatible with the still-retained signed newer recovery intents; R5K denies it. The signature/intent and first observation are under the same test administrator, **not** an independently protected remote anchor.

**Limitation:** a malicious same-administrator rollback of *all* local evidence (both SQLite stores, intents, test keys, application code and observers) still has **no global antirollback protection**. Double-reading is not a synchronized cross-process atomic snapshot; the code is research-only and never returns an admission grant. Cryptographic key possession is not real guardian organizational identity or out-of-band owner enrollment.

No operation under R5K restores real account privileges, un-revokes real capabilities, connects to a real external guardian or modifies the production kernel. The only `--exercise` command creates **new private disposable temporary directories** and has no arbitrary user-selected target paths; the CLI returns exit code **2** even for a successful scientific negative control.

## Acceptance controls (24 new tests)

**16 portable controls:** empty/invalid UTF-8, duplicate/noncanonical/nonfinite/oversized JSON, incorrect protocol/project/parent SHA, invalid role order, wrong pair cardinality, extra fields, malformed base64, packet digest substitution, malformed embedded recovery receipt, and fraudulent signature denial.

**Eight real Unix/OpenSSH + SQLite controls** (Mac Intel, Apple Silicon, Ubuntu): two *distinct genuine ephemeral Ed25519 signatures* still refused; valid duplicate replay refused; two different real signed intents controlled by the same key refused; different signers with the same nonce refused; genuine signed payload tampering and expected key substitution refused; actual separate process **primary-only rollback** between observations detected; actual separate process **both SQLite old snapshots** restored between observations detected because the first observation/intent is retained; real native `--exercise` produces the deny-only result and exit **2**.

Windows must run its **16 portable tests**, and intentionally skip exactly **eight Unix/OpenSSH** controls, which must not be represented as executed.

All R5J–R5A and R4Z/R4Y Python regression suites remain unchanged and tested. The existing R4A–R4X Rust test library remains pinned.

## Governance

Real independent guardian, owner authenticated out-of-band trust root, separately held recovery signing keys, global protected monotonic revocation floor, durable global spent-nonce history and independently governed recovery/key rotation **NOT ENROLLED / NOT IMPLEMENTED**. All **12 NORDOI native admission gates UNMET**. `PRODUCTION=NOT_AUTHORIZED`.

The entire scope is additive: a Python lab, 24 Python tests, explicit absent-custody JSON, this dossier and a dedicated four-OS workflow. **Exactly five files**, no mutation to previous scientific source/workflows, `.noi`, compiler/kernel/runtime, main, releases, owner original worktree or production. PR stays DRAFT/unmerged.

## Expected validation

- Dedicated 4-OS CI **4/4 actual SUCCESS** and general NORDOI Test Gate **5/5 actual SUCCESS**, with appropriate Windows native skips.
- Owner Mac Intel **14.5 x86_64**, Rust/Cargo **1.98.1**, exact-head isolated clone; **88 pinned Git blobs** = 24 Rust R4A–R4X, five R4Y, four R4Z, and five each R5A–R5K. Expected **559 regular PASS** = 24 new R5K + 535 prior R5J, with three Rust intentionally ignored; Rustfmt and strict Clippy PASS.
- Actual two signed fixture keys, primary-only rollback and joint rollback via distinct OS process, exact Git head unchanged before/after, isolated clone clean, owner original worktree untouched, `SCRIPT_EXIT=0`. No owner replay may be claimed before an actual owner log.
- The owner ZIP has five files **directly at ZIP root**. Verify archive outer SHA256, extract into a new `mktemp -d` directory, then `cd "$WORK"` (not a guessed nested folder). No overwriting or extraction questionnaire.

```ini
G03_R5K=RESEARCH_CANDIDATE
DOUBLE_SIGNED_RECOVERY_INTENTS=REAL_EPHEMERAL_OPENSSH_SSHSIG_ONLY
REPLAY_AND_NONCE_COLLISION=BLOCK
SECOND_PROCESS_PRIMARY_ONLY_ROLLBACK=BLOCK
SECOND_PROCESS_BOTH_LOCAL_STORES_ROLLBACK_WITH_RETAINED_INTENT=BLOCK
ALL_SAME_ADMIN_ARTIFACTS_ROLLBACK=NOT_GLOBALLY_DETECTABLE
INDEPENDENT_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
