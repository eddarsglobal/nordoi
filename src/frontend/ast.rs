use super::lexer::Token;
use super::source::{SourceId, SourceSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Delimiter {
    Parenthesis,
    Bracket,
    Brace,
}

impl Delimiter {
    pub const fn opener(self) -> char {
        match self {
            Self::Parenthesis => '(',
            Self::Bracket => '[',
            Self::Brace => '{',
        }
    }

    pub const fn closer(self) -> char {
        match self {
            Self::Parenthesis => ')',
            Self::Bracket => ']',
            Self::Brace => '}',
        }
    }

    pub const fn from_opener(character: char) -> Option<Self> {
        match character {
            '(' => Some(Self::Parenthesis),
            '[' => Some(Self::Bracket),
            '{' => Some(Self::Brace),
            _ => None,
        }
    }

    pub const fn from_closer(character: char) -> Option<Self> {
        match character {
            ')' => Some(Self::Parenthesis),
            ']' => Some(Self::Bracket),
            '}' => Some(Self::Brace),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFile {
    source: SourceId,
    span: SourceSpan,
    elements: Vec<AstElement>,
    eof: Token,
}

impl AstFile {
    pub(crate) fn new(
        source: SourceId,
        span: SourceSpan,
        elements: Vec<AstElement>,
        eof: Token,
    ) -> Self {
        Self {
            source,
            span,
            elements,
            eof,
        }
    }

    pub const fn source(&self) -> SourceId {
        self.source
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn elements(&self) -> &[AstElement] {
        &self.elements
    }

    pub const fn eof(&self) -> &Token {
        &self.eof
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstElement {
    Token(Token),
    Group(AstGroup),
}

impl AstElement {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Token(token) => token.span(),
            Self::Group(group) => group.span(),
        }
    }

    pub fn as_token(&self) -> Option<&Token> {
        match self {
            Self::Token(token) => Some(token),
            Self::Group(_) => None,
        }
    }

    pub fn as_group(&self) -> Option<&AstGroup> {
        match self {
            Self::Token(_) => None,
            Self::Group(group) => Some(group),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGroup {
    delimiter: Delimiter,
    span: SourceSpan,
    open: Token,
    elements: Vec<AstElement>,
    close: Token,
}

impl AstGroup {
    pub(crate) fn new(
        delimiter: Delimiter,
        span: SourceSpan,
        open: Token,
        elements: Vec<AstElement>,
        close: Token,
    ) -> Self {
        Self {
            delimiter,
            span,
            open,
            elements,
            close,
        }
    }

    pub const fn delimiter(&self) -> Delimiter {
        self.delimiter
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    pub const fn open(&self) -> &Token {
        &self.open
    }

    pub fn elements(&self) -> &[AstElement] {
        &self.elements
    }

    pub const fn close(&self) -> &Token {
        &self.close
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}
