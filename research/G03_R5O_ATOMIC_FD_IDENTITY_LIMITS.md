# NORDOI G0.3 R5O — Open descriptor versus path identity under real atomic replacement

**DRAFT / research-only / no native admission.**

## Exact source lineage

R5O is a direct additive successor of R5N PR #105 at pinned, owner Mac Intel validated SHA `b997d84e3a17dec010a3ec801e67ac0ce7f3b47f`: owner macOS 14.5 Intel x86_64 replay **631 regular PASS**, 3 intentionally ignored Rust, 103/103 source SHA-pinned Git blobs, Rustfmt and strict Clippy PASS, `SCRIPT_EXIT=0`; prior R5N dedicated four-OS CI 4/4 and NORDOI general Test Gate 5/5 SUCCESS. PR #105 remains DRAFT and unmerged.

R5O is limited to **five additive research files**: real open-FD/path identity Python lab, 24 adversarial tests, actual absent-external-authority JSON, this dossier, and a dedicated four-OS GitHub Actions workflow. No changes to R4A–R5N science, old workflows, main, native compiler/kernel/runtime, `.noi`, original owner checkout, release, or production.

## Actual problem and negative controls

R5N's three repeated pathname observations are not an atomic transaction. An OS file descriptor opened before an atomic file replacement may continue to refer to an older inode while the filesystem pathname now refers to a **different inode containing identical bytes**. Simply comparing SHA-256 bytes (R5M) is insufficient to identify this transient pathname swap.

R5O creates a canonical `DENY_ONLY_FD_IDENTITY_USE` lab ticket binding the exact frozen source SHA and protocol, an actual positive-height R5L denial chain head, current real R5L SQLite file SHA-256, current separately retained R5M JSON witness SHA-256, disposable nonce and **device/inode numbers** for both actual files. This is an identity-binding research ticket, **not** a signed independent authority, native permission or external enrollment certificate.

On examination, R5O uses actual `os.open(O_RDONLY | O_CLOEXEC | O_NOFOLLOW)` where supported, holds two real file descriptors across the test, checks `os.fstat`, `os.lstat`, regular-file mode, original size/time/mode/inode/device, actual bytes SHA-256 from the descriptor, and unchanged R5M read-only SQLite and witness evidence. It repeats the comparisons before and after a deterministic interleaving callback and once before a denial-only use. Even a consistent snapshot is forwarded only to unchanged R5N **`BLOCK / NOT_AUTHORIZED`** SQLite denial journaling; a repeated nonce is rejected.

The controlled callback starts a **distinct actual OS process** that writes a new private temporary file with **identical byte content** and uses **real `os.replace`** to atomically change the target path to the fresh inode while the verification process still holds its original descriptor. The lab rejects the mismatch even though the content digest is unchanged. An independent experiment confirms the original FD still reads the old inode and SHA, whereas the pathname points to the new inode. Both primary SQLite and witness replacement are covered; versions placed just before denial journaling are quarantined. Missing journal/file or malformed ticket cannot recreate a genesis silently.

**Do not promote this proof:** it is not a kernel-enforced immutable FD, an atomic cross-file snapshot, a protected filesystem mount, a real external guardian or universal race-free security boundary. The original fd is an ordinary file descriptor under the same process administration; the same admin can restore all earlier valid files and ticket/nonce history and thereby erase globally visible high-water. A file might be changed after the final check; no admission is ever granted. An attacker with root/OS privileges can alter metadata and storage. Inode numbers can be reused; stable values are not a cryptographic identity, and byte-identical file replacement after an observation can evade a future check if all evidence is replaced together.

## Test program (24 new tests)

**16 portable actual-file cases (Windows, Linux, macOS)**: legitimate file descriptor/path match can record only denial; ticket's inode claims reflect actual R5L SQLite and R5M witness files; duplicate nonce blocked, distinct nonce still denial; missing primary, missing witness, missing R5L denial journal all refuse without silent reinit; malformed/oversized/duplicate/noncanonical JSON, wrong protocol/project/parent, GRANT/UNREVOKE/RECOVER/APPROVE operation, invalid SHA-256 or nonce, boolean/zero heights, boolean/negative inode/device, forged device/inode claim, and a real child CLI deny-only assessment with exit **2**.

**Eight genuine native POSIX experiments (macOS Intel, Apple Silicon, Ubuntu)**: original open file identity matches but still refuses admission; actual second OS process atomically replaces SQLite with byte-identical new inode immediately after file descriptors were opened; same-byte witness replacement similarly refused; witness/primary replaced before the final local denial commit, quarantined; old file descriptor still refers to previous inode despite new pathname; dual same-owner new-inode matching files can generate a *new* locally consistent ticket and still only record a denial, illustrating missing independent antirollback; full real `--exercise` returns **2 / NOT_AUTHORIZED**, with old FD identity and both real replacements demonstrated.

Windows runs the **16 portable tests** and SKIPS the 8 Unix-specific open-FD replacement experiments, not counted as executed.

## Acceptance and governance

- Dedicated GitHub CI actual **4/4 SUCCESS** for macOS Intel, Apple Silicon, Ubuntu, Windows; NORDOI Test Gate actual **5/5 SUCCESS** at frozen exact R5O SHA. Preserve R5N–R4Y regression test suites and platform-specific skip accounting.
- Owner macOS 14.5 Intel x86_64, Rust/Cargo 1.98.1, exact-source isolated fresh clone, **108/108 SHA-pinned Git blob identities** = 24 Rust R4A–R4X + five R4Y + four R4Z + five each R5A–R5O. Expected **655 regular PASS** = 24 new R5O + prior 631 R5N; 3 Rust intentionally ignored, Rustfmt and strict Clippy PASS, actual R5O real native `--exercise`, exact source/observer SHA pins, clean checkout and `SCRIPT_EXIT=0`. Owner original worktree untouched. Do not claim owner replay before owner transcript.
- Owner ZIP exactly five files **at archive ROOT**; verify outer SHA256, then unique `WORK="$(mktemp -d "$HOME/Downloads/NORDOI_R5O_RUN_XXXXXX")"`, `unzip -q "$ZIP" -d "$WORK"`, `cd "$WORK"`. No overwrite prompts, no guessed folder.
- No real independent guardian, authenticated owner OOB trust root, externally protected monotonic high-water, globally durable spent-nonce register or independent key-rotation/recovery operator enrolled. **All 12 native admission gates UNMET, production NOT_AUTHORIZED.** Keep PR DRAFT/UNMERGED. The research is not deployable native access authority.

```ini
G03_R5O=RESEARCH_CANDIDATE
REAL_OS_ATOMIC_SAME_BYTES_REPLACE=OBSERVED_WITH_OLD_FD_HELD
FILE_DESCRIPTOR_AND_CURRENT_PATH_DIVERGENCE=DETECTED_AND_BLOCKED
MATCHING_FILE_IDENTITIES=DENIAL_ONLY_NOT_ACCESS
ALL_SAME_ADMIN_EVIDENCE_RESTORED=NOT_GLOBALLY_DETECTABLE
INDEPENDENT_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
