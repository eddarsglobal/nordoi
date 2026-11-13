# NORDOI G0.3 R5W — External administrative custody intake (fail-closed)

**DRAFT RESEARCH CANDIDATE — NO EXTERNAL ADMINISTRATIVE AUTHORITY ENROLLED.**

R5W is stacked directly on validated owner R5V SHA
5f39b6f3553f3e3c0578c978f8b5da7a22aaeba3, PR #113.
R5V owner macOS 14.5 Intel x86_64: 823/823 regular tests PASS, 143/143
exact Git blob pins PASS, original real R4M signed XPC/CDHash 6 PASS serial,
three deliberately ignored Rust tests, strict rustfmt/Clippy PASS and
SCRIPT_EXIT=0. GitHub R5V dedicated 4/4 and Test Gate 5/5 SUCCESS.
R5V remains DRAFT and unmerged.

## Problem and experimental boundary

R5V observed genuine distinct OS PIDs and Ed25519 keypairs under the same
UID/EUID, including a real same-admin SQLite snapshot restoration that
erases locally persisted signed revocation. This is a counterexample to any
claim that two local processes, two valid signatures or file permissions
establish independent administrative custody.

R5W introduces a strict candidate-evidence *pre-enrollment* intake contract,
not independent operator enrollment. The canonical schema binds exact
R5V owner and custodian OS-report SHA-256 bytes, signer key pins, nonce,
local history head and epoch, and the frozen parent SHA. Schema violations,
noncanonical/duplicate JSON fields, source spoofing, changed challenge,
history, signer pin or report body are refused.

A claimed owner OOB root fingerprint, guardian pin, OS attestation,
external history anchor, quorum attestation, protected revocation floor,
spent nonce floor, and key epoch floor are **self-asserted input** unless
independently authenticated. Even when syntactically valid, they are
quarantined and can never be transformed into authority by this module.
The locally created OpenSSH allowed-signers files are not OOB trust roots.

## Actual laboratory evidence

The native exercise uses two *real OS child processes* and two *real local
OpenSSH Ed25519 keypairs* in a disposable private directory. Both reports
are signed and verified with R5V's genuine SSHSIG machinery. Their observed
UID/EUID remain equal and no external custodian is actually enrolled.
Locally self-declared external key pins and revocation counters are rejected.
The CLI returns deliberately-denied exit 2; all results are BLOCK with
native_authority_granted=false and PRODUCTION=NOT_AUTHORIZED.

A different observed UID would still be insufficient without proof of
independent effective administrative control. A different physical machine,
network service, guardian label, authenticated TLS connection, or valid
signature cannot by itself prove legal/custodial independence, historical
freshness, trusted OOB roots or externally protected monotonicity.

## Twelve externally independent admission gates: all UNMET

The canonical R5W JSON preserves the twelve R5V gate IDs: independently
authenticated owner OOB root; real independent custodian identity;
distinct administrative control; guardian OOB key pin; independently
signed OS identity attestation; signed fresh challenge and history;
independent OS-domain and host witness; externally protected key epoch;
global revocation floor; global spent nonce floor; independent quorum;
independent recovery/rotation and compromise audit. These requirements
remain not tested/not enrolled. No autonomous self-certification path exists.

## Test plan

- 16 portable structural tests (schema/canonical/duplicate/protocol,
  source, operation, history, key and nonce binding, false remote
  fingerprints and counters, forged identities and tampered report bytes).
- 8 genuine POSIX/OpenSSH native tests (real distinct PIDs and keypairs,
  true signatures, signature tampering, wrong signer, claimed OOB root,
  false protected floor, fresh-process nonce mismatch and CLI deny exit 2).
- Windows intentionally runs 16 portable checks and skips eight true POSIX
  checks; unavailable OS identity evidence is not inferred.
- Dedicated CI: macOS Intel, Apple Silicon, Ubuntu, Windows; R5W and all
  R5V–R5A denial-only regressions, R4Y, original real signed R4M native
  XPC/CDHash six tests on macOS serialized with no waiver/retry.
- General NORDOI Test Gate remains a separate required check.

## Owner Mac Intel criteria: proposed, not executed

Exact frozen head and observer SHA, freshly isolated checkout, original
owner worktree/main/PR untouched, **148/148** pinned source Git blobs
(143 R5V + 5 R5W), **847** regular tests (594 Python including 24 R5W,
253 Rust), 3 deliberately ignored Rust tests, native R4M signed XPC/CDHash
6 PASS serial, strict rustfmt/Clippy PASS and SCRIPT_EXIT=0. These are
**targets** until real owner Terminal evidence is reviewed. Independent
operator custody, OOB root, protected external histories and quorum
still remain NOT_ENROLLED / NOT_VERIFIED; twelve gates still UNMET.

## Bureau decision boundary

Passing the negative-control tests can certify only that insufficient
intake evidence is refused. It **cannot** certify actual external
administrative independence, trusted custody, native admission or production.
Any true external enrollment, security root exchange and final certification
requires separately authorized independent actors and RoadMap Bureau review.

    G03_R5W=RESEARCH_CANDIDATE
    PARENT_R5V=5f39b6f3553f3e3c0578c978f8b5da7a22aaeba3
    OWNER_MAC_INTEL=PENDING
    EXPECTED_REGULAR_TESTS=847_PASS
    EXPECTED_SOURCE_GIT_BLOBS=148/148_PASS
    INDEPENDENT_ADMIN_CUSTODY=NOT_PROVEN
    EXTERNAL_GUARDIAN=NOT_ENROLLED
    OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
    GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
    NATIVE_ADMISSION_GATES=12_UNMET
    PR=DRAFT_UNMERGED
    PRODUCTION=NOT_AUTHORIZED
