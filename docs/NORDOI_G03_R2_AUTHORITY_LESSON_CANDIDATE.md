# NORDOI G0.3 R2 FIX1 — Two keys to console output (teaching candidate)

**Status: non-normative teaching prototype, not validated with beginners.**
This is not part of the frozen A/B research materials. Do not show it to a participant before or during either G0.3 R2 study condition. It must not be used to coach or revise earlier answers.

## The idea in one sentence

To print something, a NORDOI program must **say that it will use the console** and the person running it must **separately allow console output**.

Think of these as two different keys: **declare** and **grant**.

## 1. The program says what it intends to do

The existing, executable P2.1 example:

```noi
module app.main;

effect ConsoleWrite;

entry main emits "Hello from NORDOI P2.1!\n";
```

- `module app.main;` identifies the module.
- `effect ConsoleWrite;` **declares** console output as an intended effect. It does **not** authorize the effect.
- `entry main emits ...;` is the entry point that requests an output.

## 2. The command gives a narrow permission

From the NORDOI repository root:

```text
cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console
```

- `run` selects the CLI path.
- `--grant-console` **grants** only the existing console capability to that invocation.
- The resulting program stdout is exactly `Hello from NORDOI P2.1!` followed by a newline. Execution receipt information goes to stderr.

**Declaration is not permission. A permission is not a declaration. Both are checked.**

## 3. What if one key is missing?

| Source declares `ConsoleWrite`? | CLI explicitly grants console? | P2.1 console effect |
|---|---|---|
| Yes | Yes | Allowed through the implemented P2.1 command path |
| Yes | No | Denied on the existing `console-run` capability gate |
| No | Yes | Undeclared console output is rejected before emission |
| No | No | Not authorized |

Important CLI distinction: `run <path|->` **without** `--grant-console` continues through the historical V0.1 HALT-only path. Do not infer the older `console-run` exit code for that distinct path.

For the P2.1 denial case, the existing command

```text
cargo run --bin nordoi -- console-run examples/p2_1_console/hello.noi
```

returns exit code `5` with **no program stdout**.

## 4. What this does not mean

- The grant is **not** `--allow-all`. It does not grant filesystem, network or device access.
- The simplified `run --grant-console` route is scoped to the implemented **P2.1 console** path; it does not automatically execute P2.2–P2.7 file or dynamic effects.
- No new `.noi` grammar, implicit effects, ambient authority, runtime, dependency or execution privilege is proposed here.
- A shorter explanation is a hypothesis about teaching, **not measured novice usability**. The currently registered A/B study arms remain frozen and the comparative study is incomplete.
- Production remains NOT_AUTHORIZED.

## Next evidence gate

Only after independent, consenting beginner observations may this teaching prototype be evaluated against the registered material. Keep participant responses local and private, do not backfill or relabel missing consent as data, and do not interpret instrument selftests as human comprehension.
