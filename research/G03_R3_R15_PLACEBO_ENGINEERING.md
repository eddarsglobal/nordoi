# NORDOI G0.3 R3-R15 — engineering A/A placebo and A/B merge assessment

Status: experimental, non-normative. PR draft and unmerged.

## Observed Mac Intel results

R3-R13 removed one redundant existence check; its exploratory 7-block A/B pilot improved noop_set by about 22.2%. R3-R14 then ran three balanced 8-block, 30,000-iteration Mac Intel sessions. Their noop_set paired improvements were +22.422%, +18.813%, +19.257%, 24/24 blocks positive. Independent host conditions were not proven. changed_set_flush: +3.23%, +3.49%, +3.49%. denied_capability: -1.51%, -1.11%, -1.08%, even though the edited kernel method is apply_single, not require.

## Experimental procedure

Pin main to 98da03b2ea1efcf665d205189284aa8b66fda5d3 and candidate to a1ae15326eaa837f8994a75fa7cfbe2eccc44976; pin R3-R13 Python tool Git blob dac0109959e10a15764ff54a4da91bf190deff59 and Rust probe SHA256 be6bbd0b74ca8ff12035ee2e8529961285767251f26074d2ce1b4663d723204a. Inspect every tracked file: the only permissible original-file content difference is the exact removal of the redundant existence lookup in src/kernel.rs. Build offline, selftest both real Rust probes, and keep all work in temporary isolated clones.

Primary test noop_set; positive control changed_set_flush; negative-path sentinel denied_capability.

For each case/block, run:
- A/A placebo: exact same baseline executable twice, labelled left/right, 50/50 first/second order. It estimates some process, order and host timing noise; it does NOT control all binary-layout and host-condition factors.
- A/B candidate: baseline versus R3-R13 candidate, 50/50 baseline/candidate first order.
- Interleave A/A and A/B in balanced random order and rotate scenario order per block. Default 12 blocks x 30,000 Rust loop operations per arm per comparison.
- Keep raw total nanoseconds, order flags, per-block paired ns/op, percent differences, host/toolchain metadata and private report.

Interpretation: no automatic merging. If noop_set remains faster while A/A is centered around zero, evaluate denied_capability A/B difference against placebo noise. No-op correctness and denied authority tests must PASS. If the decline is larger than control noise or inconclusive, repeat/refine. An A/A placebo cannot prove time-independent sessions or explain binary-layout effects.

Synthetic-only cross-OS CI does not measure production performance. The original repo is never mutated; R3-R15 only adds research and CI, not source language or kernel semantics.

HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
