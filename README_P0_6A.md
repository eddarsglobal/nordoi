# NORDOI Production P0.6A — Rollback Witness Contract

P0.6A introduces the first explicit anti-rollback witness contract for the production track.

It adds public, bounded chain witnesses to the already durable P0.2 trust journal and P0.4 delivery ledger. Each live object retains the verified `{position,digest}` prefixes reconstructed during recovery and extended only after successful fsync-backed appends.

A new crate, `production/native-rollback-p06`, defines an abstract `MonotonicAnchor` contract, a narrow `ProductionWitnessSource` contract, and fail-closed recovery rules:

- first boot requires explicit anchor initialization;
- recovery with a missing anchor is denied;
- an anchor ahead of either local chain is rollback;
- equal position with a different digest is a fork;
- a local chain ahead of the anchor may catch the anchor up only when the anchored witness is a verified ancestor;
- compare-and-advance is required and the committed anchor is read back and verified.

The P0.6A contract crate has **no normal Cargo dependency** on P0.1, P0.3, or the kernel. Concrete P0.2/P0.4 access exists only in `dev-dependencies` for the rollback integration tests. This preserves the P0.5C closed production dependency boundary rather than weakening its policy.

The P0.6A tests include restoration of an older but internally valid P0.2 journal and restoration of an older but internally valid P0.4 ledger after the anchor was advanced. Both are denied.

## Critical scope limit

P0.6A deliberately does **not** provide a filesystem implementation of the monotonic anchor. A file stored in the same backup/restore domain would not be independent and would not solve the rollback problem.

P0.6A also does not yet place an anchor advance inside the P0.4/P0.3 dispatch sequence before every external effect. That ordering and the protected adapter from the selected P0.5C profile are reserved for P0.6B. Until an independently trusted anchor backend and per-dispatch integration exist:

**INDEPENDENT MONOTONIC ANTI-ROLLBACK: NOT CERTIFIED**

**KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED**

**PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED**
