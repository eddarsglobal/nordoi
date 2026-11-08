# NORDOI G0.3 R3-R21 — Formal engineering decision record

**Decision classification: ACCEPT_WITH_CONTROLS_FOR_OWNER_MERGE_DECISION.**
**This is an engineering recommendation, NOT owner approval to merge or deploy.**

## 1. Decision and competing options

Three engineering choices were evaluated:

1. REJECT the R3-R13 optimization because the denied-capability micro-path regressed: conservative but sacrifices a large measured no-op-set gain.
2. ACCEPT without qualifications: rejected because the observed -1.3758% denied-capability change is real within the R3-R16 FIX1 experiment and root cause remains unproven.
3. ACCEPT WITH CONTROLS **for a separate owner-directed merge decision**: selected. Preserve performance exception, quantify scope/uncertainty, require strict source pin, security/CI recheck at merge time, and no unapproved deployment.

## 2. Pinned evidence summary

- Main baseline: 98da03b2ea1efcf665d205189284aa8b66fda5d3; R3-R13 candidate PR #40: a1ae15326eaa837f8994a75fa7cfbe2eccc44976. PR #40 is draft/unmerged as inspected on 2026-11-08.
- Exactly one private-kernel code delta: remove redundant AtomicKernel::apply_single ensure_exists before a subsequent self.get lookup, with ownership control retained. Five supporting files added; 659 shared files unchanged; zero removed. R3-R20 checks verified exact original/candidate kernel SHA256.
- Owner Mac Intel R3-R20 private JSON uploaded in conversation and independently hashed: SHA256 b2aa841c9109cd23be5967fa534426cdf05541b9665c91240db752bbb17057d5. It recorded 2,496 baseline and 2,499 candidate release Rust tests PASS with no failures, cargo fmt PASS, clippy with -D warnings PASS, external Rust safety contract PASS for both. R3-R20 performed no new performance measurements.
- GitHub R3-R20 governed integration CI run 42475969873 completed SUCCESS for Ubuntu, macOS, Windows; general NORDOI test gate run 42475970206 completed SUCCESS.
- Historical R3-R16 FIX1 owner Intel Mac paired A/B results: noop_set +23.1812% (12/12 positive), changed_set_flush +3.9061% (10/12), denied_capability -1.3758% (2/12 positive; approximately 0.17 ns/op slower). Controls balanced. These are observed microbenchmarks, not a universal cross-platform performance guarantee.
- R3-R17/18/19 disassembly: witness/require normalized textual assembly agrees; contains has matching instruction text after allowing for a relocated memcmp stub target. Machine byte equality and the cause of denied-capability slowdown were **not** proven. Code layout/alignment changes remain plausible, not established causal claims.

## 3. Risk and constitutional priorities

Performance exception OPEN: a measured small regression on a security-denial micro-path is accepted as a **documented exception for evaluation**, not silently characterized as zero. No observed correctness/security failure in tested scope. Avoid assuming future hardware/workloads will behave identically. Overall measured gains are compelling in the no-op atomic write path; a later owner-directed merge decision must explicitly weigh how frequently the denial path runs.

NORDOI requires extreme public-language human simplicity and maximal speed/lightness, without weakened authority checks. The candidate changes neither the language surface nor security policy. Measured speed improvements in the main changed code path do not automatically excuse a hidden regression elsewhere; preservation of the exception and follow-up controls are mandatory.

## 4. Mandatory bounded release gate for any later merge

- Verify branch main still equals baseline SHA and PR #40 still equals candidate SHA (or start a *new* review for changed references). Re-run applicable checks after any change; never assume historical CI remains valid for a changed tree.
- Confirm strict diff/allowlist and external deny/grant/ownership/transaction security controls remain PASS.
- Ensure owner explicitly acknowledges the -1.3758% (~0.17 ns/op) denied-capability observation and separately authorizes a specific merge operation; this decision record is not that approval. No third-party evaluator/signature or human study is required.
- If integration is authorized later, retain rollback/traceability and specifically monitor denied-path performance on the target runtime. A proven material regression requires an engineering response or revert; no numerical universal production threshold is claimed here.
- Production has a **separate** NOT_AUTHORIZED boundary, even after a future merge.

## 5. Actual R3-R21 verification scope

The additive R3-R21 files are an evidence manifest, fail-closed policy checker, this record and a multi-OS governance selftest workflow. CI selftest checks the manifest and deliberately tampers with the performance exception and authorization fields; it **does not** rerun the owner Mac private report, benchmark, compile the kernel, or claim owner approval. The owner Mac verifier optionally checks the **exact** known R3-R20 report bytes against its SHA256 and validates the nested test/security/source/exception fields, writing an immutable 0600 custody receipt. No original repo or runtime changes, no GitHub merge command.

## FINAL

ENGINEERING_DECISION=ACCEPT_WITH_CONTROLS_FOR_OWNER_MERGE_DECISION
PERFORMANCE_EXCEPTION=OPEN_EXPLICIT
DENIED_CAPABILITY_ROOT_CAUSE=NOT_ESTABLISHED
MERGE_AUTHORIZATION=NONE
OWNER_MERGE_APPROVAL=NOT_GIVEN
PRODUCTION=NOT_AUTHORIZED
HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED
