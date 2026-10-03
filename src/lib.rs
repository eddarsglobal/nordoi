pub mod action;
pub mod atom;
pub mod authority;
pub mod capability;
pub mod dependency;
pub mod effect;
pub mod error;
pub mod input;
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
pub use input::{
    AtomicInputCore, InputAtomBridge, InputBatch, InputBridgeReport, InputDeviceId, InputError,
    InputEvent, InputPayload, InputResult, InputSelector, InputSequence, InputSignal, InputSource,
    InputTarget, PointerId,
};
pub use kernel::AtomicKernel;
pub use nair::{
    execute_nair, execute_nair_with_input, execute_nair_with_render,
    execute_nair_with_render_and_input, AtomSlot, DomainRef, DomainSlot, InputBridgeSlot,
    InputTargetRef, Instruction, NairError, NairExecutionReport, NairInputExecutionReport,
    NairInteractiveExecutionReport, NairProgram, NairRenderExecutionReport, NairResult, RegisterId,
    RenderNodeSlot, TransactionSlot, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR, NAIR_MAGIC,
    NAIR_MIN_SUPPORTED_MINOR,
};
pub use ownership::{DomainId, OwnershipDomain, OwnershipRegistry};
pub use render::{
    AtomicRenderCore, DirtyMask, NairRenderBridge, NairRenderBridgeError, NairRenderBridgeResult,
    NairRenderFrame, RenderBackend, RenderBatch, RenderError, RenderNode, RenderNodeId,
    RenderPrimitive, RenderResult, RenderSpace, RenderUpdate,
};
pub use transaction::{AtomicTransaction, TransactionId, TransactionReport};
pub use value::Value;
