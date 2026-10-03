mod error;
mod execute;
mod id;
mod instruction;
mod program;

pub use error::{NairError, NairResult};
pub use execute::{
    execute_nair, execute_nair_with_input, execute_nair_with_render,
    execute_nair_with_render_and_input, NairExecutionReport, NairInputExecutionReport,
    NairInteractiveExecutionReport, NairRenderExecutionReport,
};
pub use id::{
    AtomSlot, DomainSlot, InputBridgeSlot, RegisterId, RenderNodeSlot, TimerSlot, TransactionSlot,
};
pub use instruction::{DomainRef, InputTargetRef, Instruction};
pub use program::{
    NairProgram, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR, NAIR_MAGIC, NAIR_MIN_SUPPORTED_MINOR,
};
