use super::error::{LexError, LexResult};
use super::source::{SourceSpan, SourceText};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    NumericCandidate,
    QuotedText,
    Punctuation(char),
    Whitespace,
    LineComment,
    BlockComment,
    Eof,
}

impl TokenKind {
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            Self::Whitespace | Self::LineComment | Self::BlockComment
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    span: SourceSpan,
}

impl Token {
    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn is_trivia(&self) -> bool {
        self.kind.is_trivia()
    }
}

pub fn lex(source: &SourceText) -> LexResult<Vec<Token>> {
    Lexer::new(source).lex_all()
}

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a SourceText,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a SourceText) -> Self {
        Self { source, offset: 0 }
    }

    pub fn lex_all(mut self) -> LexResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if eof {
                return Ok(tokens);
            }
        }
    }

    pub fn next_token(&mut self) -> LexResult<Token> {
        if self.offset == self.source.text().len() {
            return Ok(self.token(TokenKind::Eof, self.offset, self.offset));
        }

        let start = self.offset;
        let character = self.current_char();
        self.validate_scalar(start, character)?;

        if is_ascii_whitespace(character) {
            return self.scan_whitespace();
        }

        if is_identifier_start(character) {
            return Ok(self.scan_identifier());
        }

        if character.is_ascii_digit() {
            return Ok(self.scan_numeric_candidate());
        }

        if character == '"' {
            return self.scan_quoted_text();
        }

        if character == '/' {
            if self.starts_with("//") {
                return self.scan_line_comment();
            }
            if self.starts_with("/*") {
                return self.scan_block_comment();
            }
        }

        if character.is_ascii_punctuation() {
            self.offset += 1;
            return Ok(self.token(TokenKind::Punctuation(character), start, self.offset));
        }

        let end = start + character.len_utf8();
        let span = self.span(start, end);
        if character.is_whitespace() {
            return Err(LexError::UnsupportedWhitespace { character, span });
        }
        if character.is_alphanumeric() || character == '_' {
            return Err(LexError::UnsupportedIdentifierCharacter { character, span });
        }
        Err(LexError::UnexpectedCharacter { character, span })
    }

    fn scan_whitespace(&mut self) -> LexResult<Token> {
        let start = self.offset;
        while self.offset < self.source.text().len() {
            let character = self.current_char();
            self.validate_scalar(self.offset, character)?;
            if !is_ascii_whitespace(character) {
                break;
            }
            self.offset += character.len_utf8();
        }
        Ok(self.token(TokenKind::Whitespace, start, self.offset))
    }

    fn scan_identifier(&mut self) -> Token {
        let start = self.offset;
        self.offset += 1;
        let bytes = self.source.text().as_bytes();
        while self.offset < bytes.len() {
            let byte = bytes[self.offset];
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.offset += 1;
            } else {
                break;
            }
        }
        self.token(TokenKind::Identifier, start, self.offset)
    }

    fn scan_numeric_candidate(&mut self) -> Token {
        let start = self.offset;
        self.offset += 1;
        let bytes = self.source.text().as_bytes();
        while self.offset < bytes.len() {
            let byte = bytes[self.offset];
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.offset += 1;
            } else {
                break;
            }
        }
        self.token(TokenKind::NumericCandidate, start, self.offset)
    }

    fn scan_quoted_text(&mut self) -> LexResult<Token> {
        let start = self.offset;
        self.offset += 1;

        while self.offset < self.source.text().len() {
            let current_start = self.offset;
            let character = self.current_char();
            self.validate_scalar(current_start, character)?;

            if character == '"' {
                self.offset += 1;
                return Ok(self.token(TokenKind::QuotedText, start, self.offset));
            }

            if matches!(character, '\n' | '\r') {
                return Err(LexError::NewlineInQuotedText {
                    span: self.span(current_start, current_start + 1),
                });
            }

            if character == '\\' {
                self.offset += 1;
                if self.offset == self.source.text().len() {
                    return Err(LexError::UnterminatedQuotedText {
                        span: self.span(start, self.offset),
                    });
                }
                let escaped_start = self.offset;
                let escaped = self.current_char();
                self.validate_scalar(escaped_start, escaped)?;
                if matches!(escaped, '\n' | '\r') {
                    return Err(LexError::NewlineInQuotedText {
                        span: self.span(escaped_start, escaped_start + 1),
                    });
                }
                self.offset += escaped.len_utf8();
                continue;
            }

            self.offset += character.len_utf8();
        }

        Err(LexError::UnterminatedQuotedText {
            span: self.span(start, self.offset),
        })
    }

    fn scan_line_comment(&mut self) -> LexResult<Token> {
        let start = self.offset;
        self.offset += 2;
        while self.offset < self.source.text().len() {
            let character = self.current_char();
            self.validate_scalar(self.offset, character)?;
            if matches!(character, '\n' | '\r') {
                break;
            }
            self.offset += character.len_utf8();
        }
        Ok(self.token(TokenKind::LineComment, start, self.offset))
    }

    fn scan_block_comment(&mut self) -> LexResult<Token> {
        let start = self.offset;
        self.offset += 2;
        let mut depth = 1u32;

        while self.offset < self.source.text().len() {
            if self.starts_with("/*") {
                depth += 1;
                self.offset += 2;
                continue;
            }
            if self.starts_with("*/") {
                depth -= 1;
                self.offset += 2;
                if depth == 0 {
                    return Ok(self.token(TokenKind::BlockComment, start, self.offset));
                }
                continue;
            }

            let character = self.current_char();
            self.validate_scalar(self.offset, character)?;
            self.offset += character.len_utf8();
        }

        Err(LexError::UnterminatedBlockComment {
            span: self.span(start, self.offset),
        })
    }

    fn current_char(&self) -> char {
        self.source.text()[self.offset..]
            .chars()
            .next()
            .expect("lexer offset must point inside source text")
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.source.text()[self.offset..].starts_with(prefix)
    }

    fn validate_scalar(&self, start: usize, character: char) -> LexResult<()> {
        let span = self.span(start, start + character.len_utf8());
        if character == '\0' {
            return Err(LexError::NulCharacter { span });
        }
        if is_bidirectional_control(character) {
            return Err(LexError::BidirectionalControl { character, span });
        }
        Ok(())
    }

    fn token(&self, kind: TokenKind, start: usize, end: usize) -> Token {
        Token {
            kind,
            span: self.span(start, end),
        }
    }

    fn span(&self, start: usize, end: usize) -> SourceSpan {
        SourceSpan::new_unchecked(self.source.id(), start as u32, end as u32)
    }
}

fn is_ascii_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r')
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_bidirectional_control(character: char) -> bool {
    matches!(
        character,
        '\u{061C}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{202A}'
            | '\u{202B}'
            | '\u{202C}'
            | '\u{202D}'
            | '\u{202E}'
            | '\u{2066}'
            | '\u{2067}'
            | '\u{2068}'
            | '\u{2069}'
    )
}
