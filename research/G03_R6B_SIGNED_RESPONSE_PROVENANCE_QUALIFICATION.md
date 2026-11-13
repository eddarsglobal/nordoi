# NORDOI G0.3 R6B — Exact signed external-witness response provenance (deny-only)

**DRAFT RESEARCH CANDIDATE — NO EXTERNAL ADMINISTRATIVE AUTHORITY ENROLLED.** R6B is a strictly additive child of owner-validated R6A commit `b5ce4f793b6d1a5699955998478bb93bb968c711` (PR #118), which achieved 943 owner Mac Intel regular tests PASS, 168/168 Git source identity pins PASS, real native R4M XPC/CDHash 6 PASS serial, Rustfmt/Clippy strict PASS, `SCRIPT_EXIT=0` and 9/9 GitHub CI. The parent PR is still DRAFT and unmerged.

## Scientific question

A real signed candidate response to a **cryptographically fresh R6A random challenge** may be byte-exact and mathematically authentic, but **who controls the signing key, the alleged external operator, the owner out-of-band trust root and the historical high-water ledger?** R6B directly distinguishes **(1)** canonical response integrity, **(2)** correct binding to a particular R6A challenge and its exact R5Z signed witness history, and **(3)** independent administrative signer provenance. Only (1) and (2) can be qualified with same-admin laboratory hardware. No valid local SSH signature, second process, remote-looking name, hostname, or hash can establish (3).

## Implemented experiment

- A new canonical UTF-8 JSON response type `NORDOI-G03-R6B-SIGNED-RESPONSE-PROVENANCE-V1` is pinned to R6A's frozen owner-validated commit. It binds the exact SHA-256 of the R6A request, OS-random challenge, exact R5Z signed local witness SHA-256, its height, ordered nonce-set SHA-256, locally recorded revocation floor, candidate key pin, and a bounded local timebox.
- Strict parser rejects duplicate JSON keys, noncanonical formatting, nonfinite numbers, malformed digests, oversized input, Boolean/negative/fractional counters, missing fields, false source commits, privileged grant operations and inconsistent timestamps. It requires an exact matching R6A request and byte-verified R5Z witness. Cross-request replay is refused.
- The real POSIX/OpenSSH negative-control chain creates two genuine distinct Ed25519 keys under **one actual OS administrator**, two real child-process OS observations with different PIDs and equal UID/EUID, a genuinely signed R5X receipt, an actual R5Y SQLite spent-nonce commit, a genuinely signed R5Z local witness, a genuinely signed R6A challenge, and a genuine **R6B response** signed and verified with domain-separated `ssh-keygen -Y sign/verify` in namespace `nordoi-r6b`. The result must always remain **BLOCK/NOT_AUTHORIZED**.
- A locally recorded spent challenge or candidate-key revocation refuses a response. These are **not protected remote nonce/revocation floors**. Local clock validation is not independently attested time.
- A claimed operator identifier, independent administrator domain, owner OOB key fingerprint, remotely held history root, remote nonce floor or revocation floor is untrusted, even if covered by a valid same-admin signature. The lab explicitly proves a true signature on a false claim can remain mathematically valid without qualifying external provenance.
- No network calls, no real external enrollment, no secret export, no user-switching, no privilege escalation and no native grant path are implemented.

## Twenty-four adversarial cases and cross-platform CI

**16 portable tests:** exact frozen protocol, source, operation and hashes; malformed/duplicate/noncanonical/oversized JSON; false grant; invalid key/root IDs and counter types; stale/future untrusted timebox; different R6A request and changed nonce; altered R5Z witness height/nonce set/revocation floor; missing signature; claimed external identity and root; locally spent challenge, locally revoked key and malformed local state.

**8 real POSIX/OpenSSH tests:** genuine distinct real OS processes and Ed25519 keys; correct signature always BLOCK; tampered signed body; wrong allowed signer; wrong SSHSIG namespace; *genuinely signed* same-admin false external operator claim refused; cross-challenge replay; locally revoked real key; full genuine native `--exercise` returning an intentional **exit 2**. Windows executes only 16 portable cases and deliberately skips eight POSIX-native cases; no OS evidence is invented for Windows.

Dedicated GitHub CI targets **macOS Intel, macOS Apple Silicon, Ubuntu and Windows**, reruns R6B and all R5A–R5Z, R6A, R4Y and R4Z negative research controls, and executes the **original genuine R4M signed XPC/CDHash native six tests serial once** on both macOS runners without retry/waiver. Independent general NORDOI Test Gate is also required. No prior source or production runtime is changed.

## Owner Mac Intel exact-source replay targets (not yet measured)

Run only in a fresh SHA-256-verified kit with a fresh isolated Git clone, frozen PR head and exact parent. Expect **five additive R6B source paths**, **173/173 exact Git blob source pins** (=168 from R6A plus five), and **967 regular tests =714 Python +253 Rust**, with **three intentionally ignored Rust cases**, original true R4M XPC/CDHash six serial PASS, strict rustfmt and Clippy PASS, exact HEAD before/after, clean clone, original worktree untouched, `SCRIPT_EXIT=0`. These figures are **targets**, not owner Mac results, pending an actual Terminal replay.

## Governance and Bureau qualification

The twelve externally independent native admission IDs from R6A are all retained with `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`. Independent owner OOB authentication, genuinely separate administrative operator custody, independently pinned guardian key, independently signed OS/host provenance, protected external nonce/revocation/key high-water floors and quorum require independently sourced evidence outside the same-admin laboratory. Formal RoadMap Bureau certification is not a consequence of passing negative controls. PR remains DRAFT, no main merge, native admission, privileged operation or production release.

```ini
G03_R6B=RESEARCH_CANDIDATE
PARENT_R6A=b5ce4f793b6d1a5699955998478bb93bb968c711
SIGNED_SSHSIG=VALID_SAME_ADMIN_KEY_POSSESSION_ONLY
CHALLENGE_RESPONSE=EXACT_SOURCE_BOUND_DENY_ONLY
REAL_EXTERNAL_SIGNER=NOT_ENROLLED
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
PROTECTED_GLOBAL_NONCE_AND_REVOCATION_HISTORY=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
EXPECTED_OWNER_REGULAR_TESTS=967_PASS
EXPECTED_SOURCE_GIT_BLOBS=173/173_PASS
OWNER_MAC_INTEL=PENDING
PR=DRAFT_UNMERGED
BUREAU_FORMAL_CERTIFICATION=PENDING
PRODUCTION=NOT_AUTHORIZED
```
