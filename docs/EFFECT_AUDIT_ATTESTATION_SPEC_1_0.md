# NORDOI Effect Audit Attestation Specification 1.0

Status: K1.11 candidate specification.

## 1. Scope

K1.11 authenticates selected K1.10 durable audit checkpoints without changing K1.10 journal bytes,
program replay meaning or NAIR. The attestation is a separate host-stored anchor that binds a
cryptographic signer identity to the exact K1.10 checkpoint hash and audit-chain root.

## 2. Trust identities

K1.11 defines three host-governed identities:

- `EffectAttestationKeyId([u8; 16])` identifies a signing key by opaque host-provided identity;
- `EffectAttestationAlgorithmId(u16)` identifies the signing algorithm/profile;
- `EffectTrustEpoch(u64)` identifies a non-zero trust-policy generation.

The core does not create private keys, discover certificates, select algorithms or read ambient key
stores.

## 3. Canonical signed statement

`EffectAuditAttestationStatement` canonically binds:

```text
EffectDeliveryNamespace
EffectJournalWriterId
EffectDeliveryFence
EffectTrustEpoch
EffectAttestationKeyId
EffectAttestationAlgorithmId
K1.10 audit root hash
K1.10 audit record count
hash of the complete K1.10 canonical checkpoint
```

The statement is domain-separated with `NORDOI-EFFECT-AUDIT-ATTESTATION-1.0`.

## 4. Signer and verifier boundary

`EffectAttestationSigner` is host-injected and exposes only key identity, algorithm identity and a
`sign(statement)` operation. `EffectAttestationVerifier` is independently host-injected and receives
key ID, trust epoch, algorithm ID, statement bytes and signature bytes.

A verifier rejection or verifier backend error fails closed.

## 5. Anchor storage

`EffectAttestationStore` is separate from `FencedEffectJournalStore`. Attestation bytes therefore do
not overwrite `NDEFXA01` or alter the K1.10 recovery format. Attestation commits carry the currently
active K1.8 `EffectJournalLease`; a conforming host store must reject stale leases.

The canonical attestation envelope magic is:

```text
NDEFXT01
```

The envelope has an independent SHA-256 integrity digest and bounded signature size.

## 6. Durable-state prerequisite

`GovernedEffectAttestor::attest_current` compares the live K1.10 checkpoint candidate with the bytes
currently recoverable from the governed journal. Missing durable state or divergence between live
and durable state is rejected before signing.

This prevents an anchor from certifying a process-local state that has not been durably committed.

## 7. Trust epoch and rotation

Trust epochs are explicit non-zero host inputs. A new anchor may not move to a lower trust epoch.
Within one trust epoch, key ID and algorithm ID are stable. Changing either requires a greater trust
epoch.

Writer takeover is different: writer ID and delivery fence may change while trust epoch/key remain
stable, because fencing ownership and cryptographic trust are separate dimensions.

## 8. Audit progression

Before committing a new anchor, K1.11 loads the previous anchor for the namespace and enforces:

- audit record count may not decrease;
- at the same audit record count, the audit root may not change;
- when height increases, the previously attested root must appear at the prior height in the current hash-chain;
- trust epoch may not decrease;
- key/algorithm may not change without epoch advance.

A different K1.10 checkpoint hash at the same audit height is allowed when the audit root is stable,
because program cycles may legitimately change pending outbox state without creating a new external
attempt record.

## 9. Verification

Verification recomputes the K1.10 checkpoint hash and audit root/count from the supplied checkpoint,
requires exact statement agreement, then delegates signature verification to the host verifier.

An anchor for a different checkpoint is rejected before signature verification.

## 10. Security boundary

K1.11 upgrades K1.10 from tamper evidence relative to a trusted root to cryptographically
attributable anchors according to the host verifier's trust policy. It does not by itself guarantee
hardware-backed keys, certificate validity, revocation freshness, transparency-log inclusion,
non-repudiation or protection against rollback of the external anchor store. Those guarantees depend
on the injected signer/verifier/store and future certified layers.

## 11. Replay and NAIR

Attestation key IDs, algorithms, trust epochs, signatures and anchor storage are host trust metadata.
They do not alter deterministic program replay identity.

**NAIR remains 0.5.**
