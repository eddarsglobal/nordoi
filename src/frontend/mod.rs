//! Experimental `.noi` language frontend foundation.
//!
//! L0.6 adds a separate pure-result surface on top of the certified L0.5 body boundary.
//! The new contextual form is `entry Name returns <canonical-decimal-i64>;`. `entry` and `returns`
//! remain lexical identifiers outside this exact L0.6 position. L0.5 remains unchanged and continues
//! to accept only empty bodies or `entry Name;`. General expressions and operators remain undefined.

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
mod pure_result;
mod pure_result_error;
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
pub use pure_result::{
    analyze_pure_result_unit, PureResultAnalyzer, PureResultBodyForm, PureResultUnit,
    SurfaceIntResult, SurfaceResultEntry,
};
pub use pure_result_error::{PureResultError, PureResultResult};
pub use source::{ByteOffset, SourceId, SourcePosition, SourceSpan, SourceText, MAX_SOURCE_BYTES};
pub use type_effect::{
    analyze_type_effect_unit, SurfaceDeclaration, SurfaceDeclarationKind, TypeEffectAnalyzer,
    TypeEffectUnit, MAX_TYPE_EFFECT_DECLARATIONS,
};
pub use type_effect_error::{TypeEffectError, TypeEffectResult};
