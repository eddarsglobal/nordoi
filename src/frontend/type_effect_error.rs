use super::error::SourceError;
use super::module_error::{ModuleError, NameError};
use super::source::SourceSpan;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeEffectError {
    Module(ModuleError),
    Source(SourceError),
    Name(NameError),
    ExpectedTypeName { span: SourceSpan },
    ExpectedEffectName { span: SourceSpan },
    ExpectedTypeTerminator { span: SourceSpan },
    ExpectedEffectTerminator { span: SourceSpan },
    TooManyDeclarations { maximum: u32, span: SourceSpan },
}

impl TypeEffectError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Module(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::Name(error) => error.primary_span(),
            Self::ExpectedTypeName { span }
            | Self::ExpectedEffectName { span }
            | Self::ExpectedTypeTerminator { span }
            | Self::ExpectedEffectTerminator { span }
            | Self::TooManyDeclarations { span, .. } => Some(*span),
        }
    }
}

impl Display for TypeEffectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Name(error) => Display::fmt(error, f),
            Self::ExpectedTypeName { .. } => {
                write!(f, "expected a type name after contextual 'type'")
            }
            Self::ExpectedEffectName { .. } => {
                write!(f, "expected an effect name after contextual 'effect'")
            }
            Self::ExpectedTypeTerminator { .. } => {
                write!(f, "opaque type declaration must end with ';'")
            }
            Self::ExpectedEffectTerminator { .. } => {
                write!(f, "effect declaration must end with ';'")
            }
            Self::TooManyDeclarations { maximum, .. } => write!(
                f,
                "type/effect prelude exceeds the L0.4 maximum of {maximum} declarations"
            ),
        }
    }
}

impl Error for TypeEffectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Module(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Name(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ModuleError> for TypeEffectError {
    fn from(value: ModuleError) -> Self {
        Self::Module(value)
    }
}

impl From<SourceError> for TypeEffectError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NameError> for TypeEffectError {
    fn from(value: NameError) -> Self {
        Self::Name(value)
    }
}

pub type TypeEffectResult<T> = Result<T, TypeEffectError>;
