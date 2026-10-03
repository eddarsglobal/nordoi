use nordoi_kernel::{
    execute_nair, AtomSlot, AtomicKernel, AtomicRenderCore, DirtyMask, DomainRef, Instruction,
    NairProgram, NairRenderBridge, NairRenderBridgeError, RegisterId, RenderPrimitive, RenderSpace,
    TransactionSlot, Value,
};

fn program_with_transaction(initial: i64, next: i64, commit: bool) -> NairProgram {
    let mut program = NairProgram::new();
    program.push(Instruction::Const {
        dst: RegisterId(0),
        value: Value::Int(initial),
    });
    program.push(Instruction::Const {
        dst: RegisterId(1),
        value: Value::Int(next),
    });
    program.push(Instruction::CreateAtom {
        dst: AtomSlot(0),
        owner: DomainRef::Root,
        value: RegisterId(0),
    });
    program.push(Instruction::BeginTransaction {
        dst: TransactionSlot(0),
        domain: DomainRef::Root,
    });
    program.push(Instruction::TxSet {
        tx: TransactionSlot(0),
        atom: AtomSlot(0),
        value: RegisterId(1),
    });
    if commit {
        program.push(Instruction::Commit {
            tx: TransactionSlot(0),
        });
    } else {
        program.push(Instruction::Rollback {
            tx: TransactionSlot(0),
        });
    }
    program.push(Instruction::Halt);
    program
}

fn clean_node(
    render: &mut AtomicRenderCore,
    primitive: RenderPrimitive,
) -> nordoi_kernel::RenderNodeId {
    let node = render.create_node(primitive, RenderSpace::Screen);
    assert_eq!(render.flush().len(), 1);
    node
}

#[test]
fn nair_atom_slot_can_bind_to_render_node() {
    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program_with_transaction(0, 1, true)).unwrap();

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Text);
    let bridge = NairRenderBridge::new();

    assert!(bridge
        .bind_slot(&mut render, &report, AtomSlot(0), node, DirtyMask::CONTENT)
        .unwrap());
}

#[test]
fn committed_nair_change_reaches_render_frontier() {
    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program_with_transaction(0, 1, true)).unwrap();

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Text);
    let bridge = NairRenderBridge::new();
    bridge
        .bind_slot(&mut render, &report, AtomSlot(0), node, DirtyMask::CONTENT)
        .unwrap();

    let frame = bridge.pump(&mut kernel, &mut render).unwrap();

    assert_eq!(frame.scheduled_atoms.len(), 1);
    assert_eq!(frame.newly_invalidated_nodes, 1);
    assert_eq!(frame.batch.len(), 1);
    assert_eq!(frame.batch.updates[0].id, node);
    assert!(frame.batch.updates[0].dirty.contains(DirtyMask::CONTENT));
}

#[test]
fn identical_nair_write_creates_zero_render_work() {
    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program_with_transaction(7, 7, true)).unwrap();

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Text);
    let bridge = NairRenderBridge::new();
    bridge
        .bind_slot(&mut render, &report, AtomSlot(0), node, DirtyMask::CONTENT)
        .unwrap();

    let frame = bridge.pump(&mut kernel, &mut render).unwrap();

    assert!(frame.is_empty());
    assert_eq!(frame.newly_invalidated_nodes, 0);
}

#[test]
fn rolled_back_nair_transaction_creates_zero_render_work() {
    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program_with_transaction(0, 9, false)).unwrap();

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Quad);
    let bridge = NairRenderBridge::new();
    bridge
        .bind_slot(
            &mut render,
            &report,
            AtomSlot(0),
            node,
            DirtyMask::APPEARANCE,
        )
        .unwrap();

    let frame = bridge.pump(&mut kernel, &mut render).unwrap();
    assert!(frame.is_empty());
}

#[test]
fn nair_dependency_closure_can_drive_a_dependent_render_node() {
    let mut program = NairProgram::new();
    program.push(Instruction::Const {
        dst: RegisterId(0),
        value: Value::Int(0),
    });
    program.push(Instruction::Const {
        dst: RegisterId(1),
        value: Value::Int(1),
    });
    program.push(Instruction::CreateAtom {
        dst: AtomSlot(0),
        owner: DomainRef::Root,
        value: RegisterId(0),
    });
    program.push(Instruction::CreateAtom {
        dst: AtomSlot(1),
        owner: DomainRef::Root,
        value: RegisterId(0),
    });
    program.push(Instruction::Connect {
        source: AtomSlot(0),
        dependent: AtomSlot(1),
    });
    program.push(Instruction::BeginTransaction {
        dst: TransactionSlot(0),
        domain: DomainRef::Root,
    });
    program.push(Instruction::TxSet {
        tx: TransactionSlot(0),
        atom: AtomSlot(0),
        value: RegisterId(1),
    });
    program.push(Instruction::Commit {
        tx: TransactionSlot(0),
    });
    program.push(Instruction::Halt);

    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program).unwrap();

    let mut render = AtomicRenderCore::new();
    let dependent_node = clean_node(&mut render, RenderPrimitive::Mesh);
    let bridge = NairRenderBridge::new();
    bridge
        .bind_slot(
            &mut render,
            &report,
            AtomSlot(1),
            dependent_node,
            DirtyMask::TRANSFORM,
        )
        .unwrap();

    let frame = bridge.pump(&mut kernel, &mut render).unwrap();

    assert_eq!(frame.scheduled_atoms.len(), 2);
    assert_eq!(frame.batch.len(), 1);
    assert_eq!(frame.batch.updates[0].id, dependent_node);
}

#[test]
fn bridge_merges_multiple_dirty_reasons_for_one_nair_slot() {
    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program_with_transaction(0, 1, true)).unwrap();

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Text);
    let bridge = NairRenderBridge::new();

    bridge
        .bind_slot(&mut render, &report, AtomSlot(0), node, DirtyMask::CONTENT)
        .unwrap();
    bridge
        .bind_slot(
            &mut render,
            &report,
            AtomSlot(0),
            node,
            DirtyMask::APPEARANCE,
        )
        .unwrap();

    let frame = bridge.pump(&mut kernel, &mut render).unwrap();
    assert_eq!(frame.batch.len(), 1);
    assert!(frame.batch.updates[0].dirty.contains(DirtyMask::CONTENT));
    assert!(frame.batch.updates[0].dirty.contains(DirtyMask::APPEARANCE));
}

#[test]
fn unknown_nair_atom_slot_is_rejected_before_render_binding() {
    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program_with_transaction(0, 0, true)).unwrap();

    let mut render = AtomicRenderCore::new();
    let node = clean_node(&mut render, RenderPrimitive::Text);
    let bridge = NairRenderBridge::new();

    assert_eq!(
        bridge.bind_slot(&mut render, &report, AtomSlot(99), node, DirtyMask::CONTENT,),
        Err(NairRenderBridgeError::UnknownAtomSlot(AtomSlot(99)))
    );
}

#[test]
fn screen_and_world_nodes_can_be_driven_by_same_nair_atom() {
    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program_with_transaction(0, 1, true)).unwrap();

    let mut render = AtomicRenderCore::new();
    let screen = render.create_node(RenderPrimitive::Text, RenderSpace::Screen);
    let world = render.create_node(RenderPrimitive::Mesh, RenderSpace::World);
    render.flush();

    let bridge = NairRenderBridge::new();
    bridge
        .bind_slot(
            &mut render,
            &report,
            AtomSlot(0),
            screen,
            DirtyMask::CONTENT,
        )
        .unwrap();
    bridge
        .bind_slot(
            &mut render,
            &report,
            AtomSlot(0),
            world,
            DirtyMask::TRANSFORM,
        )
        .unwrap();

    let frame = bridge.pump(&mut kernel, &mut render).unwrap();
    assert_eq!(frame.batch.len(), 2);
    assert_eq!(frame.batch.updates[0].space, RenderSpace::Screen);
    assert_eq!(frame.batch.updates[1].space, RenderSpace::World);
}
