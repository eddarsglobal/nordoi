pub mod action;
pub mod atom;
pub mod authority;
pub mod capability;
pub mod dependency;
pub mod effect;
pub mod error;
pub mod kernel;
pub mod nair;
pub mod ownership;
pub mod render;
pub mod scheduler;
pub mod transaction;
pub mod value;

pub use action::ActionSpec;
pub use atom::{Atom, AtomId};
pub use authority::{required_capability, EffectGuard};
pub use capability::{Capability, CapabilitySet};
pub use effect::{Effect, EffectSet};
pub use error::{AtomicError, AtomicResult};
pub use kernel::AtomicKernel;
pub use nair::{
    execute_nair, AtomSlot, DomainRef, DomainSlot, Instruction, NairError, NairExecutionReport,
    NairProgram, NairResult, RegisterId, TransactionSlot, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR,
    NAIR_MAGIC,
};
pub use ownership::{DomainId, OwnershipDomain, OwnershipRegistry};
pub use render::{
    AtomicRenderCore, DirtyMask, NairRenderBridge, NairRenderBridgeError, NairRenderBridgeResult,
    NairRenderFrame, RenderBackend, RenderBatch, RenderError, RenderNode, RenderNodeId,
    RenderPrimitive, RenderResult, RenderSpace, RenderUpdate,
};
pub use transaction::{AtomicTransaction, TransactionId, TransactionReport};
pub use value::Value;
