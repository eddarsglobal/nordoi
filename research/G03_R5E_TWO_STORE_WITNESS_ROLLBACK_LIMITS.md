# NORDOI G0.3 R5E — Two real SQLite stores and the non-independence boundary

**Research only. A second local file is not an independent operator or protected remote monotonic witness. There is no production or admission authority.**

## Provenance

This candidate is directly stacked on **G0.3 R5D FIX1 PR #95** exact commit `f870bc0a869d0dcf478a02c7ad1df87122135125`. Owner Mac Intel 14.5 R5D returned **391/391 regular research tests PASS**, 3 deliberately ignored Rust tests, **53/53 Git blob source pins**, Rustfmt/Clippy strict PASS, `SCRIPT_EXIT=0`. R5D dedicated CI 4/4 SUCCESS; the general Test Gate on the same source was ultimately **5/5 SUCCESS on the second attempt**, after a cancelled macOS runner job was individually rerun. The first unsuccessful attempt remains historical, not reclassified. R5D original Windows tests needed a FIX1 connection-lifetime correction; the R5E parent is that corrected exact commit.

## New genuine work, no external custody claims

R5E creates **two real, separate on-disk SQLite databases** in explicitly supplied private sibling directories:

1. Publisher-side: unchanged real R5D `r5d_lab.sqlite3`, with SHA-256 event chain, unique locally consumed challenge nonce, epoch and height.
2. Second-store witness-side: new `r5e_witness.sqlite3`, with its own SQLite FULL-sync setting, rollback journal, transactional `BEGIN IMMEDIATE` checkpoint table, unique nonce, last published primary head, linked witness SHA-256 checkpoint chain, epoch and metadata integrity verification.

The second-store checkpoint runs in its **own real OS process** when invoked via the R5E CLI. Its inspection of the actual primary database acquires a local SQLite transaction and verifies the primary R5D hash chain. When both separately persisted stores are consistent, the result still says `status=BLOCK`, `admission=NOT_AUTHORIZED`, and `reason=LOCAL_TWO_STORE_MATCH_NOT_INDEPENDENT`.

### Actual adversarial counterexamples, not generated authority

- **Primary-only file rollback:** checkpoint publisher height 1 and 2, restore the older primary SQLite file while preserving second-store height 2. The second store reports `PRIMARY_ROLLBACK_DETECTED_BY_PRESERVED_SECOND_STORE`, and admission stays BLOCK.
- **Same-height fork:** restore publisher height 1 and create an alternative height-2 branch. The witness reports `WITNESS_PRIMARY_EQUIVOCATION`, not admission.
- **Both databases restored to older snapshots:** copy both files at height 1, advance/checkpoint height 2, restore **both** older files. The two-store consistency checker sees matching height 1 and **cannot distinguish them from the current history**. The research verdict remains `LOCAL_TWO_STORE_MATCH_NOT_INDEPENDENT`. This is the key scientifically demonstrated failure of same-administrator custody. The second SQLite file is not a remote durable global high-water.
- **Two actual concurrent checkpoint processes:** they attempt to advance a single checkpoint. SQLite serialization yields exactly one local `LAB_WITNESS_COMMIT` and one refused replay/gap. Both CLI processes still return exit code **2** and `admission=NOT_AUTHORIZED`.

The study also demonstrates distinct/non-nested file paths, corruption and metadata mutation detection, epoch advancement and downgrade rejection, primary-vs-witness lag, file symlink and FIFO denial, and private file-directory access checks. No external network connection or real custodian is activated; no long-running service is installed. All data stays in disposable laboratory directories. These tests do not establish power-loss guarantees or robust cross-database atomic commit; separate publisher and witness databases do **not** share an atomic distributed transaction.

## Research controls (24 new tests)

The R5E Python tests genuinely exercise actual disk SQLite and actual multiple Python subprocesses: genesis, separate checkpoint file, restart across process, strict sequence, unwitnessed publisher advance, two checkpoints, nonce replay, primary-only rollback detected, same-height fork detected, **joint snapshot rollback NOT detectable**, witness-only rollback leaves lag, mutated witness chain/metadata/history detected, refused stale checkpoint does not modify head, epoch advance/downgrade, concurrent dual-process checkpoint race, identical/nested directories denied, symlink/FIFO denied, insecure POSIX permissions denied, and real CLI local commit exit 2. Windows POSIX-specific checks may be intentionally skipped, but all available tests must pass.

The real R5D tests are retained unchanged, including their own single-file complete rollback negative control. No simulation of a third-party guardian, no fabricated external security attestation.

## Independent custody prerequisites (not satisfied)

- G1: real independent guardian with verified legally/operationally distinct administrative custody.
- G2: owner-approved signing/root pin authenticated over a verifiable out-of-band channel.
- G3: independently protected monotonic high-water outside same-owner modification and rollback, with durable consumed nonce history and fork/equivocation protection.
- G4: an installed independently protected verifier and trust root, not replaceable unilaterally by the publisher.
- G5: real independently reviewed recovery, revocation, outage and rotation.
- G6: independently reproducible, authenticated third-party signed provenance and attestations, not self-supplied labels or local databases.

No owner-approved OOB root, genuinely separate external administrator, durable external witness or global monotonic antirollback is currently enrolled. Even successful real local proof of rollback detection must **not** unlock the 12 NORDOI native admission gates.

## Acceptance

- Stack one DRAFT/UNMERGED PR on exact R5D head. Exactly five new research files: Python second-store witness, 24 adverse tests, truthful authority-gap JSON, this scientific dossier and a dedicated four-platform GitHub CI workflow.
- Preserve every R4A–R5D Rust/Python source, existing CI, public `.noi`, compiler, kernel, runtime, main, original owner source tree and production. Use no paid infrastructure.
- GitHub four-platform dedicated CI **4/4 actually completed SUCCESS** (macOS Intel, Apple Silicon, Ubuntu, Windows). Real R5E 24 cases on POSIX; Windows skipped tests must be reported rather than counted as executed. R5D/R5C/R5B/R5A/R4Z/R4Y Python regression and no-admission checks PASS. General Test Gate must complete five actual SUCCESS jobs.
- Owner Mac Intel exact-revision isolated replay verifies **58 exact Git blobs** (24 Rust R4A–R4X + 5 R4Y + 4 R4Z + 5 R5A + 5 R5B + 5 R5C + 5 R5D + 5 R5E). Test totals: **415 regular PASS** = R5E 24 + R5D 24 + R5C 24 + R5B 24 + R5A 24 + R4Z 24 + R4Y 18 + Rust 253, **3 intentionally ignored Rust**, Rustfmt and strict Clippy PASS, source checkout unchanged and `SCRIPT_EXIT=0`.
- Owner Mac replay must **actually occur** before any `PASS_WITH_CONTROLS_RESEARCH_ONLY` acceptance. A GitHub workflow and local preparation do not prove execution on the owner's Mac or enrollment by a third party.

```ini
G03_R5E=RESEARCH_CANDIDATE
SECOND_STORE_SQLITE=REAL_LOCAL_DISK
PRIMARY_ONLY_ROLLBACK=DETECTABLE_WHILE_SECOND_STORE_PRESERVED
BOTH_SNAPSHOTS_ROLLBACK=UNDETECTABLE_WITHOUT_EXTERNAL_ROOT
REAL_INDEPENDENT_CUSTODIAN=NOT_ENROLLED
OWNER_AUTHENTICATED_OOB_ROOT=NOT_ENROLLED
EXTERNAL_PROTECTED_HIGHWATER=NOT_IMPLEMENTED
GLOBAL_DURABLE_NONCE_LEDGER=NOT_IMPLEMENTED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
