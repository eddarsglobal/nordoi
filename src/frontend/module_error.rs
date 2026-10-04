use super::error::SourceError;
use super::parse_error::ParseError;
use super::source::SourceSpan;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameError {
    Source(SourceError),
    ExpectedIdentifier {
        span: SourceSpan,
    },
    TooLong {
        bytes: u32,
        maximum: u32,
        span: SourceSpan,
    },
}

impl NameError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Source(_) => None,
            Self::ExpectedIdentifier { span } | Self::TooLong { span, .. } => Some(*span),
        }
    }
}

impl Display for NameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => Display::fmt(error, f),
            Self::ExpectedIdentifier { .. } => write!(f, "expected an identifier name"),
            Self::TooLong { bytes, maximum, .. } => write!(
                f,
                "name has {bytes} bytes; the L0.3 maximum is {maximum} bytes"
            ),
        }
    }
}

impl Error for NameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SourceError> for NameError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

pub type NameResult<T> = Result<T, NameError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleError {
    Parse(ParseError),
    Source(SourceError),
    Name(NameError),
    ExpectedModuleName {
        span: SourceSpan,
    },
    ExpectedModuleNameAfterSeparator {
        separator_span: SourceSpan,
        span: SourceSpan,
    },
    ExpectedModulePathSeparatorOrTerminator {
        span: SourceSpan,
    },
    MissingModuleTerminator {
        span: SourceSpan,
    },
    TooManyModuleSegments {
        maximum: u32,
        span: SourceSpan,
    },
}

impl ModuleError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Parse(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::Name(error) => error.primary_span(),
            Self::ExpectedModuleName { span }
            | Self::ExpectedModuleNameAfterSeparator { span, .. }
            | Self::ExpectedModulePathSeparatorOrTerminator { span }
            | Self::MissingModuleTerminator { span }
            | Self::TooManyModuleSegments { span, .. } => Some(*span),
        }
    }
}

impl Display for ModuleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Name(error) => Display::fmt(error, f),
            Self::ExpectedModuleName { .. } => {
                write!(f, "expected a module name after contextual 'module'")
            }
            Self::ExpectedModuleNameAfterSeparator { .. } => {
                write!(f, "expected a module-name segment after '.'")
            }
            Self::ExpectedModulePathSeparatorOrTerminator { .. } => {
                write!(f, "expected '.' or ';' after module-name segment")
            }
            Self::MissingModuleTerminator { .. } => {
                write!(f, "module declaration must end with ';'")
            }
            Self::TooManyModuleSegments { maximum, .. } => write!(
                f,
                "module path exceeds the L0.3 maximum of {maximum} segments"
            ),
        }
    }
}

impl Error for ModuleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Name(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ParseError> for ModuleError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<SourceError> for ModuleError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NameError> for ModuleError {
    fn from(value: NameError) -> Self {
        Self::Name(value)
    }
}

pub type ModuleResult<T> = Result<T, ModuleError>;
