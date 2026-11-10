# NORDOI G0.3 R4Z — Offline external guardian enrollment handoff preflight

**RESEARCH-ONLY. OPERATOR ENROLLMENT NOT PERFORMED. NO ADMISSION TO PRODUCTION.**

## Baseline and scope

This draft candidate is stacked directly on G0.3 R4Y PR #90 exact source commit `be462ead174dec4332882cce7620448a9db41d34`. Owner Mac Intel 14.5 R4Y FIX1 terminal replay: 18/18 Python PASS, 253/253 regular Rust PASS, 3 deliberately ignored, 29/29 combined Git blob pins, Rustfmt/strict Clippy PASS, `SCRIPT_EXIT=0`. R4Y dedicated GitHub CI 4/4 SUCCESS and Test Gate 5/5 SUCCESS.

**R4Z solves a limited interoperability question:** how to format a fresh owner-issued offline enrollment challenge, bind it to one candidate response, and mechanically reject stale/substituted/noncanonical request references while **always withholding authorization**. A replayable pair of metadata files does NOT verify a genuine signature, operator identity, authenticated owner out-of-band approval, protected monotonic remote witness, consumed challenge state or external recovery governance. Those remain unresolved.

## Proposed candidate exchange, not verified authority

The owner-side request contains exact canonical JSON fields:
- `protocol=NORDOI-G03-R4Z-OFFLINE-HANDOFF-V1`, project and frozen R4Y origin commit;
- a publisher-controller **identifier claim** and SHA256 public root **reference claims** for owner and candidate, not certified pins;
- an OS-generated random 256-bit nonce, issued/expiry Unix seconds, maximum TTL **300 seconds**, epoch floor at least **4**;
- a SHA256 digest binding the canonical request bytes before `binding_sha256`.

A candidate response contains only untrusted references to the request binding, candidate key, signature digest, owner out-of-band receipt digest, witness head digest, claimed separate administrative-controller ID and claimed witness epoch. **No signature verification takes place in this R4Z module.** Hashes protect formatting/integrity comparisons but are not attested signatures; controller IDs and witness epochs can be self-declared.

The tool can parse local bounded regular files (8 KiB max, no symlink) read-only and produces `status=BLOCK`, `admission=NOT_AUTHORIZED`, and exit code `2` for *all* inputs. Even a perfectly shaped reply with distinct claimed controller IDs and fresh epoch is blocked as `INDEPENDENT_PROVENANCE_NOT_EXTERNALLY_VERIFIED`.

## Research adversarial controls (24 standalone Python cases)

1. Canonical request recognized but no receipt grants nothing.
2. Fully populated claim bundle still blocks.
3. Fresh generator draws nonces from OS cryptographic RNG.
4. Old receipt bound to another challenge is rejected.
5. Guardian-key substitution is rejected.
6. Same administrator identified even with case changes is rejected.
7. Different operator name alone is unverified.
8. Lower witness epoch than externally expected floor is rejected.
9. Local epoch floor downgrade refused.
10. Expired request refused.
11. Future issue beyond clock tolerance refused.
12. TTL 0, negative, 301 or boolean refused.
13. Wrong protocol, project or parent commit refused.
14. Changed nonce invalidates binding.
15. Same owner and candidate pin refused.
16. Duplicate or additional fields refused.
17. Noncanonical JSON serialization refused.
18. Invalid UTF-8 and non-finite JSON refused.
19. Invalid candidate signature digest refused.
20. Boolean/non-integer epoch refused.
21. Real command-line exit two and read-only input.
22. Symlink entry refused.
23. Absent input file refused.
24. No successful admission branch on candidate references.

These tests exercise claim integrity and refusal, **not** external administrative separation. The test uses fixture-only digest strings with no real guardian private keys. No sensitive identity documents or secrets are recorded.

## Still required outside this lab

G1: real separately controlled external operator authenticated and approved.

G2: independent owner-approved out-of-band root/fingerprint obtained before relying on candidate-supplied keys.

G3: separately protected durable, monotonic and non-equivocating witness state, with consumed nonce ledger and rollback/fork detection.

G4: protected installed verifier/root pin not replaceable unilaterally by the primary publisher.

G5: independently reviewed revocation, rotation, outage, key-loss and recovery policy.

G6: actual independently reproducible provenance and real third-party signed receipts verified against their anchored roots.

**No connected independent custodian, no authenticated external evidence, no protected global anti-rollback. All 12 NORDOI native admission gates remain UNMET. No external IO grant; no production.** A further milestone may evaluate real evidence only when it exists and after separate safety review.

## Acceptance

- Exact stacked draft PR; new research-only Python reader, tests, this dossier, and dedicated GitHub workflow. No previous R4A–R4Y research sources, existing workflows, `.noi`, compiler, runtime, kernel, original owner tree, main or production changed.
- Four actually created dedicated CI jobs: macOS Intel, macOS Apple Silicon, Ubuntu, Windows, all PASS; full general Test Gate 5/5 PASS. Do not confuse a queued/missing job with a pass.
- Owner macOS Intel pinned isolated checkout of **exact R4Z commit**, verified all **24 R4A–R4X old Rust test blobs, 5 R4Y blobs and 4 R4Z additive source blobs**, real 24/24 new Python tests + 18/18 R4Y tests + 253/253 original Rust tests (3 deliberately ignored), Rustfmt and strict Clippy PASS, script exit zero. Owner original worktree unmodified.
- The only valid research verdict after those checks: `PASS_WITH_CONTROLS_RESEARCH_ONLY`. Never `PRODUCTION_AUTHORIZED`.

```ini
G03_R4Z=RESEARCH_CANDIDATE
ACTUAL_EXTERNAL_OPERATOR=NOT_ENROLLED
OWNER_OOB_PIN=NOT_VERIFIED
EXTERNAL_SIGNATURES=NOT_VERIFIED_BY_R4Z
DURABLE_NONCE_LEDGER=NOT_IMPLEMENTED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
