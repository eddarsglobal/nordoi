//! Experimental `.noi` language frontend foundation.
//!
//! L0.5 adds the first fully understood minimal body form on top of L0.4: an empty body or one
//! contextual `entry Name;` declaration. `entry` remains a lexical identifier outside this exact body
//! position. General functions, parameters, calls, expressions, handlers, packages and NAIR lowering
//! remain deliberately undefined.

mod ast;
mod body;
mod body_error;
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
pub use body::{
    analyze_minimal_body_unit, MinimalBodyAnalyzer, MinimalBodyForm, MinimalBodyUnit, SurfaceEntry,
};
pub use body_error::{BodyError, BodyResult};
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
