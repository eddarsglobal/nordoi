# NORDOI G0.3 R4Y — Real independent guardian custody qualification

**RESEARCH-ONLY / EVIDENCE INTAKE. NO real external administrator has been enrolled. NO production authority.**

## Exact provenance

Stacked on G0.3 R4X PR #89 commit `d711d6054fea2acfd52921b4ccc7cba8bf2d9b6e`. Owner supplied a macOS Intel 14.5 transcript with **253 regular R4A–R4X tests PASS**, **0 FAIL**, **3 deliberately ignored**, **24/24 source Git blob pins**, Rustfmt and Clippy strict PASS, isolated HEAD unchanged, `SCRIPT_EXIT=0`. R4X GitHub dedicated CI completed **4/4 SUCCESS** and general Test Gate **5/5 SUCCESS**. These are real research tests, *not* proof of independently administered custody.

R4Y introduces only read-only Python 3 standard-library candidate evidence inventory validation, 18 adversarial tests, an honestly empty inventory, this dossier and one CI workflow. No changes to previous Rust science, public `.noi`, compiler, kernel, runtime, original owner worktree, existing workflows, main or production.

## Six required externally verified gates

| Gate | Independent real-world proof needed | R4Y evidence status |
|---|---|---|
| G1 | Identifiable third-party guardian, actually verified separate administrator and no unilateral original publisher control | NOT VERIFIED |
| G2 | Owner-approved public root fingerprint authenticated *outside* GitHub, prior to reliance on network key material | NOT VERIFIED |
| G3 | Independently protected monotonic append-only high-water, rollback/fork detection, and durable one-use challenge consumption | NOT VERIFIED |
| G4 | Protected deployed verifier and root pin, resistant to unilateral primary-admin replacement | NOT VERIFIED |
| G5 | Explicit revocation, compromise, rotation, outage, split-brain and authorized recovery procedures | NOT VERIFIED |
| G6 | Externally authenticated, independently reproducible evidence lineage joining operator, keys, signed receipts and witness | NOT VERIFIED |

The R4Y reader is **not** an authentication mechanism. All candidate fields, references and digests are untrusted *claims*. A filled inventory, an ostensibly independent source label, two locally generated SSHSIG keys, a same-owner GitHub branch or an all-green CI never qualifies the gates. A real external enrollment can only be evaluated after actual separate authenticated operator onboarding and security review, outside this research-only tool. **There is deliberately no positive verdict path.**

## Candidate inventory

The committed inventory is empty and truthful: `research/G03_R4Y_EXTERNAL_CUSTODY_EMPTY_INTAKE.json`. Each future candidate reference (not the underlying sensitive evidence) has exact fields `gate`, `source`, `reference`, and `sha256`. Sources are explicitly labeled `REPOSITORY_ONLY`, `OWNER_LOCAL_RECORD`, `CLAIMED_OUT_OF_BAND`, `CLAIMED_INDEPENDENT_CONTROLLER`, or `CLAIMED_PROTECTED_WITNESS`. These labels have **no authority**. Do not check in private keys, personal records, secrets, recovery material or raw OOB correspondence.

The reader processes a single local regular file (max 32 KiB), rejects symbolic links, invalid UTF-8, unknown fields, duplicate JSON keys, duplicate gate entries, noncanonical digests and unsafe identifiers. It makes no network calls or writes and prints a JSON `BLOCK` verdict, exit code **2**, even for fully claimed evidence. It never reports an accepted external guardian.

From repository root:

```sh
python3 -B -m unittest discover -s research -p 'test_g03_r4y_external_custody_intake.py' -v
python3 -B research/g03_r4y_external_custody_intake.py --manifest research/G03_R4Y_EXTERNAL_CUSTODY_EMPTY_INTAKE.json
# Second command intentionally returns 2, meaning BLOCK.
```

## Qualification gates for this research milestone

1. Candidate branch must be pinned directly to R4X unchanged parent, draft and unmerged.
2. 18 adversarial Python tests PASS in real OS runners: macOS Intel, Apple Silicon, Ubuntu and Windows. The default inventory returns BLOCK / exit 2 (this is expected, not a test failure).
3. All four dedicated CI jobs and all five general Test Gate jobs must *actually exist and succeed*; incomplete histories remain incomplete.
4. Owner Mac Intel must independently replay exact candidate commit and its source pins, run real Python and unchanged 253 R4A–R4X Rust regular tests with exactly 3 ignored, then Rustfmt and Clippy strict. Do not claim owner execution before receiving it.
5. R4Y cannot enroll a custodian, authorize external IO, enable production, close 12 native gates or claim global antirollback.

## Real-world enrollment prerequisites

An unrelated independent operator must genuinely exist and accept custody, under separate administrative control. The owner must independently verify its identity and key fingerprint via authenticated out-of-band communication, outside GitHub/repository control. The guardian must provide a protected durable monotonic service, verifiable signed current heads, non-equivocation and replay protection, independent recovery paths and a protected verifier trust root. Independent audit of *actual* receipts is required before a later governance decision could even consider admission.

```ini
G03_R4Y=RESEARCH_CANDIDATE_INTAKE_ONLY
EXTERNAL_CUSTODIAN_IDENTITY=NOT_VERIFIED
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
DURABLE_NONCE_CONSUMPTION=NOT_IMPLEMENTED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
