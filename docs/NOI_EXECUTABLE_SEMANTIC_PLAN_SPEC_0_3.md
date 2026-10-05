# NORDOI C0.3 — Executable Semantic Plan Specification 0.3

Status: **candidate** until the complete NORDOI Release Gate, exact-commit GitHub CI, and annotated
`c0.3` tag are all green.

## 1. Scope

C0.3 introduces a compiler-owned execution-plan boundary above certified L0.5 minimal body
semantics. It does **not** introduce NAIR lowering or runtime execution.

Accepted source semantics are exactly those already certified by L0.5:

- empty/trivia-only body;
- one contextual pure zero-work `entry Name;` body.

C0.3 converts those semantics into a canonical `SemanticExecutionPlan`.

## 2. Plan forms

The plan form is exactly one of:

```text
EMPTY
ENTRY(name)
```

Both forms have:

```text
work_item_count = 0
required_effects = []
requires_host_authority = false
```

No other plan operation exists in C0.3.

## 3. Validation law

A plan is published only after the L0.5 body boundary has succeeded completely. An entry with any
resolved effect requirement is rejected by C0.3 validation. This is a defense-in-depth invariant even
though certified L0.5 currently creates only pure entries.

Unsupported source syntax remains fail-closed before plan publication.

## 4. Authority separation

C0.3 does not query, serialize, infer, grant, borrow, or consume host authority. A semantic plan is
compiler intent only.

```text
declared effect != authority
resolved effect != authority
semantic execution plan != authority
```

## 5. Canonical identity

C0.3 adds:

```text
canonical_c03_bytes()
```

Its domain is:

```text
NORDOI-C0.3-PLAN\0
```

The witness binds the certified L0.5 witness plus the plan form and zero-work/zero-effect/no-authority
state. Source IDs, source paths, spans, whitespace, and comments do not participate.

Earlier witnesses remain unchanged:

- C0.1 `canonical_identity_bytes()`;
- L0.4 `canonical_semantic_bytes()`;
- C0.2 `canonical_c02_bytes()`;
- L0.5 `canonical_l05_bytes()`.

## 6. CLI inspection

C0.3 adds:

```text
nordoi plan <path|->
```

Representative entry output:

```text
plan module="demo" form=ENTRY entry="main" work=0 effects=0 authority=NONE l05=... c03=...
lowering=UNDEFINED runtime=NOT_INVOKED nair=UNCHANGED
```

This output is human inspection text, not a frozen serialization protocol.

## 7. Deliberately undefined

C0.3 does not define:

- statements or expressions;
- calls or parameters;
- return values;
- control flow;
- effect operations or handlers;
- memory/storage semantics;
- capability grants;
- host calls;
- plan steps or work items;
- NSIR -> NAIR lowering;
- runtime execution.

## 8. Protected certified surfaces

C0.3 must not change certified K1.18/NAIR/runtime semantics. The following remain protected:

- `src/nair/`;
- `src/runtime/`;
- `src/runtime_checkpoint/`;
- `src/kernel.rs`;
- `src/semantic_stability.rs`;
- `.github/`;
- `scripts/release_gate.sh`.

## 9. Release law

C0.3 is certified only after:

1. `cargo fmt --all -- --check`;
2. `cargo clippy --all-targets -- -D warnings`;
3. `cargo check --all-targets`;
4. `cargo test --all-targets`;
5. exact-commit GitHub CI passes all required jobs;
6. annotated tag `c0.3` is pushed.
