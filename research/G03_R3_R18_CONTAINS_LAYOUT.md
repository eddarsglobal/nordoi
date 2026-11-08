# NORDOI G0.3 R3-R18 — CapabilitySet::contains call-path code layout diagnosis

Research-only, no kernel modification. No automatic merge. Production NOT_AUTHORIZED.

## Evidence carried forward

Mac Intel R3-R16 FIX1: 4/4 security contracts PASS, balanced AA/BB/AB group positions; noop_set AB +23.1812% (12/12 faster), changed_set_flush AB +3.9061% (10/12 faster), denied_capability AB -1.3758% (2/12 faster), ~0.17 ns. Denied AA -0.0883% and BB +0.0114%.

Mac Intel R3-R17 FIX1: both external Rust witnesses checked default deny and explicit grant, PASS. Baseline/candidate AtomicKernel::require contained 41 textual disassembled instructions each; witness contained 47 each. Relative branch-target normalization gave textual matches 41/41 and 47/47. Raw text hashes differed because of addresses. require moved from 0x100003210 to 0x100002d40 (-1232 bytes); witness from 0x100001ca0 to 0x100001c90 (-16 bytes). This does not demonstrate full machine-byte identity or performance causation.

## Frozen sources

Baseline main SHA 98da03b2ea1efcf665d205189284aa8b66fda5d3.
Candidate R3-R13 SHA a1ae15326eaa837f8994a75fa7cfbe2eccc44976.
R3-R13 Python verifier Git blob dac0109959e10a15764ff54a4da91bf190deff59.
R3-R17 FIX1 Python Git blob 713f50c217aa9cc2f0604e24089ed8749af07a08.
Unchanged R3-R17 witness Rust Git blob 599f2353777e86de4a85ce6ba4e1cbc951a4249f.

No code from these pinned sources is modified. The only kernel difference is the redundant existence-check removal in private apply_single. Other tracked original source bytes are guarded. The baseline/candidate are cloned in separate temporary source folders; witness output builds use separate build folders to prevent the old R3-R17 candidate-path collision.

## Protocol

On owner Mac Intel, rebuild baseline/candidate as independently linked release Rust witnesses using cargo build --offline --release --lib. Run real security contract: denied by default, grant accepted, fresh kernel denied in both.

Inspect unique nm/otool symbols for the no-inline witness, AtomicKernel::require and CapabilitySet::contains. Extract and retain per-function raw disassembly, branch-normalized instruction text, raw SHA256, normalized SHA256, instruction counts, direct call edges, executable bytes and SHA256. Report each code address and alignment mod 16/32/64/128, plus baseline-to-candidate shifts. The normalization changes local conditional branch targets into function-relative offsets while preserving instruction mnemonics and other operands. If contains is not linker visible or its body cannot be bounded, record NOT_COMPARABLE, not invented equivalence. The exact Mac report is private 0600 JSON, non-overwriting.

Cross-OS GitHub CI uses synthetic parser negative cases and compiles/runs a real baseline Rust security witness on Ubuntu, macOS and Windows; this is not Mac Intel disassembly or benchmark evidence.

## Interpretation

The source calls AtomicKernel::require -> CapabilitySet::contains -> HashSet::contains. The first two are unchanged by the candidate source delta. Textually identical disassembled functions are not guaranteed byte identical, and alignment shifts do not prove the tiny measured latency difference. The full call path may include hash-table/hasher internals outside this scope. R3-R18 is a finite assembly audit, not another speed campaign. An explicit subsequent engineering decision should weigh a strong reproducible ~20% noop-set gain against a ~1.4% (0.17 ns) measured denied micro-path slowdown, unaffected correctness tests, call frequency and environment-specific uncertainty.

No parser/NAIR/authority policy changes, no public language change, no external human study/evaluator/signature, no automatic merge or production. R3-R13 PR #40 remains unmerged. Final protocol verdict: ENGINEERING_REVIEW_ONLY, MERGE_AUTHORIZATION=NONE, PRODUCTION=NOT_AUTHORIZED.
