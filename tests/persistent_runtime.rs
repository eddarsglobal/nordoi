use nordoi_kernel::{
    AtomSlot, DirtyMask, DomainRef, InputBatch, InputBridgeSlot, InputDeviceId, InputError,
    InputEvent, InputPayload, InputSequence, InputSignal, InputSource, InputTarget, InputTargetRef,
    Instruction, NairProgram, PersistentAtomicRuntime, RegisterId, RenderNodeId, RenderNodeSlot,
    RenderPrimitive, RenderSpace, RuntimeError, Value,
};

fn key_batch(sequence: u64, code: u32, pressed: bool) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(sequence),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed,
                repeat: false,
            },
        }],
    }
}

fn targeted_key_batch(sequence: u64, code: u32, pressed: bool) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(sequence),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::RenderNode(RenderNodeId(1)),
            payload: InputPayload::Key {
                code,
                pressed,
                repeat: false,
            },
        }],
    }
}

fn interactive_program(code: u32, target: InputTargetRef) -> NairProgram {
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
            target,
            signal: InputSignal::KeyPressed { code },
        },
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
        Instruction::RenderFlush,
        Instruction::Halt,
    ])
}

#[test]
fn boot_creates_one_persistent_quiescent_world() {
    let runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();

    assert_eq!(runtime.tick_index(), 0);
    assert_eq!(
        runtime.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(false)
    );
    assert!(runtime.is_quiescent());
}

#[test]
fn first_tick_mutates_existing_atom_and_emits_render_work() {
    let mut runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();
    let report = runtime.tick(&key_batch(1, 13, true)).unwrap();

    assert_eq!(report.tick, 1);
    assert_eq!(report.final_atoms[&AtomSlot(0)].value, Value::Bool(true));
    assert_eq!(report.final_atoms[&AtomSlot(0)].version, 1);
    assert_eq!(report.frame.as_ref().unwrap().scheduled_atoms.len(), 1);
    assert_eq!(report.frame.as_ref().unwrap().batch.len(), 1);
    assert!(runtime.is_quiescent());
}

#[test]
fn state_persists_across_multiple_ticks() {
    let mut runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();

    runtime.tick(&key_batch(1, 13, true)).unwrap();
    let released = runtime.tick(&key_batch(2, 13, false)).unwrap();

    assert_eq!(released.final_atoms[&AtomSlot(0)].value, Value::Bool(false));
    assert_eq!(released.final_atoms[&AtomSlot(0)].version, 2);
    assert_eq!(runtime.tick_index(), 2);
}

#[test]
fn identical_persistent_write_creates_zero_render_work() {
    let mut runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();

    runtime.tick(&key_batch(1, 13, true)).unwrap();
    let repeated = runtime.tick(&key_batch(2, 13, true)).unwrap();

    assert_eq!(repeated.final_atoms[&AtomSlot(0)].version, 1);
    assert!(repeated.frame.is_none());
}

#[test]
fn unmatched_input_creates_zero_state_and_render_work() {
    let mut runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();
    let report = runtime.tick(&key_batch(1, 99, true)).unwrap();

    assert_eq!(report.final_atoms[&AtomSlot(0)].version, 0);
    assert!(report.frame.is_none());
    assert_eq!(report.input_applications[0].matched_bindings, 0);
}

#[test]
fn sequence_must_remain_monotonic_between_ticks() {
    let mut runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();
    runtime.tick(&key_batch(5, 13, true)).unwrap();

    assert_eq!(
        runtime.tick(&key_batch(5, 13, false)),
        Err(RuntimeError::Input(InputError::NonMonotonicSequence {
            previous: InputSequence(5),
            current: InputSequence(5),
        }))
    );
    assert_eq!(
        runtime.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(true)
    );
    assert_eq!(runtime.tick_index(), 1);
}

#[test]
fn failed_tick_does_not_advance_session_identity() {
    let mut runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();
    runtime.tick(&key_batch(1, 13, true)).unwrap();
    let before = runtime.replay_key();

    let invalid = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(2),
            source: InputSource::Gamepad,
            device: InputDeviceId(9),
            target: InputTarget::Global,
            payload: InputPayload::Axis {
                axis: 0,
                value: 2.0,
            },
        }],
    };

    assert!(matches!(
        runtime.tick(&invalid),
        Err(RuntimeError::Input(_))
    ));
    assert_eq!(runtime.replay_key(), before);
    assert_eq!(runtime.tick_index(), 1);
    assert_eq!(
        runtime.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(true)
    );
}

#[test]
fn equal_tick_sequences_produce_equal_session_replay_keys() {
    let program = interactive_program(13, InputTargetRef::Global);
    let mut a = PersistentAtomicRuntime::boot(&program).unwrap();
    let mut b = PersistentAtomicRuntime::boot(&program).unwrap();

    a.tick(&key_batch(1, 13, true)).unwrap();
    a.tick(&key_batch(2, 13, false)).unwrap();
    b.tick(&key_batch(1, 13, true)).unwrap();
    b.tick(&key_batch(2, 13, false)).unwrap();

    assert_eq!(a.replay_key(), b.replay_key());
    assert_eq!(a.snapshot().unwrap(), b.snapshot().unwrap());
}

#[test]
fn session_replay_key_changes_when_tick_trace_changes() {
    let program = interactive_program(13, InputTargetRef::Global);
    let mut a = PersistentAtomicRuntime::boot(&program).unwrap();
    let mut b = PersistentAtomicRuntime::boot(&program).unwrap();

    a.tick(&key_batch(1, 13, true)).unwrap();
    b.tick(&key_batch(1, 13, false)).unwrap();

    assert_ne!(a.replay_key(), b.replay_key());
}

#[test]
fn empty_ticks_are_valid_quiescent_session_steps() {
    let mut runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();
    let before = runtime.replay_key();
    let report = runtime.tick(&InputBatch::default()).unwrap();

    assert_eq!(report.tick, 1);
    assert_eq!(report.input_events, 0);
    assert!(report.frame.is_none());
    assert_ne!(runtime.replay_key(), before);
    assert!(runtime.is_quiescent());
}

#[test]
fn boot_report_preserves_original_initial_render_frames() {
    let runtime =
        PersistentAtomicRuntime::boot(&interactive_program(13, InputTargetRef::Global)).unwrap();

    assert_eq!(runtime.boot_report().frames.len(), 2);
    assert_eq!(runtime.boot_report().frames[0].batch.len(), 1);
}

#[test]
fn render_node_target_bindings_survive_bootstrap() {
    let mut runtime = PersistentAtomicRuntime::boot(&interactive_program(
        13,
        InputTargetRef::RenderNode(RenderNodeSlot(0)),
    ))
    .unwrap();

    let report = runtime.tick(&targeted_key_batch(1, 13, true)).unwrap();
    assert_eq!(report.final_atoms[&AtomSlot(0)].value, Value::Bool(true));
    assert!(report.frame.is_some());
}

#[test]
fn repeated_apply_boundaries_are_preserved_per_tick() {
    let mut instructions = interactive_program(13, InputTargetRef::Global)
        .instructions()
        .to_vec();
    let halt = instructions.pop().unwrap();
    instructions.insert(
        instructions.len() - 1,
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
    );
    instructions.push(halt);

    let mut runtime =
        PersistentAtomicRuntime::boot(&NairProgram::from_instructions(instructions)).unwrap();
    let report = runtime.tick(&key_batch(1, 13, true)).unwrap();

    assert_eq!(report.input_applications.len(), 2);
    assert_eq!(report.final_atoms[&AtomSlot(0)].version, 1);
}

#[test]
fn state_only_program_remains_stable_across_persistent_ticks() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(42),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::Halt,
    ]);
    let mut runtime = PersistentAtomicRuntime::boot(&program).unwrap();

    let first = runtime.tick(&InputBatch::default()).unwrap();
    let second = runtime.tick(&InputBatch::default()).unwrap();

    assert_eq!(first.final_atoms[&AtomSlot(0)].value, Value::Int(42));
    assert_eq!(second.final_atoms[&AtomSlot(0)].version, 0);
    assert!(first.frame.is_none());
    assert!(second.frame.is_none());
}
