use std::error::Error;
use std::fmt::{Display, Formatter};

use super::id::RenderNodeId;

#[derive(Debug, Clone, PartialEq)]
pub enum RenderError {
    UnknownNode(RenderNodeId),
    InvalidOpacity(f32),
    NonFiniteTransform,
    EmptyDirtyMask,
}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownNode(id) => write!(f, "render node {id:?} does not exist"),
            Self::InvalidOpacity(value) => {
                write!(
                    f,
                    "render opacity must be finite and within 0..=1, got {value}"
                )
            }
            Self::NonFiniteTransform => write!(f, "render transform contains a non-finite value"),
            Self::EmptyDirtyMask => write!(f, "render binding cannot use an empty dirty mask"),
        }
    }
}

impl Error for RenderError {}

pub type RenderResult<T> = Result<T, RenderError>;
