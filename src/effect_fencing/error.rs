use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    effect_dispatch::{EffectDeliveryFence, EffectDeliveryNamespace, EffectDispatchError},
    effect_persistence::EffectPersistenceError,
};

use super::EffectJournalWriterId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectFenceStoreError {
    message: String,
}

impl EffectFenceStoreError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for EffectFenceStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EffectFenceStoreError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectFencingError {
    Store(EffectFenceStoreError),
    Persistence(EffectPersistenceError),
    Dispatch(EffectDispatchError),
    LeaseRequired,
    LeaseAlreadyHeld,
    InvalidFence(EffectDeliveryFence),
    LeaseNamespaceMismatch {
        expected: EffectDeliveryNamespace,
        actual: EffectDeliveryNamespace,
    },
    LeaseWriterMismatch {
        expected: EffectJournalWriterId,
        actual: EffectJournalWriterId,
    },
}

impl Display for EffectFencingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(f, "effect fencing store failure: {error}"),
            Self::Persistence(error) => write!(f, "effect persistence failure: {error}"),
            Self::Dispatch(error) => write!(f, "effect dispatch failure: {error}"),
            Self::LeaseRequired => write!(f, "an active effect journal lease is required"),
            Self::LeaseAlreadyHeld => write!(f, "this journal already holds an active lease"),
            Self::InvalidFence(fence) => {
                write!(f, "effect journal fence must be non-zero, got {}", fence.0)
            }
            Self::LeaseNamespaceMismatch { expected, actual } => write!(
                f,
                "effect journal lease namespace mismatch: expected {expected}, actual {actual}"
            ),
            Self::LeaseWriterMismatch { expected, actual } => write!(
                f,
                "effect journal lease writer mismatch: expected {expected}, actual {actual}"
            ),
        }
    }
}

impl Error for EffectFencingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Persistence(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EffectFenceStoreError> for EffectFencingError {
    fn from(value: EffectFenceStoreError) -> Self {
        Self::Store(value)
    }
}

impl From<EffectPersistenceError> for EffectFencingError {
    fn from(value: EffectPersistenceError) -> Self {
        Self::Persistence(value)
    }
}

impl From<EffectDispatchError> for EffectFencingError {
    fn from(value: EffectDispatchError) -> Self {
        Self::Dispatch(value)
    }
}

pub type EffectFencingResult<T> = Result<T, EffectFencingError>;
