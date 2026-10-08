# NORDOI Production Profile 2 Progress

## Baseline

Production Profile 1 is certified and frozen at `v1.7`.

## P2.1 — Capability-Secured Observable I/O

Candidate milestone introducing one explicitly authorized observable console channel.

Target proof set:

- explicit `effect ConsoleWrite;` source declaration;
- exact `Capability::ConsoleWrite` host grant;
- deny-by-default and zero stdout bytes on denial;
- 4096-byte decoded UTF-8 quota;
- deterministic plan and receipt SHA-256 identities;
- host/path/time independent canonical receipt;
- no changes to kernel/runtime/NAIR Profile 1 encodings;
- Release Gate PASS;
- 20/20 focused tests;
- exact-SHA cross-platform CI;
- immutable annotated `p2.1` tag only after full certification.

P2.1 is the first Production Profile 2 milestone; no completion percentage for the entire Profile 2 is claimed yet because the profile scope remains intentionally incremental.
