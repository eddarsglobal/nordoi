# NORDOI Research — Effect Audit Attestation Intelligence 0.1

K1.10 deliberately stopped at hash chaining: a hash chain can prove internal consistency relative to
a trusted root, but it cannot identify the writer that endorsed that root. K1.11 adds the smallest
possible authentication layer without coupling NORDOI to one cryptographic library or PKI.

## Architectural conclusions

1. **Signing keys must not enter canonical program authority.** A `.noi`/NAIR program must never be
   able to serialize a private key or silently grant itself signing authority.
2. **Algorithms must be replaceable.** The core signs canonical bytes through an opaque host
   backend instead of embedding Ed25519, ECDSA, RSA, a cloud KMS or a hardware token as a mandatory
   dependency.
3. **Trust rotation and writer fencing are different.** A process takeover changes writer/fence;
   compromise recovery changes trust epoch/key. Conflating them would make ordinary failover look
   like cryptographic key rotation.
4. **Sign only durable state.** A signature over process-local candidate state is dangerous because
   it can authenticate state that never became recoverable. K1.11 therefore checks the live
   checkpoint against the durable K1.10 checkpoint before signing.
5. **Anchors should not rewrite the journal.** Keeping `NDEFXT01` separate from `NDEFXA01` preserves
   K1.10 recovery compatibility and permits independent anchor replication or transparency systems.
6. **Rollback resistance is layered.** A local latest-anchor store can reject backward progression
   while it retains its latest record, but a hostile host that rolls back the entire anchor store can
   defeat that local memory. Strong rollback resistance needs an external monotonic authority,
   append-only transparency service, hardware counter or consensus-backed store.
7. **Verification policy belongs to the host.** Certificate chains, revocation, hardware attestation,
   key provenance and approved algorithms evolve independently of NORDOI program semantics.

## Future work

A later layer may add external transparency anchoring, threshold/multi-signature policy, hardware
attestation evidence, signed build provenance and reproducible artifact identity. Those should reuse
the K1.11 canonical statement discipline rather than making the kernel dependent on one vendor.
