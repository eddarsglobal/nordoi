//! Experimental compiler semantic boundary.
//!
//! C0.5 adds a separate pure-result execution-plan boundary on top of certified L0.6 without
//! changing C0.3/C0.4, NAIR 0.6, V0.1 runtime execution, or host authority. Earlier witnesses
//! remain available unchanged.

mod body;
mod effects;
mod error;
mod hir;
mod lower;
mod nair_lowering;
mod nsir;
mod plan;
mod pure_result;
mod result_plan;
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
pub use pure_result::{
    compile_pure_result_boundary, lower_pure_result_unit_to_hir, validate_pure_result_hir,
    HirPureIntResult, HirPureResultEntry, HirPureResultForm, HirPureResultUnit, NsirPureIntResult,
    NsirPureResultEntry, NsirPureResultForm, NsirPureResultUnit,
};
pub use result_plan::{
    compile_pure_result_execution_plan_boundary, validate_pure_result_execution_plan,
    PureResultEntryPlan, PureResultExecutionPlan, PureResultPlanForm, PureResultPlanValue,
};
pub use symbols::{
    resolve_effect_set, NsirEffectSymbol, NsirTypeSymbol, ResolvedEffectSet, SemanticEffectId,
    SemanticRegistry, SemanticTypeId,
};
