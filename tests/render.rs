use nordoi_kernel::{
    AtomicKernel, AtomicRenderCore, DirtyMask, RenderError, RenderPrimitive, RenderSpace,
};

fn clean_node(
    render: &mut AtomicRenderCore,
    primitive: RenderPrimitive,
) -> nordoi_kernel::RenderNodeId {
    let id = render.create_node(primitive, RenderSpace::Screen);
    let batch = render.flush();
    assert_eq!(batch.len(), 1);
    id
}

#[test]
fn new_node_emits_one_initial_update() {
    let mut render = AtomicRenderCore::new();
    let node = render.create_node(RenderPrimitive::Quad, RenderSpace::Screen);

    let batch = render.flush();

    assert_eq!(batch.len(), 1);
    assert_eq!(batch.updates[0].id, node);
    assert_eq!(batch.updates[0].dirty, DirtyMask::ALL);
    assert_eq!(render.pending_nodes(), 0);
}

#[test]
fn identical_property_write_creates_zero_work() {
    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Quad);

    assert!(!render.set_opacity(node, 1.0).unwrap());
    assert_eq!(render.pending_nodes(), 0);
    assert!(render.flush().is_empty());
}

#[test]
fn unrelated_nodes_remain_clean() {
    let mut render = AtomicRenderCore::new();
    let left = render.create_node(RenderPrimitive::Quad, RenderSpace::Screen);
    let right = render.create_node(RenderPrimitive::Quad, RenderSpace::Screen);
    render.flush();

    render.set_opacity(left, 0.5).unwrap();
    let batch = render.flush();

    assert_eq!(batch.len(), 1);
    assert_eq!(batch.updates[0].id, left);
    assert_ne!(batch.updates[0].id, right);
    assert!(batch.updates[0].dirty.contains(DirtyMask::APPEARANCE));
}

#[test]
fn parent_transform_invalidates_only_its_subtree() {
    let mut render = AtomicRenderCore::new();
    let root = render.create_node(RenderPrimitive::Group, RenderSpace::World);
    let child = render
        .create_child(root, RenderPrimitive::Mesh, RenderSpace::World)
        .unwrap();
    let grandchild = render
        .create_child(child, RenderPrimitive::Mesh, RenderSpace::World)
        .unwrap();
    let unrelated = render.create_node(RenderPrimitive::Mesh, RenderSpace::World);
    render.flush();

    render.set_position(root, [1.0, 2.0, 3.0]).unwrap();
    let batch = render.flush();
    let ids: Vec<_> = batch.updates.iter().map(|update| update.id).collect();

    assert_eq!(ids, vec![root, child, grandchild]);
    assert!(!ids.contains(&unrelated));
    assert!(batch
        .updates
        .iter()
        .all(|update| update.dirty.contains(DirtyMask::TRANSFORM)));
}

#[test]
fn repeated_atom_invalidation_collapses_to_one_render_job() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Text);
    render.bind_atom(atom, node, DirtyMask::CONTENT).unwrap();

    assert_eq!(render.invalidate_atom(atom).unwrap(), 1);
    assert_eq!(render.invalidate_atom(atom).unwrap(), 0);
    assert_eq!(render.pending_nodes(), 1);

    let batch = render.flush();
    assert_eq!(batch.len(), 1);
    assert!(batch.updates[0].dirty.contains(DirtyMask::CONTENT));
}

#[test]
fn atom_bindings_union_dirty_reasons_without_duplicate_work() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Text);

    assert!(render.bind_atom(atom, node, DirtyMask::CONTENT).unwrap());
    assert!(render.bind_atom(atom, node, DirtyMask::APPEARANCE).unwrap());

    render.invalidate_atom(atom).unwrap();
    let batch = render.flush();

    assert_eq!(batch.len(), 1);
    assert!(batch.updates[0].dirty.contains(DirtyMask::CONTENT));
    assert!(batch.updates[0].dirty.contains(DirtyMask::APPEARANCE));
}

#[test]
fn one_atom_can_invalidate_multiple_render_nodes() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);

    let mut render = AtomicRenderCore::new();
    let a = render.create_node(RenderPrimitive::Text, RenderSpace::Screen);
    let b = render.create_node(RenderPrimitive::Quad, RenderSpace::Screen);
    render.flush();

    render.bind_atom(atom, a, DirtyMask::CONTENT).unwrap();
    render.bind_atom(atom, b, DirtyMask::APPEARANCE).unwrap();

    assert_eq!(render.invalidate_atom(atom).unwrap(), 2);
    let batch = render.flush();
    let ids: Vec<_> = batch.updates.iter().map(|update| update.id).collect();

    assert_eq!(ids, vec![a, b]);
}

#[test]
fn invalid_render_values_are_rejected_without_work() {
    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Quad);

    assert_eq!(
        render.set_opacity(node, 1.5),
        Err(RenderError::InvalidOpacity(1.5))
    );
    assert_eq!(
        render.set_position(node, [0.0, f32::NAN, 0.0]),
        Err(RenderError::NonFiniteTransform)
    );
    assert_eq!(render.pending_nodes(), 0);
}

#[test]
fn flush_order_is_deterministic() {
    let mut render = AtomicRenderCore::new();
    let first = render.create_node(RenderPrimitive::Quad, RenderSpace::Screen);
    let second = render.create_node(RenderPrimitive::Quad, RenderSpace::Screen);
    let third = render.create_node(RenderPrimitive::Quad, RenderSpace::Screen);
    render.flush();

    render.set_opacity(third, 0.3).unwrap();
    render.set_opacity(first, 0.1).unwrap();
    render.set_opacity(second, 0.2).unwrap();

    let ids: Vec<_> = render
        .flush()
        .updates
        .into_iter()
        .map(|update| update.id)
        .collect();
    assert_eq!(ids, vec![first, second, third]);
}

#[test]
fn screen_and_world_share_the_same_render_core() {
    let mut render = AtomicRenderCore::new();
    let ui = render.create_node(RenderPrimitive::Text, RenderSpace::Screen);
    let mesh = render.create_node(RenderPrimitive::Mesh, RenderSpace::World);

    let batch = render.flush();

    assert_eq!(batch.len(), 2);
    assert_eq!(batch.updates[0].id, ui);
    assert_eq!(batch.updates[0].space, RenderSpace::Screen);
    assert_eq!(batch.updates[1].id, mesh);
    assert_eq!(batch.updates[1].space, RenderSpace::World);
}

#[test]
fn empty_render_binding_is_rejected() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(1_i64);

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Text);

    assert_eq!(
        render.bind_atom(atom, node, DirtyMask::NONE),
        Err(RenderError::EmptyDirtyMask)
    );
    assert_eq!(render.pending_nodes(), 0);
}
