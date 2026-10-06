pub mod action;
pub mod atom;
pub mod authority;
pub mod capability;
pub mod compiler;
pub mod conditional_core_v05;
pub mod dependency;
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
pub mod nair;
pub mod ownership;
pub mod program_upgrade;
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

pub use conditional_core_v05::{
    compile_pure_condition_nair_v05, compile_static_if_nair_v05, compile_static_if_plan_v05,
    execute_pure_condition_source_v05, execute_static_if_source_v05, lower_pure_condition_plan_v05,
    lower_static_if_v05, ConditionalCoreError, ConditionalCoreResult, StaticIfBranch,
    StaticIfCondition, StaticIfOperand, V05ConditionExecutionReport, V05ConditionNairArtifact,
    V05StaticIfExecutionReport, V05StaticIfNairArtifact, V05StaticIfPlan,
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
pub use nair::{
    execute_nair, execute_nair_with_input, execute_nair_with_render,
    execute_nair_with_render_and_input, execute_nair_with_render_and_input_observed, AtomSlot,
    CompletionSlot, DomainRef, DomainSlot, InputBridgeSlot, InputTargetRef, Instruction,
    NairCompletionAuthority, NairCompletionBinding, NairCompletionProjection,
    NairCompletionProjectionValue, NairEffectSet, NairError, NairExecutionReport,
    NairInputExecutionReport, NairInteractiveExecutionReport,
    NairObservedInteractiveExecutionReport, NairProgram, NairReactionAuthority,
    NairReactionCycleReport, NairReactionStep, NairReactionTrigger, NairReactionValue,
    NairRenderExecutionReport, NairResult, ReactionSlot, RegisterId, RenderNodeSlot, TimerSlot,
    TransactionSlot, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR, NAIR_INTEGER_ARITHMETIC_MINOR,
    NAIR_INTEGER_COMPARISON_MINOR, NAIR_LATEST_FORMAT_MINOR, NAIR_MAGIC, NAIR_MIN_SUPPORTED_MINOR,
};
pub use ownership::{DomainId, OwnershipDomain, OwnershipRegistry};
pub use program_upgrade::{
    AtomUpgradeRule, DynamicTimerUpgradeRule, ProgramEpoch, RuntimeAllTimerUpgradePlan,
    RuntimeAllTimerUpgradeReport, RuntimeDynamicTimerUpgradePlan, RuntimeTimerAwareUpgradePlan,
    RuntimeTimerUpgradePlan, RuntimeUpgradeAuthority, RuntimeUpgradeError, RuntimeUpgradeHash,
    RuntimeUpgradeLineageRecord, RuntimeUpgradePlan, RuntimeUpgradeReport, RuntimeUpgradeResult,
    TimerUpgradeRule, MAX_RUNTIME_UPGRADE_RULES,
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
    run_closed, run_closed_observed, AtomicRuntime, PersistentAtomicRuntime,
    PersistentRuntimeTickReport, RuntimeAtomSnapshot, RuntimeError, RuntimeObservedReport,
    RuntimeReplayKey, RuntimeReport, RuntimeResult,
};

pub use runtime_checkpoint::{
    FencedRuntimeCheckpointStore, RuntimeCheckpointCommitReceipt, RuntimeCheckpointError,
    RuntimeCheckpointRecoveryReport, RuntimeCheckpointResult, RuntimeCheckpointStoreError,
    RuntimeSemanticCheckpoint, MAX_RUNTIME_CHECKPOINT_ATOMS, MAX_RUNTIME_CHECKPOINT_BYTES,
    MAX_RUNTIME_CHECKPOINT_COMPLETED_DELIVERIES, MAX_RUNTIME_CHECKPOINT_COMPLETION_SOURCES,
    MAX_RUNTIME_CHECKPOINT_TEXT_BYTES, MAX_RUNTIME_CHECKPOINT_TIMERS,
};
