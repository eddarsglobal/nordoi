use crate::frontend::{ModuleError, SourceError, SourceSpan};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    Frontend(ModuleError),
    Source(SourceError),
    EmptySemanticName,
    SemanticNameTooLong {
        bytes: u32,
        maximum: u32,
    },
    InvalidSemanticName {
        name: String,
    },
    EmptySemanticPath,
    TooManySemanticSegments {
        count: u32,
        maximum: u32,
    },
    SourceMismatch {
        file: SourceSpan,
        other: SourceSpan,
    },
    ModuleSpanOutsideFile {
        file: SourceSpan,
        module: SourceSpan,
    },
    BodySpanOutsideFile {
        file: SourceSpan,
        body: SourceSpan,
    },
    BodyStartsBeforeModuleEnds {
        module: SourceSpan,
        body: SourceSpan,
    },
}

impl CompilerError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Frontend(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::SourceMismatch { other, .. } => Some(*other),
            Self::ModuleSpanOutsideFile { module, .. } => Some(*module),
            Self::BodySpanOutsideFile { body, .. }
            | Self::BodyStartsBeforeModuleEnds { body, .. } => Some(*body),
            Self::EmptySemanticName
            | Self::SemanticNameTooLong { .. }
            | Self::InvalidSemanticName { .. }
            | Self::EmptySemanticPath
            | Self::TooManySemanticSegments { .. } => None,
        }
    }
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Frontend(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::EmptySemanticName => write!(f, "semantic name must not be empty"),
            Self::SemanticNameTooLong { bytes, maximum } => write!(
                f,
                "semantic name has {bytes} bytes; the C0.1 maximum is {maximum} bytes"
            ),
            Self::InvalidSemanticName { name } => write!(
                f,
                "semantic name '{}' does not satisfy the C0.1 ASCII identifier profile",
                name.chars()
                    .flat_map(char::escape_default)
                    .collect::<String>()
            ),
            Self::EmptySemanticPath => {
                write!(
                    f,
                    "named semantic module path must contain at least one segment"
                )
            }
            Self::TooManySemanticSegments { count, maximum } => write!(
                f,
                "semantic module path has {count} segments; the C0.1 maximum is {maximum}"
            ),
            Self::SourceMismatch { .. } => {
                write!(
                    f,
                    "HIR spans from different source units cannot be combined"
                )
            }
            Self::ModuleSpanOutsideFile { .. } => {
                write!(f, "HIR module span must be contained by the file span")
            }
            Self::BodySpanOutsideFile { .. } => {
                write!(f, "HIR body span must be contained by the file span")
            }
            Self::BodyStartsBeforeModuleEnds { .. } => {
                write!(
                    f,
                    "HIR body cannot begin before the module declaration ends"
                )
            }
        }
    }
}

impl Error for CompilerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frontend(error) => Some(error),
            Self::Source(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ModuleError> for CompilerError {
    fn from(value: ModuleError) -> Self {
        Self::Frontend(value)
    }
}

impl From<SourceError> for CompilerError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

pub type CompilerResult<T> = Result<T, CompilerError>;
