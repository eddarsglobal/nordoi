use super::source::{ByteOffset, SourceId, SourceSpan};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    SourceTooLarge {
        bytes: u64,
        maximum: u64,
    },
    SourceMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    InvalidSpanOrder {
        start: ByteOffset,
        end: ByteOffset,
    },
    OffsetOutOfBounds {
        offset: ByteOffset,
        source_len: ByteOffset,
    },
    OffsetNotCharBoundary {
        offset: ByteOffset,
    },
}

impl Display for SourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceTooLarge { bytes, maximum } => {
                write!(
                    f,
                    "source has {bytes} bytes; maximum supported size is {maximum}"
                )
            }
            Self::SourceMismatch { expected, actual } => write!(
                f,
                "source mismatch: expected source {}, got source {}",
                expected.get(),
                actual.get()
            ),
            Self::InvalidSpanOrder { start, end } => write!(
                f,
                "invalid source span: start {} is after end {}",
                start.get(),
                end.get()
            ),
            Self::OffsetOutOfBounds { offset, source_len } => write!(
                f,
                "source offset {} is outside source length {}",
                offset.get(),
                source_len.get()
            ),
            Self::OffsetNotCharBoundary { offset } => write!(
                f,
                "source offset {} is not a UTF-8 scalar boundary",
                offset.get()
            ),
        }
    }
}

impl Error for SourceError {}

pub type SourceResult<T> = Result<T, SourceError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    Source(SourceError),
    NulCharacter { span: SourceSpan },
    BidirectionalControl { character: char, span: SourceSpan },
    UnsupportedWhitespace { character: char, span: SourceSpan },
    UnsupportedIdentifierCharacter { character: char, span: SourceSpan },
    UnexpectedCharacter { character: char, span: SourceSpan },
    NewlineInQuotedText { span: SourceSpan },
    UnterminatedQuotedText { span: SourceSpan },
    UnterminatedBlockComment { span: SourceSpan },
}

impl LexError {
    pub fn span(&self) -> Option<SourceSpan> {
        match self {
            Self::Source(_) => None,
            Self::NulCharacter { span }
            | Self::BidirectionalControl { span, .. }
            | Self::UnsupportedWhitespace { span, .. }
            | Self::UnsupportedIdentifierCharacter { span, .. }
            | Self::UnexpectedCharacter { span, .. }
            | Self::NewlineInQuotedText { span }
            | Self::UnterminatedQuotedText { span }
            | Self::UnterminatedBlockComment { span } => Some(*span),
        }
    }
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => Display::fmt(error, f),
            Self::NulCharacter { .. } => write!(f, "NUL is not permitted in NORDOI source text"),
            Self::BidirectionalControl { character, .. } => write!(
                f,
                "bidirectional control U+{:04X} is not permitted in NORDOI source text",
                *character as u32
            ),
            Self::UnsupportedWhitespace { character, .. } => write!(
                f,
                "whitespace U+{:04X} is not supported by the L0.1 lexer",
                *character as u32
            ),
            Self::UnsupportedIdentifierCharacter { character, .. } => write!(
                f,
                "non-ASCII identifier character U+{:04X} is not supported by the L0.1 lexer",
                *character as u32
            ),
            Self::UnexpectedCharacter { character, .. } => {
                write!(f, "unexpected source character U+{:04X}", *character as u32)
            }
            Self::NewlineInQuotedText { .. } => {
                write!(
                    f,
                    "raw line breaks are not permitted inside L0.1 quoted text"
                )
            }
            Self::UnterminatedQuotedText { .. } => write!(f, "unterminated quoted text"),
            Self::UnterminatedBlockComment { .. } => write!(f, "unterminated block comment"),
        }
    }
}

impl Error for LexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SourceError> for LexError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

pub type LexResult<T> = Result<T, LexError>;
