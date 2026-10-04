use super::ast::{AstElement, AstFile, AstGroup, Delimiter};
use super::lexer::{lex, Token, TokenKind};
use super::parse_error::{ParseError, ParseResult};
use super::source::{SourceSpan, SourceText};

pub const MAX_PARSE_NESTING: u32 = 256;

pub fn parse(source: &SourceText) -> ParseResult<AstFile> {
    Parser::new(source)?.parse()
}

#[derive(Debug)]
pub struct Parser<'a> {
    source: &'a SourceText,
    tokens: Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a SourceText) -> ParseResult<Self> {
        Ok(Self {
            source,
            tokens: lex(source)?,
        })
    }

    pub fn parse(self) -> ParseResult<AstFile> {
        let mut frames = vec![Frame::root()];
        let mut eof = None;

        for token in self.tokens {
            let kind = token.kind().clone();
            match kind {
                TokenKind::Eof => {
                    if eof.is_some() {
                        return Err(ParseError::MultipleEof { span: token.span() });
                    }
                    eof = Some(token);
                }
                TokenKind::Punctuation(character) => {
                    if eof.is_some() {
                        return Err(ParseError::TokenAfterEof { span: token.span() });
                    }

                    if let Some(delimiter) = Delimiter::from_opener(character) {
                        let nesting = (frames.len() - 1) as u32;
                        if nesting >= MAX_PARSE_NESTING {
                            return Err(ParseError::NestingLimitExceeded {
                                maximum: MAX_PARSE_NESTING,
                                span: token.span(),
                            });
                        }
                        frames.push(Frame::group(delimiter, token));
                        continue;
                    }

                    if let Some(found) = Delimiter::from_closer(character) {
                        if frames.len() == 1 {
                            return Err(ParseError::UnexpectedClosingDelimiter {
                                found,
                                span: token.span(),
                            });
                        }

                        let frame = frames
                            .pop()
                            .expect("non-root parse frame must exist before closing delimiter");
                        let expected = frame
                            .delimiter
                            .expect("non-root parse frame must carry a delimiter");

                        if found != expected {
                            return Err(ParseError::MismatchedClosingDelimiter {
                                expected,
                                found,
                                open_span: frame
                                    .open
                                    .as_ref()
                                    .expect("group frame must carry an opening token")
                                    .span(),
                                close_span: token.span(),
                            });
                        }

                        let open = frame.open.expect("group frame must carry an opening token");
                        let span = SourceSpan::new_unchecked(
                            self.source.id(),
                            open.span().start().get(),
                            token.span().end().get(),
                        );
                        let group = AstGroup::new(expected, span, open, frame.elements, token);
                        frames
                            .last_mut()
                            .expect("parent parse frame must exist")
                            .elements
                            .push(AstElement::Group(group));
                        continue;
                    }

                    frames
                        .last_mut()
                        .expect("root parse frame must exist")
                        .elements
                        .push(AstElement::Token(token));
                }
                _ => {
                    if eof.is_some() {
                        return Err(ParseError::TokenAfterEof { span: token.span() });
                    }
                    frames
                        .last_mut()
                        .expect("root parse frame must exist")
                        .elements
                        .push(AstElement::Token(token));
                }
            }
        }

        let eof = eof.ok_or_else(|| ParseError::MissingEof {
            span: SourceSpan::new_unchecked(
                self.source.id(),
                self.source.len().get(),
                self.source.len().get(),
            ),
        })?;

        if frames.len() != 1 {
            let frame = frames.last().expect("an unclosed group frame must remain");
            return Err(ParseError::UnclosedDelimiter {
                delimiter: frame
                    .delimiter
                    .expect("non-root parse frame must carry a delimiter"),
                open_span: frame
                    .open
                    .as_ref()
                    .expect("group frame must carry an opening token")
                    .span(),
                eof_span: eof.span(),
            });
        }

        let root = frames.pop().expect("root parse frame must exist");
        Ok(AstFile::new(
            self.source.id(),
            self.source.full_span(),
            root.elements,
            eof,
        ))
    }
}

#[derive(Debug)]
struct Frame {
    delimiter: Option<Delimiter>,
    open: Option<Token>,
    elements: Vec<AstElement>,
}

impl Frame {
    fn root() -> Self {
        Self {
            delimiter: None,
            open: None,
            elements: Vec::new(),
        }
    }

    fn group(delimiter: Delimiter, open: Token) -> Self {
        Self {
            delimiter: Some(delimiter),
            open: Some(open),
            elements: Vec::new(),
        }
    }
}
