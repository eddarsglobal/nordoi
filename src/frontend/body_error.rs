use super::error::SourceError;
use super::module_error::NameError;
use super::source::SourceSpan;
use super::type_effect_error::TypeEffectError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyError {
    TypeEffect(TypeEffectError),
    Source(SourceError),
    Name(NameError),
    ExpectedEntryOrEnd { span: SourceSpan },
    ExpectedEntryName { span: SourceSpan },
    ExpectedEntryTerminator { span: SourceSpan },
    UnexpectedAfterEntry { span: SourceSpan },
}

impl BodyError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::Name(error) => error.primary_span(),
            Self::ExpectedEntryOrEnd { span }
            | Self::ExpectedEntryName { span }
            | Self::ExpectedEntryTerminator { span }
            | Self::UnexpectedAfterEntry { span } => Some(*span),
        }
    }
}

impl Display for BodyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Name(error) => Display::fmt(error, f),
            Self::ExpectedEntryOrEnd { .. } => write!(
                f,
                "L0.5 body must be empty or begin with contextual 'entry'"
            ),
            Self::ExpectedEntryName { .. } => {
                write!(f, "expected an entry name after contextual 'entry'")
            }
            Self::ExpectedEntryTerminator { .. } => {
                write!(f, "L0.5 entry declaration must end with ';'")
            }
            Self::UnexpectedAfterEntry { .. } => write!(
                f,
                "L0.5 permits no significant body element after the single entry declaration"
            ),
        }
    }
}

impl Error for BodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Name(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TypeEffectError> for BodyError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<SourceError> for BodyError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NameError> for BodyError {
    fn from(value: NameError) -> Self {
        Self::Name(value)
    }
}

pub type BodyResult<T> = Result<T, BodyError>;
