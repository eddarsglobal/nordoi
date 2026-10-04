//! Experimental `.noi` language frontend foundation.
//!
//! L0.3 adds bounded source names and a deliberately narrow contextual module header on top of the
//! certified L0.1 lexer and L0.2 structural parser. It still does not define imports, symbol
//! resolution, declarations, expressions, types, effects, packages, HIR/NSIR or NAIR lowering.

mod ast;
mod error;
mod lexer;
mod module;
mod module_error;
mod name;
mod parse_error;
mod parser;
mod source;

pub use ast::{AstElement, AstFile, AstGroup, Delimiter};
pub use error::{LexError, LexResult, SourceError, SourceResult};
pub use lexer::{lex, Lexer, Token, TokenKind};
pub use module::{
    analyze_module_unit, ModuleAnalyzer, ModuleDecl, ModulePath, ModuleUnit, MAX_MODULE_SEGMENTS,
};
pub use module_error::{ModuleError, ModuleResult, NameError, NameResult};
pub use name::{Name, MAX_NAME_BYTES};
pub use parse_error::{ParseError, ParseResult};
pub use parser::{parse, Parser, MAX_PARSE_NESTING};
pub use source::{ByteOffset, SourceId, SourcePosition, SourceSpan, SourceText, MAX_SOURCE_BYTES};
