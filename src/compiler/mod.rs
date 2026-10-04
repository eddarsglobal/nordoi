//! Experimental compiler semantic boundary.
//!
//! C0.1 introduces source-backed HIR and validated NSIR identities without defining declarations,
//! expressions, source-level types/effects, symbol resolution or lowering to NAIR. The frontend
//! remains experimental and the certified K1.18 kernel semantic floor is unchanged.

mod error;
mod hir;
mod lower;
mod nsir;

pub use error::{CompilerError, CompilerResult};
pub use hir::{
    HirBodyState, HirUnit, SemanticModuleIdentity, SemanticName, SemanticPath,
    MAX_SEMANTIC_NAME_BYTES, MAX_SEMANTIC_PATH_SEGMENTS,
};
pub use lower::{compile_semantic_boundary, lower_module_unit_to_hir};
pub use nsir::{validate_hir, NsirBodyState, NsirOrigin, NsirUnit};
