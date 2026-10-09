# P2.7 Intelligence Record — Atomic Multi-File Bundle Output

## Council conclusion

The next useful step after P2.6 is not broader filesystem authority and not networking. It is to prove a bounded multi-output commit discipline so NORDOI does not normalize partially committed observable state.

This follows the constitutional priorities:

- impossible states first;
- atomic transactions;
- least authority;
- validate before execute;
- no work without effect;
- evidence before evolution.

## Rejected designs

### Sequential writes directly into the granted directory

Rejected because failure after file N would expose a partial result set.

### General directory capability

Rejected because granting an entire writable directory is wider than the declared program targets.

### Arbitrary nested paths

Rejected because P2.7 should not silently become a general filesystem namespace API.

### Overwrite/append

Rejected because mutation of existing host data has a larger rollback and authority problem than create-new publication.

### Claiming crash-safe ACID

Rejected. A directory rename provides a narrow publication commit point, but P2.7 has no certified fsync/journal/power-loss durability contract.

## Chosen architecture

A new bundle directory is the publication unit. All outputs are evaluated before staging. Staging occurs under the already granted root. One rename publishes the completed directory. Canonical receipts exclude the root path and staging implementation details.

The design deliberately reuses P2.6 rendering semantics instead of creating a new Text runtime or a second expression evaluator.

## Future implication

After P2.7 certification, the Councils recommend a Constitutional Conformance Matrix before another capability family is opened. This prevents Profile 2 from becoming a filesystem-only tunnel and rechecks universal execution, verified performance, concurrency, spatial, heterogeneous compute and stronger verification obligations.
