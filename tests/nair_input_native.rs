use nordoi_kernel::{
    execute_nair, execute_nair_with_input, execute_nair_with_render,
    execute_nair_with_render_and_input, AtomSlot, AtomicInputCore, AtomicKernel, AtomicRenderCore,
    DirtyMask, DomainRef, DomainSlot, InputBridgeSlot, InputDeviceId, InputPayload, InputSignal,
    InputSource, InputTargetRef, Instruction, NairError, NairProgram, RegisterId, RenderNodeSlot,
    RenderPrimitive, RenderSpace, Value, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR,
};

fn keyboard_batch(code: u32, pressed: bool, device: InputDeviceId) -> nordoi_kernel::InputBatch {
    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Keyboard,
            device,
            None,
            InputPayload::Key {
                code,
                pressed,
                repeat: false,
            },
        )
        .unwrap();
    input.drain()
}

fn input_state_program(code: u32) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(0),
            source: Some(InputSource::Keyboard),
            device: None,
            target: InputTargetRef::Global,
            signal: InputSignal::KeyPressed { code },
        },
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_0_3_is_current_and_0_2_render_programs_still_decode() {
    assert_eq!(NAIR_FORMAT_MAJOR, 0);
    assert_eq!(NAIR_FORMAT_MINOR, 3);

    let program = NairProgram::from_instructions(vec![
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Quad,
            space: RenderSpace::Screen,
        },
        Instruction::Halt,
    ]);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 2;
    bytes[7] = 0;

    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, program);
    let reencoded = decoded.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([reencoded[6], reencoded[7]]), 3);
}

#[test]
fn input_opcodes_are_rejected_under_declared_nair_0_2() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::Halt,
    ]);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 2;
    bytes[7] = 0;

    assert_eq!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidOpcode(0x40))
    );
}

#[test]
fn input_bridge_slots_are_single_assignment() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::DuplicateInputBridgeSlot(InputBridgeSlot(0)))
    );
}

#[test]
fn input_bridge_must_exist_before_binding() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(9),
            atom: AtomSlot(0),
            source: None,
            device: None,
            target: InputTargetRef::Any,
            signal: InputSignal::PointerX,
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownInputBridgeSlot(InputBridgeSlot(9)))
    );
}

#[test]
fn atom_must_exist_before_input_binding() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(7),
            source: None,
            device: None,
            target: InputTargetRef::Any,
            signal: InputSignal::PointerY,
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownAtomSlot(AtomSlot(7)))
    );
}

#[test]
fn input_bridge_must_exist_before_apply() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(3),
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownInputBridgeSlot(InputBridgeSlot(3)))
    );
}

#[test]
fn render_target_slot_must_exist_before_input_binding() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(0),
            source: Some(InputSource::Mouse),
            device: None,
            target: InputTargetRef::RenderNode(RenderNodeSlot(8)),
            signal: InputSignal::PointerButtonPressed { button: 0 },
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownRenderNodeSlot(RenderNodeSlot(8)))
    );
}

#[test]
fn interaction_program_canonical_round_trip_is_byte_stable() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Float(0.0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Mesh,
            space: RenderSpace::World,
        },
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(0),
            source: Some(InputSource::XrController),
            device: Some(InputDeviceId(77)),
            target: InputTargetRef::RenderNode(RenderNodeSlot(0)),
            signal: InputSignal::PoseZ,
        },
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
        Instruction::Halt,
    ]);

    let bytes = program.canonical_bytes().unwrap();
    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, program);
    assert_eq!(decoded.canonical_bytes().unwrap(), bytes);
}

#[test]
fn state_only_executor_rejects_native_input_before_execution() {
    let program = input_state_program(42);
    let mut kernel = AtomicKernel::new();

    assert_eq!(
        execute_nair(&mut kernel, &program),
        Err(NairError::InputContextRequired)
    );
}

#[test]
fn render_only_executor_rejects_native_input_before_execution() {
    let program = input_state_program(42);
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();

    assert_eq!(
        execute_nair_with_render(&mut kernel, &mut render, &program),
        Err(NairError::InputContextRequired)
    );
}

#[test]
fn native_input_applies_keyboard_state_to_nam_atom() {
    let program = input_state_program(42);
    let batch = keyboard_batch(42, true, InputDeviceId(1));
    let mut kernel = AtomicKernel::new();

    let report = execute_nair_with_input(&mut kernel, &batch, &program).unwrap();
    let atom = report.execution.atom_bindings[&AtomSlot(0)];

    assert_eq!(report.created_input_bridges, 1);
    assert_eq!(report.input_applications.len(), 1);
    assert_eq!(report.input_applications[0].changed_atoms, 1);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(true));
}

#[test]
fn native_input_source_and_device_filters_are_respected() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(0),
            source: Some(InputSource::Keyboard),
            device: Some(InputDeviceId(1)),
            target: InputTargetRef::Global,
            signal: InputSignal::KeyPressed { code: 9 },
        },
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
        Instruction::Halt,
    ]);
    let batch = keyboard_batch(9, true, InputDeviceId(99));
    let mut kernel = AtomicKernel::new();

    let report = execute_nair_with_input(&mut kernel, &batch, &program).unwrap();
    let atom = report.execution.atom_bindings[&AtomSlot(0)];

    assert_eq!(report.input_applications[0].matched_bindings, 0);
    assert_eq!(report.input_applications[0].changed_atoms, 0);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(false));
}

#[test]
fn unmatched_native_input_creates_zero_nam_work() {
    let program = input_state_program(42);
    let batch = keyboard_batch(7, true, InputDeviceId(1));
    let mut kernel = AtomicKernel::new();

    let report = execute_nair_with_input(&mut kernel, &batch, &program).unwrap();

    assert_eq!(report.input_applications[0].matched_bindings, 0);
    assert_eq!(report.input_applications[0].scheduled_atoms, 0);
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn multiple_input_transitions_collapse_to_final_atomic_snapshot() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(0),
            source: Some(InputSource::Keyboard),
            device: None,
            target: InputTargetRef::Global,
            signal: InputSignal::KeyPressed { code: 7 },
        },
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
        Instruction::Halt,
    ]);

    let mut input = AtomicInputCore::new();
    for pressed in [true, false] {
        input
            .submit(
                InputSource::Keyboard,
                InputDeviceId(1),
                None,
                InputPayload::Key {
                    code: 7,
                    pressed,
                    repeat: false,
                },
            )
            .unwrap();
    }
    let batch = input.drain();
    let mut kernel = AtomicKernel::new();

    let report = execute_nair_with_input(&mut kernel, &batch, &program).unwrap();
    let atom = report.execution.atom_bindings[&AtomSlot(0)];
    let apply = &report.input_applications[0];

    assert_eq!(apply.matched_bindings, 2);
    assert_eq!(apply.staged_writes, 1);
    assert_eq!(apply.changed_atoms, 1);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(false));
}

#[test]
fn native_input_bridge_respects_domain_ownership() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateDomain {
            dst: DomainSlot(0),
            name: "isolated-input".to_string(),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Slot(DomainSlot(0)),
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(0),
            source: None,
            device: None,
            target: InputTargetRef::Any,
            signal: InputSignal::ButtonPressed { button: 1 },
        },
        Instruction::Halt,
    ]);
    let batch = nordoi_kernel::InputBatch::default();
    let mut kernel = AtomicKernel::new();

    assert!(matches!(
        execute_nair_with_input(&mut kernel, &batch, &program),
        Err(NairError::Input(_))
    ));
}

#[test]
fn native_interaction_closes_input_nam_render_loop() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
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
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(0),
            source: Some(InputSource::Keyboard),
            device: None,
            target: InputTargetRef::Global,
            signal: InputSignal::KeyPressed { code: 13 },
        },
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
        Instruction::RenderFlush,
        Instruction::Halt,
    ]);
    let batch = keyboard_batch(13, true, InputDeviceId(1));
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();

    let report =
        execute_nair_with_render_and_input(&mut kernel, &mut render, &batch, &program).unwrap();
    let atom = report.execution.atom_bindings[&AtomSlot(0)];

    assert_eq!(report.frames.len(), 2);
    assert_eq!(report.input_applications.len(), 1);
    assert_eq!(report.input_applications[0].changed_atoms, 1);
    assert_eq!(report.frames[1].scheduled_atoms.len(), 1);
    assert_eq!(report.frames[1].batch.len(), 1);
    assert!(report.frames[1].batch.updates[0]
        .dirty
        .contains(DirtyMask::CONTENT));
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(true));
}
