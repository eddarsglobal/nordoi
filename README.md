# NORDOI K1.11 — Signed Effect Audit Anchor & Trust Epoch Protocol

K1.11 extends certified K1.10 audit/in-doubt recovery with a separate cryptographic attestation layer
for durable effect-audit checkpoints.

The governed trust path is:

```text
K1.10 durable NDEFXA01 checkpoint
        ↓
exact durable/live equality check
        ↓
canonical EffectAuditAttestationStatement
        ↓
host-injected EffectAttestationSigner
        ↓
NDEFXT01 signed anchor
        ↓
fenced EffectAttestationStore
        ↓
independent host verifier
```

## What K1.11 adds

- `EffectAttestationKeyId` opaque key identity;
- `EffectAttestationAlgorithmId` replaceable algorithm/profile identity;
- explicit non-zero `EffectTrustEpoch`;
- canonical `EffectAuditAttestationStatement`;
- opaque bounded `EffectAuditAttestation` signature envelope;
- canonical anchor format `NDEFXT01` with SHA-256 envelope digest;
- host-injected `EffectAttestationSigner` and `EffectAttestationVerifier`;
- separate host-injected `EffectAttestationStore`;
- `GovernedEffectAttestor` for signing and verifying durable K1.10 checkpoints;
- fenced anchor commits under the active K1.8 lease;
- monotonic audit-height/root checks;
- trust-epoch rollback protection while the latest anchor is retained;
- key/algorithm rotation only on explicit trust-epoch advance;
- exact live-vs-durable checkpoint comparison before signing;
- no new Rust dependency;
- NAIR remains 0.5.

## Why K1.11 exists

K1.10 gives a SHA-256 hash chain and checkpoint digest, but explicitly does not identify who endorsed
the root. If an attacker can replace both the checkpoint and its local hash root, hash consistency
alone cannot authenticate the writer.

K1.11 signs a canonical statement that binds:

```text
namespace
writer
fence
trust epoch
key ID
algorithm ID
audit root
audit record count
complete K1.10 checkpoint hash
```

The private key never enters the NORDOI core.

## Separate anchor store

`NDEFXT01` does not replace `NDEFXA01`. K1.10 remains the delivery-recovery journal and K1.11 is an
independent trust anchor. This means signing can be replicated to a secure database, HSM-backed
service, transparency system or other provider without changing K1.10 recovery bytes.

## Durable-state law

The high-level `attest_current` path refuses to sign unless the live event-loop audit checkpoint is
byte-equivalent to the checkpoint recoverable from the durable K1.10 journal. NORDOI therefore does
not authenticate state that exists only in process memory.

## Key rotation

Trust rotation is explicit:

```text
same trust epoch  → same key ID + same algorithm ID
higher trust epoch → key/algorithm rotation permitted
lower trust epoch  → rejected
```

A writer takeover is not a key rotation. Writer ID and fencing token may change independently while
trust epoch/key remain stable.

## Audit progression

A retained anchor cannot be replaced by one with fewer audit records. At the same audit height, a
different audit root is rejected as a fork. When the audit grows, the previously signed root must
appear at the prior height in the new K1.10 chain. A different full checkpoint hash at the same audit
height is allowed when the audit root is unchanged because new program cycles can change pending
outbox state without creating a new external-attempt audit event.

## Security boundary

K1.11 does not claim that every signer is secure or that every anchor store is rollback-proof. The
actual assurance depends on the injected signer, verifier, trust policy and storage backend.
Hardware-backed keys, revocation, transparency inclusion, threshold signatures and build provenance
remain future separable layers.

## Replay and NAIR

Attestation metadata is host trust metadata and never changes deterministic program replay identity.

**NAIR remains 0.5.**

## Tests

K1.11 adds **28 attestation/trust-anchor tests** on top of the **295 certified K1.10 tests**, for an expected corpus of **323 tests**. The new coverage includes canonical envelope round-trips, tamper detection, signer/verifier failures, separate anchor storage, durable-state matching, trust-epoch rollback, key/algorithm rotation, audit rollback/fork/non-descendant history, stale fencing, takeover and replay independence.

## Certification

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

K1.11 is certified only after the local release gate and cross-platform GitHub CI are fully green,
followed by publication of the `k1.11` tag.
