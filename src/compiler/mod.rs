//! Experimental compiler semantic boundary.
//!
//! C0.2 builds a canonical semantic registry over the C0.1/L0.4 HIR-NSIR boundary. Opaque nominal
//! `type` declarations and named `effect` identities receive typed, deterministic module-local IDs
//! in separate namespaces. Effect identity remains declared intent only: it grants no host authority
//! and performs no runtime work. Expressions, functions, handlers, effect operations and NSIR → NAIR
//! lowering remain deliberately undefined.

mod effects;
mod error;
mod hir;
mod lower;
mod nsir;
mod symbols;

pub use effects::{SemanticEffectSet, MAX_SEMANTIC_EFFECT_REQUIREMENTS};
pub use error::{CompilerError, CompilerResult};
pub use hir::{
    HirBodyState, HirDeclaration, HirUnit, SemanticDeclarationKind, SemanticModuleIdentity,
    SemanticName, SemanticPath, MAX_SEMANTIC_DECLARATIONS, MAX_SEMANTIC_NAME_BYTES,
    MAX_SEMANTIC_PATH_SEGMENTS,
};
pub use lower::{
    compile_resolved_semantic_boundary, compile_semantic_boundary, compile_type_effect_boundary,
    lower_module_unit_to_hir, lower_type_effect_unit_to_hir,
};
pub use nsir::{validate_hir, NsirBodyState, NsirDeclaration, NsirOrigin, NsirUnit};
pub use symbols::{
    resolve_effect_set, NsirEffectSymbol, NsirTypeSymbol, ResolvedEffectSet, SemanticEffectId,
    SemanticRegistry, SemanticTypeId,
};
