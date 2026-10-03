use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{error::AtomicError, input::InputError};

#[derive(Debug, PartialEq)]
pub enum ReactionError {
    Atomic(AtomicError),
    Input(InputError),
    EmptyAction,
    ValueSourceMismatch,
    IntegerProjectionOverflow(u64),
    ReactionIdExhausted,
}

impl Display for ReactionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atomic(error) => write!(f, "reaction atomic failure: {error}"),
            Self::Input(error) => write!(f, "reaction input failure: {error}"),
            Self::EmptyAction => write!(f, "reaction action must contain at least one step"),
            Self::ValueSourceMismatch => write!(
                f,
                "reaction value source is incompatible with the reaction trigger"
            ),
            Self::IntegerProjectionOverflow(value) => write!(
                f,
                "reaction integer projection {value} does not fit in NORDOI Value::Int"
            ),
            Self::ReactionIdExhausted => write!(f, "reaction identity space is exhausted"),
        }
    }
}

impl Error for ReactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Atomic(error) => Some(error),
            Self::Input(error) => Some(error),
            Self::EmptyAction
            | Self::ValueSourceMismatch
            | Self::IntegerProjectionOverflow(_)
            | Self::ReactionIdExhausted => None,
        }
    }
}

impl From<AtomicError> for ReactionError {
    fn from(value: AtomicError) -> Self {
        Self::Atomic(value)
    }
}

impl From<InputError> for ReactionError {
    fn from(value: InputError) -> Self {
        Self::Input(value)
    }
}

pub type ReactionResult<T> = Result<T, ReactionError>;
