mod backend;
mod bridge;
mod core;
mod dirty;
mod error;
mod id;
mod node;

pub use backend::RenderBackend;
pub use bridge::{
    NairRenderBridge, NairRenderBridgeError, NairRenderBridgeResult, NairRenderFrame,
};
pub use core::{AtomicRenderCore, RenderBatch, RenderUpdate};
pub use dirty::DirtyMask;
pub use error::{RenderError, RenderResult};
pub use id::RenderNodeId;
pub use node::{RenderNode, RenderPrimitive, RenderSpace};
