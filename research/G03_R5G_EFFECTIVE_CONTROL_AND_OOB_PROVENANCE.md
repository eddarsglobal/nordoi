# NORDOI G0.3 R5G — Effective administrative control and external-custody claim rejection

**RESEARCH ONLY. No independent third-party operator enrolled; no owner-authenticated OOB trust root; every admission BLOCK.**

## Frozen foundation

R5G is a DRAFT, unmerged child of **R5F PR #97** head `b84f49ce1c993f2c1f770a8d9882bb38b188aaae`. Owner's Mac Intel macOS 14.5 R5F full replay recorded **439 regular research tests PASS**, **3 deliberately ignored Rust tests**, **63/63 exact Git blob pins**, Rustfmt and strict Clippy PASS, `SCRIPT_EXIT=0`. Dedicated R5F GitHub CI **4/4 SUCCESS** and general Test Gate **5/5 SUCCESS**. R5F identified same-UID custody on two real R5D/R5E SQLite databases. R5G tests actual *effectiveness* of this control using a separate OS process.

## Real new adversarial experiment

The R5G Python module creates its own **randomly named private temporary directory** and two **real SQLite databases** using the unmodified R5D primary and R5E witness modules. It creates and checkpoints local heights 1 and 2, takes actual height-1 backup snapshots, then chmods the local witness SQLite database to POSIX mode **0400 (read only)**.

A **second real Python OS process**, running under the *same* OS UID, enters only that temporary lab directory using a one-use ephemeral token, verifies both SQLite file headers and ownership, chmods the supposed read-only witness database back to mode **0600**, and restores **both** earlier height-1 database snapshots.

When the parent process reopens R5E, it sees two internally consistent height-1 databases. The earlier height 2 is **undetectable by these same-admin stores**. This does NOT mean the test hacked a real external service, bypassed a different OS user, defeated a hardened protected remote journal or proved any actual external attacker capability. It shows exactly that Unix file mode 0400 and a separate OS process are **not** independent custody against the file owner.

The `--run` CLI creates and cleans up only its own disposable lab. It does not accept caller-specified storage paths in that mode. The internal `--actor` helper only runs against a private ephemeral directory with a matching one-time lab token. It NEVER modifies the original NORDOI workspace or production systems. This is a safety/hygiene restriction, not a hardened security sandbox against a hostile local user.

The actual result is:
- `reason=SAME_UID_CAN_RESTORE_READONLY_LOCAL_WITNESS`
- `witness_mode_before_actor=256` (= 0400)
- `witness_mode_after_actor=384` (= 0600)
- `witness_before_restore_height=2`
- `witness_after_restore_height=1`
- `both_store_rollback_detections=NO`
- `status=BLOCK; admission=NOT_AUTHORIZED`
- `independent_external_operator=NOT_ENROLLED`
- CLI exit **2**.

Windows cannot claim POSIX UID/mode semantics: the eight POSIX native cases are intentionally skipped there; its real ACL/domain independence remains **NOT_ATTESTED**. Mac Intel, Apple Silicon and Ubuntu execute the eight actual POSIX experiment cases and 16 portable manifest negative controls.

## 16 portable claims qualification controls

A strict, bounded canonical JSON declaration may claim different publisher, guardian, witness and recovery operator names, SHA256 keys and “CLAIMED_OFFLINE” provenance. Even with four distinct string names and nonzero hashes it **always BLOCKS**, because self-declared controller labels or hash digests are **not** cryptographic operator authentication, proof of owner OOB channel, independently protected global highwater or protected recovery.

Negative tests reject same-controller aliases, protocol/source/project substitutions, zero/invalid claimed hashes, boolean/malformed controllers, duplicate keys, invalid UTF8, noncanonical/oversized JSON, and CLI requests to exercise caller-selected storage paths.

**No code path returns admission=true or authorizes a native capability.** The untrusted claim format is merely a candidate evidence contract, not a signed external attestation. Even signatures proven with R5B's temporary local test keys do not establish an independent owner or third-party guardian.

## Six real outstanding requirements

1. **Independent operator** — real separate organization/control-plane account, verifiable administrative, root, backup, CI, host, recovery and revocation isolation from the NORDOI publisher.
2. **Owner-approved OOB root** — actual independently authenticated owner fingerprint verification and offline retention, not a file named OOB or a matching SHA256 field.
3. **Externally protected durable highwater** — independently governed append-only monotonic history and global spent-nonce ledger, tamper/rollback/fork resistance in practice.
4. **Protected verifier** — release, code identity and trusted roots insulated from unilateral publisher substitution.
5. **Governed recovery** — real exercised independent revocation, key rotation, outages, compromise response and split-brain rules.
6. **Real custodian proofs** — externally attributable signed evidence chain, independently reproducible under separate administration with explicit authorization.

None are established by these two SQLite databases or same-owner local ephemeral process. Do not request, collect or commit third-party private keys, personal identification documents, off-band communication secrets or sensitive recovery materials into a public research PR.

## Acceptance and release policy

Exactly five additive R5G research files: one Python effective-control lab, one **24-case** adversarial test file, one explicit absence JSON, this dossier, and a dedicated four-platform CI workflow. Existing R4A–R5F sources, GitHub workflows, `.noi`, compiler/kernel/runtime, main, production and original owner worktree must remain unchanged; PR must stay DRAFT and unmerged.

Dedicated four-platform CI must complete **4/4 actual SUCCESS** and general Test Gate **5/5 actual SUCCESS**. POSIX native test count 24; Windows 16 PASS with 8 native controls intentionally skipped. Historical R5F/R5E/R5D/R5C/R5B/R5A/R4Z/R4Y Python regressions must pass, with appropriate platform skips transparently recorded.

Owner macOS 14.5 Intel exact-head isolated replay requires **68 Git blob SHA pins**: 24 Rust R4A–R4X + 5 R4Y + 4 R4Z + 5 each R5A–R5G, and **463 regular PASS** = 24 R5G + 24 R5F + 24 R5E + 24 R5D + 24 R5C + 24 R5B + 24 R5A + 24 R4Z + 18 R4Y + 253 Rust. Three Rust intentionally ignored, strict Rustfmt / Clippy and unchanged clean source HEAD. Any actual Mac replay must be received before research-only acceptance; CI is not an owner replay.

**No independent physical or organizational custody has yet been observed. Research PASS_WITH_CONTROLS can only mean actual local negative-control experiment succeeded; it never grants one of the 12 native admission gates or production.**

```ini
G03_R5G=RESEARCH_CANDIDATE
LIVE_SAME_UID_CHMOD_AND_JOINT_RESTORE=REAL_LAB
EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_ENROLLED
EXTERNAL_MONOTONIC_HIGHWATER=NOT_IMPLEMENTED
GLOBAL_DURABLE_NONCE_LEDGER=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
