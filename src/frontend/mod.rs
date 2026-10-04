//! Experimental `.noi` language frontend foundation.
//!
//! L0.2 adds a deterministic, lossless structural parser and experimental AST on top of L0.1.
//! It still does not define keywords, operators, declarations, HIR, types, effects or NAIR lowering.

mod ast;
mod error;
mod lexer;
mod parse_error;
mod parser;
mod source;

pub use ast::{AstElement, AstFile, AstGroup, Delimiter};
pub use error::{LexError, LexResult, SourceError, SourceResult};
pub use lexer::{lex, Lexer, Token, TokenKind};
pub use parse_error::{ParseError, ParseResult};
pub use parser::{parse, Parser, MAX_PARSE_NESTING};
pub use source::{ByteOffset, SourceId, SourcePosition, SourceSpan, SourceText, MAX_SOURCE_BYTES};
