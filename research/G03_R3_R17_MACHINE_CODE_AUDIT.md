# NORDOI G0.3 R3-R17 — Function-level assembly and layout audit

ENGINEERING ONLY — NOT A MERGE GATE APPROVAL — PRODUCTION NOT AUTHORIZED.

## Motivation and frozen evidence

- R3-R16 FIX1 has 4/4 security contracts PASS and balanced A/A, B/B and A/B positions (4 per slot per case).
- Owner Intel Mac: noop_set A/B +23.1812% (12/12 blocks), changed_set_flush +3.9061% (10/12), denied_capability -1.3758% (2/12). Denial A/A -0.0883%, B/B +0.0114%. R3-R15 and R3-R14 had similar small denied-capability negatives.
- The sole R3-R13 source change removes a redundant ensure_exists in AtomicKernel::apply_single. AtomicKernel::require and CapabilitySet::contains are **unchanged** in source. No new permission is granted; no correctness regression has been detected in the tested scope.
- Different code-section fingerprints of the two binaries do not establish a change in require's actual function machine code, because binary layout and other functions differ.

## R3-R17 test design

The baseline is **exactly** 98da03b2ea1efcf665d205189284aa8b66fda5d3 and the candidate is **exactly** a1ae15326eaa837f8994a75fa7cfbe2eccc44976. The R3-R13 Python verifier is pinned to Git blob dac0109959e10a15764ff54a4da91bf190deff59. The R3-R17 witness Rust source has a pinned Git blob validated before executing. No repository source edits.

The witness defines a named, #[no_mangle], #[inline(never)] function nordoi_r17_require_witness, with live AtomicKernel and Capability parameters and runtime black_box references. It calls the public require API and returns 1 for denial, 0 for success. A separate main function checks: 32 default denials, pending_work unchanged, explicit grant success, default denial of a fresh independent kernel. Both probes must print the exact PASS receipt.

On Mac Intel:
1. Clone pinned baseline/candidate into disposable directories and validate the exact original-file delta plus identical other tracked sources.
2. Independently compile both Rust release libraries offline with the same Cargo flags, then compile/link the **same** witness with identical explicit rustc optimization flags into two executables.
3. Require exact security PASS from both executed witnesses. Record rustc/cargo identities and exact binary SHA-256/byte count.
4. With nm -an, locate a unique executable Mach-O symbol for the no-inline witness. Use otool -tvV to retain the symbol's disassembly and its SHA256. Also inspect linker-visible symbols that appear to belong to nordoi_kernel's require path; if those symbols are not independently visible, record NOT_LINKER_VISIBLE_OR_INLINED.
5. Preserve full evidence in a new, nonoverwriting local JSON report with 0600 permissions; **no uploaded executable, code dump or private report to GitHub**.

This is a *function-level textual assembly inspection*, not an exact instruction-byte equivalence proof: disassembled call offsets and binary layout may differ. The Rust compiler can inline, fold or outline require, and a direct function symbol may be absent. No dynamic trace or per-instruction latency has been measured. An equal wrapper assembly fingerprint does not prove equal full require machinery; a different fingerprint does not prove causation of the ~0.17 ns latency delta. If both require paths are visible, review their bodies and call sites manually/with another targeted compiler diagnostic. If not, report the limitation and consider a controlled compiler experiment.

## Engineering acceptance

- Source/content pins, Rust security probe on both variants and nm/otool symbol extraction PASS are mandatory for a Mac audit. A missing standalone require symbol is an **expected informative result**, not a fabricated PASS.
- Cross-OS (macOS/Ubuntu/Windows) CI builds a real release library and runs the same external Rust security witness, plus Python synthetic negative tests for symbol parser. CI does not pretend to disassemble production Mac x86_64 or establish speed.
- The results can say the witness security behavior remains equivalent in the tested scope; they cannot claim binary-layout causality without further evidence.
- R3-R17 cannot merge PR #40, cannot authorize production or modify source-language grammar, NAIR, atomic kernel, scheduler or permission grants. No external human evaluator/volunteers/signature required. All existing drafts remain unmerged.

HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
