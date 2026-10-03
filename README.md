# NORDOI Kernel K0.3

**K0.3 — Atomic Transactions & Ownership Boundaries**

NORDOI K0.3 builds on the K0.2 effect/capability kernel and introduces the first executable ownership and transaction model for NAM.

## New in K0.3

- Explicit `DomainId` ownership domains.
- Every atom has an owner.
- Root-domain compatibility for K0.1/K0.2 code.
- Domain-owned atom creation.
- Explicit ownership transfer.
- `AtomicTransaction` staging with zero mutation before commit.
- Optimistic version conflict detection.
- All-or-nothing validation before transaction mutation.
- Rollback as zero-work.
- Multiple writes to one atom collapse to one staged write.
- Dependency invalidations are unioned and scheduled once.
- GitHub Actions CI definition.
- `LAW_0001_NORDOI_MASTER_LAW.md` added as a foundational law.

## Transaction path

```text
begin(domain)
   ↓
stage writes
   ↓
ownership check
   ↓
version snapshot
   ↓
commit
   ↓
validate ALL writes first
   ↓
compute true change set
   ↓
compute dependency union
   ↓
apply state atomically
   ↓
schedule each affected atom once
```

If validation fails before commit, staged writes do not partially leak into kernel state.

## Ownership rule

```text
atom -> exactly one ownership domain
```

A domain cannot mutate another domain's atom through the normal K0.3 state APIs.

Ownership transfer is explicit:

```rust
kernel.transfer_atom(atom, current_owner, new_owner)?;
```

## Example

```rust
let wallet = kernel.create_domain("wallet")?;
let balance = kernel.create_atom_owned(wallet, 100_i64)?;

let mut tx = kernel.begin_transaction(wallet)?;
tx.set(&kernel, balance, 75_i64)?;

// still 100 here — transaction is not committed
let report = kernel.commit(tx)?;
// now 75
```

## Master Law

See:

```text
laws/LAW_0001_NORDOI_MASTER_LAW.md
```

It includes the founder's performance declaration:

> "NORDOI est le plus leger langage au monde et le plus vite dans l'univer"

NORDOI treats this as a performance mission that must progressively be supported by reproducible measurements.

## Tests

```bash
cargo test --all-targets
```

The provided CI workflow runs the test suite on GitHub.

## Current kernel layers

```text
K0.1  Atoms + dependency graph + minimal scheduler
K0.2  Typed effects + explicit capabilities
K0.3  Atomic transactions + ownership domains
```

## Next development direction

K0.4 should begin the first **NAIR core**, but only after the K0.3 ownership/transaction invariants pass CI. NAIR must encode these semantics instead of bypassing them.

## Mandatory version test gate

Every NORDOI version ends with automated tests. The next version begins only after the current version is green in GitHub Actions.

Local gate:

```bash
./scripts/release_gate.sh
```

GitHub gate runs `cargo check --all-targets` and `cargo test --all-targets`, with tests on Linux, macOS and Windows.

See `docs/TESTING_AND_RELEASE_LAW.md`.
