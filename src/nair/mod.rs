mod completion;
mod error;
mod execute;
mod id;
mod instruction;
mod program;
mod reaction;

pub use completion::{
    NairCompletionAuthority, NairCompletionBinding, NairCompletionProjection,
    NairCompletionProjectionValue,
};
pub use error::{NairError, NairResult};
pub use execute::{
    execute_nair, execute_nair_with_input, execute_nair_with_render,
    execute_nair_with_render_and_input, execute_nair_with_render_and_input_call_observed,
    execute_nair_with_render_and_input_observed,
    execute_nair_with_render_and_input_selective_observed, NairBranchWorkReport,
    NairCallObservedInteractiveExecutionReport, NairCallWorkReport, NairExecutionReport,
    NairInputExecutionReport, NairInteractiveExecutionReport,
    NairObservedInteractiveExecutionReport, NairRenderExecutionReport,
    NairSelectiveObservedInteractiveExecutionReport,
};
pub use id::{
    AtomSlot, CompletionSlot, DomainSlot, InputBridgeSlot, ReactionSlot, RegisterId,
    RenderNodeSlot, TimerSlot, TransactionSlot,
};
pub use instruction::{BranchExpr, CallExpr, DomainRef, InputTargetRef, Instruction};
pub use program::{
    NairProgram, MAX_NAIR_BRANCH_EXPR_DEPTH, MAX_NAIR_BRANCH_EXPR_NODES, MAX_NAIR_CALL_ARGS,
    MAX_NAIR_CALL_EXPR_DEPTH, MAX_NAIR_CALL_EXPR_NODES, MAX_NAIR_CALL_GRAPH_DEPTH,
    NAIR_ACYCLIC_CALL_GRAPH_MINOR, NAIR_DYNAMIC_BRANCH_MINOR, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR,
    NAIR_INPUT_REGISTER_MINOR, NAIR_INTEGER_ARITHMETIC_MINOR, NAIR_INTEGER_COMPARISON_MINOR,
    NAIR_LATEST_FORMAT_MINOR, NAIR_MAGIC, NAIR_MIN_SUPPORTED_MINOR, NAIR_RUNTIME_CALL_MINOR,
    NAIR_SELECTIVE_BRANCH_MINOR,
};
pub use reaction::{
    NairEffectSet, NairReactionAuthority, NairReactionCycleReport, NairReactionStep,
    NairReactionTrigger, NairReactionValue,
};

pub(crate) use completion::bootstrap_native_completions;
pub(crate) use reaction::bootstrap_native_reactions;
