//! Experimental compiler semantic boundary.
//!
//! C0.1 introduced source-backed HIR and validated NSIR module identity. L0.4 adds a deliberately
//! narrow semantic declaration prelude: opaque nominal `type` declarations and named `effect`
//! identities. Effect identity is declared intent only; it grants no host authority and performs no
//! runtime work. Expressions, functions, handlers, effect operations and NSIR → NAIR lowering remain
//! deliberately undefined.

mod effects;
mod error;
mod hir;
mod lower;
mod nsir;

pub use effects::{SemanticEffectSet, MAX_SEMANTIC_EFFECT_REQUIREMENTS};
pub use error::{CompilerError, CompilerResult};
pub use hir::{
    HirBodyState, HirDeclaration, HirUnit, SemanticDeclarationKind, SemanticModuleIdentity,
    SemanticName, SemanticPath, MAX_SEMANTIC_DECLARATIONS, MAX_SEMANTIC_NAME_BYTES,
    MAX_SEMANTIC_PATH_SEGMENTS,
};
pub use lower::{
    compile_semantic_boundary, compile_type_effect_boundary, lower_module_unit_to_hir,
    lower_type_effect_unit_to_hir,
};
pub use nsir::{validate_hir, NsirBodyState, NsirDeclaration, NsirOrigin, NsirUnit};
