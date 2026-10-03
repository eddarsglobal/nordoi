use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::atom::AtomId;

use super::{
    dirty::DirtyMask,
    error::{RenderError, RenderResult},
    id::RenderNodeId,
    node::{RenderNode, RenderPrimitive, RenderSpace},
};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderUpdate {
    pub id: RenderNodeId,
    pub dirty: DirtyMask,
    pub revision: u64,
    pub primitive: RenderPrimitive,
    pub space: RenderSpace,
    pub parent: Option<RenderNodeId>,
    pub visible: bool,
    pub opacity: f32,
    pub position: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenderBatch {
    pub updates: Vec<RenderUpdate>,
}

impl RenderBatch {
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }

    pub fn len(&self) -> usize {
        self.updates.len()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AtomicRenderCore {
    nodes: HashMap<RenderNodeId, RenderNode>,
    pending: BTreeSet<RenderNodeId>,
    bindings: HashMap<AtomId, BTreeMap<RenderNodeId, DirtyMask>>,
    next_node_id: u64,
}

impl AtomicRenderCore {
    pub fn new() -> Self {
        Self {
            next_node_id: 1,
            ..Self::default()
        }
    }

    pub fn create_node(&mut self, primitive: RenderPrimitive, space: RenderSpace) -> RenderNodeId {
        self.create_node_internal(None, primitive, space)
    }

    pub fn create_child(
        &mut self,
        parent: RenderNodeId,
        primitive: RenderPrimitive,
        space: RenderSpace,
    ) -> RenderResult<RenderNodeId> {
        self.require_node(parent)?;

        let id = self.create_node_internal(Some(parent), primitive, space);

        let parent_node = self
            .nodes
            .get_mut(&parent)
            .expect("parent existence was validated");
        parent_node.children.insert(id);
        if parent_node.mark_dirty(DirtyMask::STRUCTURE) {
            self.pending.insert(parent);
        }

        Ok(id)
    }

    fn create_node_internal(
        &mut self,
        parent: Option<RenderNodeId>,
        primitive: RenderPrimitive,
        space: RenderSpace,
    ) -> RenderNodeId {
        let id = RenderNodeId(self.next_node_id);
        self.next_node_id += 1;

        let mut node = RenderNode::new(id, primitive, space, parent);
        node.mark_dirty(DirtyMask::ALL);
        self.nodes.insert(id, node);
        self.pending.insert(id);
        id
    }

    pub fn node(&self, id: RenderNodeId) -> RenderResult<&RenderNode> {
        self.nodes.get(&id).ok_or(RenderError::UnknownNode(id))
    }

    pub fn pending_nodes(&self) -> usize {
        self.pending.len()
    }

    pub fn set_opacity(&mut self, id: RenderNodeId, opacity: f32) -> RenderResult<bool> {
        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(RenderError::InvalidOpacity(opacity));
        }

        let node = self
            .nodes
            .get_mut(&id)
            .ok_or(RenderError::UnknownNode(id))?;

        if node.opacity == opacity {
            return Ok(false);
        }

        node.opacity = opacity;
        self.mark_node_dirty(id, DirtyMask::APPEARANCE)?;
        Ok(true)
    }

    pub fn set_position(&mut self, id: RenderNodeId, position: [f32; 3]) -> RenderResult<bool> {
        if position.iter().any(|value| !value.is_finite()) {
            return Err(RenderError::NonFiniteTransform);
        }

        {
            let node = self
                .nodes
                .get_mut(&id)
                .ok_or(RenderError::UnknownNode(id))?;

            if node.position == position {
                return Ok(false);
            }

            node.position = position;
        }

        self.mark_subtree_dirty(id, DirtyMask::TRANSFORM)?;
        Ok(true)
    }

    pub fn set_visible(&mut self, id: RenderNodeId, visible: bool) -> RenderResult<bool> {
        {
            let node = self
                .nodes
                .get_mut(&id)
                .ok_or(RenderError::UnknownNode(id))?;

            if node.visible == visible {
                return Ok(false);
            }

            node.visible = visible;
        }

        self.mark_subtree_dirty(id, DirtyMask::VISIBILITY)?;
        Ok(true)
    }

    pub fn set_space(&mut self, id: RenderNodeId, space: RenderSpace) -> RenderResult<bool> {
        {
            let node = self
                .nodes
                .get_mut(&id)
                .ok_or(RenderError::UnknownNode(id))?;

            if node.space == space {
                return Ok(false);
            }

            node.space = space;
        }

        self.mark_subtree_dirty(id, DirtyMask::TRANSFORM | DirtyMask::STRUCTURE)?;
        Ok(true)
    }

    pub fn bind_atom(
        &mut self,
        atom: AtomId,
        node: RenderNodeId,
        mask: DirtyMask,
    ) -> RenderResult<bool> {
        self.require_node(node)?;
        if mask.is_empty() {
            return Err(RenderError::EmptyDirtyMask);
        }

        let node_bindings = self.bindings.entry(atom).or_default();
        let previous = node_bindings.get(&node).copied().unwrap_or(DirtyMask::NONE);
        let next = previous | mask;

        if next == previous {
            return Ok(false);
        }

        node_bindings.insert(node, next);
        Ok(true)
    }

    /// Converts state invalidation into the minimum render invalidation frontier.
    /// Multiple bindings targeting the same node are merged before flush.
    pub fn invalidate_atom(&mut self, atom: AtomId) -> RenderResult<usize> {
        let bindings = match self.bindings.get(&atom) {
            Some(bindings) => bindings.clone(),
            None => return Ok(0),
        };

        let before = self.pending.len();
        for (node, mask) in bindings {
            self.mark_node_dirty(node, mask)?;
        }
        Ok(self.pending.len().saturating_sub(before))
    }

    pub fn invalidate_atoms<I>(&mut self, atoms: I) -> RenderResult<usize>
    where
        I: IntoIterator<Item = AtomId>,
    {
        let before = self.pending.len();
        for atom in atoms {
            self.invalidate_atom(atom)?;
        }
        Ok(self.pending.len().saturating_sub(before))
    }

    pub fn flush(&mut self) -> RenderBatch {
        let pending = std::mem::take(&mut self.pending);
        let mut updates = Vec::with_capacity(pending.len());

        for id in pending {
            let node = self
                .nodes
                .get_mut(&id)
                .expect("pending render node must exist");

            updates.push(RenderUpdate {
                id: node.id,
                dirty: node.dirty,
                revision: node.revision,
                primitive: node.primitive,
                space: node.space,
                parent: node.parent,
                visible: node.visible,
                opacity: node.opacity,
                position: node.position,
            });

            node.clean();
        }

        RenderBatch { updates }
    }

    fn mark_node_dirty(&mut self, id: RenderNodeId, mask: DirtyMask) -> RenderResult<bool> {
        let node = self
            .nodes
            .get_mut(&id)
            .ok_or(RenderError::UnknownNode(id))?;

        let changed = node.mark_dirty(mask);
        if changed {
            self.pending.insert(id);
        }
        Ok(changed)
    }

    fn mark_subtree_dirty(&mut self, root: RenderNodeId, mask: DirtyMask) -> RenderResult<usize> {
        let subtree = self.subtree(root)?;
        let mut changed = 0usize;

        for id in subtree {
            if self.mark_node_dirty(id, mask)? {
                changed += 1;
            }
        }

        Ok(changed)
    }

    fn subtree(&self, root: RenderNodeId) -> RenderResult<Vec<RenderNodeId>> {
        self.require_node(root)?;

        let mut frontier = vec![root];
        let mut found = BTreeSet::new();

        while let Some(id) = frontier.pop() {
            if !found.insert(id) {
                continue;
            }

            let node = self
                .nodes
                .get(&id)
                .expect("subtree node must exist while graph is immutable");

            frontier.extend(node.children.iter().copied());
        }

        Ok(found.into_iter().collect())
    }

    fn require_node(&self, id: RenderNodeId) -> RenderResult<()> {
        if self.nodes.contains_key(&id) {
            Ok(())
        } else {
            Err(RenderError::UnknownNode(id))
        }
    }
}
