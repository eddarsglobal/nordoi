mod error;
mod execute;
mod id;
mod instruction;
mod program;

pub use error::{NairError, NairResult};
pub use execute::{
    execute_nair, execute_nair_with_render, NairExecutionReport, NairRenderExecutionReport,
};
pub use id::{AtomSlot, DomainSlot, RegisterId, RenderNodeSlot, TransactionSlot};
pub use instruction::{DomainRef, Instruction};
pub use program::{
    NairProgram, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR, NAIR_MAGIC, NAIR_MIN_SUPPORTED_MINOR,
};
