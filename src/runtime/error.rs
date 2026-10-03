use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    error::AtomicError, input::InputError, nair::NairError, reaction::ReactionError,
    render::RenderError,
};

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    Input(InputError),
    Nair(NairError),
    Kernel(AtomicError),
    Render(RenderError),
    Reaction(ReactionError),
    ResidualWork { nam: usize, render: usize },
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(error) => write!(f, "runtime input boundary rejected activation: {error}"),
            Self::Nair(error) => write!(f, "runtime NAIR execution failed: {error}"),
            Self::Kernel(error) => write!(f, "runtime snapshot failed: {error}"),
            Self::Render(error) => write!(f, "runtime render propagation failed: {error}"),
            Self::Reaction(error) => write!(f, "runtime reaction activation failed: {error}"),
            Self::ResidualWork { nam, render } => write!(
                f,
                "closed runtime activation ended with residual work: NAM={nam}, render={render}"
            ),
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Input(error) => Some(error),
            Self::Nair(error) => Some(error),
            Self::Kernel(error) => Some(error),
            Self::Render(error) => Some(error),
            Self::Reaction(error) => Some(error),
            Self::ResidualWork { .. } => None,
        }
    }
}

impl From<InputError> for RuntimeError {
    fn from(value: InputError) -> Self {
        Self::Input(value)
    }
}

impl From<NairError> for RuntimeError {
    fn from(value: NairError) -> Self {
        Self::Nair(value)
    }
}

impl From<RenderError> for RuntimeError {
    fn from(value: RenderError) -> Self {
        Self::Render(value)
    }
}

impl From<AtomicError> for RuntimeError {
    fn from(value: AtomicError) -> Self {
        Self::Kernel(value)
    }
}

impl From<ReactionError> for RuntimeError {
    fn from(value: ReactionError) -> Self {
        Self::Reaction(value)
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
