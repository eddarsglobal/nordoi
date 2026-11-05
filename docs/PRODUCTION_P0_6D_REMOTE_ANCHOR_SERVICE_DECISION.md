# P0.6D Decision — Durable authenticated remote anchor service

## Decision

Add a standalone remote anchor service without modifying P0.6C or the previously certified protected path.

The service must speak the exact P0.6C protocol and persist every accepted monotonic transition before acknowledging success.

## Durable ordering

For an accepted compare-and-advance:

1. authenticate the client request;
2. verify exact expected state;
3. verify monotonic generation and witness positions;
4. append a signed journal entry chained to the previous entry;
5. call `sync_all()` on the journal;
6. update in-memory state;
7. sign and return the success response.

A persistence failure poisons the in-process store and fails closed.

## Authenticated journal

Each entry contains:

- previous entry hash;
- the complete `ProductionAnchorRecord`;
- an Ed25519 signature by the configured server key.

The next-entry chain hash is SHA-256 over a domain-separated encoding of the complete authenticated entry.

This prevents silent complete-entry rewriting without the server key. It does **not** prevent wholesale rollback of the entire journal to an older valid signed prefix.

## Partial-tail recovery

A trailing byte sequence shorter than one complete journal entry is treated as an interrupted, unacknowledged append. On open, P0.6D truncates only that incomplete tail after validating the preceding complete authenticated chain.

A complete but invalid entry is never truncated or repaired automatically; startup fails closed as corrupt.

## Key custody

The runnable service takes paths to two raw 32-byte key files:

- authorized client public key;
- server signing secret.

The secret itself is not passed as a command-line value. P0.6D does not yet certify OS keychain, TPM, Secure Enclave, HSM, or equivalent key custody.

## Performance and simplicity

P0.6D uses blocking `std::net`, fixed-size messages, fixed-size journal entries, Ed25519, and SHA-256.

It deliberately excludes async runtimes, HTTP, JSON, Serde, and RPC frameworks. Remote anchoring remains an optional production-security subsystem and imposes zero cost on ordinary compiler/runtime execution when unused.

## Forbidden claims after P0.6D alone

Even with all local and CI tests green, the following must remain NOT CERTIFIED:

- independent failure-domain deployment;
- remote storage rollback resistance against full valid snapshot restoration;
- hardware-backed server key custody;
- kernel-wide mandatory enforcement;
- production deployment authorization.
