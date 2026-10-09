# NORDOI G0.1 — Constitutional Conformance Matrix & Future-Native Gate

NORDOI G0.1 is an additive governance milestone built on the immutable certified `p2.7` baseline.
It introduces no new `.noi` syntax, NAIR opcode, NAM semantics, runtime authority or capability.

Its purpose is to prevent two governance failures:

1. confusing a large number of certified low-level invariants with completion of the whole NORDOI mission;
2. allowing the roadmap to drift into historical feature-parity work that does not advance NORDOI's constitutional future-oriented goals.

## Two counters, never one

`nordoi conformance` reports two independent measures:

- **constitutional evidence coverage** — whether each of C1..C322 has certified evidence in its current explicit scope;
- **full NORDOI v1 delivery estimate** — a transparent weighted planning model across the complete architecture families.

The second value is explicitly a planning estimate, never a certification claim.

```bash
nordoi conformance
nordoi conformance --json
```

Expected G0.1 candidate baseline:

```text
principles=322
certified=310
partial=12
evidence-coverage=96.27%
full-v1-delivery-estimate=58.35%
delivery-estimate-kind=PLANNING_NOT_CERTIFICATION
future-native-gate=MANDATORY_GOVERNANCE
runtime-semantics=UNCHANGED
authority=NONE
```

## Future-Native Gate

Every future major capability family must satisfy all six rules:

- constitutional driver;
- no feature-parity-only milestone;
- simplicity gain;
- security or provability gain;
- universal architecture;
- certified-boundary preservation.

This explicitly preserves the NORDOI goal of being externally simple while internally sophisticated. A proposal that adds power but needlessly exposes historical complexity to the programmer must be redesigned.

## Machine-readable governance artifacts

- `governance/constitutional_conformance_v1.tsv`
- `governance/nordoi_v1_delivery_model_v1.tsv`
- `governance/future_native_gate_v1.tsv`

Normative report: `docs/NORDOI_CONSTITUTIONAL_CONFORMANCE_MATRIX_G0_1.md`.
Research record: `research/CONSTITUTIONAL_CONFORMANCE_INTELLIGENCE_G0_1.md`.

Certified baseline consumed by G0.1:

```text
p2.7
commit 0a634d6cab52074e18b42d1ffea00ce02cdba359
CI 37956122196
```

Public certified version output remains intentionally frozen:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
