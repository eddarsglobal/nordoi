use super::error::SourceResult;
use super::lexer::{Token, TokenKind};
use super::module_error::{NameError, NameResult};
use super::source::{SourceSpan, SourceText};

pub const MAX_NAME_BYTES: u32 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Name {
    span: SourceSpan,
}

impl Name {
    pub fn from_token(source: &SourceText, token: &Token) -> NameResult<Self> {
        if token.kind() != &TokenKind::Identifier {
            return Err(NameError::ExpectedIdentifier { span: token.span() });
        }

        let text = source.slice(token.span())?;
        let bytes = text.len() as u32;
        if bytes > MAX_NAME_BYTES {
            return Err(NameError::TooLong {
                bytes,
                maximum: MAX_NAME_BYTES,
                span: token.span(),
            });
        }

        Ok(Self { span: token.span() })
    }

    pub const fn span(self) -> SourceSpan {
        self.span
    }

    pub fn text<'a>(&self, source: &'a SourceText) -> SourceResult<&'a str> {
        source.slice(self.span)
    }
}
