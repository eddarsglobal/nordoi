# NORDOI G0.3 R6F / BUREAU-001 — E8

## Two genuine local SQLite forks, one signed nonce, incompatible local histories

**E7 prerequisite:** exact head `81c84f116b3ae6003b84c335e47af4d1b84637ef`; 4/4 specialized run [44033036777](https://github.com/eddarsglobal/nordoi/actions/runs/44033036777), 5/5 Rust gate [44033036774](https://github.com/eddarsglobal/nordoi/actions/runs/44033036774). This research adds only new files and DOES NOT alter native Rust, R1–E7, RoadMap or Bureau.

### What E7 did not show

E7 protected one shared SQLite file against local concurrent duplicate insertion, but a copy of the same original empty database creates **two separate atomicity domains**. E8 uses actual closed SQLite files A/B copied from one original, true distinct OS child processes and verification of one genuinely Ed25519 SSHSIG-signed local receipt. Each child makes one valid *local research record*, for the same nonce, without native effects. After both return BLOCK, a local reconciliation notices that both branch event records refer to the same signed nonce and head but different fork identities.

The event SHA is deliberately only a local digest, NOT a protected signature or independent witness. Reconciliation detects a conflict only if **all relevant fork evidence is supplied**. Withholding one branch causes an explicit INCOMPLETE BLOCK in this bounded two-fork lab, but if a malicious administrator hides the existence of the other branch altogether, local evidence **cannot** prove completeness. Two local successes do not imply two native authorization grants, and conflict detection is not global prevention.

### Automated, source-anchored evaluation

- 18 cross-platform source, policy and actual SQLite/file integrity tests; six POSIX genuine OpenSSH/real OS-process tests explicitly skipped on Windows.
- `--audit` and `--exercise` return **exit 2 / BLOCK** when their research assertions succeed; error is exit 3. No positive native admission path exists.
- On POSIX: six real tests plus complete exercise; original E2–E7 regression suites, RoadMapGuardian, 99 original R1 source blobs. On both macOS machines: genuine signed R4M XPC/CDHash six tests serial, no waiver/retry.
- Mac Intel evidence is preserved as machine JSON with SHA256 and GitHub artifact, bound to PR event exact HEAD. General Rust CI must remain green.

### Bureau contradiction review

ATLAS verifies separate SQLite atomicity domains; FERRUM checks actual OS and XPC boundaries; BOUNDARY distinguishes true child PIDs and database paths; CIPHER verifies the actual local E6 SSHSIG; LEDGER tests event overlap and incomplete evidence; CAPSTONE ensures local commits do not grant effects; ADVERSARY withholds a branch; VERITY checks exact PR SHA and CI outputs; GUARDIAN retains all twelve native gates. These are nine *functions*, not nine actually commissioned independent agents or independent signatures.

### Irreducible boundary

```ini
E8_TWO_LOCAL_SQLITE_FORKS=RESEARCH_HYPOTHESIS_PENDING_EXACT_HEAD_CI
E8_BOTH_LOCAL_ATOMIC_COMMITS=TO_BE_OBSERVED
E8_RECONCILIATION_CONFLICT_DETECTION=TO_BE_OBSERVED
E8_WITHHELD_BRANCH_EVIDENCE=FAIL_CLOSED_WHEN_KNOWN
E8_ALL_POSSIBLE_BRANCHES_EXTERNALLY_KNOWN=NOT_VERIFIED
HUMAN_TESTERS=0_PERMANENTLY_FORBIDDEN
INDEPENDENT_SPECIALIST_REVIEWS=0
INDEPENDENT_EXTERNAL_RECEIPTS=0
EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_AUTHENTICATED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
R6F=IN_PROGRESS_UNCERTIFIED
R6G=LOCKED_BY_SEQUENCE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

No human tests, owner key export, network, merge, deployment, external enrollment, admission or production. **E8 must not be called PASS until all exact-head CI is verified.**
