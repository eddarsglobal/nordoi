# NORDOI G0.3 R3-R9 — Independent-novice study integrity and descriptive analysis

**RESEARCH ONLY · NON-NORMATIVE · NO HUMAN EVIDENCE YET · PRODUCTION NOT AUTHORIZED.**

## Starting point (confirmed on 2026-11-07)

- R3-R6 original Mac Intel CLI error audit: 10/10 PASS; private R3-R6 JSON stays on the owner's Mac.
- R3-R7 novice help research: owner Mac read-only verification PASS, human comprehension **NOT MEASURED**.
- R3-R8 preparation on owner Mac: 20 private participant packets created, 10 A / 10 B; **no human trials** at that stage. Synthetic GitHub CI was successful.
- Separate unmerged draft PRs: #26 R3-R6, #28 R3-R7, #30 R3-R8; frozen `main` SHA `98da03b2ea1efcf665d205189284aa8b66fda5d3`, frozen production SHA `e2898bf7c511e53636ff28c9fd3efc93165a0215`.

R3-R9 continues **the R3-R8 preregistered protocol unchanged**, with six specified tasks, 20 real independent volunteer novices, two 10-person conditions and 5 blocks of 4. It adds only a **read-only readiness and integrity gate**, plus an optional explicitly **self-reported, descriptive** analysis. It cannot and must not manufacture people, claim human comprehension, infer statistical significance, or substitute machine agents for independent participants.

## Research questions and scopes

- **Primary descriptive outcome:** per-participant share of six scenarios where *both* diagnosis category and safe next action are correct; average within each arm.
- **Secondary:** median seconds per scenario and number of unsafe automatic-grant answers, reported by arm.
- **A:** unmodified exact observed raw NORDOI CLI stderr. **B:** exactly the same stderr, plus the additive R3-R7 educational hint. Both arms retain full help when present. This comparison studies *assisted error interpretation*, not retention or transfer learning.
- No random reallocation after packet delivery, no changes to the scoring key, no interim comparison, no p-values or causal adoption claim. In particular, no suggestion that `run` without permission equals `console-run` with a refused grant.

## Gates

1. **Source pin:** the executable R3-R8 helper must match Git blob `696dc7089f20b449aa77f7a553f7973a2558282f` exactly, before loading it. R3-R9 itself should also be checked by its published blob hash when downloaded.
2. **Read-only study verification:** check the private R3-R8 allocation (exact 20 slots in original order, correct arm design, 5 balanced blocks) and the complete unchanged packet bytes for all 20 against the original private R3-R6 audit transcript. Reject altered/missing packets, incorrect raw stderr, changed hints, modified response template and too-broad POSIX private permissions.
3. **No invented human trial:** missing or blank response file is a *correct, blocked/non-analyzable* state; success at preparation is not success of the human study.
4. **Optional user-supplied responses:** analyze only the exact private file `responses_ACTUAL.json` inside the same study folder, and only when the original R3-R8 scoring tool accepts all 20 complete data rows. No raw personal answers are printed, no names/emails/IPs should be collected. The tool reports only arm-level aggregates and declares human provenance **UNVERIFIED BY SOFTWARE**.
5. **External verification required:** a real human process must confirm independent participant recruitment, genuinely informed consent, actual completion and correct manual timing. Checkbox-style or JSON `true` fields alone are **not** proof. Preserve a private, non-identifying study-operation log outside GitHub; do not import it into CI. If participants are unavailable, stop at technical preparation.
6. **Privacy:** all study folders remain private on the owner's Mac; do not upload audit JSON, individual packets, response files, raw trial transcripts or identifying metadata to GitHub, ChatGPT or any public system. Review published aggregate outputs for residual privacy concerns before publication.
7. **Constitutional constraints:** no `.noi` grammar, compiler, CLI, receipt, runtime, effect or capability change; no performance overhead. Original PRs remain draft/unmerged, no production permission.

## Commands (the R3-R8 `prepare` step has already been performed on the owner's Mac)

Set paths:

```sh
R8="$HOME/Downloads/NORDOI_G03_R3_R8_STUDY_20261107.py"
AUDIT="$HOME/Downloads/NORDOI_G03_R3_R6_20261107T150311Z_90154.json"
DIR="$HOME/Downloads/NORDOI_G03_R3_R8_PRIVATE_20261107"
R9="$HOME/Downloads/NORDOI_G03_R3_R9_GATE_20261107.py"
```

Check readiness only (no human results inferred):

```sh
python3 -B "$R9" selftest --r8-tool "$R8"
python3 -B "$R9" check --r8-tool "$R8" --study-dir "$DIR" --report "$AUDIT"
```

**ONLY after an actual completed, consenting independent 20-person pilot**: make a private copy of the provided `responses_TEMPLATE.json` named `responses_ACTUAL.json` and fill it from the actual observed results, leaving the original template unchanged. Then:

```sh
chmod 600 "$DIR/responses_ACTUAL.json"
python3 -B "$R9" analyze --r8-tool "$R8" --study-dir "$DIR" --report "$AUDIT" --responses "$DIR/responses_ACTUAL.json"
```

The `analyze` command does not certify participants, independence or superior results. For fewer than 20 participants, report **STUDY_INCOMPLETE** without passing partial response files to the analysis tool; do not replace missing respondents with imagined data or convenience-substituted AI testers.

## Validation evidence allowed

- Synthetic R3-R9 selftest in CI on macOS, Ubuntu and Windows: checks structural and negative gates, but contains **no actual human records**. CI may use the exact pinned public R3-R8 code at commit `aacad6626ae6e792a75c05e6f2fc5e233543bc58` and never the private Mac R3-R6 report.
- Owner Mac R3-R9 `check` proves integrity of the *already prepared* R3-R8 study materials and nothing more.
- Verified real human evidence, if later collected, is a separate, independently reviewed gate. The software cannot certify it.

## Exit criteria

`R3_R9_TECHNICAL=PASS` only if pinned-source selftests and Mac material integrity pass. `R3_R9_HUMAN=NOT_COMPLETED` until **actual** consenting independent participants and separately evaluated responses exist. `PEDAGOGICAL_SUPERIORITY=NOT_AUTHORIZED`; `PR=DRAFT_UNMERGED`; `PRODUCTION=NOT_AUTHORIZED`.
