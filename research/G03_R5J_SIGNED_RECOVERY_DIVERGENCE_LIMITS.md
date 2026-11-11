# NORDOI G0.3 R5J — Signed recovery intent quarantine against divergent revocation histories

**Research-only, deny-only. This stage does not enroll a genuine guardian, third-party operator, owner OOB root or production recovery power.**

## Immutable parent and previous evidence

R5J is directly stacked on R5I FIX1 PR #100 exact head `32cdb98628f35cf08a965e1a2efe37bdd64e4d7b`. The owner executed R5I in an isolated macOS Intel 14.5 checkout: **511 regular tests PASS, 3 Rust intentionally ignored, 78/78 exact pinned Git blobs PASS, strict Rustfmt/Clippy PASS and script exit 0**. GitHub dedicated R5I 4/4 and general Test Gate 5/5 were SUCCESS. R5I demonstrates that a separately stored *local* revocation checkpoint can detect R5H primary rollback if it is preserved, but both same-admin SQLite databases can be simultaneously restored without a protected outside high-water.

## Objective — quarantine recovery requests without manufacturing a guardian

R5J defines an explicitly non-authorizing **RECOVERY_INTENT_DENY_ONLY** format, signed by a *real ephemeral Ed25519 test key* through macOS/Linux OpenSSH `ssh-keygen -Y sign`, with verification by `ssh-keygen -Y verify` under a new domain-separated `nordoi-r5j-recovery-intent-fixture@eddarsglobal` namespace. This signature demonstrates only disposable key possession. It cannot authenticate a real independent custodian, prove authority over a real separate recovery service, enroll a trusted OOB root, or revoke/restore a native capability.

An intent binds the exact R5I FIX1 source commit; strict JSON protocol/project; action `RECOVERY_INTENT_DENY_ONLY` (never `GRANT`, `UNREVOKE`, `RECOVER` or `APPROVE`); actual R5H/R5I local database primary SHA-256 head, floor SHA-256 head, positive floor height and epoch; nonce; fixture Ed25519 key digest; issuance and expiry at most 300 seconds. The packet uses canonical UTF-8 JSON and a native OpenSSH SSHSIG; keys with unknown root are rejected. No secret or real guardian key is imported.

The R5J examiner verifies the actual R5I checkpoint chain via unchanged R5I audit, and reads the two real R5H/R5I SQLite state rows through read-only SQLite URI `mode=ro`. Missing databases are refused **without silently creating another genesis**. A source state must be *revoked* at nonzero height and agree on height, epoch and primary head, then the signed intent must bind those exact values. A valid native signature and perfectly matching local state **still yield `BLOCK / NOT_AUTHORIZED`** because a recognized external independent authority is absent. The sole command-line mode `--exercise` uses newly created private temporary files, does not accept a caller-selected storage path, and always returns **exit 2**.

The verifier is a research laboratory and not a hardened concurrent process/host security boundary. SQLite state may change between reading the two databases; the result never grants admission, and cannot replace authenticated remote high-water, protected root storage, or runtime trust.

## 24 meaningful tests

**16 portable denials:** empty/invalid/noncanonical/duplicate/oversized JSON, incorrect project/protocol/source, grant/unrevoke/recover/approve operation substitution, genesis/boolean/invalid heights and epochs, digest substitution, timestamp and TTL violations, bad Ed25519 wire public keys/fingerprints, fake SSH signatures, and CLI prohibition of user-supplied database paths. All are actual code paths; no test asserts fictitious remote administration.

**Eight actual Unix/OpenSSH/SQLite cases** (Mac Intel, Apple Silicon and Ubuntu): key-possession signature valid yet recovery still blocked; real separately generated signer substitution denied; post-signature nonce tampering fails native SSHSIG verification; expired intent denied; primary-only rollback detected and quarantined while floor remains; different genuinely signed revocation at same height fails history comparison; jointly restored older primary+floor snapshots do not satisfy a *preserved signed newer intent* and remain denied; real native `--exercise` proves the outcomes and exits 2.

**Important limitation:** retaining a signed recovery intent lets the lab reject two old SQLite snapshots that conflict with that *retained intent*. It does **not** make the signature or intent into an independent non-rollbackable witness. A same-admin adversary able to remove/restore the intent and both files can still conceal lost history. The scientific question is whether inconsistent/stale *presented* evidence can be refused, **not** whether independent remote recovery has been implemented.

Windows intentionally runs the **16 portable tests**, skipping the **8 native `/usr/bin/ssh-keygen` cases**. Never count native skips as executed.

## Open independent-custody gates

1. Genuine externally independent guardian and recovery authority, verifiably isolated from publisher administrator/root, CI, backups and recovery.
2. Real owner-authenticated, separately held OOB key root and explicit owner-approved delegation.
3. Global protected monotonic revocation ledger/high-water and durable spent nonce register outside the publisher's rollback powers.
4. Independent trusted verifier and anti-fork logic deployed with protected code identity and roots.
5. Externally controlled revocation, key rotation, outages, compromise and tested disaster recovery.
6. Independently attributable signed custody chain and evidence that is reproducible by a genuinely separate operator.

**None has been enrolled or observed.** All **12 native admission gates remain UNMET**, `PRODUCTION=NOT_AUTHORIZED`.

## Acceptance and governance

- Exactly five additive research-only files in one new **DRAFT UNMERGED** PR directly on exact R5I FIX1 SHA. No edits to earlier scientific source, CI, `.noi`, compiler/kernel/runtime, main, owner original worktree, services or production.
- Dedicated four-OS CI actual **4/4 SUCCESS** and NORDOI general Test Gate **5/5 SUCCESS** at unchanged exact R5J SHA. Prior R5I/R5H/R5G/R5F/R5E/R5D/R5C/R5B/R5A/R4Z/R4Y Python regressions must pass, with platform skips reported honestly.
- Owner Mac Intel 14.5 exact-head isolated replay: **83 pinned Git blobs** = 24 Rust R4A–R4X + five R4Y + four R4Z + five each R5A–R5J. Target **535 regular PASS** = R5J 24 + R5I baseline 511; three intentionally ignored Rust, Rustfmt and strict Clippy PASS; real R5H signature/R5I floor and R5J signed recovery quarantine, unchanged HEAD/worktree and `SCRIPT_EXIT=0`. Mac owner transcript required before research-only acceptance.
- Mac kit ZIP has **five files at archive ROOT**, no nested path. Verify outer SHA256, then use a new `mktemp -d` directory, `unzip -q "$ZIP" -d "$WORK"` and `cd "$WORK"`. No overwrite prompts; no extraction into existing folders.

```ini
G03_R5J=RESEARCH_CANDIDATE
SIGNED_RECOVERY_INTENT=REAL_OPENSSH_EPHEMERAL_FIXTURE_ONLY
AUTHENTIC_INDEPENDENT_RECOVERY=NOT_ENROLLED
PRESERVED_REVOCATION_FLOOR_ROLLBACK=DETECTED_AND_REFUSED
SIGNED_SAME_HEIGHT_FORK=REFUSED
JOINT_OLD_LOCAL_SNAPSHOTS_WITH_PRESERVED_INTENT=REFUSED
JOINT_OLD_LOCAL_SNAPSHOTS_WITH_NO_EXTERNAL_ANCHOR=NOT_GLOBALLY_DETECTABLE
OWNER_OOB_ROOT=NOT_ENROLLED
PROTECTED_GLOBAL_MONOTONIC_REVOCATION=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
