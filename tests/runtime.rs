use nordoi_kernel::{
    run_closed, AtomSlot, AtomicInputCore, AtomicRuntime, DirtyMask, DomainRef, DomainSlot,
    InputBatch, InputBridgeSlot, InputDeviceId, InputError, InputEvent, InputPayload,
    InputSequence, InputSignal, InputSource, InputTarget, InputTargetRef, Instruction, NairError,
    NairProgram, PointerId, RegisterId, RenderNodeSlot, RenderPrimitive, RenderSpace, RuntimeError,
    Value,
};

fn keyboard_batch(code: u32, pressed: bool) -> InputBatch {
    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Keyboard,
            InputDeviceId(1),
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

fn interactive_program(code: u32) -> NairProgram {
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
            signal: InputSignal::KeyPressed { code },
        },
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
        Instruction::RenderFlush,
        Instruction::Halt,
    ])
}

fn manual_scroll(sequence: u64, x: f32, y: f32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(sequence),
            source: InputSource::Mouse,
            device: InputDeviceId(2),
            target: InputTarget::Global,
            payload: InputPayload::Scroll { delta: [x, y] },
        }],
    }
}

#[test]
fn closed_runtime_executes_input_nam_render_loop() {
    let report = run_closed(&interactive_program(13), &keyboard_batch(13, true)).unwrap();
    let snapshot = &report.final_atoms[&AtomSlot(0)];

    assert_eq!(report.input_events, 1);
    assert_eq!(snapshot.value, Value::Bool(true));
    assert_eq!(snapshot.version, 1);
    assert_eq!(report.execution.frames.len(), 2);
    assert_eq!(report.execution.frames[1].scheduled_atoms.len(), 1);
    assert_eq!(report.execution.frames[1].batch.len(), 1);
    assert!(report.is_quiescent());
}

#[test]
fn repeated_closed_execution_is_deterministic() {
    let runtime = AtomicRuntime::new();
    let program = interactive_program(13);
    let batch = keyboard_batch(13, true);

    let a = runtime.execute(&program, &batch).unwrap();
    let b = runtime.execute(&program, &batch).unwrap();

    assert_eq!(a, b);
    assert_eq!(a.replay_key, b.replay_key);
}

#[test]
fn replay_key_changes_when_input_changes() {
    let runtime = AtomicRuntime::new();
    let program = interactive_program(13);

    let pressed = runtime
        .execute(&program, &keyboard_batch(13, true))
        .unwrap();
    let released = runtime
        .execute(&program, &keyboard_batch(13, false))
        .unwrap();

    assert_ne!(pressed.replay_key, released.replay_key);
}

#[test]
fn replay_key_changes_when_program_changes() {
    let runtime = AtomicRuntime::new();
    let batch = keyboard_batch(13, true);

    let a = runtime.execute(&interactive_program(13), &batch).unwrap();
    let b = runtime.execute(&interactive_program(14), &batch).unwrap();

    assert_ne!(a.replay_key, b.replay_key);
}

#[test]
fn negative_zero_input_has_one_canonical_replay_identity() {
    let runtime = AtomicRuntime::new();
    let program = NairProgram::from_instructions(vec![Instruction::Halt]);

    let negative = runtime
        .execute(&program, &manual_scroll(1, -0.0, 0.0))
        .unwrap();
    let positive = runtime
        .execute(&program, &manual_scroll(1, 0.0, 0.0))
        .unwrap();

    assert_eq!(negative.replay_key, positive.replay_key);
}

#[test]
fn non_monotonic_public_input_batch_is_rejected_before_execution() {
    let batch = InputBatch {
        events: vec![
            InputEvent {
                sequence: InputSequence(2),
                source: InputSource::Keyboard,
                device: InputDeviceId(1),
                target: InputTarget::Global,
                payload: InputPayload::Key {
                    code: 1,
                    pressed: true,
                    repeat: false,
                },
            },
            InputEvent {
                sequence: InputSequence(1),
                source: InputSource::Keyboard,
                device: InputDeviceId(1),
                target: InputTarget::Global,
                payload: InputPayload::Key {
                    code: 1,
                    pressed: false,
                    repeat: false,
                },
            },
        ],
    };

    assert_eq!(
        run_closed(
            &NairProgram::from_instructions(vec![Instruction::Halt]),
            &batch
        ),
        Err(RuntimeError::Input(InputError::NonMonotonicSequence {
            previous: InputSequence(2),
            current: InputSequence(1),
        }))
    );
}

#[test]
fn non_finite_public_input_batch_is_rejected_before_execution() {
    let batch = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Mouse,
            device: InputDeviceId(2),
            target: InputTarget::Global,
            payload: InputPayload::PointerMove {
                pointer: PointerId(1),
                position: [f32::NAN, 0.0],
                delta: [0.0, 0.0],
            },
        }],
    };

    assert_eq!(
        run_closed(
            &NairProgram::from_instructions(vec![Instruction::Halt]),
            &batch
        ),
        Err(RuntimeError::Input(InputError::NonFiniteValue))
    );
}

#[test]
fn xr_pose_is_canonicalized_at_runtime_boundary() {
    let batch = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::XrController,
            device: InputDeviceId(8),
            target: InputTarget::Global,
            payload: InputPayload::Pose {
                position: [-0.0, 1.0, 2.0],
                orientation: [0.0, 0.0, 0.0, -2.0],
            },
        }],
    };

    let canonical = batch.canonicalized().unwrap();
    match &canonical.events[0].payload {
        InputPayload::Pose {
            position,
            orientation,
        } => {
            assert!(!position[0].is_sign_negative());
            assert_eq!(*orientation, [0.0, 0.0, 0.0, 1.0]);
        }
        _ => panic!("expected canonical XR pose"),
    }
}

#[test]
fn state_only_program_finishes_quiescent_in_universal_runtime() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(7),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::Halt,
    ]);

    let report = run_closed(&program, &InputBatch::default()).unwrap();

    assert_eq!(report.final_atoms[&AtomSlot(0)].value, Value::Int(7));
    assert_eq!(report.final_atoms[&AtomSlot(0)].version, 0);
    assert!(report.is_quiescent());
}

#[test]
fn empty_input_batch_has_deterministic_identity() {
    let program = NairProgram::from_instructions(vec![Instruction::Halt]);
    let a = run_closed(&program, &InputBatch::default()).unwrap();
    let b = run_closed(&program, &InputBatch::default()).unwrap();

    assert_eq!(a.replay_key, b.replay_key);
    assert_eq!(a.input_events, 0);
}

#[test]
fn final_atom_snapshot_uses_semantic_slot_order() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(20),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(10),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(2),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(1),
            owner: DomainRef::Root,
            value: RegisterId(1),
        },
        Instruction::Halt,
    ]);

    let report = run_closed(&program, &InputBatch::default()).unwrap();
    let slots: Vec<_> = report.final_atoms.keys().copied().collect();

    assert_eq!(slots, vec![AtomSlot(1), AtomSlot(2)]);
    assert_eq!(report.final_atoms[&AtomSlot(1)].value, Value::Int(10));
    assert_eq!(report.final_atoms[&AtomSlot(2)].value, Value::Int(20));
}

#[test]
fn failed_closed_activation_exposes_no_partial_runtime_report() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateDomain {
            dst: DomainSlot(0),
            name: "foreign".to_string(),
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

    assert!(matches!(
        run_closed(&program, &InputBatch::default()),
        Err(RuntimeError::Nair(NairError::Input(_)))
    ));
}
