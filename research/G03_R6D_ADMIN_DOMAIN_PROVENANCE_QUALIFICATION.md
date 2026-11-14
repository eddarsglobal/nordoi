# NORDOI G0.3 R6D — Actual administrative-domain provenance pre-enrollment, deny-only

**RESEARCH CANDIDATE / NO EXTERNAL CUSTODIAN ENROLLED / NATIVE ADMISSION BLOCKED.** Exact additive child of owner Mac Intel-validated R6C head `87bcd2bf968533e502e68dc7472b76c1d80ec3c8` (PR #120). The owner-supplied Mac Intel log established **991 regular tests PASS** (738 Python+253 Rust), **178/178 Git source blobs PASS**, genuine native R4M XPC/CDHash **6 PASS serial**, rustfmt/strict Clippy, exact unchanged HEAD and `SCRIPT_EXIT=0`. Direct GitHub CI on R6C was independently checked **9/9 SUCCESS**. R6C stays DRAFT and unmerged.

## Research question

R6C observed two real Ed25519 keys and three mathematically genuine OpenSSH signatures under a single OS administration account. R6D tests a stronger form of the unsafe claim: **can purportedly separate administrative-domain identity, two genuine separate OS process IDs, and further cryptographically signed statements create real independent custody merely by being included in signed evidence?** No.

R6D composes the **exact source-bound real R5Z witness → R6A 256-bit challenge → R6B signed response → R6C two-key signed envelope**. A new canonical R6D document ties the exact source SHA-256, real same-admin OS identity observations, owner/candidate key pins and local clock. The owner and guardian key each sign the new R6D evidence in separate SSHSIG namespaces. With the inherited two R6C signatures and one R6B response signature, there are **five genuine cryptographic signatures** (two distinct local keys), yet no independent external signer's control domain.

## Controls implemented

- A strict canonical JSON schema `NORDOI-G03-R6D-ADMIN-DOMAIN-PROVENANCE-INTAKE-V1`, pinned to the precise R6C source commit; exact SHA-256 of R6C envelope, R6B response, R6A random challenge request and R5Z witness; exact challenge and both key fingerprints; two PIDs, two UIDs and explicit owner/custodian proxy roles.
- Rejects ambiguous/malformed/counterfeit UTF-8 JSON, duplicate keys, noncanonical serialization, extra/missing fields, NaN/Infinity, negative/Boolean/fractional counts, spoofed source, privileged operation and local expiry. A duplicate or zero PID cannot be presented as two observed processes.
- Same source hashes and genuine local signatures require actual `ssh-keygen -Y sign/verify` under separate SSHSIG namespaces `nordoi-r6d-owner` and `nordoi-r6d-candidate`; the three earlier genuine signatures are verified too. Wrong key, swapped signature, mismatched public pin, wrong namespace and tampered bytes fail closed.
- Explicit owner/candidate administrative-domain *labels* are recorded **only as untrusted claims**. A forged additional UID does not authenticate an independently controlled operator. A locally signed external operator ID, claimed owner OOB trust root, admin attestation hash, remote history hash, revocation floor, nonce floor or `quorum=2` is denied even when the signature itself verifies cryptographically.
- Locally spent challenges or revoked keys are refused but the local state can be restored under the same administrator; no independent global monotonicity is inferred.
- Strictly offline and contained in a private disposable research directory. No network, new authentication/authorization API, production runtime changes, key export, privilege escalation or enrollment of an external legal/operator custodian.

## 24 adversarial cases

**16 portable:** source/protocol/roles, four exact parent content hashes, missing signatures, duplicate/noncanonical/invalid UTF-8, missing/extra/oversize fields, wrong parent and production verb, malformed hashes/domains, bad PIDs/UIDs/time, duplicate PID, same name-only domain, claimed UID separation, every false OOB/remote/quorum claim, stale local clock, altered sources, nonce replay/key revocation, malformed local deny-only sets.

**8 POSIX-native:** real five-signature verification with distinct OS PIDs and same UID/EUID; altered genuine signed evidence, wrong signer allowlist, swapped signatures/wrong namespace, actually signed fake external operator+root, fake UID divergence, real locally spent nonce and key revocation, and standalone native experiment with intentional exit code **2** and `BLOCK`. Windows intentionally runs the sixteen portable tests and skips the eight POSIX-native tests; it does not fabricate Windows process results.

Dedicated CI covers **macOS Intel, Apple Silicon, Ubuntu, Windows**; retains R6A–R6C, R5A–R5Z, R4Y and R4Z denial-only regressions and the original genuine R4M signed native XPC/CDHash six serial on both macOS runners (no retry or waiver). The general NORDOI Test Gate must pass independently.

## Owner Mac Intel acceptance criteria — targets, not witnessed results

Reproduce only inside a fresh SHA-256 verified kit and separate exact-head Git checkout, with five new R6D files, **183/183 frozen Git source blob identity checks**, and **1015 regular PASS =762 Python+253 Rust**, three intentionally ignored Rust, original R4M six real signed native XPC/CDHash PASS serial, Rustfmt/strict Clippy PASS, exact HEAD unchanged, clean clone, original worktree untouched and `SCRIPT_EXIT=0`. The numbers remain **targets** until the owner provides a full x86_64 Mac Terminal log.

## Independent-custody evidence missing

In order to qualify a real external administrator, an actual independently controlled legal/operator custodian and genuine OOB owner-authenticated trust root are necessary, with externally managed private key, independently signed and witnessed system evidence, protected remote key epoch / nonce / revocation high-water marks, compromise/recovery controls and independent cross-domain quorum. None is supplied by local R6D experiments. All **12 native admission gates remain UNMET**; Bureau formal certification PENDING and production NOT_AUTHORIZED. A technically green negative-control lab is not automatic permission to merge the PR.

```ini
G03_R6D=RESEARCH_CANDIDATE
PARENT=87bcd2bf968533e502e68dc7472b76c1d80ec3c8
FIVE_AUTHENTIC_SSHSIG=LOCAL_KEY_POSSESSION_ONLY
REAL_EXTERNAL_ADMIN_CUSTODY=NOT_PROVEN
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
INDEPENDENT_GLOBAL_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
EXPECTED_GIT_BLOBS=183/183_PASS
EXPECTED_REGULAR_TESTS=1015_PASS
OWNER_MAC_INTEL=PENDING
PR=DRAFT_UNMERGED
BUREAU_FORMAL_CERTIFICATION=PENDING
PRODUCTION=NOT_AUTHORIZED
```
