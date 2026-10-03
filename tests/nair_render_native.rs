use nordoi_kernel::{
    execute_nair, execute_nair_with_render, AtomSlot, AtomicKernel, AtomicRenderCore, DirtyMask,
    DomainRef, Instruction, NairError, NairProgram, RegisterId, RenderNodeSlot, RenderPrimitive,
    RenderSpace, TransactionSlot, Value, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR,
};

fn state_program(value: i64) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(value),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::Halt,
    ])
}

fn render_binding_program(initial: i64, next: i64, rollback: bool) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(initial),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Text,
            space: RenderSpace::Screen,
        },
        Instruction::BindRenderAtom {
            atom: AtomSlot(0),
            node: RenderNodeSlot(0),
            dirty: DirtyMask::CONTENT,
        },
        Instruction::RenderFlush,
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(next),
        },
        Instruction::BeginTransaction {
            dst: TransactionSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::TxSet {
            tx: TransactionSlot(0),
            atom: AtomSlot(0),
            value: RegisterId(1),
        },
        if rollback {
            Instruction::Rollback {
                tx: TransactionSlot(0),
            }
        } else {
            Instruction::Commit {
                tx: TransactionSlot(0),
            }
        },
        Instruction::RenderFlush,
        Instruction::Halt,
    ])
}

#[test]
fn nair_0_5_is_current_and_0_1_state_programs_still_decode() {
    assert_eq!(NAIR_FORMAT_MAJOR, 0);
    assert_eq!(NAIR_FORMAT_MINOR, 5);

    let program = state_program(7);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 1;
    bytes[7] = 0;

    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, program);

    let reencoded = decoded.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([reencoded[6], reencoded[7]]), 5);
}

#[test]
fn render_instructions_require_explicit_render_context() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Quad,
            space: RenderSpace::Screen,
        },
        Instruction::Halt,
    ]);

    let mut kernel = AtomicKernel::new();
    assert_eq!(
        execute_nair(&mut kernel, &program),
        Err(NairError::RenderContextRequired)
    );
}

#[test]
fn render_slots_are_single_assignment() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Group,
            space: RenderSpace::Screen,
        },
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Quad,
            space: RenderSpace::Screen,
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::DuplicateRenderNodeSlot(RenderNodeSlot(0)))
    );
}

#[test]
fn render_parent_must_exist_before_child_definition() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderChild {
            dst: RenderNodeSlot(1),
            parent: RenderNodeSlot(0),
            primitive: RenderPrimitive::Mesh,
            space: RenderSpace::World,
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownRenderNodeSlot(RenderNodeSlot(0)))
    );
}

#[test]
fn render_program_canonical_round_trip_is_byte_stable() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Group,
            space: RenderSpace::World,
        },
        Instruction::CreateRenderChild {
            dst: RenderNodeSlot(1),
            parent: RenderNodeSlot(0),
            primitive: RenderPrimitive::Mesh,
            space: RenderSpace::World,
        },
        Instruction::BindRenderAtom {
            atom: AtomSlot(0),
            node: RenderNodeSlot(1),
            dirty: DirtyMask::TRANSFORM | DirtyMask::CONTENT,
        },
        Instruction::SetRenderVisible {
            node: RenderNodeSlot(1),
            visible: true,
        },
        Instruction::SetRenderOpacity {
            node: RenderNodeSlot(1),
            opacity: 0.75,
        },
        Instruction::SetRenderPosition {
            node: RenderNodeSlot(1),
            position: [1.0, 2.0, 3.0],
        },
        Instruction::RenderFlush,
        Instruction::Halt,
    ]);

    let bytes = program.canonical_bytes().unwrap();
    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    let reencoded = decoded.canonical_bytes().unwrap();

    assert_eq!(decoded, program);
    assert_eq!(reencoded, bytes);
}

#[test]
fn native_nair_creates_screen_world_hierarchy_and_properties() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Group,
            space: RenderSpace::World,
        },
        Instruction::CreateRenderChild {
            dst: RenderNodeSlot(1),
            parent: RenderNodeSlot(0),
            primitive: RenderPrimitive::Mesh,
            space: RenderSpace::World,
        },
        Instruction::SetRenderPosition {
            node: RenderNodeSlot(1),
            position: [4.0, 5.0, 6.0],
        },
        Instruction::SetRenderOpacity {
            node: RenderNodeSlot(1),
            opacity: 0.5,
        },
        Instruction::SetRenderVisible {
            node: RenderNodeSlot(1),
            visible: false,
        },
        Instruction::RenderFlush,
        Instruction::Halt,
    ]);

    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render(&mut kernel, &mut render, &program).unwrap();

    let root = report.render_bindings[&RenderNodeSlot(0)];
    let child = report.render_bindings[&RenderNodeSlot(1)];
    let root_node = render.node(root).unwrap();
    let child_node = render.node(child).unwrap();

    assert_eq!(report.created_render_nodes(), 2);
    assert_eq!(child_node.parent, Some(root));
    assert!(root_node.children.contains(&child));
    assert_eq!(child_node.position, [4.0, 5.0, 6.0]);
    assert_eq!(child_node.opacity, 0.5);
    assert!(!child_node.visible);
    assert_eq!(report.frames.len(), 1);
    assert_eq!(report.frames[0].batch.len(), 2);
}

#[test]
fn explicit_render_flush_defines_frame_boundaries() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Quad,
            space: RenderSpace::Screen,
        },
        Instruction::RenderFlush,
        Instruction::SetRenderOpacity {
            node: RenderNodeSlot(0),
            opacity: 0.25,
        },
        Instruction::RenderFlush,
        Instruction::Halt,
    ]);

    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render(&mut kernel, &mut render, &program).unwrap();

    assert_eq!(report.frames.len(), 2);
    assert_eq!(report.frames[0].batch.len(), 1);
    assert_eq!(report.frames[1].batch.len(), 1);
    assert!(report.frames[1].batch.updates[0]
        .dirty
        .contains(DirtyMask::APPEARANCE));
}

#[test]
fn committed_atom_change_reaches_bound_render_node() {
    let program = render_binding_program(0, 1, false);
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render(&mut kernel, &mut render, &program).unwrap();

    assert_eq!(report.frames.len(), 2);
    assert_eq!(report.frames[1].scheduled_atoms.len(), 1);
    assert_eq!(report.frames[1].batch.len(), 1);
    assert!(report.frames[1].batch.updates[0]
        .dirty
        .contains(DirtyMask::CONTENT));
}

#[test]
fn identical_atom_write_after_clean_frame_creates_zero_render_work() {
    let program = render_binding_program(5, 5, false);
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render(&mut kernel, &mut render, &program).unwrap();

    assert_eq!(report.frames.len(), 2);
    assert!(report.frames[1].scheduled_atoms.is_empty());
    assert!(report.frames[1].batch.is_empty());
}

#[test]
fn rollback_after_clean_frame_creates_zero_render_work() {
    let program = render_binding_program(5, 9, true);
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render(&mut kernel, &mut render, &program).unwrap();

    assert_eq!(report.frames.len(), 2);
    assert!(report.frames[1].scheduled_atoms.is_empty());
    assert!(report.frames[1].batch.is_empty());
}

#[test]
fn halt_implicitly_flushes_pending_render_work_once() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Text,
            space: RenderSpace::Screen,
        },
        Instruction::Halt,
    ]);

    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render(&mut kernel, &mut render, &program).unwrap();

    assert_eq!(report.frames.len(), 1);
    assert_eq!(report.frames[0].batch.len(), 1);
    assert_eq!(render.pending_nodes(), 0);
}

#[test]
fn invalid_render_opacity_is_rejected_before_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Quad,
            space: RenderSpace::Screen,
        },
        Instruction::SetRenderOpacity {
            node: RenderNodeSlot(0),
            opacity: 1.5,
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::InvalidRenderOpacity(1.5))
    );
}

#[test]
fn non_finite_render_position_is_rejected_before_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Mesh,
            space: RenderSpace::World,
        },
        Instruction::SetRenderPosition {
            node: RenderNodeSlot(0),
            position: [0.0, f32::INFINITY, 0.0],
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::NonFiniteRenderPosition(RenderNodeSlot(0)))
    );
}

#[test]
fn empty_render_binding_mask_is_rejected_before_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Text,
            space: RenderSpace::Screen,
        },
        Instruction::BindRenderAtom {
            atom: AtomSlot(0),
            node: RenderNodeSlot(0),
            dirty: DirtyMask::NONE,
        },
        Instruction::Halt,
    ]);

    assert_eq!(program.validate(), Err(NairError::EmptyRenderDirtyMask));
}

#[test]
fn nair_0_1_cannot_smuggle_0_2_render_opcodes() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Quad,
            space: RenderSpace::Screen,
        },
        Instruction::Halt,
    ]);

    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 1;
    bytes[7] = 0;

    assert_eq!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidOpcode(0x30))
    );
}
