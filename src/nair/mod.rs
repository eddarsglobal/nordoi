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
    execute_nair_with_render_and_input, execute_nair_with_render_and_input_observed,
    NairExecutionReport, NairInputExecutionReport, NairInteractiveExecutionReport,
    NairObservedInteractiveExecutionReport, NairRenderExecutionReport,
};
pub use id::{
    AtomSlot, CompletionSlot, DomainSlot, InputBridgeSlot, ReactionSlot, RegisterId,
    RenderNodeSlot, TimerSlot, TransactionSlot,
};
pub use instruction::{DomainRef, InputTargetRef, Instruction};
pub use program::{
    NairProgram, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR, NAIR_INPUT_REGISTER_MINOR,
    NAIR_INTEGER_ARITHMETIC_MINOR, NAIR_INTEGER_COMPARISON_MINOR, NAIR_LATEST_FORMAT_MINOR,
    NAIR_MAGIC, NAIR_MIN_SUPPORTED_MINOR,
};
pub use reaction::{
    NairEffectSet, NairReactionAuthority, NairReactionCycleReport, NairReactionStep,
    NairReactionTrigger, NairReactionValue,
};

pub(crate) use completion::bootstrap_native_completions;
pub(crate) use reaction::bootstrap_native_reactions;
