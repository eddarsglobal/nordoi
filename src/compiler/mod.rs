//! Experimental compiler semantic boundary.
//!
//! C0.3 layers a canonical executable semantic plan over certified L0.5 body semantics. The plan
//! contains zero work items, requires zero effects, grants zero host authority, and never invokes the
//! runtime. The existing C0.1, L0.4, C0.2 and L0.5 boundaries and witnesses remain available
//! unchanged. General functions, calls, expressions, handlers and NSIR → NAIR lowering remain
//! deliberately undefined.

mod body;
mod effects;
mod error;
mod hir;
mod lower;
mod nsir;
mod plan;
mod symbols;

pub use body::{
    compile_minimal_body_boundary, lower_minimal_body_unit_to_hir, validate_body_hir, HirBodyUnit,
    HirEntryPoint, HirMinimalBody, NsirBodyUnit, NsirEntryPoint, NsirMinimalBody,
};
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
pub use plan::{
    compile_execution_plan_boundary, validate_execution_plan, SemanticEntryPlan,
    SemanticExecutionPlan, SemanticPlanForm,
};
pub use symbols::{
    resolve_effect_set, NsirEffectSymbol, NsirTypeSymbol, ResolvedEffectSet, SemanticEffectId,
    SemanticRegistry, SemanticTypeId,
};
