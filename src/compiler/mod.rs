//! Experimental compiler semantic boundary.
//!
//! C0.7 adds a separate pure-expression execution-plan boundary on top of certified L0.7.
//! Earlier compiler/runtime boundaries and witnesses remain available unchanged. C0.7 preserves
//! the exact pure postfix expression structure without lowering, executing, performing I/O,
//! consulting capabilities, or granting authority.

mod body;
mod effects;
mod error;
mod expression_plan;
mod hir;
mod lower;
mod nair_lowering;
mod nsir;
mod plan;
mod pure_expression;
mod pure_result;
mod result_nair;
mod result_plan;
mod symbols;

pub use body::{
    compile_minimal_body_boundary, lower_minimal_body_unit_to_hir, validate_body_hir, HirBodyUnit,
    HirEntryPoint, HirMinimalBody, NsirBodyUnit, NsirEntryPoint, NsirMinimalBody,
};
pub use effects::{SemanticEffectSet, MAX_SEMANTIC_EFFECT_REQUIREMENTS};
pub use error::{CompilerError, CompilerResult};
pub use expression_plan::{
    compile_pure_expression_execution_plan_boundary, validate_pure_expression_execution_plan,
    PlannedPureExpression, PureExpressionEntryPlan, PureExpressionExecutionPlan,
    PureExpressionPlanForm,
};
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
pub use pure_expression::{
    compile_pure_expression_boundary, lower_pure_expression_unit_to_hir,
    validate_pure_expression_hir, HirPureExpression, HirPureExpressionEntry, HirPureExpressionForm,
    HirPureExpressionOp, HirPureExpressionUnit, NsirPureExpression, NsirPureExpressionEntry,
    NsirPureExpressionForm, NsirPureExpressionUnit, SemanticPureExpressionOp,
};
pub use pure_result::{
    compile_pure_result_boundary, lower_pure_result_unit_to_hir, validate_pure_result_hir,
    HirPureIntResult, HirPureResultEntry, HirPureResultForm, HirPureResultUnit, NsirPureIntResult,
    NsirPureResultEntry, NsirPureResultForm, NsirPureResultUnit,
};
pub use result_nair::{
    compile_pure_result_nair_boundary, lower_pure_result_plan_to_nair, PureResultNairArtifact,
};
pub use result_plan::{
    compile_pure_result_execution_plan_boundary, validate_pure_result_execution_plan,
    PureResultEntryPlan, PureResultExecutionPlan, PureResultPlanForm, PureResultPlanValue,
};
pub use symbols::{
    resolve_effect_set, NsirEffectSymbol, NsirTypeSymbol, ResolvedEffectSet, SemanticEffectId,
    SemanticRegistry, SemanticTypeId,
};
