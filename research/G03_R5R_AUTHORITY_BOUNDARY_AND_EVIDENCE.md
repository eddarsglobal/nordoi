# NORDOI G0.3 R5R — Qualification of genuinely independent historical authority

**Scientific research-only. No native admission. Any locally supplied assertion is NEVER accepted as proof of external independent custody.**

## 1. Frozen owner-qualified source and previous counterexample

R5R is a direct five-file additive child of R5Q PR #108, exact source SHA `8d15f9661d97a8cc30a1eca66322be28ca500abc`. Owner macOS 14.5 Intel x86_64 actually replayed R5Q with **703 regular tests PASS** (450 Python + 253 Rust), **118 exact Git source hashes PASS**, three deliberately ignored Rust, native signed XPC R4M six tests sequential PASS with no retries/waivers, Rustfmt and strict Clippy PASS, `SCRIPT_EXIT=0`. R5Q dedicated four-OS CI 4/4 and general Test Gate 5/5 PASS; PR DRAFT/UNMERGED.

R5Q physically proved that one same-administrator actor can coherently restore an old primary SQLite, local JSON witness, and separate local SQLite nonce journal; a newly started OS process then has no authenticated global evidence that a later committed history was ever visible. A nonce genuinely consumed in the later history can be written again locally as **a new BLOCK only**. That is the authoritative negative research fact. Signing a current local record, copying files to another local directory, changing inode/mtime or running a second process cannot establish an independent historical root.

## 2. R5R problem: verifiable independence is distinct from plausible claims

The engineering deliverable is an **executable read-only refusal boundary**, not a fictitious newly commissioned remote guardian. Real independent historical authority would require authentic out-of-band owner-root enrollment, genuinely distinct accountable external custodian, independently pinned guardian key and verified live signature, fresh owner challenge, cross-signed history, protected external monotonic height, globally persistent revocation and spent nonce floors, quorum independence and documented compromise/recovery/rotation process.

The JSON file `G03_R5R_EXTERNAL_HISTORY_AUTHORITY_REQUIREMENTS.json` names **exactly twelve machine-readable admission gates**, each `UNMET_EXTERNAL_EVIDENCE`. These conditions are requirements, not claims that signed test fixtures or an untrusted local file satisfy them. A future externally enrolled proof system would need a separate authenticated interface and independent verification. **R5R does not implement such enrollment or any production admission path.**

The strict `OBSERVE_ONLY_NEVER_ENROLL` local-candidate schema pins R5Q's exact Git SHA, protocol, project, nonce, SHA256 of the actual R5Q three-file historical projection, asserted owner root and guardian label, claimed global height and spent-nonce floor, and twelve explicitly typed Boolean claims. It refuses malformed, duplicate-key, oversized, noncanonical, incorrect source and unknown fields; lower-than-local claimed counters are quarantined. **It does not trust a claim merely because a local file contains 12 true values.** Even a matching, parseable, syntactically complete claim returns `SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE / BLOCK / NOT_AUTHORIZED` with `independently_enrolled_owner_root=false`, `external_provenance_verified=false`, and `independent_history_authority=NOT_ENROLLED`.

Evidence read is grounded in the actual R5Q read-only SQL/history projection and uses R5O's open-file descriptor checks. A second real OS process can swap in a **byte-identical but new-inode JSON bundle** while it is being checked; this is quarantined. The filesystem evidence path remains within the disposable laboratory fixtures; no production credential or system trust store is modified.

## 3. Genuine native cryptographic and rollback controls

The **actual macOS/Ubuntu POSIX research experiment** creates a disposable private Ed25519 key using installed `/usr/bin/ssh-keygen`, signs an actual local candidate with `ssh-keygen -Y sign`, and verifies the detached signature against an actual local `allowed_signers` file using `ssh-keygen -Y verify`. A modified message is genuinely **signature-rejected**. The unmodified genuine signature is **valid as evidence of same-user test-key possession only**, not externally controlled signing authority; an all-twelve-true signed local candidate is still **BLOCK**.

A genuine R5Q history A is committed. A later genuine world B is persisted; the old signed local candidate is rejected as bound to the wrong history SHA. An **actual second process restores all three old files** and the old local candidate matches again, but remains **SELF_ASSERTED only**. Another process can atomically replace the candidate file after R5O opens its descriptor; the identity change is quarantined. A newly spawned OS assessor reads the restored files and the local signed assertion but still returns **NOT_ENROLLED / NOT_AUTHORIZED**, exit **2**.

**Critical distinction:** neither cryptographic verification with same-admin-generated keys nor an apparently matching proof establishes a trust root, independent quorum or globally protected high-water. The locally created OpenSSH key is a genuine algorithmic test fixture but has no independent provenance. No legal/contractual custody is created. The owner Mac replay is first-party execution evidence, not an independent external signature.

## 4. Test and acceptance program

**Exactly five additive research-only files:** real preflight module, **24 adversarial tests**, 12-gate JSON evidence requirements, this dossier, dedicated four-OS workflow. No modifications to earlier R4A–R5Q science/workflows, `.noi`, native kernel/compiler/runtime, main, releases or the owner's original checkout. DRAFT PR remains UNMERGED.

**16 portable controls:** real R5Q three-file local evidence and project/source pin, 12 exact required gates, missing external evidence always refused, all 12 true not admitted, one false incomplete, claimed height or nonce count lower than reality quarantined, wrong local history digest, spoofed project/parent/protocol/GRANT operation, missing/extra gate keys, duplicate/noncanonical/oversized JSON, malicious guardian labels, invalid integer/hex/control types, all-false candidate refused, corrupted real R5M witness refused, real independent Python subprocess assessor exit **2 / NOT_AUTHORIZED**.

**8 POSIX native controls:** genuine local Ed25519 sign+verify and altered-message rejection, self-signed all-true candidate denied, real other-process byte-identical bundle swap quarantined while an original FD remains open, actual history B disqualifies old-world A claim, three-file rollback makes it locally match again but cannot make it independent, deleted bundle refuses without auto-init, fresh assessor subprocess after rollback continues to deny, complete native `--exercise` exit 2 and exactly bounded truthful negative-result keys. Windows deliberately runs **16 portable PASS** and **SKIPS 8 Unix/OpenSSH-native tests** rather than treating them as executed.

Dedicated real CI **4/4 SUCCESS required** macOS Intel, macOS Apple Silicon, Ubuntu, Windows; general NORDOI Test Gate **5/5 SUCCESS required**, with R5Q–R4Y research regressions preserved. The real R4M CodeDirectory/XPC native signature differential must be fully run **serially** on both macOS platforms: six tests, no retries, no waivers, no weakened CDHash gate.

## 5. Source-pinned Intel owner replay

Owner Mac baseline: macOS **14.5 Intel x86_64**, Rust/Cargo **1.98.1**. Verify exact source/observer SHA, private fresh isolated clone, **123 exact pinned Git blob identities** = 24 Rust R4A–R4X + 5 R4Y + 4 R4Z + five each R5A–R5R. Expected **727 regular tests** = 474 Python (24 new R5R + 450 R5Q) + 253 Rust; three Rust intentionally ignored, Rustfmt and strict Clippy PASS, true R5R `--exercise` plus prior live negative controls, R4M native six serial, original owner worktree untouched, `SCRIPT_EXIT=0`. This is a *target*, not an owner PASS, until the actual Mac log is supplied.

Owner ZIP exactly **five files at archive root**; exterior SHA256 first, unique new `mktemp -d`, internal 4/4 SHA256SUMS, Bash syntax check, then run. No overwrites, no production or independent guardian enrollment. **Native admission gates 12 UNMET, PRODUCTION NOT_AUTHORIZED.**

```ini
G03_R5R=RESEARCH_CANDIDATE
LOCAL_ALL_TWELVE_TRUE=NOT_ENROLLED
LOCAL_GENUINE_ED25519_SIGNATURE=KEY_POSSESSION_ONLY
ALTERED_LOCALLY_SIGNED_CLAIM=SIGNATURE_REJECTED
REAL_HISTORY_ADVANCE_STALE_CLAIM=QUARANTINED
JOINT_OLD_HISTORY_ROLLBACK_AND_MATCHING_CLAIM=SELF_ASSERTED_ONLY
REAL_SECOND_PROCESS_ATOMIC_CANDIDATE_SWAP=QUARANTINED
NEW_OS_PROCESS_AFTER_ROLLBACK=NO_EXTERNAL_GUARDIAN
OWNER_OOB_ROOT=NOT_ENROLLED
INDEPENDENT_CUSTODIAN=NOT_ENROLLED
PROTECTED_GLOBAL_HEIGHT=NOT_VERIFIED
GLOBAL_NONCE_FLOOR=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
