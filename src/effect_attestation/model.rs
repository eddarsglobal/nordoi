use crate::{
    effect_audit::{EffectAuditCheckpoint, EffectAuditHash},
    effect_dispatch::{EffectDeliveryFence, EffectDeliveryNamespace},
    effect_fencing::EffectJournalWriterId,
};

use super::{
    EffectAttestationAlgorithmId, EffectAttestationError, EffectAttestationKeyId,
    EffectAttestationResult, EffectTrustEpoch, MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES,
};

const STATEMENT_DOMAIN: &[u8] = b"NORDOI-EFFECT-AUDIT-ATTESTATION-1.0";
const CHECKPOINT_HASH_DOMAIN: &[u8] = b"NORDOI-EFFECT-AUDIT-ATTESTED-CHECKPOINT-1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAuditAttestationStatement {
    pub namespace: EffectDeliveryNamespace,
    pub writer: EffectJournalWriterId,
    pub fence: EffectDeliveryFence,
    pub trust_epoch: EffectTrustEpoch,
    pub key_id: EffectAttestationKeyId,
    pub algorithm_id: EffectAttestationAlgorithmId,
    pub audit_root: EffectAuditHash,
    pub audit_records: u64,
    pub checkpoint_hash: EffectAuditHash,
}

impl EffectAuditAttestationStatement {
    pub fn for_checkpoint(
        checkpoint: &EffectAuditCheckpoint,
        writer: EffectJournalWriterId,
        fence: EffectDeliveryFence,
        trust_epoch: EffectTrustEpoch,
        key_id: EffectAttestationKeyId,
        algorithm_id: EffectAttestationAlgorithmId,
    ) -> EffectAttestationResult<Self> {
        if fence.0 == 0 {
            return Err(EffectAttestationError::ZeroFence);
        }
        if trust_epoch.0 == 0 {
            return Err(EffectAttestationError::ZeroTrustEpoch);
        }
        if algorithm_id.0 == 0 {
            return Err(EffectAttestationError::ZeroAlgorithmId);
        }
        let audit_records = u64::try_from(checkpoint.audit().len())
            .map_err(|_| EffectAttestationError::AuditRecordCountOverflow)?;
        Ok(Self {
            namespace: checkpoint.namespace(),
            writer,
            fence,
            trust_epoch,
            key_id,
            algorithm_id,
            audit_root: checkpoint.audit().root_hash(),
            audit_records,
            checkpoint_hash: checkpoint_hash(checkpoint),
        })
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes =
            Vec::with_capacity(STATEMENT_DOMAIN.len() + 16 + 16 + 8 + 8 + 16 + 2 + 32 + 8 + 32);
        bytes.extend_from_slice(STATEMENT_DOMAIN);
        bytes.extend_from_slice(&self.namespace.0);
        bytes.extend_from_slice(&self.writer.0);
        bytes.extend_from_slice(&self.fence.0.to_le_bytes());
        bytes.extend_from_slice(&self.trust_epoch.0.to_le_bytes());
        bytes.extend_from_slice(&self.key_id.0);
        bytes.extend_from_slice(&self.algorithm_id.0.to_le_bytes());
        bytes.extend_from_slice(&self.audit_root.0);
        bytes.extend_from_slice(&self.audit_records.to_le_bytes());
        bytes.extend_from_slice(&self.checkpoint_hash.0);
        bytes
    }

    pub fn matches_checkpoint(&self, checkpoint: &EffectAuditCheckpoint) -> bool {
        self.namespace == checkpoint.namespace()
            && self.audit_root == checkpoint.audit().root_hash()
            && self.audit_records == checkpoint.audit().len() as u64
            && self.checkpoint_hash == checkpoint_hash(checkpoint)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAuditAttestation {
    pub statement: EffectAuditAttestationStatement,
    pub signature: Vec<u8>,
}

impl EffectAuditAttestation {
    pub fn new(
        statement: EffectAuditAttestationStatement,
        signature: Vec<u8>,
    ) -> EffectAttestationResult<Self> {
        if signature.is_empty() {
            return Err(EffectAttestationError::EmptySignature);
        }
        if signature.len() > MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES {
            return Err(EffectAttestationError::SignatureTooLarge {
                bytes: signature.len(),
                limit: MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES,
            });
        }
        Ok(Self {
            statement,
            signature,
        })
    }
}

pub(crate) fn checkpoint_hash(checkpoint: &EffectAuditCheckpoint) -> EffectAuditHash {
    let canonical = checkpoint.canonical_bytes();
    let mut bytes = Vec::with_capacity(CHECKPOINT_HASH_DOMAIN.len() + canonical.len());
    bytes.extend_from_slice(CHECKPOINT_HASH_DOMAIN);
    bytes.extend_from_slice(&canonical);
    EffectAuditHash(crate::effect_audit::hash::sha256(&bytes))
}
