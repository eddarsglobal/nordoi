use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    effect_dispatch::{EffectDispatchError, EffectIntentId},
    effect_fencing::{EffectFenceStoreError, EffectFencingError},
    effect_persistence::EffectPersistenceError,
};

use super::EffectRetryTick;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectRetryError {
    Fencing(EffectFencingError),
    Store(EffectFenceStoreError),
    Persistence(EffectPersistenceError),
    Dispatch(EffectDispatchError),
    InvalidPolicy(String),
    InvalidFailureCount(u32),
    RetryTickMovedBackward {
        previous: EffectRetryTick,
        current: EffectRetryTick,
    },
    RetryTickOverflow,
    RetryCheckpointTooLarge {
        bytes: usize,
        limit: usize,
    },
    RetryEntryLimitExceeded {
        entries: u64,
        limit: usize,
    },
    DeadLetterLimitExceeded {
        entries: u64,
        limit: usize,
    },
    InvalidRetryCheckpointMagic,
    UnsupportedRetryCheckpointVersion {
        major: u16,
        minor: u16,
    },
    RetryCheckpointChecksumMismatch {
        expected: u64,
        actual: u64,
    },
    RetryCheckpointTruncated,
    RetryCheckpointInvalidUtf8,
    RetryCheckpointStringTooLarge {
        bytes: u64,
        limit: usize,
    },
    RetryNamespaceMismatch,
    RetryPolicyMismatch,
    UnknownRetryIntent(EffectIntentId),
    RetryIntentNotPending(EffectIntentId),
    DuplicateOrUnorderedRetryIntent(EffectIntentId),
    DuplicateOrUnorderedDeadLetter(EffectIntentId),
    UnknownDeadLetter(EffectIntentId),
    InvalidDeadLetterReason(u8),
}

impl Display for EffectRetryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fencing(error) => write!(f, "effect fencing failure: {error}"),
            Self::Store(error) => write!(f, "effect retry store failure: {error}"),
            Self::Persistence(error) => write!(f, "effect persistence failure: {error}"),
            Self::Dispatch(error) => write!(f, "effect dispatch failure: {error}"),
            Self::InvalidPolicy(message) => write!(f, "invalid effect retry policy: {message}"),
            Self::InvalidFailureCount(count) => {
                write!(f, "effect retry failure count must be non-zero, got {count}")
            }
            Self::RetryTickMovedBackward { previous, current } => write!(
                f,
                "effect retry tick moved backward: previous {previous}, current {current}"
            ),
            Self::RetryTickOverflow => write!(f, "effect retry tick overflow"),
            Self::RetryCheckpointTooLarge { bytes, limit } => write!(
                f,
                "effect retry checkpoint is {bytes} bytes, limit is {limit} bytes"
            ),
            Self::RetryEntryLimitExceeded { entries, limit } => write!(
                f,
                "effect retry checkpoint has {entries} retry entries, limit is {limit}"
            ),
            Self::DeadLetterLimitExceeded { entries, limit } => write!(
                f,
                "effect retry checkpoint has {entries} dead letters, limit is {limit}"
            ),
            Self::InvalidRetryCheckpointMagic => write!(f, "invalid effect retry checkpoint magic"),
            Self::UnsupportedRetryCheckpointVersion { major, minor } => write!(
                f,
                "unsupported effect retry checkpoint version {major}.{minor}"
            ),
            Self::RetryCheckpointChecksumMismatch { expected, actual } => write!(
                f,
                "effect retry checkpoint checksum mismatch: expected {expected:016x}, actual {actual:016x}"
            ),
            Self::RetryCheckpointTruncated => write!(f, "truncated effect retry checkpoint"),
            Self::RetryCheckpointInvalidUtf8 => write!(f, "invalid UTF-8 in effect retry checkpoint"),
            Self::RetryCheckpointStringTooLarge { bytes, limit } => write!(
                f,
                "effect retry checkpoint string is {bytes} bytes, limit is {limit} bytes"
            ),
            Self::RetryNamespaceMismatch => write!(f, "effect retry checkpoint namespace mismatch"),
            Self::RetryPolicyMismatch => write!(f, "effect retry checkpoint policy does not match the configured policy"),
            Self::UnknownRetryIntent(id) => write!(f, "unknown retry intent {}", id.0),
            Self::RetryIntentNotPending(id) => {
                write!(f, "retry state references non-pending intent {}", id.0)
            }
            Self::DuplicateOrUnorderedRetryIntent(id) => write!(
                f,
                "duplicate or unordered retry intent {} in checkpoint",
                id.0
            ),
            Self::DuplicateOrUnorderedDeadLetter(id) => write!(
                f,
                "duplicate or unordered dead-letter intent {} in checkpoint",
                id.0
            ),
            Self::UnknownDeadLetter(id) => write!(f, "unknown dead-letter intent {}", id.0),
            Self::InvalidDeadLetterReason(tag) => {
                write!(f, "invalid dead-letter reason tag 0x{tag:02x}")
            }
        }
    }
}

impl Error for EffectRetryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Fencing(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::Persistence(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EffectFencingError> for EffectRetryError {
    fn from(value: EffectFencingError) -> Self {
        Self::Fencing(value)
    }
}

impl From<EffectFenceStoreError> for EffectRetryError {
    fn from(value: EffectFenceStoreError) -> Self {
        Self::Store(value)
    }
}

impl From<EffectPersistenceError> for EffectRetryError {
    fn from(value: EffectPersistenceError) -> Self {
        Self::Persistence(value)
    }
}

impl From<EffectDispatchError> for EffectRetryError {
    fn from(value: EffectDispatchError) -> Self {
        Self::Dispatch(value)
    }
}

pub type EffectRetryResult<T> = Result<T, EffectRetryError>;
