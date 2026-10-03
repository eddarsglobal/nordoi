use crate::{
    atom::AtomId,
    kernel::AtomicKernel,
    nair::{AtomSlot, NairExecutionReport},
};

use super::{
    core::{AtomicRenderCore, RenderBatch},
    dirty::DirtyMask,
    error::RenderError,
    id::RenderNodeId,
};

#[derive(Debug, Clone, PartialEq)]
pub enum NairRenderBridgeError {
    UnknownAtomSlot(AtomSlot),
    Render(RenderError),
}

impl std::fmt::Display for NairRenderBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAtomSlot(slot) => {
                write!(f, "NAIR atom slot {slot:?} has no runtime atom binding")
            }
            Self::Render(err) => write!(f, "NAIR render bridge rejected render operation: {err}"),
        }
    }
}

impl std::error::Error for NairRenderBridgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnknownAtomSlot(_) => None,
            Self::Render(err) => Some(err),
        }
    }
}

impl From<RenderError> for NairRenderBridgeError {
    fn from(value: RenderError) -> Self {
        Self::Render(value)
    }
}

pub type NairRenderBridgeResult<T> = Result<T, NairRenderBridgeError>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NairRenderFrame {
    pub scheduled_atoms: Vec<AtomId>,
    pub newly_invalidated_nodes: usize,
    pub batch: RenderBatch,
}

impl NairRenderFrame {
    pub fn is_empty(&self) -> bool {
        self.scheduled_atoms.is_empty() && self.batch.is_empty()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NairRenderBridge;

impl NairRenderBridge {
    pub fn new() -> Self {
        Self
    }

    /// Binds a semantic NAIR AtomSlot to a runtime render node after NAIR execution.
    /// The execution report is the canonical slot -> AtomId authority for this run.
    pub fn bind_slot(
        &self,
        render: &mut AtomicRenderCore,
        report: &NairExecutionReport,
        slot: AtomSlot,
        node: RenderNodeId,
        mask: DirtyMask,
    ) -> NairRenderBridgeResult<bool> {
        let atom = report
            .atom_bindings
            .get(&slot)
            .copied()
            .ok_or(NairRenderBridgeError::UnknownAtomSlot(slot))?;

        Ok(render.bind_atom(atom, node, mask)?)
    }

    /// Consumes NAM's deduplicated atomic work frontier and translates it into
    /// the minimum correct render frontier. No NAM work means no bridge work.
    pub fn pump(
        &self,
        kernel: &mut AtomicKernel,
        render: &mut AtomicRenderCore,
    ) -> NairRenderBridgeResult<NairRenderFrame> {
        let scheduled_atoms = kernel.flush();
        let newly_invalidated_nodes = render.invalidate_atoms(scheduled_atoms.iter().copied())?;
        let batch = render.flush();

        Ok(NairRenderFrame {
            scheduled_atoms,
            newly_invalidated_nodes,
            batch,
        })
    }
}
