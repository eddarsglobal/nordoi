use crate::{
    effect_audit::{EffectAuditCheckpoint, GovernedAuditedEffectJournal},
    effect_dispatch::AtomicEffectOutbox,
    effect_fencing::{EffectJournalLease, FencedEffectJournalStore},
};

use super::{
    EffectAttestationCommitReceipt, EffectAttestationError, EffectAttestationResult,
    EffectAttestationSigner, EffectAttestationStore, EffectAttestationVerifier,
    EffectAuditAttestation, EffectAuditAttestationStatement, EffectTrustEpoch,
    MAX_EFFECT_ATTESTATION_BYTES,
};

#[derive(Debug)]
pub struct GovernedEffectAttestor<S> {
    store: S,
    trust_epoch: EffectTrustEpoch,
}

impl<S> GovernedEffectAttestor<S> {
    pub fn new(store: S, trust_epoch: EffectTrustEpoch) -> EffectAttestationResult<Self> {
        if trust_epoch.0 == 0 {
            return Err(EffectAttestationError::ZeroTrustEpoch);
        }
        Ok(Self { store, trust_epoch })
    }

    pub const fn trust_epoch(&self) -> EffectTrustEpoch {
        self.trust_epoch
    }
    pub fn store(&self) -> &S {
        &self.store
    }
    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }
    pub fn into_store(self) -> S {
        self.store
    }
}

impl<S: EffectAttestationStore> GovernedEffectAttestor<S> {
    pub fn attest_current<J: FencedEffectJournalStore, SIGN: EffectAttestationSigner>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<J>,
        outbox: &AtomicEffectOutbox,
        signer: &mut SIGN,
    ) -> EffectAttestationResult<(EffectAuditAttestation, EffectAttestationCommitReceipt)> {
        let lease = journal.assert_active()?;
        let live = journal.capture_checkpoint(outbox);
        let durable = journal
            .recover()?
            .ok_or(EffectAttestationError::DurableCheckpointMissing)?;
        if live != durable {
            return Err(EffectAttestationError::DurableCheckpointMismatch);
        }
        self.attest(&live, lease, signer)
    }

    pub fn verify_current<J: FencedEffectJournalStore, V: EffectAttestationVerifier>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<J>,
        outbox: &AtomicEffectOutbox,
        verifier: &V,
    ) -> EffectAttestationResult<Option<EffectAuditAttestation>> {
        let live = journal.capture_checkpoint(outbox);
        let durable = journal
            .recover()?
            .ok_or(EffectAttestationError::DurableCheckpointMissing)?;
        if live != durable {
            return Err(EffectAttestationError::DurableCheckpointMismatch);
        }
        self.verify_latest(&durable, verifier)
    }

    pub fn attest<SIGN: EffectAttestationSigner>(
        &mut self,
        checkpoint: &EffectAuditCheckpoint,
        lease: EffectJournalLease,
        signer: &mut SIGN,
    ) -> EffectAttestationResult<(EffectAuditAttestation, EffectAttestationCommitReceipt)> {
        if checkpoint.namespace() != lease.namespace {
            return Err(EffectAttestationError::NamespaceMismatch {
                expected: checkpoint.namespace(),
                actual: lease.namespace,
            });
        }
        let previous = self.load_latest_raw(checkpoint.namespace())?;
        let statement = EffectAuditAttestationStatement::for_checkpoint(
            checkpoint,
            lease.writer,
            lease.fence,
            self.trust_epoch,
            signer.key_id(),
            signer.algorithm_id(),
        )?;
        if let Some(previous) = previous.as_ref() {
            validate_progression(previous, &statement, checkpoint)?;
        }
        let statement_bytes = statement.canonical_bytes();
        let signature = signer.sign(&statement_bytes)?;
        let attestation = EffectAuditAttestation::new(statement, signature)?;
        let bytes = attestation.canonical_bytes();
        if bytes.len() > MAX_EFFECT_ATTESTATION_BYTES {
            return Err(EffectAttestationError::AttestationTooLarge {
                bytes: bytes.len(),
                limit: MAX_EFFECT_ATTESTATION_BYTES,
            });
        }
        let receipt = self.store.commit_fenced_attestation(lease, &bytes)?;
        Ok((attestation, receipt))
    }

    pub fn verify_latest<V: EffectAttestationVerifier>(
        &mut self,
        checkpoint: &EffectAuditCheckpoint,
        verifier: &V,
    ) -> EffectAttestationResult<Option<EffectAuditAttestation>> {
        let Some(attestation) = self.load_latest_raw(checkpoint.namespace())? else {
            return Ok(None);
        };
        if !attestation.statement.matches_checkpoint(checkpoint) {
            return Err(EffectAttestationError::CheckpointMismatch);
        }
        let statement_bytes = attestation.statement.canonical_bytes();
        let verified = verifier.verify(
            attestation.statement.key_id,
            attestation.statement.trust_epoch,
            attestation.statement.algorithm_id,
            &statement_bytes,
            &attestation.signature,
        )?;
        if !verified {
            return Err(EffectAttestationError::SignatureRejected);
        }
        Ok(Some(attestation))
    }

    pub fn load_latest(
        &mut self,
        namespace: crate::effect_dispatch::EffectDeliveryNamespace,
    ) -> EffectAttestationResult<Option<EffectAuditAttestation>> {
        self.load_latest_raw(namespace)
    }

    fn load_latest_raw(
        &mut self,
        namespace: crate::effect_dispatch::EffectDeliveryNamespace,
    ) -> EffectAttestationResult<Option<EffectAuditAttestation>> {
        let Some(bytes) = self.store.load_attestation(namespace)? else {
            return Ok(None);
        };
        let attestation = EffectAuditAttestation::from_canonical_bytes(&bytes)?;
        if attestation.statement.namespace != namespace {
            return Err(EffectAttestationError::NamespaceMismatch {
                expected: namespace,
                actual: attestation.statement.namespace,
            });
        }
        Ok(Some(attestation))
    }
}

fn validate_progression(
    previous: &EffectAuditAttestation,
    current: &EffectAuditAttestationStatement,
    checkpoint: &EffectAuditCheckpoint,
) -> EffectAttestationResult<()> {
    let previous = &previous.statement;
    if current.trust_epoch < previous.trust_epoch {
        return Err(EffectAttestationError::TrustEpochRollback {
            previous: previous.trust_epoch,
            current: current.trust_epoch,
        });
    }
    if current.trust_epoch == previous.trust_epoch {
        if current.key_id != previous.key_id {
            return Err(EffectAttestationError::KeyChangedWithoutEpochAdvance {
                previous: previous.key_id,
                current: current.key_id,
            });
        }
        if current.algorithm_id != previous.algorithm_id {
            return Err(
                EffectAttestationError::AlgorithmChangedWithoutEpochAdvance {
                    previous: previous.algorithm_id,
                    current: current.algorithm_id,
                },
            );
        }
    }
    if current.audit_records < previous.audit_records {
        return Err(EffectAttestationError::AuditRollback {
            previous_records: previous.audit_records,
            current_records: current.audit_records,
        });
    }
    if current.audit_records == previous.audit_records && current.audit_root != previous.audit_root
    {
        return Err(EffectAttestationError::AuditForkAtSameHeight);
    }
    if current.audit_records > previous.audit_records && previous.audit_records > 0 {
        let ancestor_index = usize::try_from(previous.audit_records - 1)
            .map_err(|_| EffectAttestationError::AuditHistoryNotDescendant)?;
        let ancestor = checkpoint
            .audit()
            .records()
            .get(ancestor_index)
            .ok_or(EffectAttestationError::AuditHistoryNotDescendant)?;
        if ancestor.hash != previous.audit_root {
            return Err(EffectAttestationError::AuditHistoryNotDescendant);
        }
    }
    Ok(())
}
