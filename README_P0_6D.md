# NORDOI Production P0.6D — Remote Anchor Service Candidate

P0.6D adds the server counterpart to the already-certified P0.6C remote anchor client.

## What this candidate adds

- a standalone `nordoi-remote-anchor-service-p06d` crate;
- exact compatibility with the bounded P0.6C binary protocol;
- client request authentication with Ed25519;
- signed server responses with request-id binding;
- an append-only authenticated journal;
- per-entry Ed25519 signatures chained through a SHA-256 entry hash;
- `sync_all()` before a successful CAS response is emitted;
- recovery from an incomplete trailing write by truncating only the incomplete tail;
- fail-closed rejection of complete-entry corruption;
- restart persistence tests;
- P0.6A rollback detection over a restarted P0.6D service;
- a minimal blocking executable with key files, not secret-key command-line material.

## Deliberate lightness

The normal dependency set is only:

- `nordoi-rollback-anchor-p06`
- `ed25519-dalek`
- `sha2`

The P0.6C client is a **dev dependency only** for cross-compatibility tests.

No Tokio, HTTP, Serde, JSON, gRPC, or general RPC framework is introduced.

## Certification boundary

A passing P0.6D gate can certify the **service implementation and durable journal behavior**.

It still cannot certify:

- that a real deployment is outside the local failure/rollback domain;
- that the server filesystem itself cannot be restored to an older valid snapshot;
- that server private-key custody is hardware-backed;
- kernel-wide mandatory enforcement;
- production deployment authorization.

Those remain separate milestones.
