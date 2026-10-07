pub mod action;
mod acyclic_calls_v11;
pub mod atom;
pub mod authority;
mod bounded_calls_v10;
pub mod capability;
pub mod compiler;
pub mod conditional_core_v05;
pub mod core_functions_v06;
pub mod dependency;
mod dynamic_branch_v09;
mod dynamic_control_v08;
mod dynamic_input_v07;
pub mod effect;
pub mod effect_attestation;
pub mod effect_audit;
pub mod effect_completion;
pub mod effect_dispatch;
pub mod effect_fencing;
pub mod effect_persistence;
pub mod effect_retry;
pub mod error;
pub mod frontend;
pub mod input;
pub mod kernel;
pub mod modules_v13;
pub mod nair;
mod nested_control_v12;
pub mod ownership;
pub mod program_upgrade;
pub mod project_v14;
pub mod pure_binding_execution;
pub mod pure_expression_execution;
pub mod pure_result_execution;
pub mod reaction;
pub mod render;
pub mod runtime;
pub mod runtime_checkpoint;
pub mod scheduler;
pub mod semantic_stability;
pub mod source_execution;
pub mod time;
pub mod transaction;
pub mod value;

pub use action::ActionSpec;
pub use atom::{Atom, AtomId};
pub use authority::{required_capability, EffectGuard};
pub use capability::{Capability, CapabilitySet};
pub use compiler::{
    compile_execution_plan_boundary, compile_minimal_body_boundary, compile_nair_lowering_boundary,
    compile_pure_binding_boundary, compile_pure_binding_execution_plan_boundary,
    compile_pure_binding_nair_boundary, compile_pure_condition_boundary,
    compile_pure_condition_execution_plan_boundary, compile_pure_expression_boundary,
    compile_pure_expression_execution_plan_boundary, compile_pure_expression_nair_boundary,
    compile_pure_result_boundary, compile_pure_result_execution_plan_boundary,
    compile_pure_result_nair_boundary, compile_resolved_semantic_boundary,
    compile_semantic_boundary, compile_type_effect_boundary, lower_execution_plan_to_nair,
    lower_minimal_body_unit_to_hir, lower_module_unit_to_hir, lower_pure_binding_plan_to_nair,
    lower_pure_binding_unit_to_hir, lower_pure_condition_unit_to_hir,
    lower_pure_expression_plan_to_nair, lower_pure_expression_unit_to_hir,
    lower_pure_result_plan_to_nair, lower_pure_result_unit_to_hir, lower_type_effect_unit_to_hir,
    resolve_effect_set, validate_body_hir, validate_execution_plan, validate_hir,
    validate_pure_binding_execution_plan, validate_pure_binding_hir,
    validate_pure_condition_execution_plan, validate_pure_condition_hir,
    validate_pure_expression_execution_plan, validate_pure_expression_hir,
    validate_pure_result_execution_plan, validate_pure_result_hir, CompilerError, CompilerResult,
    HirBodyState, HirBodyUnit, HirDeclaration, HirEntryPoint, HirMinimalBody, HirPureBinding,
    HirPureBindingEntry, HirPureBindingExpression, HirPureBindingExpressionOp, HirPureBindingForm,
    HirPureBindingUnit, HirPureConditionEntry, HirPureConditionForm, HirPureConditionKind,
    HirPureConditionUnit, HirPureExpression, HirPureExpressionEntry, HirPureExpressionForm,
    HirPureExpressionOp, HirPureExpressionUnit, HirPureIntResult, HirPureResultEntry,
    HirPureResultForm, HirPureResultUnit, HirUnit, NairLoweringArtifact, NsirBodyState,
    NsirBodyUnit, NsirDeclaration, NsirEffectSymbol, NsirEntryPoint, NsirMinimalBody, NsirOrigin,
    NsirPureBindingEntry, NsirPureBindingExpression, NsirPureBindingForm, NsirPureBindingSymbol,
    NsirPureBindingUnit, NsirPureConditionEntry, NsirPureConditionForm, NsirPureConditionUnit,
    NsirPureExpression, NsirPureExpressionEntry, NsirPureExpressionForm, NsirPureExpressionUnit,
    NsirPureIntResult, NsirPureResultEntry, NsirPureResultForm, NsirPureResultUnit, NsirTypeSymbol,
    NsirUnit, PlannedPureBindingExpression, PlannedPureCondition, PlannedPureExpression,
    PureBindingEntryPlan, PureBindingExecutionPlan, PureBindingNairArtifact, PureBindingPlanForm,
    PureBindingRegistry, PureConditionCompilerError, PureConditionCompilerResult,
    PureConditionEntryPlan, PureConditionExecutionPlan, PureConditionPlanError,
    PureConditionPlanForm, PureConditionPlanResult, PureExpressionEntryPlan,
    PureExpressionExecutionPlan, PureExpressionNairArtifact, PureExpressionPlanForm,
    PureResultEntryPlan, PureResultExecutionPlan, PureResultNairArtifact, PureResultPlanForm,
    PureResultPlanValue, ResolvedEffectSet, SemanticDeclarationKind, SemanticEffectId,
    SemanticEffectSet, SemanticEntryPlan, SemanticExecutionPlan, SemanticModuleIdentity,
    SemanticName, SemanticPath, SemanticPlanForm, SemanticPureBindingExpressionOp,
    SemanticPureBindingId, SemanticPureComparator, SemanticPureCondition, SemanticPureExpressionOp,
    SemanticRegistry, SemanticTypeId, MAX_SEMANTIC_DECLARATIONS, MAX_SEMANTIC_EFFECT_REQUIREMENTS,
    MAX_SEMANTIC_NAME_BYTES, MAX_SEMANTIC_PATH_SEGMENTS,
};

pub use acyclic_calls_v11::{
    compile_acyclic_runtime_call_plan_v11, execute_acyclic_runtime_call_source_v11,
    lower_acyclic_runtime_call_plan_v11, AcyclicRuntimeCallError, AcyclicRuntimeCallResult,
    V11AcyclicCallExecutionReport, V11AcyclicCallNairArtifact, V11AcyclicCallPlan,
    MAX_V11_CONSTANTS, MAX_V11_EXPR_NODES, MAX_V11_FUNCTIONS, MAX_V11_NAME_BYTES, MAX_V11_PARAMS,
    MAX_V11_RUNTIME_CALLS, MAX_V11_RUNTIME_CALL_DEPTH,
};
pub use bounded_calls_v10::{
    compile_bounded_runtime_call_plan_v10, execute_bounded_runtime_call_source_v10,
    lower_bounded_runtime_call_plan_v10, BoundedRuntimeCallError, BoundedRuntimeCallResult,
    V10RuntimeCallExecutionReport, V10RuntimeCallNairArtifact, V10RuntimeCallPlan,
    MAX_V10_CONSTANTS, MAX_V10_EXPR_NODES, MAX_V10_FUNCTIONS, MAX_V10_NAME_BYTES, MAX_V10_PARAMS,
    MAX_V10_RUNTIME_CALLS, MAX_V10_RUNTIME_CALL_DEPTH,
};
pub use conditional_core_v05::{
    compile_pure_condition_nair_v05, compile_static_if_nair_v05, compile_static_if_plan_v05,
    execute_pure_condition_source_v05, execute_static_if_source_v05, lower_pure_condition_plan_v05,
    lower_static_if_v05, ConditionalCoreError, ConditionalCoreResult, StaticIfBranch,
    StaticIfCondition, StaticIfOperand, V05ConditionExecutionReport, V05ConditionNairArtifact,
    V05StaticIfExecutionReport, V05StaticIfNairArtifact, V05StaticIfPlan,
};
pub use core_functions_v06::{
    compile_core_plan_v06, execute_core_source_v06, lower_core_plan_v06, CoreFunctionsError,
    CoreFunctionsResult, CoreValue, V06CoreExecutionReport, V06CoreNairArtifact, V06CorePlan,
    MAX_V06_BINDINGS, MAX_V06_CALL_DEPTH, MAX_V06_EXPR_NODES, MAX_V06_FUNCTIONS,
    MAX_V06_NAME_BYTES, MAX_V06_PARAMS,
};
pub use dynamic_branch_v09::{
    compile_dynamic_branch_body_plan_v09, execute_dynamic_branch_body_source_v09,
    lower_dynamic_branch_body_plan_v09, DynamicBranchBodyError, DynamicBranchBodyResult,
    V09BranchBodyExecutionReport, V09BranchBodyNairArtifact, V09BranchBodyPlan, MAX_V09_CONSTANTS,
    MAX_V09_EXPR_NODES, MAX_V09_NAME_BYTES,
};
pub use dynamic_control_v08::{
    compile_dynamic_control_plan_v08, execute_dynamic_control_source_v08,
    lower_dynamic_control_plan_v08, DynamicControlError, DynamicControlResult,
    V08ControlExecutionReport, V08ControlNairArtifact, V08ControlPlan, MAX_V08_CONSTANTS,
    MAX_V08_EXPR_NODES, MAX_V08_NAME_BYTES,
};
pub use dynamic_input_v07::{
    compile_dynamic_plan_v07, execute_dynamic_source_v07, lower_dynamic_plan_v07,
    DynamicInputError, DynamicInputResult, DynamicValue, DynamicValueKind,
    V07DynamicExecutionReport, V07DynamicNairArtifact, V07DynamicPlan, MAX_V07_CONSTANTS,
    MAX_V07_EXPR_NODES, MAX_V07_NAME_BYTES,
};
pub use effect::{Effect, EffectSet};
pub use effect_attestation::{
    EffectAttestationAlgorithmId, EffectAttestationBackendError, EffectAttestationCommitReceipt,
    EffectAttestationError, EffectAttestationKeyId, EffectAttestationResult,
    EffectAttestationSigner, EffectAttestationStore, EffectAttestationStoreError,
    EffectAttestationVerifier, EffectAuditAttestation, EffectAuditAttestationStatement,
    EffectTrustEpoch, GovernedEffectAttestor, MAX_EFFECT_ATTESTATION_BYTES,
    MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES,
};
pub use effect_audit::{
    EffectAttemptId, EffectAuditCheckpoint, EffectAuditDispatchOutcome, EffectAuditError,
    EffectAuditEvent, EffectAuditHash, EffectAuditLedger, EffectAuditRecord, EffectAuditResult,
    EffectAuditSequence, EffectInDoubtAttempt, GovernedAuditedEffectJournal,
    MAX_EFFECT_AUDIT_CHECKPOINT_BYTES, MAX_EFFECT_AUDIT_EVENTS,
};
pub use effect_completion::{
    AtomicEffectCompletionCore, EffectCompletion, EffectCompletionApplicationReport,
    EffectCompletionAuthority, EffectCompletionBatch, EffectCompletionBatchReport,
    EffectCompletionError, EffectCompletionOutcome, EffectCompletionProjection,
    EffectCompletionProjectionValue, EffectCompletionResult, EffectCompletionSequence,
    EffectCompletionSourceId, EffectCompletionWriteReport, MAX_EFFECT_COMPLETION_BATCH,
    MAX_EFFECT_COMPLETION_TEXT_BYTES,
};
pub use effect_dispatch::{
    AtomicEffectOutbox, EffectBackend, EffectBackendError, EffectBackendReceipt,
    EffectDeliveryFence, EffectDeliveryKey, EffectDeliveryNamespace, EffectDispatchAuthority,
    EffectDispatchError, EffectDispatchReceipt, EffectDispatchRequest, EffectDispatchResult,
    EffectIntentId, EffectOutboxStageReport, GovernedEffectDispatcher, QueuedEffectIntent,
};
pub use effect_fencing::{
    EffectFenceStoreError, EffectFencingError, EffectFencingResult, EffectJournalLease,
    EffectJournalWriterId, FencedEffectJournalStore, GovernedFencedEffectJournal,
};
pub use effect_persistence::{
    EffectJournalCommitReceipt, EffectJournalStore, EffectJournalStoreError,
    EffectOutboxCheckpoint, EffectPersistenceError, EffectPersistenceResult, GovernedEffectJournal,
    MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES, MAX_EFFECT_JOURNAL_PENDING,
    MAX_EFFECT_JOURNAL_STRING_BYTES,
};
pub use effect_retry::{
    DeadLetteredEffect, EffectDeadLetterReason, EffectRetryCheckpoint, EffectRetryDispatchOutcome,
    EffectRetryError, EffectRetryLedger, EffectRetryPolicy, EffectRetryRecord, EffectRetryResult,
    EffectRetryTick, GovernedRetryEffectJournal, MAX_EFFECT_DEAD_LETTERS,
    MAX_EFFECT_RETRY_CHECKPOINT_BYTES, MAX_EFFECT_RETRY_ENTRIES,
};
pub use error::{AtomicError, AtomicResult};
pub use frontend::{
    analyze_minimal_body_unit, analyze_module_unit, analyze_pure_binding_unit,
    analyze_pure_condition_unit, analyze_pure_expression_unit, analyze_pure_result_unit,
    analyze_type_effect_unit, lex, parse, AstElement, AstFile, AstGroup, BodyError, BodyResult,
    ByteOffset, Delimiter, LexError, LexResult, Lexer, MinimalBodyAnalyzer, MinimalBodyForm,
    MinimalBodyUnit, ModuleAnalyzer, ModuleDecl, ModuleError, ModulePath, ModuleResult, ModuleUnit,
    Name, NameError, NameResult, ParseError, ParseResult, Parser, PureBindingAnalyzer,
    PureBindingBodyForm, PureBindingError, PureBindingResult, PureBindingUnit,
    PureConditionAnalyzer, PureConditionBodyForm, PureConditionError, PureConditionResult,
    PureConditionUnit, PureExpressionAnalyzer, PureExpressionBodyForm, PureExpressionError,
    PureExpressionResult, PureExpressionUnit, PureResultAnalyzer, PureResultBodyForm,
    PureResultError, PureResultResult, PureResultUnit, SourceError, SourceId, SourcePosition,
    SourceResult, SourceSpan, SourceText, SurfaceConditionEntry, SurfaceDeclaration,
    SurfaceDeclarationKind, SurfaceEntry, SurfaceExpressionEntry, SurfaceIntResult,
    SurfacePureBinding, SurfacePureBindingEntry, SurfacePureBindingExpression,
    SurfacePureBindingExpressionOp, SurfacePureComparator, SurfacePureCondition,
    SurfacePureConditionKind, SurfacePureExpression, SurfacePureExpressionOp, SurfaceResultEntry,
    Token, TokenKind, TypeEffectAnalyzer, TypeEffectError, TypeEffectResult, TypeEffectUnit,
    MAX_MODULE_SEGMENTS, MAX_NAME_BYTES, MAX_PARSE_NESTING, MAX_PURE_BINDINGS,
    MAX_PURE_EXPRESSION_NODES, MAX_SOURCE_BYTES, MAX_TYPE_EFFECT_DECLARATIONS,
};
pub use input::{
    AtomicInputCore, InputAtomBridge, InputBatch, InputBridgeReport, InputDeviceId, InputError,
    InputEvent, InputPayload, InputResult, InputSelector, InputSequence, InputSignal, InputSource,
    InputTarget, PointerId,
};
pub use kernel::AtomicKernel;
pub use modules_v13::{
    compile_module_graph_v13, discover_module_imports_v13, execute_module_graph_v13,
    lower_module_graph_v13, validate_module_name_v13, ModuleGraphError, ModuleGraphResult,
    V13ModuleDiscovery, V13ModuleGraphExecutionReport, V13ModuleGraphNairArtifact,
    V13ModuleGraphPlan, MAX_V13_IMPORTS_PER_MODULE, MAX_V13_MODULES, MAX_V13_TOTAL_IMPORT_EDGES,
};
pub use nair::{
    execute_nair, execute_nair_with_input, execute_nair_with_render,
    execute_nair_with_render_and_input, execute_nair_with_render_and_input_call_observed,
    execute_nair_with_render_and_input_observed,
    execute_nair_with_render_and_input_selective_observed, AtomSlot, BranchExpr, CallExpr,
    CompletionSlot, DomainRef, DomainSlot, InputBridgeSlot, InputTargetRef, Instruction,
    NairBranchWorkReport, NairCallObservedInteractiveExecutionReport, NairCallWorkReport,
    NairCompletionAuthority, NairCompletionBinding, NairCompletionProjection,
    NairCompletionProjectionValue, NairEffectSet, NairError, NairExecutionReport,
    NairInputExecutionReport, NairInteractiveExecutionReport,
    NairObservedInteractiveExecutionReport, NairProgram, NairReactionAuthority,
    NairReactionCycleReport, NairReactionStep, NairReactionTrigger, NairReactionValue,
    NairRenderExecutionReport, NairResult, NairSelectiveObservedInteractiveExecutionReport,
    ReactionSlot, RegisterId, RenderNodeSlot, TimerSlot, TransactionSlot,
    MAX_NAIR_BRANCH_EXPR_DEPTH, MAX_NAIR_BRANCH_EXPR_NODES, MAX_NAIR_CALL_ARGS,
    MAX_NAIR_CALL_EXPR_DEPTH, MAX_NAIR_CALL_EXPR_NODES, MAX_NAIR_CALL_GRAPH_DEPTH,
    NAIR_ACYCLIC_CALL_GRAPH_MINOR, NAIR_DYNAMIC_BRANCH_MINOR, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR,
    NAIR_INPUT_REGISTER_MINOR, NAIR_INTEGER_ARITHMETIC_MINOR, NAIR_INTEGER_COMPARISON_MINOR,
    NAIR_LATEST_FORMAT_MINOR, NAIR_MAGIC, NAIR_MIN_SUPPORTED_MINOR, NAIR_RUNTIME_CALL_MINOR,
    NAIR_SELECTIVE_BRANCH_MINOR, NAIR_STRUCTURED_CALL_CONTROL_MINOR,
};
pub use nested_control_v12::{
    compile_nested_function_control_plan_v12, execute_nested_function_control_source_v12,
    lower_nested_function_control_plan_v12, NestedFunctionControlError,
    NestedFunctionControlResult, V12NestedControlExecutionReport, V12NestedControlNairArtifact,
    V12NestedControlPlan, MAX_V12_CONSTANTS, MAX_V12_EXPR_NODES, MAX_V12_FUNCTIONS,
    MAX_V12_NAME_BYTES, MAX_V12_PARAMS, MAX_V12_RUNTIME_CALLS, MAX_V12_RUNTIME_CALL_DEPTH,
};
pub use ownership::{DomainId, OwnershipDomain, OwnershipRegistry};
pub use program_upgrade::{
    AtomUpgradeRule, DynamicTimerUpgradeRule, ProgramEpoch, RuntimeAllTimerUpgradePlan,
    RuntimeAllTimerUpgradeReport, RuntimeDynamicTimerUpgradePlan, RuntimeTimerAwareUpgradePlan,
    RuntimeTimerUpgradePlan, RuntimeUpgradeAuthority, RuntimeUpgradeError, RuntimeUpgradeHash,
    RuntimeUpgradeLineageRecord, RuntimeUpgradePlan, RuntimeUpgradeReport, RuntimeUpgradeResult,
    TimerUpgradeRule, MAX_RUNTIME_UPGRADE_RULES,
};
pub use project_v14::{
    compile_project_v14, inspect_project_package_v14, parse_project_manifest_v14,
    ProjectBuildError, ProjectBuildResult, V14PackageInfo, V14ProjectBuild, V14ProjectManifest,
    MAX_V14_MANIFEST_BYTES, MAX_V14_PACKAGE_BYTES, MAX_V14_PROJECT_NAME_BYTES,
    MAX_V14_SOURCE_ROOT_BYTES, MAX_V14_VERSION_BYTES, V14_LOCK_FILE, V14_MANIFEST_FILE,
    V14_PACKAGE_MAGIC, V14_PACKAGE_MAJOR, V14_PACKAGE_MINOR,
};
pub use reaction::{
    AtomicReactionCore, EffectIntent, ReactionBatchReport, ReactionError, ReactionId,
    ReactionResult, ReactionSpec, ReactionStep, ReactionTrigger, ReactionValue, TimerSelector,
};
pub use render::{
    AtomicRenderCore, DirtyMask, NairRenderBridge, NairRenderBridgeError, NairRenderBridgeResult,
    NairRenderFrame, RenderBackend, RenderBatch, RenderError, RenderNode, RenderNodeId,
    RenderPrimitive, RenderResult, RenderSpace, RenderUpdate,
};

pub use semantic_stability::{
    kernel_semantic_stability_manifest, kernel_semantic_stability_manifest_bytes,
    kernel_semantic_stability_manifest_hash, ChangePolicy, SemanticKind,
    SemanticStabilityManifestHash, SemanticSurface, StabilityClass,
    KERNEL_SEMANTIC_STABILITY_MANIFEST, KERNEL_STABILITY_MANIFEST_DOMAIN,
    KERNEL_STABILITY_MANIFEST_MAJOR, KERNEL_STABILITY_MANIFEST_MINOR,
};

pub use source_execution::{
    execute_source_v01, validate_v01_execution, SourceExecutionError, SourceExecutionReport,
    SourceExecutionResult,
};

pub use pure_binding_execution::{
    execute_pure_binding_source_v04, validate_v04_execution, PureBindingExecutionError,
    PureBindingExecutionReport, PureBindingExecutionResult,
};

pub use pure_expression_execution::{
    execute_pure_expression_source_v03, validate_v03_execution, PureExpressionExecutionError,
    PureExpressionExecutionReport, PureExpressionExecutionResult,
};

pub use pure_result_execution::{
    execute_pure_result_source_v02, validate_v02_execution, PureResultExecutionError,
    PureResultExecutionReport, PureResultExecutionResult,
};

pub use time::{
    AtomicEventLoop, AtomicTimeCore, EventLoopCycleReport, EventLoopError, EventLoopReplayKey,
    EventLoopResult, LogicalDuration, LogicalTime, TimeAdvanceReport, TimeError, TimeResult,
    TimerFire, TimerId, TimerSnapshot, DEFAULT_TIMER_FIRE_BUDGET,
};
pub use transaction::{AtomicTransaction, TransactionId, TransactionReport};
pub use value::Value;

pub use runtime::{
    run_closed, run_closed_call_observed, run_closed_observed, run_closed_selective_observed,
    AtomicRuntime, PersistentAtomicRuntime, PersistentRuntimeTickReport, RuntimeAtomSnapshot,
    RuntimeCallObservedReport, RuntimeError, RuntimeObservedReport, RuntimeReplayKey,
    RuntimeReport, RuntimeResult, RuntimeSelectiveObservedReport,
};

pub use runtime_checkpoint::{
    FencedRuntimeCheckpointStore, RuntimeCheckpointCommitReceipt, RuntimeCheckpointError,
    RuntimeCheckpointRecoveryReport, RuntimeCheckpointResult, RuntimeCheckpointStoreError,
    RuntimeSemanticCheckpoint, MAX_RUNTIME_CHECKPOINT_ATOMS, MAX_RUNTIME_CHECKPOINT_BYTES,
    MAX_RUNTIME_CHECKPOINT_COMPLETED_DELIVERIES, MAX_RUNTIME_CHECKPOINT_COMPLETION_SOURCES,
    MAX_RUNTIME_CHECKPOINT_TEXT_BYTES, MAX_RUNTIME_CHECKPOINT_TIMERS,
};
