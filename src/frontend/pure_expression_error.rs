use super::error::SourceError;
use super::module_error::NameError;
use super::source::SourceSpan;
use super::type_effect_error::TypeEffectError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureExpressionError {
    TypeEffect(TypeEffectError),
    Source(SourceError),
    Name(NameError),
    ExpectedEntryOrEnd { span: SourceSpan },
    ExpectedEntryName { span: SourceSpan },
    ExpectedTerminatorOrReturns { span: SourceSpan },
    ExpectedExpression { span: SourceSpan },
    ExpectedExpressionTerminator { span: SourceSpan },
    ExpectedOperand { span: SourceSpan },
    ExpectedPlusOrEnd { span: SourceSpan },
    InvalidIntegerLiteral { span: SourceSpan },
    IntegerLiteralOutOfRange { span: SourceSpan },
    UnsupportedGroup { span: SourceSpan },
    EmptyParenthesizedExpression { span: SourceSpan },
    TooManyExpressionNodes { maximum: u32, span: SourceSpan },
    UnexpectedAfterEntry { span: SourceSpan },
}

impl PureExpressionError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::Name(error) => error.primary_span(),
            Self::ExpectedEntryOrEnd { span }
            | Self::ExpectedEntryName { span }
            | Self::ExpectedTerminatorOrReturns { span }
            | Self::ExpectedExpression { span }
            | Self::ExpectedExpressionTerminator { span }
            | Self::ExpectedOperand { span }
            | Self::ExpectedPlusOrEnd { span }
            | Self::InvalidIntegerLiteral { span }
            | Self::IntegerLiteralOutOfRange { span }
            | Self::UnsupportedGroup { span }
            | Self::EmptyParenthesizedExpression { span }
            | Self::TooManyExpressionNodes { span, .. }
            | Self::UnexpectedAfterEntry { span } => Some(*span),
        }
    }
}

impl Display for PureExpressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Name(error) => Display::fmt(error, f),
            Self::ExpectedEntryOrEnd { .. } => write!(
                f,
                "L0.7 body must be empty or begin with contextual 'entry'"
            ),
            Self::ExpectedEntryName { .. } => {
                write!(f, "expected an entry name after contextual 'entry'")
            }
            Self::ExpectedTerminatorOrReturns { .. } => write!(
                f,
                "L0.7 entry must continue with ';' or contextual 'returns'"
            ),
            Self::ExpectedExpression { .. } => write!(
                f,
                "expected a pure integer expression after contextual 'returns'"
            ),
            Self::ExpectedExpressionTerminator { .. } => {
                write!(f, "L0.7 pure-expression entry must end with ';'")
            }
            Self::ExpectedOperand { .. } => write!(
                f,
                "expected a canonical non-negative decimal integer or parenthesized expression"
            ),
            Self::ExpectedPlusOrEnd { .. } => write!(
                f,
                "L0.7 pure expressions currently permit only '+' between operands"
            ),
            Self::InvalidIntegerLiteral { .. } => write!(
                f,
                "L0.7 integer literal must be canonical decimal: '0' or a digit 1-9 followed only by digits"
            ),
            Self::IntegerLiteralOutOfRange { .. } => write!(
                f,
                "L0.7 integer literal exceeds the supported i64 non-negative range"
            ),
            Self::UnsupportedGroup { .. } => write!(
                f,
                "L0.7 pure expressions support only parenthesis groups; '[' and '{{' groups are undefined"
            ),
            Self::EmptyParenthesizedExpression { .. } => write!(
                f,
                "L0.7 parenthesized pure expression must not be empty"
            ),
            Self::TooManyExpressionNodes { maximum, .. } => write!(
                f,
                "L0.7 pure expression exceeds the maximum of {maximum} semantic nodes"
            ),
            Self::UnexpectedAfterEntry { .. } => write!(
                f,
                "L0.7 permits no significant body element after the single entry declaration"
            ),
        }
    }
}

impl Error for PureExpressionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Name(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TypeEffectError> for PureExpressionError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<SourceError> for PureExpressionError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NameError> for PureExpressionError {
    fn from(value: NameError) -> Self {
        Self::Name(value)
    }
}

pub type PureExpressionResult<T> = Result<T, PureExpressionError>;
