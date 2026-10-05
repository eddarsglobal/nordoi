use super::error::SourceError;
use super::module_error::NameError;
use super::source::SourceSpan;
use super::type_effect_error::TypeEffectError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureResultError {
    TypeEffect(TypeEffectError),
    Source(SourceError),
    Name(NameError),
    ExpectedEntryOrEnd { span: SourceSpan },
    ExpectedEntryName { span: SourceSpan },
    ExpectedTerminatorOrReturns { span: SourceSpan },
    ExpectedResultLiteral { span: SourceSpan },
    InvalidResultLiteral { span: SourceSpan },
    ResultLiteralOutOfRange { span: SourceSpan },
    ExpectedResultTerminator { span: SourceSpan },
    UnexpectedAfterEntry { span: SourceSpan },
}

impl PureResultError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::Name(error) => error.primary_span(),
            Self::ExpectedEntryOrEnd { span }
            | Self::ExpectedEntryName { span }
            | Self::ExpectedTerminatorOrReturns { span }
            | Self::ExpectedResultLiteral { span }
            | Self::InvalidResultLiteral { span }
            | Self::ResultLiteralOutOfRange { span }
            | Self::ExpectedResultTerminator { span }
            | Self::UnexpectedAfterEntry { span } => Some(*span),
        }
    }
}

impl Display for PureResultError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Name(error) => Display::fmt(error, f),
            Self::ExpectedEntryOrEnd { .. } => write!(
                f,
                "L0.6 body must be empty or begin with contextual 'entry'"
            ),
            Self::ExpectedEntryName { .. } => {
                write!(f, "expected an entry name after contextual 'entry'")
            }
            Self::ExpectedTerminatorOrReturns { .. } => write!(
                f,
                "L0.6 entry must continue with ';' or contextual 'returns'"
            ),
            Self::ExpectedResultLiteral { .. } => write!(
                f,
                "expected a canonical non-negative decimal integer literal after contextual 'returns'"
            ),
            Self::InvalidResultLiteral { .. } => write!(
                f,
                "L0.6 result literal must be canonical decimal: '0' or a digit 1-9 followed only by digits"
            ),
            Self::ResultLiteralOutOfRange { .. } => write!(
                f,
                "L0.6 result literal exceeds the supported i64 non-negative range"
            ),
            Self::ExpectedResultTerminator { .. } => {
                write!(f, "L0.6 result entry must end with ';'")
            }
            Self::UnexpectedAfterEntry { .. } => write!(
                f,
                "L0.6 permits no significant body element after the single entry declaration"
            ),
        }
    }
}

impl Error for PureResultError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Name(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TypeEffectError> for PureResultError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<SourceError> for PureResultError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NameError> for PureResultError {
    fn from(value: NameError) -> Self {
        Self::Name(value)
    }
}

pub type PureResultResult<T> = Result<T, PureResultError>;
