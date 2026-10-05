//! Experimental compiler semantic boundary.
//!
//! C0.4 adds the first explicit compiler-owned lowering from the certified C0.3 zero-work semantic
//! plan to existing NAIR 0.6. Only the already-understood zero-work subset is accepted; the emitted
//! program is exactly one terminal `Halt`. The runtime is never invoked and host authority remains
//! outside program bytes. Earlier C0.1/L0.4/C0.2/L0.5/C0.3 witnesses remain available unchanged.

mod body;
mod effects;
mod error;
mod hir;
mod lower;
mod nair_lowering;
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
pub use nair_lowering::{
    compile_nair_lowering_boundary, lower_execution_plan_to_nair, NairLoweringArtifact,
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
