# NORDOI G0.3 R3-R0 — Human simplicity and atomic performance study

**Status:** RESEARCH CANDIDATE — NON-NORMATIVE; **not** a change in public `.noi` syntax.  
**Source pin:** `main` `7442b06238dfa9351ae932c0cc636c0c99c3dfea`.  
**Frozen production:** `production/p0.1` `e2898bf7c511e53636ff28c9fd3efc93165a0215`.  
**Parent evidence:** G0.3 R2 issue #11 — structurally valid sessions A=1, B=0; human independence unverified; FIX3 real Mac audit stopped before execution on missing AI archive. The fictional AI session **is not** human evidence.

## 1. Observed baseline: executable today

The pinned P2.1 example is:

```noi
module app.main;

effect ConsoleWrite;

entry main emits "Hello from NORDOI P2.1!\n";
```

The checked-in source is **86 UTF-8 bytes**, **three nonblank lines**, with **four distinct surface keywords** `module`, `effect`, `entry`, `emits` (text proxies, not measures of cognition or runtime cost). Git blob: `e1b74cdabb63716744832c518e9e81e41aa3f14b`.

The certified first-use command is:

```text
cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console
```

`effect ConsoleWrite;` is the **declared intent** in source. `--grant-console` is a **separate, narrow runtime authorization** provided at invocation. The ungranted plain `run` preserves its historical V0.1 HALT-only contract. The `console-run` explicit-denial path is separate (exit 5, no program stdout). The R2 alias does not authorize P2.2–P2.7 effects.

Source snapshot: `docs/NORDOI_G02_FIRST_USE_QUICKSTART.md` blob `0aa6b2af0d67c55b9d70d89d9b8b0fb5d970646e`.

## 2. Friction hypotheses, separated

| Friction | What is observable | What remains unknown |
|---|---|---|
| Source concepts | Four keywords across three executable source lines | Whether a novice finds any particular keyword confusing |
| Terminal/bootstrap | `cargo run --bin nordoi -- ...` precedes the NORDOI command | Number of first-use errors due to Cargo versus NORDOI |
| Authorization | Source effect and CLI grant are intentionally distinct | How long an independent novice needs to learn this separation |
| Output/receipt | Program output on stdout, receipt on stderr | Whether the distinction helps or hinders early comprehension |
| Execution cost | G0.2 R3 found no statistically established CLI alias speed gain | Actual runtime cost of an unimplemented proposed syntax or tooling |

The existing pilot only checked one local A session and zero B sessions. It is **not** grounds to rank these causes or to claim an improvement.

## 3. Four design alternatives — no silent syntax invention

| Candidate | Mechanism | Executable now? | Authority effect | Cost hypothesis | R3 decision |
|---|---|---|---|---|---|
| A. Retain existing source and guide | Baseline `.noi` and `cargo run ... --grant-console` | Yes | Explicit | Existing baseline | CONTROL |
| B. Separate build instruction from everyday command | Teach `cargo build --release --bin nordoi`, then `./target/release/nordoi run ... --grant-console` on Unix; Windows needs its executable naming/path | Uses existing binary/CLI; build required | Explicit | No new runtime dependency or code path | **RESEARCH PRIORITY** |
| C. Optional guided first-run helper | Display a *preview* and explanation, request an independent narrow grant; never infer permission or auto-run | **Not implemented** | Must remain explicit | Tool startup/maintenance cost **unknown** | DESIGN HOLD |
| D. Shorter grammar / fewer boilerplate declarations | Speculative translation into existing typed semantics | **Not implemented** | Must preserve effect+grant and proofs | Parser/compiler/runtime cost **unknown** | **SYNTAX HOLD** |

B does **not** reduce `.noi` source size. It attempts to reduce *repeated terminal complexity* after a separately explained build step. This is a candidate educational hypothesis, not measured novice success. A fully packaged binary might offer a different experience later, but no packaging or distribution is authorized in R3.

### Non-normative concept only, **not runnable source**

A hypothetical shorter `.noi` surface could omit or consolidate ceremony, but **no grammar is proposed for adoption** here. Nothing may infer `ConsoleWrite` permission from the presence of `emits` alone; an effect declaration and an explicit execution grant must remain verifiable. Any later syntax RFC requires semantic equivalence proof, negative authorization tests, exact output/receipt parity, compilation and startup measurements, and real consenting user evidence.

## 4. Language Intelligence Charter — ten axes

Each question applies to a *proposed NORDOI mechanism*, not to the popularity of another language.

| Charter axis | A / B — existing language, changed teaching | C — helper concept | D — future source grammar |
|---|---|---|---|
| 1. Original problem | First-use command and conceptual overload | Setup guidance/interpretation | Visible ceremony |
| 2. Mechanism | Keep grammar; reorganize steps | Preview + explicit grant request | New parse/semantic mapping (unspecified) |
| 3. Strengths | Reversible and already executable | Could explain errors in context | Might reduce surface tokens |
| 4. Structural weaknesses | Still requires full source and initial toolchain | Additional tool is itself complexity | Hidden complexity or ambiguity risk |
| 5. Compatibility debt | None from program semantics | Long-lived tool support possible | New grammar compatibility obligation |
| 6. Security | Declaration and grant unchanged | Must never auto-grant | Must prove effect+grant equivalence |
| 7. Performance | No additional runtime path | Tool overhead unknown | Parser/runtime overhead unknown |
| 8. Human cost | Requires a newcomer study | User study required | User study required |
| 9. AI generation ambiguity | Existing verified examples | Prompt/tool state may drift | Parsing ambiguity not examined |
| 10. Impossible states | Existing unauthorized-output gates retained | Could prevent invalid permission choices if designed well | Needs formal invariants before adoption |

## 5. Learning from other language ecosystems (official documentation)

- [Python Tutorial](https://docs.python.org/3/tutorial/) demonstrates concise interactive first steps but explicitly assumes *general programming familiarity*. Ease of language syntax is not the same as teaching first-time programmers.
- [A Tour of Go](https://go.dev/tour/) provides an editable in-browser Run workflow: it removes installation friction **for tour exercises**, not necessarily for production deployment.
- [The Rust Book / Hello World](https://doc.rust-lang.org/stable/book/ch01-02-hello-world.html) separates creating a file, understanding a program, and using the toolchain, while assuming command-line familiarity.
- [Official Zig language documentation](https://ziglang.org/documentation/) distinguishes `stdout` from `stderr` and presents output I/O explicitly. An output demonstration is not directly comparable across differing stream semantics.

These are documented design patterns, **not controlled cross-language benchmarks**. No speed, binary-size or measured-learning superiority claims follow from the examples.

## 6. Constitutional Future-Native gate

| Gate | Candidate A/B | Candidate C | Candidate D |
|---|---|---|---|
| FNG1 Constitutional purpose | PASS_DESIGN — simpler human onboarding | OPEN | OPEN |
| FNG2 Beyond historical feature parity | PASS_DESIGN | OPEN | OPEN |
| FNG3 Simplicity benefit | HYPOTHESIS; needs participants | NOT_MEASURED | NOT_MEASURED |
| FNG4 Security/provability | PRESERVED by unchanged executable semantics | MUST_PROVE | MUST_PROVE |
| FNG5 Universal platform/backend architecture | UNCHANGED; document Windows path separately | MUST_DESIGN | MUST_DESIGN |
| FNG6 Certified boundary preservation | PASS_DESIGN; read-only | HOLD | HOLD |

`PASS_DESIGN` **is not** production or novice-usability certification.

## 7. Decision and follow-up evidence

**R3-R0 design decision: prefer B as the next reversible *documentation experiment*, retain A as the control; do not implement C or D.** Before publishing even B as the recommended onboarding, compare it with A using independent, consenting novice observations and defined objective tasks (first print, explain declared effect vs grant, correct denial, time-to-completion, errors and hints). Separate setup time from program-understanding time. Existing R2 study is incomplete; its A/B arm comparison cannot be reused as a positive result.

Make any performance claims only after exact-source and hardware-controlled tests. Never infer security or speed from the shortest code sample.

**No `.noi` syntax, compiler, runtime, NAIR, NAM, grants, production branch or release is changed or authorized by this document.**
