# NORDOI Production P0.6B — Per-Dispatch Monotonic Anchor Barrier

P0.6B integrates the P0.6A `MonotonicAnchor` contract into the selected fenced production path.

The protected ordering is:

1. P0.4 delivery reservation is appended and fsynced.
2. P0.3/P0.2 signed admission is verified, appended and fsynced.
3. P0.6B advances and read-back verifies the monotonic anchor.
4. Only then may the external backend execute.
5. A successful backend result is recorded as a P0.4 local ACK and fsynced.
6. P0.6B advances and verifies the anchor again.

A pre-effect anchor failure blocks the external destination. A failure after a successful destination call returns an error, but the P0.4 delivery identity has already been locally acknowledged and cannot be automatically executed again.

This milestone does **not** certify any specific hardware, OS, remote service, TPM, secure enclave, HSM, or other independent anchor implementation. The injected `MonotonicAnchor` remains a host responsibility and must live outside the rollback domain of the local P0.2/P0.4 files.

`INDEPENDENT ANCHOR BACKEND QUALITY: NOT CERTIFIED`

`KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED`

`PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED`
