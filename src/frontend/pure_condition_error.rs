use super::error::SourceError;
use super::module_error::NameError;
use super::source::SourceSpan;
use super::type_effect_error::TypeEffectError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureConditionError {
    TypeEffect(TypeEffectError),
    Source(SourceError),
    Name(NameError),
    ExpectedEntryOrEnd { span: SourceSpan },
    ExpectedEntryName { span: SourceSpan },
    ExpectedTerminatorOrReturns { span: SourceSpan },
    ExpectedCondition { span: SourceSpan },
    ExpectedConditionTerminator { span: SourceSpan },
    InvalidBooleanOrComparison { span: SourceSpan },
    InvalidIntegerLiteral { span: SourceSpan },
    IntegerLiteralOutOfRange { span: SourceSpan },
    InvalidComparator { span: SourceSpan },
    MissingRightOperand { span: SourceSpan },
    UnexpectedAfterCondition { span: SourceSpan },
    UnexpectedAfterEntry { span: SourceSpan },
}

impl PureConditionError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::Name(error) => error.primary_span(),
            Self::ExpectedEntryOrEnd { span }
            | Self::ExpectedEntryName { span }
            | Self::ExpectedTerminatorOrReturns { span }
            | Self::ExpectedCondition { span }
            | Self::ExpectedConditionTerminator { span }
            | Self::InvalidBooleanOrComparison { span }
            | Self::InvalidIntegerLiteral { span }
            | Self::IntegerLiteralOutOfRange { span }
            | Self::InvalidComparator { span }
            | Self::MissingRightOperand { span }
            | Self::UnexpectedAfterCondition { span }
            | Self::UnexpectedAfterEntry { span } => Some(*span),
        }
    }
}

impl Display for PureConditionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Name(error) => Display::fmt(error, f),
            Self::ExpectedEntryOrEnd { .. } => write!(
                f,
                "L0.9 body must be empty or begin with contextual 'entry'"
            ),
            Self::ExpectedEntryName { .. } => write!(f, "L0.9 expected entry name"),
            Self::ExpectedTerminatorOrReturns { .. } => write!(
                f,
                "L0.9 entry must end with ';' or continue with contextual 'returns'"
            ),
            Self::ExpectedCondition { .. } => write!(f, "L0.9 expected a pure boolean condition"),
            Self::ExpectedConditionTerminator { .. } => {
                write!(f, "L0.9 condition entry must end with ';'")
            }
            Self::InvalidBooleanOrComparison { .. } => write!(
                f,
                "L0.9 condition must be 'true', 'false', or one canonical integer comparison"
            ),
            Self::InvalidIntegerLiteral { .. } => write!(
                f,
                "L0.9 integer comparison operands must be canonical non-negative decimal literals"
            ),
            Self::IntegerLiteralOutOfRange { .. } => {
                write!(f, "L0.9 integer comparison operand exceeds i64 range")
            }
            Self::InvalidComparator { .. } => write!(
                f,
                "L0.9 comparator must be one of ==, !=, <, <=, >, >= with no trivia inside a two-character operator"
            ),
            Self::MissingRightOperand { .. } => {
                write!(f, "L0.9 comparison is missing its right integer operand")
            }
            Self::UnexpectedAfterCondition { .. } => {
                write!(f, "L0.9 condition has unexpected trailing syntax")
            }
            Self::UnexpectedAfterEntry { .. } => {
                write!(f, "L0.9 allows at most one entry declaration")
            }
        }
    }
}

impl Error for PureConditionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Name(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TypeEffectError> for PureConditionError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<SourceError> for PureConditionError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NameError> for PureConditionError {
    fn from(value: NameError) -> Self {
        Self::Name(value)
    }
}

pub type PureConditionResult<T> = Result<T, PureConditionError>;
