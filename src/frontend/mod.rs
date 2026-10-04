//! Experimental `.noi` language frontend foundation.
//!
//! L0.1 deliberately stops at source text, source spans and lossless lexical tokens.
//! It does not define parser, AST, HIR, type, effect or NAIR lowering semantics.

mod error;
mod lexer;
mod source;

pub use error::{LexError, LexResult, SourceError, SourceResult};
pub use lexer::{lex, Lexer, Token, TokenKind};
pub use source::{ByteOffset, SourceId, SourcePosition, SourceSpan, SourceText, MAX_SOURCE_BYTES};
