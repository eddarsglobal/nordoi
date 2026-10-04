use super::ast::Delimiter;
use super::error::LexError;
use super::source::SourceSpan;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Lex(LexError),
    UnexpectedClosingDelimiter {
        found: Delimiter,
        span: SourceSpan,
    },
    MismatchedClosingDelimiter {
        expected: Delimiter,
        found: Delimiter,
        open_span: SourceSpan,
        close_span: SourceSpan,
    },
    UnclosedDelimiter {
        delimiter: Delimiter,
        open_span: SourceSpan,
        eof_span: SourceSpan,
    },
    NestingLimitExceeded {
        maximum: u32,
        span: SourceSpan,
    },
    MissingEof {
        span: SourceSpan,
    },
    MultipleEof {
        span: SourceSpan,
    },
    TokenAfterEof {
        span: SourceSpan,
    },
}

impl ParseError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Lex(error) => error.span(),
            Self::UnexpectedClosingDelimiter { span, .. }
            | Self::NestingLimitExceeded { span, .. }
            | Self::MissingEof { span }
            | Self::MultipleEof { span }
            | Self::TokenAfterEof { span } => Some(*span),
            Self::MismatchedClosingDelimiter { close_span, .. } => Some(*close_span),
            Self::UnclosedDelimiter { open_span, .. } => Some(*open_span),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex(error) => Display::fmt(error, f),
            Self::UnexpectedClosingDelimiter { found, .. } => {
                write!(f, "unexpected closing delimiter '{}'", found.closer())
            }
            Self::MismatchedClosingDelimiter {
                expected, found, ..
            } => write!(
                f,
                "mismatched closing delimiter: expected '{}', found '{}'",
                expected.closer(),
                found.closer()
            ),
            Self::UnclosedDelimiter { delimiter, .. } => write!(
                f,
                "unclosed delimiter '{}'; expected '{}' before end of source",
                delimiter.opener(),
                delimiter.closer()
            ),
            Self::NestingLimitExceeded { maximum, .. } => write!(
                f,
                "syntax nesting exceeds the L0.2 maximum depth of {maximum}"
            ),
            Self::MissingEof { .. } => write!(f, "token stream is missing its EOF token"),
            Self::MultipleEof { .. } => write!(f, "token stream contains more than one EOF token"),
            Self::TokenAfterEof { .. } => write!(f, "token stream contains a token after EOF"),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lex(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LexError> for ParseError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
