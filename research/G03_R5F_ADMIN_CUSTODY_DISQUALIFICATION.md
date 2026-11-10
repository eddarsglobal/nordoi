# NORDOI G0.3 R5F — Real OS-account custody negative controls

**RESEARCH ONLY — independent external custodian remains NOT ENROLLED; native admission BLOCK and PRODUCTION NOT_AUTHORIZED.**

## Frozen source / actual owner evidence

R5F directly stacks on **G0.3 R5E PR #96** at pinned commit `04e5670cdfe9682e2d467c9f3678ba2b069afd54`. The actual owner-provided Intel Mac macOS 14.5 replay reported **415 regular research tests PASS**, 3 intentionally ignored Rust tests, 58/58 exact Git source pins PASS, Rustfmt/strict Clippy PASS, source worktree untouched, `SCRIPT_EXIT=0`. Separately checked R5E GitHub dedicated CI **4/4 SUCCESS**, general Test Gate **5/5 SUCCESS**. R5E scientific finding: local second-store can detect publisher-only rollback when preserved, but **joint full snapshot restoration remains undetectable** under the same administrator.

R5F must not modify existing R4A–R5E source, GitHub CI, the `.noi` public language, NORDOI compiler/kernel/runtime, main, production or owner's original worktree.

## New objective: actual, narrowly scoped OS ownership evidence

R5F audits **existing genuine R5D/R5E SQLite files** by READ-ONLY filesystem access. It inspects both existing private parent directories and the actual R5D and R5E database files. On POSIX, if the two parent directories and two database files are owned by the **same OS UID**, it returns the disqualifier `SAME_OS_UID_CONTROLS_BOTH_SQLITE_STORES`. Different names, different directories and different database files are not proof of separate administrative control.

The audit also detects missing/nonregular files, symlink paths, identical/nested directories, hardlink aliasing, nonprivate directories and database files (on POSIX), and corrupted SQLite file headers. It uses an OS read-only handle for each database and closes it without SQLite write transactions, mutation, logging, external calls or key material. It does NOT claim to attest end-to-end file security, because high-privilege users, host administrators or the source publisher can still edit or replace the files. On Windows, POSIX UID-based separation is not available as authoritative Windows ACL or domain-control evidence; the audit explicitly says `NT_OWNER_ACL_AND_ADMIN_DOMAIN_NOT_ATTESTED`.

The real R5E databases are created in distinct sibling directories by the tests, and the R5F inspection reads them. Each test environment's databases are **under the same laboratory administrative domain**. This is a useful disqualifier, not a positive proof of genuine third-party custody. Other systems with different UIDs would likewise need independent operator identity, access-control, root management and live witness attestation; UID difference alone never grants authorization.

Every outcome returns `status=BLOCK`, `admission=NOT_AUTHORIZED`, and CLI exit **2**. Even a successfully readable, private, separately named SQLite pair has **zero production authority**. Source paths provided by the caller are evidence hints, not a security sandbox.

## 24 new adversarial cases

- Two actual existing SQLite databases are audited but **never** admitted.
- Same OS UID custody explicitly disqualified on POSIX; Windows reports no ACL attestation.
- Different file/directory names and same storage device do not establish independence.
- External operator identity, owner OOB key root, remote protected high-water, external nonce consumption, admin independence all remain unverified or absent.
- Actual CLI returns exit 2 for a valid local pair, with read-only files unchanged byte-for-byte.
- Missing primary or witness database refused; corrupt SQLite header of either refused.
- Symlink primary/witness databases refused; symlink directories refused.
- Identical or nested directories refused; POSIX world-readable witness directory refused.
- Hardlink alias between the two database paths refused.
- Windows tests must report POSIX-only checks and unavailable symlink/hardlink checks as **skipped**, never PASS.

The inventory `G03_R5F_NO_INDEPENDENT_ADMINISTRATION.json` explicitly records the currently known shared local administrative domain and null external authorities. It is a *truthful research record*, **not third-party enrollment evidence**.

## Six independent-custody gates remain unfulfilled

1. G1 — a genuine externally controlled account/operator, independently identified and assessed for organizational/control independence, including administrator, backup, audit, recovery and revocation powers. An OS UID is at most one negative technical indicator.
2. G2 — explicit owner approval of the guardian key fingerprint verified via a genuinely independent out-of-band channel with recorded provenance.
3. G3 — real external protected monotonic witness that cannot be rolled back, rewritten or forked by the primary publisher, with durable global consumed nonce register.
4. G4 — deployed verifier and pinned roots protected against unilateral source-publisher substitution.
5. G5 — independent key-rotation, compromise, revocation, outage and recovery controls, including tested split-brain governance.
6. G6 — real independently reproducible signed provenance tying distinct operator identities, roots, witness and witnessed history. A second file, branch or lab-generated key is insufficient.

No actual independent operator, separately governed witness root, OOB owner fingerprint or remote durable nonce ledger has been supplied. **All 12 native admission gates UNMET.** Do not commit identity documents, private keys, recovery secrets or sensitive communication records to the public repository.

## Qualification gates for R5F

- One new **DRAFT / UNMERGED** PR stacked directly on R5E frozen head, with exactly five additive research-only files: source, 24-case tests, truthful absence inventory, this qualification document, dedicated four-OS CI.
- Four actual dedicated GitHub jobs SUCCESS: macOS Intel, Apple Silicon, Ubuntu, Windows; run 24 new R5F tests with platform skips transparently recorded, plus prior R5E/R5D/R5C/R5B/R5A/R4Z/R4Y research Python regressions.
- NORDOI Test Gate five actual jobs SUCCESS.
- Owner Mac Intel exact-commit isolated replay: **63 pinned Git source blobs** = 24 Rust R4A–R4X, 5 R4Y, 4 R4Z, 5 each R5A through R5F. New R5F 24 + R5E 24 + R5D 24 + R5C 24 + R5B 24 + R5A 24 + R4Z 24 + R4Y 18 + previous Rust 253 = **439 regular tests PASS**, 3 prior Rust intentionally ignored. Strict Rustfmt/Clippy, source head unchanged, isolated worktree clean, script exit 0. Any skipped R5F cases on macOS must be reported; no false 24/24 execution claims.
- **Research-only** PASS_WITH_CONTROLS may be declared after exact-head CI and actual owner replay, but **must not** be described as positive administrative independence or operational external custody. Even perfect tests must leave PRODUCTION NOT_AUTHORIZED.

## Governance state

```ini
G03_R5F=RESEARCH_CANDIDATE
REAL_R5D_R5E_SQLITE_FILES=READ_ONLY_OS_CUSTODY_AUDIT
SAME_POSIX_UID=DISQUALIFIER_FOR_INDEPENDENT_OS_ACCOUNT_CUSTODY
DIFFERENT_UIDS=NOT_SUFFICIENT_FOR_INDEPENDENT_GUARDIAN
WINDOWS_ACL_ATTESTATION=NOT_IMPLEMENTED
REAL_INDEPENDENT_OPERATOR=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_ENROLLED
EXTERNAL_MONOTONIC_WITNESS=NOT_IMPLEMENTED
GLOBAL_DURABLE_NONCE_LEDGER=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
