use std::collections::BTreeSet;

use super::{dirty::DirtyMask, id::RenderNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderSpace {
    Screen,
    World,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderPrimitive {
    Group,
    Quad,
    Text,
    Mesh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderNode {
    pub id: RenderNodeId,
    pub primitive: RenderPrimitive,
    pub space: RenderSpace,
    pub parent: Option<RenderNodeId>,
    pub children: BTreeSet<RenderNodeId>,
    pub visible: bool,
    pub opacity: f32,
    pub position: [f32; 3],
    pub revision: u64,
    pub dirty: DirtyMask,
}

impl RenderNode {
    pub fn new(
        id: RenderNodeId,
        primitive: RenderPrimitive,
        space: RenderSpace,
        parent: Option<RenderNodeId>,
    ) -> Self {
        Self {
            id,
            primitive,
            space,
            parent,
            children: BTreeSet::new(),
            visible: true,
            opacity: 1.0,
            position: [0.0, 0.0, 0.0],
            revision: 0,
            dirty: DirtyMask::NONE,
        }
    }

    pub(crate) fn mark_dirty(&mut self, mask: DirtyMask) -> bool {
        if mask.is_empty() {
            return false;
        }

        let next = self.dirty | mask;
        if next == self.dirty {
            return false;
        }

        self.dirty = next;
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub(crate) fn clean(&mut self) {
        self.dirty = DirtyMask::NONE;
    }
}
