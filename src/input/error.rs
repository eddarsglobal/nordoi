use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::error::AtomicError;

#[derive(Debug, PartialEq)]
pub enum InputError {
    NonFiniteValue,
    AxisOutOfRange(f32),
    ZeroOrientation,
    SequenceExhausted,
    Atomic(AtomicError),
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFiniteValue => write!(f, "input contains a non-finite numeric value"),
            Self::AxisOutOfRange(value) => {
                write!(f, "normalized input axis {value} is outside [-1, 1]")
            }
            Self::ZeroOrientation => write!(f, "XR pose orientation cannot have zero length"),
            Self::SequenceExhausted => write!(f, "input sequence space is exhausted"),
            Self::Atomic(error) => write!(f, "atomic input bridge error: {error}"),
        }
    }
}

impl Error for InputError {}

impl From<AtomicError> for InputError {
    fn from(value: AtomicError) -> Self {
        Self::Atomic(value)
    }
}

pub type InputResult<T> = Result<T, InputError>;
