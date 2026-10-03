use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    effect_dispatch::{EffectDispatchError, EffectIntentId},
    effect_fencing::{EffectFenceStoreError, EffectFencingError},
    effect_persistence::EffectPersistenceError,
    effect_retry::{EffectRetryError, EffectRetryTick},
};

use super::{EffectAttemptId, EffectAuditHash, EffectAuditSequence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectAuditError {
    Retry(EffectRetryError),
    Fencing(EffectFencingError),
    Store(EffectFenceStoreError),
    Persistence(EffectPersistenceError),
    Dispatch(EffectDispatchError),
    AttemptIdExhausted,
    AuditSequenceExhausted,
    InDoubtAttemptExists(EffectAttemptId),
    NoInDoubtAttempt,
    InDoubtIntentMismatch {
        expected: EffectIntentId,
        actual: EffectIntentId,
    },
    InDoubtIntentNotPending(EffectIntentId),
    ResolutionTickMovedBackward {
        previous: EffectRetryTick,
        current: EffectRetryTick,
    },
    AuditCheckpointTooLarge {
        bytes: usize,
        limit: usize,
    },
    AuditEventLimitExceeded {
        events: u64,
        limit: usize,
    },
    AuditCheckpointStringTooLarge {
        bytes: u64,
        limit: usize,
    },
    InvalidAuditCheckpointMagic,
    UnsupportedAuditCheckpointVersion {
        major: u16,
        minor: u16,
    },
    AuditCheckpointTruncated,
    AuditCheckpointInvalidUtf8,
    AuditCheckpointDigestMismatch {
        expected: EffectAuditHash,
        actual: EffectAuditHash,
    },
    AuditHashChainMismatch {
        sequence: EffectAuditSequence,
    },
    AuditPreviousHashMismatch {
        sequence: EffectAuditSequence,
    },
    AuditSequenceMismatch {
        expected: EffectAuditSequence,
        actual: EffectAuditSequence,
    },
    InvalidAuditEventTag(u8),
    InvalidAuditEventState(String),
    InvalidDeadLetterReason(u8),
    AuditNamespaceMismatch,
    AuditPolicyMismatch,
}

impl Display for EffectAuditError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retry(error) => write!(f, "effect retry failure: {error}"),
            Self::Fencing(error) => write!(f, "effect fencing failure: {error}"),
            Self::Store(error) => write!(f, "effect audit store failure: {error}"),
            Self::Persistence(error) => write!(f, "effect persistence failure: {error}"),
            Self::Dispatch(error) => write!(f, "effect dispatch failure: {error}"),
            Self::AttemptIdExhausted => write!(f, "effect attempt identity space is exhausted"),
            Self::AuditSequenceExhausted => write!(f, "effect audit sequence space is exhausted"),
            Self::InDoubtAttemptExists(id) => write!(f, "in-doubt effect attempt {} must be resolved before further dispatch", id.0),
            Self::NoInDoubtAttempt => write!(f, "no in-doubt effect attempt exists"),
            Self::InDoubtIntentMismatch { expected, actual } => write!(f, "in-doubt intent mismatch: expected {}, got {}", expected.0, actual.0),
            Self::InDoubtIntentNotPending(id) => write!(f, "in-doubt intent {} is not pending", id.0),
            Self::ResolutionTickMovedBackward { previous, current } => write!(f, "effect audit resolution tick moved backward: previous {previous}, current {current}"),
            Self::AuditCheckpointTooLarge { bytes, limit } => write!(f, "effect audit checkpoint is {bytes} bytes, limit is {limit} bytes"),
            Self::AuditEventLimitExceeded { events, limit } => write!(f, "effect audit checkpoint has {events} events, limit is {limit}"),
            Self::AuditCheckpointStringTooLarge { bytes, limit } => write!(f, "effect audit checkpoint string is {bytes} bytes, limit is {limit} bytes"),
            Self::InvalidAuditCheckpointMagic => write!(f, "invalid effect audit checkpoint magic"),
            Self::UnsupportedAuditCheckpointVersion { major, minor } => write!(f, "unsupported effect audit checkpoint version {major}.{minor}"),
            Self::AuditCheckpointTruncated => write!(f, "truncated effect audit checkpoint"),
            Self::AuditCheckpointInvalidUtf8 => write!(f, "invalid UTF-8 in effect audit checkpoint"),
            Self::AuditCheckpointDigestMismatch { expected, actual } => write!(f, "effect audit checkpoint digest mismatch: expected {expected}, actual {actual}"),
            Self::AuditHashChainMismatch { sequence } => write!(f, "effect audit hash mismatch at sequence {}", sequence.0),
            Self::AuditPreviousHashMismatch { sequence } => write!(f, "effect audit previous-hash mismatch at sequence {}", sequence.0),
            Self::AuditSequenceMismatch { expected, actual } => write!(f, "effect audit sequence mismatch: expected {}, got {}", expected.0, actual.0),
            Self::InvalidAuditEventTag(tag) => write!(f, "invalid effect audit event tag 0x{tag:02x}"),
            Self::InvalidAuditEventState(message) => write!(f, "invalid effect audit event state: {message}"),
            Self::InvalidDeadLetterReason(tag) => write!(f, "invalid effect audit dead-letter reason tag 0x{tag:02x}"),
            Self::AuditNamespaceMismatch => write!(f, "effect audit checkpoint namespace mismatch"),
            Self::AuditPolicyMismatch => write!(f, "effect audit checkpoint retry policy does not match configured policy"),
        }
    }
}

impl Error for EffectAuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Retry(error) => Some(error),
            Self::Fencing(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::Persistence(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EffectRetryError> for EffectAuditError {
    fn from(value: EffectRetryError) -> Self {
        Self::Retry(value)
    }
}
impl From<EffectFencingError> for EffectAuditError {
    fn from(value: EffectFencingError) -> Self {
        Self::Fencing(value)
    }
}
impl From<EffectFenceStoreError> for EffectAuditError {
    fn from(value: EffectFenceStoreError) -> Self {
        Self::Store(value)
    }
}
impl From<EffectPersistenceError> for EffectAuditError {
    fn from(value: EffectPersistenceError) -> Self {
        Self::Persistence(value)
    }
}
impl From<EffectDispatchError> for EffectAuditError {
    fn from(value: EffectDispatchError) -> Self {
        Self::Dispatch(value)
    }
}

pub type EffectAuditResult<T> = Result<T, EffectAuditError>;
