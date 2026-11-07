# NORDOI G0.3 R3-R10 — Independent novice session operations

**NON-NORMATIVE RESEARCH CANDIDATE · NO HUMAN EVIDENCE · NO PRODUCTION AUTHORITY.**

## Source and scope

- Source baseline `main`: `98da03b2ea1efcf665d205189284aa8b66fda5d3`.
- Frozen production: `e2898bf7c511e53636ff28c9fd3efc93165a0215`.
- R3-R6 diagnostic transcript: 10/10 Mac Intel checks passed; private JSON must stay off GitHub.
- R3-R7 beginner guidance: technical checks passed; **human readability never measured**.
- R3-R8: 20 existing private study packets, 10 A / 10 B, five balanced blocks, prepared on the owner's Mac and verified in R3-R9. The study has **zero completed independent human sessions** according to the latest report.
- R3-R9: existing read-only packet gate and descriptive scoring; no actual participant responses present. PRs #26, #28, #30 and #32 remain distinct draft/unmerged unless separately reviewed.

R3-R10 adds a **private operator packet** to administer the existing, preregistered A/B pilot. No syntax, engine, compiler, I/O permissions, capability gates, output text, performance, or runtime changes. **Do not regenerate or reassign R3-R8 allocation.**

## Pilot remains unchanged

**Six cases:** missing console grant, undeclared ConsoleWrite effect, misspelled `emits`, invalid CLI option with original help, missing file path, and legacy `run` route. Arm A reads original unmodified CLI error; arm B reads the **same raw error** plus the R3-R7 research hint. Same language, instructions, order and answer choices. No live code execution or granting permissions. This is a *hint-available interpretation pilot*, not a test of memory or long-term learning.

**Enrollment target:** 20 genuinely independent, consenting, unexposed adult beginners, in sequential order of previously randomized slots P001–P020. Do not include the team, previous testers, AI agents, close collaborators, or people shown answers beforehand. Participation is optional; no incentives or coercion implied. If participants are unavailable, leave human results **NOT COMPLETED**.

**Main endpoint (R3-R8 unchanged):** for each real participant, fraction of six cases where both the category and the next action match the preregistered key, averaged by arm. Secondary endpoints: median task seconds and unsafe automatic-grant choices. Descriptive pilot only, **no p-value or superiority/adoption verdict**. Technical validation is not human comprehension.

## Human consent and researcher procedure

1. Recruit an independent adult volunteer. Screen prior NORDOI exposure *before* revealing the tasks. Avoid collecting names, contacts, exact age, devices, IPs, employer or sensitive free text.
2. Read the identical `CONSENT_SCRIPT.md` from the local R3-R10 operator folder in English for both groups. Ask explicitly for voluntary informed consent and allow withdrawal at any point without penalty. **No consent → no packet, no completed trial.**
3. Assign the next unoccupied sequential slot without seeing future results. Use ONLY the matching Pxxx.md from the existing private R3-R8 folder. Do not re-randomize, edit, replace, shorten the original lengthy CLI help or cross-contaminate A/B packets.
4. Ask the participant to read the six static cases and select one category and one next action for each. Do not provide extra explanations, run NORDOI, request permissions, or disclose the key. Record elapsed seconds externally and consistently. No recordings or screenshots needed.
5. Record answers in an **offline private copy** of `responses_TEMPLATE.json` named `responses_ACTUAL.json`, using the R3-R8 schema and answer codes. Separately maintain a private, **non-identifying** `session_log_ACTUAL.json` based on R3-R10 `session_log_TEMPLATE.json`, filling checks from genuine observed events only. Never write identities, answers, session receipts or study raw material into GitHub or ChatGPT.
6. If a volunteer declines, withdraws or leaves trials incomplete, **do not silently replace** the participant or fabricate six answers. Mark a protocol deviation and stop the pre-registered complete-case gate. Record exclusions only in non-identifying aggregate researcher notes; document any change of protocol before collecting more data.
7. `review-log` only verifies JSON structure and recorded statements. It **cannot** authenticate people or consent. R3-R9 `analyze` similarly requires 20 complete responses but does not independently verify them. A separate independent review must check actual study operations before human-evidence claims.

## Privacy and technical gates

The R3-R10 standard-library Python tool checks the exact pinned Git blobs of R9 and R8, verifies 20 original packets via R9 against the owner's private R3-R6 report, and creates **a fresh directory only** containing:

- `CONSENT_SCRIPT.md` (uniform script)
- `OPERATOR_CHECKLIST.md` (researcher checklist)
- `session_log_TEMPLATE.json` (20 empty entries; **not proof of humans**)

The directory is mode `0700`, files `0600` on POSIX. The tool never edits R3-R8 packets, the original audit, NORDOI source or production. It does not create `responses_ACTUAL.json`, populate consent flags, contact participants, send invitations or certify identity. The `review-log` action refuses blank/incomplete fields, withdrawals and reported deviations; passing that structural check is **not** independent attestation. `selftest` is synthetic only, and CI must NEVER receive private human evidence.

## Local Mac invocation after validating the published R10 Git blob

```sh
R8="$HOME/Downloads/NORDOI_G03_R3_R8_STUDY_20261107.py"
R9="$HOME/Downloads/NORDOI_G03_R3_R9_FOR_R10_20261107.py"  # Download from the exact pinned R3-R9 commit first
AUDIT="$HOME/Downloads/NORDOI_G03_R3_R6_20261107T150311Z_90154.json"
STUDY="$HOME/Downloads/NORDOI_G03_R3_R8_PRIVATE_20261107"
R10="$HOME/Downloads/NORDOI_G03_R3_R10_OPERATOR_20261107.py"
OUT="$HOME/Downloads/NORDOI_G03_R3_R10_OPERATOR_PRIVATE_20261107"
python3 -B "$R10" selftest
python3 -B "$R10" check --r8-tool "$R8" --r9-tool "$R9" --study-dir "$STUDY" --report "$AUDIT"
python3 -B "$R10" prepare --r8-tool "$R8" --r9-tool "$R9" --study-dir "$STUDY" --report "$AUDIT" --out "$OUT"
```

**Only following genuine human sessions**: create `session_log_ACTUAL.json` inside `OUT` from the private template with actual observed fields, phase `ACTUAL_MANUALLY_RECORDED`, and `chmod 600`. Run `review-log` with the same `--r8-tool`, `--r9-tool`, `--study-dir`, `--report` plus `--operator-dir "$OUT" --log "$OUT/session_log_ACTUAL.json"`. This does not prove independent people. Never fill log values just to make the tool pass.

## End state

`R3_R10_TECHNICAL=PASS` only after code selftests, exact-head CI and owner Mac packet/kit verification. `HUMAN_SESSIONS=NOT_COMPLETED` unless genuine study records independently checked. `PEDAGOGICAL_SUPERIORITY=NOT_AUTHORIZED`; R3-R10 PR draft/unmerged; `PRODUCTION=NOT_AUTHORIZED`.
