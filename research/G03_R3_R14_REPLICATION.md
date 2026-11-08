# NORDOI G0.3 R3-R14 — Independent Mac-session replication of single-lookup atomic kernel

**ENGINEERING RESEARCH ONLY · NO HUMAN VOLUNTEERS · UNMERGED · PRODUCTION NOT AUTHORIZED**

## Engineering aim and evidence

The owner Mac Intel R3-R13 pilot (7 paired blocks, 30,000 iterations each) measured a baseline `noop_set` median of **56.460 ns/op** versus candidate **43.921 ns/op** (22.2% faster by quotient of the two medians). The `noop_set` candidate ran first in **6/7 blocks**. This is **promising but exploratory**; ordering, time drift and jitter prevent an unconditional speedup or merge claim. Other measured effects were small and not necessarily caused by the code modification. The PR #40 remains draft; GitHub tests are green but are not performance evidence.

NORDOI C2, C3, C6, C7, C14, C17, C18, C20 and the Master Law remain binding. Human recruitment, sign-off and human-comprehension validation are not part of this engineering gate.

## Frozen source and workload

- Baseline `main`: `98da03b2ea1efcf665d205189284aa8b66fda5d3`.
- Candidate R3-R13: `a1ae15326eaa837f8994a75fa7cfbe2eccc44976`; **no change to source code in R3-R14**.
- R3-R13 Python runner Git blob: `dac0109959e10a15764ff54a4da91bf190deff59`. Download only its exact commit and verify before invocation.
- R3-R13 Rust probe SHA-256: `be6bbd0b74ca8ff12035ee2e8529961285767251f26074d2ce1b4663d723204a`.
- Existing seven-case in-process probe, using the same identical Rust probe source and same `cargo build --offline --release --lib` and `rustc -C opt-level=3 -C codegen-units=1` linkage in both arms.
- `noop_set`, `changed_set_flush`, `dependency_set_flush`, `noop_transaction`, `denied_capability`, `nair_validate`, `nair_execute_with_validation`. The last includes validation inside `execute_nair` again. All API calls include their invariant checks and loop overhead; they are **not CPU cycles**, whole-process CLI times or comparative-language benchmarks.

## Prespecified replication procedure

1. Initiate a **new private study directory** once with `init --out <directory>`. It contains a private, immutable protocol JSON, but no performance samples.
2. Execute **S1**, **S2**, **S3** in **separate deliberate Mac sessions**, preferably with a different restart or intervening host workload. Record the actual execution context separately if needed; the runner never falsely claims that timestamp or nonce proves host-session independence.
3. Each `run-session` clones exact pinned `main` and exact pinned R3-R13 candidate branch to disposable isolated directories, checks the exact single-lookup code delta and **all tracked-file content**, verifies the exact Rust probe, builds two release libraries offline and selftests each binary before sampling.
4. For **each of the seven scenarios**, each session performs eight randomly ordered paired blocks of 30,000 operations **per arm per block**. Exactly four blocks use baseline→candidate and four candidate→baseline, using Python `secrets.SystemRandom`. The per-case round-robin rotates invocation order across the session. The private report retains all *total nanosecond* values (no integer-rounded ns/op), arm order, builds, exact source pins and a fresh nonce.
5. Session slots **cannot be overwritten**, and S2/S3 require the prior slot's complete structurally valid report. The study directory is `0700` and each private report `0600` on macOS; no individual raw data goes to GitHub.
6. `analyze --study-dir <directory>` returns `INCOMPLETE` until **all three valid reports** exist. It recomputes every median, paired delta and per-case AB/BA balance from raw block measurements. The R3-R13 pilot is not counted as one of the three sessions. It does not collect names, ask for volunteers or certify human evidence.

## Decision interpretation

**Primary:** `noop_set` median paired (baseline−candidate) nanoseconds/op **per session**, normalized by that session's median baseline time. Positive means candidate faster. Also report count of faster blocks. A **directional trend for engineering review** requires each of 3 sessions to have positive median paired deltas, a ≥5% median paired relative difference and ≥5/8 faster blocks. Failure returns `MIXED_OR_WEAK_REMEASURE`; success returns **`ENGINEERING_REVIEW_ONLY`**, *not* an automatic performance certification or permission to merge.

**Secondary safeguards:** review all six other paths for consistently adverse medians or changing security properties. A 6–7 ns refusal check is especially sensitive to measurement/optimization noise. Exact `verify_delta` and Rust invariants preserve fail-closed denied permissions, ownership and atomic transaction guards. If a safety test fails, reject the candidate regardless of speed.

**Limitations:** one Mac Intel host, noisy wall-clock microbenchmarks with validation and loops included, no allocation counters, no power controls or cache traces, three sessions insufficient for sweeping statistical conclusions. Per-case measurements from the same session are correlated. No scientific p-values or claim of cross-language superiority. A human owner starts each separate machine run, but **human performance testing remains closed/not required**.

## Mac commands

Download R3-R14 from its *future exact committed head*, verify its Git blob, and download R3-R13 from its exact pinned commit and Git blob before running it. Do not copy commands or blobs across unverified commits.

```bash
python3 -B "$R14" selftest
python3 -B "$R14" init --out "$HOME/Downloads/NORDOI_G03_R3_R14_PRIVATE_20261108"
python3 -B "$R14" run-session --study-dir "$HOME/Downloads/NORDOI_G03_R3_R14_PRIVATE_20261108" --slot S1 --r13-tool "$R13"
# In separate user-triggered Mac sessions, repeat with --slot S2 then --slot S3
python3 -B "$R14" analyze --study-dir "$HOME/Downloads/NORDOI_G03_R3_R14_PRIVATE_20261108"
```

Do **not** invoke all S1/S2/S3 in one automated batch. If the candidate branch/main SHA changes, fail closed and re-evaluate the pinned procedure rather than silently comparing different code.

## Release isolation

- `.noi` grammar, compiler, runtime, NAIR encoding, capabilities and source examples are unchanged. R3-R14 adds only one research Python runner, this protocol and a Python 3.11 synthetic-only three-OS CI workflow.
- GitHub synthetic tests validate the analysis and rejection gates; they do **not** create Mac benchmark evidence or prove independent sessions.
- The original repository is never written to by the script; build artifacts stay inside disposable directories. No GitHub push, merge, tag or production release is authorized.

Exit status until owner completes distinct sessions: `R3_R14_SYNTHETIC=PASS` (if CI), `R3_R14_SESSION_S1/S2/S3=PENDING`, `R3_R14_ENGINEERING_REVIEW=PENDING`, `HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED`, `PRODUCTION=NOT_AUTHORIZED`.
