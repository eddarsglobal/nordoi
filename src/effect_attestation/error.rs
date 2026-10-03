use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    effect_audit::EffectAuditError, effect_dispatch::EffectDeliveryNamespace,
    effect_fencing::EffectFencingError,
};

use super::{
    EffectAttestationAlgorithmId, EffectAttestationBackendError, EffectAttestationKeyId,
    EffectAttestationStoreError, EffectTrustEpoch,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectAttestationError {
    Audit(EffectAuditError),
    Fencing(EffectFencingError),
    Backend(EffectAttestationBackendError),
    Store(EffectAttestationStoreError),
    ZeroFence,
    ZeroTrustEpoch,
    ZeroAlgorithmId,
    AuditRecordCountOverflow,
    EmptySignature,
    SignatureTooLarge {
        bytes: usize,
        limit: usize,
    },
    AttestationTooLarge {
        bytes: usize,
        limit: usize,
    },
    InvalidAttestationMagic,
    UnsupportedAttestationVersion {
        major: u16,
        minor: u16,
    },
    AttestationTruncated,
    AttestationDigestMismatch,
    NamespaceMismatch {
        expected: EffectDeliveryNamespace,
        actual: EffectDeliveryNamespace,
    },
    CheckpointMismatch,
    DurableCheckpointMissing,
    DurableCheckpointMismatch,
    SignatureRejected,
    TrustEpochRollback {
        previous: EffectTrustEpoch,
        current: EffectTrustEpoch,
    },
    KeyChangedWithoutEpochAdvance {
        previous: EffectAttestationKeyId,
        current: EffectAttestationKeyId,
    },
    AlgorithmChangedWithoutEpochAdvance {
        previous: EffectAttestationAlgorithmId,
        current: EffectAttestationAlgorithmId,
    },
    AuditRollback {
        previous_records: u64,
        current_records: u64,
    },
    AuditForkAtSameHeight,
    AuditHistoryNotDescendant,
}

impl Display for EffectAttestationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Audit(error) => write!(f, "effect audit failure: {error}"),
            Self::Fencing(error) => write!(f, "effect fencing failure: {error}"),
            Self::Backend(error) => write!(f, "effect attestation backend failure: {error}"),
            Self::Store(error) => write!(f, "effect attestation store failure: {error}"),
            Self::ZeroFence => write!(f, "effect attestation fence must be non-zero"),
            Self::ZeroTrustEpoch => write!(f, "effect attestation trust epoch must be non-zero"),
            Self::ZeroAlgorithmId => write!(f, "effect attestation algorithm id must be non-zero"),
            Self::AuditRecordCountOverflow => write!(f, "effect audit record count exceeds u64"),
            Self::EmptySignature => write!(f, "effect attestation signature is empty"),
            Self::SignatureTooLarge { bytes, limit } => write!(f, "effect attestation signature is {bytes} bytes, limit is {limit}"),
            Self::AttestationTooLarge { bytes, limit } => write!(f, "effect attestation is {bytes} bytes, limit is {limit}"),
            Self::InvalidAttestationMagic => write!(f, "invalid effect attestation magic"),
            Self::UnsupportedAttestationVersion { major, minor } => write!(f, "unsupported effect attestation version {major}.{minor}"),
            Self::AttestationTruncated => write!(f, "truncated effect attestation"),
            Self::AttestationDigestMismatch => write!(f, "effect attestation digest mismatch"),
            Self::NamespaceMismatch { expected, actual } => write!(f, "effect attestation namespace mismatch: expected {expected}, got {actual}"),
            Self::CheckpointMismatch => write!(f, "effect attestation does not match the supplied audit checkpoint"),
            Self::DurableCheckpointMissing => write!(f, "no durable effect audit checkpoint exists to attest"),
            Self::DurableCheckpointMismatch => write!(f, "live effect audit state does not match the durable checkpoint"),
            Self::SignatureRejected => write!(f, "effect attestation signature was rejected"),
            Self::TrustEpochRollback { previous, current } => write!(f, "effect attestation trust epoch moved backward: previous {previous}, current {current}"),
            Self::KeyChangedWithoutEpochAdvance { previous, current } => write!(f, "effect attestation key changed without trust-epoch advance: previous {previous}, current {current}"),
            Self::AlgorithmChangedWithoutEpochAdvance { previous, current } => write!(f, "effect attestation algorithm changed without trust-epoch advance: previous {previous}, current {current}"),
            Self::AuditRollback { previous_records, current_records } => write!(f, "effect attestation audit height moved backward: previous {previous_records}, current {current_records}"),
            Self::AuditForkAtSameHeight => write!(f, "effect attestation attempted to sign a different audit root at the same height"),
            Self::AuditHistoryNotDescendant => write!(f, "effect attestation audit history does not descend from the previously attested root"),
        }
    }
}

impl Error for EffectAttestationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Audit(error) => Some(error),
            Self::Fencing(error) => Some(error),
            Self::Backend(error) => Some(error),
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EffectAuditError> for EffectAttestationError {
    fn from(value: EffectAuditError) -> Self {
        Self::Audit(value)
    }
}
impl From<EffectFencingError> for EffectAttestationError {
    fn from(value: EffectFencingError) -> Self {
        Self::Fencing(value)
    }
}
impl From<EffectAttestationBackendError> for EffectAttestationError {
    fn from(value: EffectAttestationBackendError) -> Self {
        Self::Backend(value)
    }
}
impl From<EffectAttestationStoreError> for EffectAttestationError {
    fn from(value: EffectAttestationStoreError) -> Self {
        Self::Store(value)
    }
}

pub type EffectAttestationResult<T> = Result<T, EffectAttestationError>;
