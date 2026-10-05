use super::error::SourceError;
use super::module_error::NameError;
use super::source::SourceSpan;
use super::type_effect_error::TypeEffectError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureBindingError {
    TypeEffect(TypeEffectError),
    Source(SourceError),
    Name(NameError),
    ExpectedConstOrEntryOrEnd { span: SourceSpan },
    ExpectedBindingName { span: SourceSpan },
    ExpectedBindingEquals { span: SourceSpan },
    ExpectedBindingValue { span: SourceSpan },
    InvalidBindingIntegerLiteral { span: SourceSpan },
    BindingIntegerLiteralOutOfRange { span: SourceSpan },
    ExpectedBindingTerminator { span: SourceSpan },
    TooManyBindings { maximum: u32, span: SourceSpan },
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

impl PureBindingError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::TypeEffect(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::Name(error) => error.primary_span(),
            Self::ExpectedConstOrEntryOrEnd { span }
            | Self::ExpectedBindingName { span }
            | Self::ExpectedBindingEquals { span }
            | Self::ExpectedBindingValue { span }
            | Self::InvalidBindingIntegerLiteral { span }
            | Self::BindingIntegerLiteralOutOfRange { span }
            | Self::ExpectedBindingTerminator { span }
            | Self::TooManyBindings { span, .. }
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

impl Display for PureBindingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Name(error) => Display::fmt(error, f),
            Self::ExpectedConstOrEntryOrEnd { .. } => write!(
                f,
                "L0.8 body must contain only leading contextual 'const' declarations followed by at most one contextual 'entry'"
            ),
            Self::ExpectedBindingName { .. } => {
                write!(f, "expected a binding name after contextual 'const'")
            }
            Self::ExpectedBindingEquals { .. } => {
                write!(f, "L0.8 pure binding must continue with '=' after its name")
            }
            Self::ExpectedBindingValue { .. } => write!(
                f,
                "L0.8 pure binding initializer must be a canonical non-negative decimal integer literal"
            ),
            Self::InvalidBindingIntegerLiteral { .. } => write!(
                f,
                "L0.8 binding integer literal must be canonical decimal: '0' or a digit 1-9 followed only by digits"
            ),
            Self::BindingIntegerLiteralOutOfRange { .. } => write!(
                f,
                "L0.8 binding integer literal exceeds the supported i64 non-negative range"
            ),
            Self::ExpectedBindingTerminator { .. } => {
                write!(f, "L0.8 pure binding declaration must end with ';'")
            }
            Self::TooManyBindings { maximum, .. } => write!(
                f,
                "L0.8 pure binding prelude exceeds the maximum of {maximum} bindings"
            ),
            Self::ExpectedEntryName { .. } => {
                write!(f, "expected an entry name after contextual 'entry'")
            }
            Self::ExpectedTerminatorOrReturns { .. } => write!(
                f,
                "L0.8 entry must continue with ';' or contextual 'returns'"
            ),
            Self::ExpectedExpression { .. } => write!(
                f,
                "expected an L0.8 pure expression after contextual 'returns'"
            ),
            Self::ExpectedExpressionTerminator { .. } => {
                write!(f, "L0.8 pure-binding entry must end with ';'")
            }
            Self::ExpectedOperand { .. } => write!(
                f,
                "expected a canonical non-negative integer, a declared pure binding name, or a parenthesized expression"
            ),
            Self::ExpectedPlusOrEnd { .. } => write!(
                f,
                "L0.8 pure expressions currently permit only '+' between operands"
            ),
            Self::InvalidIntegerLiteral { .. } => write!(
                f,
                "L0.8 integer literal must be canonical decimal: '0' or a digit 1-9 followed only by digits"
            ),
            Self::IntegerLiteralOutOfRange { .. } => write!(
                f,
                "L0.8 integer literal exceeds the supported i64 non-negative range"
            ),
            Self::UnsupportedGroup { .. } => write!(
                f,
                "L0.8 pure expressions support only parenthesis groups; '[' and '{{' groups are undefined"
            ),
            Self::EmptyParenthesizedExpression { .. } => write!(
                f,
                "L0.8 parenthesized pure expression must not be empty"
            ),
            Self::TooManyExpressionNodes { maximum, .. } => write!(
                f,
                "L0.8 pure expression exceeds the maximum of {maximum} semantic nodes"
            ),
            Self::UnexpectedAfterEntry { .. } => write!(
                f,
                "L0.8 permits no significant body element after the single entry declaration"
            ),
        }
    }
}

impl Error for PureBindingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeEffect(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Name(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TypeEffectError> for PureBindingError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<SourceError> for PureBindingError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NameError> for PureBindingError {
    fn from(value: NameError) -> Self {
        Self::Name(value)
    }
}

pub type PureBindingResult<T> = Result<T, PureBindingError>;
