//! Experimental `.noi` language frontend foundation.
//!
//! L0.4 adds a bounded contextual declaration prelude on top of the certified L0.1/L0.2/L0.3
//! frontend. Only opaque nominal `type Name;` and named `effect Name;` declarations are recognized.
//! It still does not define imports, symbol resolution, expressions, functions, handlers, packages or
//! NAIR lowering.

mod ast;
mod error;
mod lexer;
mod module;
mod module_error;
mod name;
mod parse_error;
mod parser;
mod source;
mod type_effect;
mod type_effect_error;

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
pub use type_effect::{
    analyze_type_effect_unit, SurfaceDeclaration, SurfaceDeclarationKind, TypeEffectAnalyzer,
    TypeEffectUnit, MAX_TYPE_EFFECT_DECLARATIONS,
};
pub use type_effect_error::{TypeEffectError, TypeEffectResult};
