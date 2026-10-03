use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{capability::Capability, effect::Effect};

use super::EffectIntentId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectBackendError {
    message: String,
    retryable: bool,
}

impl EffectBackendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: true,
        }
    }

    pub fn permanent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: false,
        }
    }

    pub fn with_retryable(message: impl Into<String>, retryable: bool) -> Self {
        Self {
            message: message.into(),
            retryable,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn is_retryable(&self) -> bool {
        self.retryable
    }
}

impl Display for EffectBackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EffectBackendError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDispatchError {
    IntentIdExhausted,
    UnknownIntent(EffectIntentId),
    InternalEffectNotDispatchable(Effect),
    CapabilityDenied(Capability),
    BackendUnsupported {
        intent: EffectIntentId,
        effect: Effect,
    },
    BackendFailed {
        intent: EffectIntentId,
        error: EffectBackendError,
    },
}

impl Display for EffectDispatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IntentIdExhausted => write!(f, "effect-intent identity space is exhausted"),
            Self::UnknownIntent(id) => write!(f, "unknown effect intent {}", id.0),
            Self::InternalEffectNotDispatchable(effect) => write!(
                f,
                "internal effect is not dispatchable through an external backend: {effect:?}"
            ),
            Self::CapabilityDenied(capability) => {
                write!(f, "effect dispatch capability denied: {capability:?}")
            }
            Self::BackendUnsupported { intent, effect } => write!(
                f,
                "effect backend does not support intent {} ({effect:?})",
                intent.0
            ),
            Self::BackendFailed { intent, error } => {
                write!(f, "effect backend failed for intent {}: {error}", intent.0)
            }
        }
    }
}

impl Error for EffectDispatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BackendFailed { error, .. } => Some(error),
            _ => None,
        }
    }
}

pub type EffectDispatchResult<T> = Result<T, EffectDispatchError>;
