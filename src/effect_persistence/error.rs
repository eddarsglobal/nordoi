use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::effect_dispatch::{EffectDeliveryNamespace, EffectDispatchError, EffectIntentId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectJournalStoreError {
    message: String,
}

impl EffectJournalStoreError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for EffectJournalStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EffectJournalStoreError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectPersistenceError {
    Store(EffectJournalStoreError),
    Dispatch(EffectDispatchError),
    InvalidMagic,
    UnsupportedVersion {
        major: u16,
        minor: u16,
    },
    Truncated,
    CheckpointTooLarge {
        bytes: usize,
        limit: usize,
    },
    PendingLimitExceeded {
        pending: u64,
        limit: usize,
    },
    StringTooLarge {
        bytes: u64,
        limit: usize,
    },
    ChecksumMismatch {
        expected: u64,
        actual: u64,
    },
    InvalidUtf8,
    UnknownEffectTag(u8),
    InvalidIntentId(EffectIntentId),
    DuplicateOrUnorderedIntent(EffectIntentId),
    InvalidReactionId(u64),
    InvalidNextIntentId(u64),
    NamespaceMismatch {
        expected: EffectDeliveryNamespace,
        actual: EffectDeliveryNamespace,
    },
    RecoveryAfterCycleStarted {
        cycle: u64,
    },
}

impl Display for EffectPersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(f, "effect journal store failure: {error}"),
            Self::Dispatch(error) => write!(f, "effect dispatch failure: {error}"),
            Self::InvalidMagic => write!(f, "invalid effect journal magic"),
            Self::UnsupportedVersion { major, minor } => {
                write!(f, "unsupported effect journal format {major}.{minor}")
            }
            Self::Truncated => write!(f, "truncated effect journal checkpoint"),
            Self::CheckpointTooLarge { bytes, limit } => write!(
                f,
                "effect journal checkpoint is too large: {bytes} bytes exceeds limit {limit}"
            ),
            Self::PendingLimitExceeded { pending, limit } => write!(
                f,
                "effect journal pending count {pending} exceeds limit {limit}"
            ),
            Self::StringTooLarge { bytes, limit } => write!(
                f,
                "effect journal string length {bytes} exceeds limit {limit}"
            ),
            Self::ChecksumMismatch { expected, actual } => write!(
                f,
                "effect journal checksum mismatch: expected {expected:016x}, actual {actual:016x}"
            ),
            Self::InvalidUtf8 => write!(f, "effect journal contains invalid UTF-8"),
            Self::UnknownEffectTag(tag) => write!(f, "unknown persisted effect tag 0x{tag:02x}"),
            Self::InvalidIntentId(id) => write!(f, "invalid persisted effect intent id {}", id.0),
            Self::DuplicateOrUnorderedIntent(id) => write!(
                f,
                "duplicate or non-canonical persisted effect intent id {}",
                id.0
            ),
            Self::InvalidReactionId(id) => write!(f, "invalid persisted reaction id {id}"),
            Self::InvalidNextIntentId(id) => write!(f, "invalid persisted next effect intent id {id}"),
            Self::NamespaceMismatch { expected, actual } => write!(
                f,
                "effect journal namespace mismatch: expected {expected}, actual {actual}"
            ),
            Self::RecoveryAfterCycleStarted { cycle } => write!(
                f,
                "effect journal recovery is only allowed before the first event-loop cycle; current cycle={cycle}"
            ),
        }
    }
}

impl Error for EffectPersistenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EffectJournalStoreError> for EffectPersistenceError {
    fn from(value: EffectJournalStoreError) -> Self {
        Self::Store(value)
    }
}

impl From<EffectDispatchError> for EffectPersistenceError {
    fn from(value: EffectDispatchError) -> Self {
        Self::Dispatch(value)
    }
}

pub type EffectPersistenceResult<T> = Result<T, EffectPersistenceError>;
