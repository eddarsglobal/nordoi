use nordoi_kernel::{
    execute_nair, execute_nair_with_render_and_input_observed, run_closed_observed, AtomicKernel,
    AtomicRenderCore, InputBatch, InputDeviceId, InputEvent, InputPayload, InputSequence,
    InputSource, InputTarget, Instruction, NairError, NairProgram, RegisterId, Value,
    NAIR_INPUT_REGISTER_MINOR, NAIR_LATEST_FORMAT_MINOR,
};

fn key_batch(code: u32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(7),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        }],
    }
}

fn dynamic_add_program() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(2),
        },
        Instruction::IntAddChecked {
            dst: RegisterId(2),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_09_input_register_minor_remains_supported() {
    assert_eq!(NAIR_INPUT_REGISTER_MINOR, 9);
    assert_eq!(NAIR_LATEST_FORMAT_MINOR, 10);
}

#[test]
fn dynamic_input_register_requires_09_header_and_round_trips() {
    let program = dynamic_add_program();
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(program.required_format_minor(), 9);
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 9);
    assert_eq!(NairProgram::from_canonical_bytes(&bytes).unwrap(), program);
}

#[test]
fn dynamic_key_code_add_executes_to_42() {
    let report = run_closed_observed(&dynamic_add_program(), &key_batch(40)).unwrap();
    assert_eq!(report.register(RegisterId(2)), Some(&Value::Int(42)));
    assert_eq!(report.runtime.input_events, 1);
}

#[test]
fn dynamic_register_is_valid_without_compile_time_value() {
    assert!(dynamic_add_program().validate().is_ok());
}

#[test]
fn state_only_executor_rejects_dynamic_input_before_execution() {
    let mut kernel = AtomicKernel::new();
    assert_eq!(
        execute_nair(&mut kernel, &dynamic_add_program()),
        Err(NairError::InputContextRequired)
    );
}

#[test]
fn missing_input_event_fails_closed() {
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let error = execute_nair_with_render_and_input_observed(
        &mut kernel,
        &mut render,
        &InputBatch::default(),
        &dynamic_add_program(),
    )
    .unwrap_err();
    assert_eq!(error, NairError::InputEventMissing(0));
}

#[test]
fn non_keyboard_event_fails_closed() {
    let batch = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Gamepad,
            device: InputDeviceId(7),
            target: InputTarget::Global,
            payload: InputPayload::Button {
                button: 1,
                pressed: true,
            },
        }],
    };
    let error = run_closed_observed(&dynamic_add_program(), &batch).unwrap_err();
    assert!(format!("{error}").contains("not a keyboard key event"));
}

#[test]
fn replay_identity_binds_dynamic_input() {
    let program = dynamic_add_program();
    let a = run_closed_observed(&program, &key_batch(40)).unwrap();
    let b = run_closed_observed(&program, &key_batch(40)).unwrap();
    let c = run_closed_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(a.runtime.replay_key, b.runtime.replay_key);
    assert_ne!(a.runtime.replay_key, c.runtime.replay_key);
}
