# NORDOI Testing & Release Law

## Law

**No NORDOI version is complete until its automated tests pass.**

Development follows this mandatory sequence:

```text
Design / Law
    ↓
Implementation
    ↓
Automated Tests
    ↓
GitHub CI
    ↓
GREEN
    ↓
Version accepted
    ↓
Next version may begin
```

A version with failing, missing, or unexecuted mandatory tests is a **candidate**, not a completed NORDOI version.

## Minimum test gate for every version

Every version must include tests appropriate to the layer it introduces. At minimum, the repository gate must run:

```bash
cargo check --all-targets
cargo test --all-targets
```

For kernel invariants, tests must include positive and adversarial cases whenever applicable.

## Required evidence

A release is accepted only when:

1. source code for the version is committed;
2. automated tests for new invariants are committed with it;
3. previous regression tests remain present;
4. GitHub Actions reports success;
5. no known failing mandatory test is hidden, skipped, or removed merely to obtain a green build.

## Regression rule

A bug fixed once should receive a regression test whenever technically practical, so the same class of failure cannot silently return.

## Security rule

Security guarantees require negative/adversarial tests in addition to happy-path tests.

## Performance rule

When NORDOI begins making measurable performance claims, those claims must be backed by reproducible benchmark suites. Performance claims are not substitutes for correctness tests.

## Version progression rule

**K0.4 must not be treated as started until K0.3 has passed its GitHub test gate.** The same rule applies recursively to every later version.
