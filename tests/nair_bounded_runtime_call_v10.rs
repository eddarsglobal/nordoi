use nordoi_kernel::{
    runtime::run_closed_call_observed, CallExpr, InputBatch, InputDeviceId, InputEvent,
    InputPayload, InputSequence, InputSource, InputTarget, Instruction, NairError, NairProgram,
    RegisterId, RuntimeError, Value, MAX_NAIR_CALL_ARGS, NAIR_LATEST_FORMAT_MINOR,
    NAIR_RUNTIME_CALL_MINOR,
};

fn key_batch(code: u32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(9),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        }],
    }
}

fn add_bias_body() -> CallExpr {
    CallExpr::IntAddChecked {
        lhs: Box::new(CallExpr::Parameter(0)),
        rhs: Box::new(CallExpr::Value(Value::Int(100))),
    }
}

fn call_program() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 0,
            args: vec![RegisterId(0)],
            body: add_bias_body(),
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_012_runtime_call_minor_remains_supported() {
    assert_eq!(NAIR_RUNTIME_CALL_MINOR, 12);
    assert_eq!(NAIR_LATEST_FORMAT_MINOR, 14);
}

#[test]
fn bounded_runtime_call_requires_012_header_and_round_trips() {
    let program = call_program();
    assert_eq!(program.required_format_minor(), 12);
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 12);
    assert_eq!(NairProgram::from_canonical_bytes(&bytes).unwrap(), program);
}

#[test]
fn bounded_runtime_call_executes_one_frame() {
    let report = run_closed_call_observed(&call_program(), &key_batch(41)).unwrap();
    assert_eq!(report.register(RegisterId(1)), Some(&Value::Int(141)));
    assert_eq!(report.call_work.runtime_calls, 1);
    assert_eq!(report.call_work.call_body_instructions, 3);
    assert_eq!(report.call_work.max_call_depth, 1);
    assert_eq!(report.runtime.execution.execution.runtime_branches, 0);
}

#[test]
fn bounded_runtime_call_can_return_bool() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 7,
            args: vec![RegisterId(0)],
            body: CallExpr::IntGt {
                lhs: Box::new(CallExpr::Parameter(0)),
                rhs: Box::new(CallExpr::Value(Value::Int(40))),
            },
        },
        Instruction::Halt,
    ]);
    let report = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(report.register(RegisterId(1)), Some(&Value::Bool(true)));
    assert_eq!(report.call_work.runtime_calls, 1);
}

#[test]
fn call_parameter_must_exist() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 0,
            args: vec![RegisterId(0)],
            body: CallExpr::Parameter(1),
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        program.validate(),
        Err(NairError::CallParameterOutOfRange(1))
    );
}

#[test]
fn call_argument_count_is_bounded() {
    let args = (0..=MAX_NAIR_CALL_ARGS)
        .map(|index| RegisterId(index as u32))
        .collect::<Vec<_>>();
    let mut instructions = Vec::new();
    for index in 0..=MAX_NAIR_CALL_ARGS {
        instructions.push(Instruction::Const {
            dst: RegisterId(index as u32),
            value: Value::Int(index as i64),
        });
    }
    instructions.push(Instruction::CallEval {
        dst: RegisterId(100),
        function_id: 0,
        args,
        body: CallExpr::Value(Value::Int(1)),
    });
    instructions.push(Instruction::Halt);
    let program = NairProgram::from_instructions(instructions);
    assert_eq!(
        program.validate(),
        Err(NairError::CallArgumentCountExceeded(MAX_NAIR_CALL_ARGS + 1))
    );
}

#[test]
fn checked_overflow_in_selected_call_fails_closed() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(i64::MAX),
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 0,
            args: vec![RegisterId(0)],
            body: CallExpr::IntAddChecked {
                lhs: Box::new(CallExpr::Parameter(0)),
                rhs: Box::new(CallExpr::Value(Value::Int(1))),
            },
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        run_closed_call_observed(&program, &InputBatch::default()),
        Err(RuntimeError::Nair(NairError::CallExpressionIntegerOverflow))
    );
}

#[test]
fn bounded_runtime_call_observation_is_deterministic() {
    let program = call_program();
    let a = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    let b = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(a.runtime.replay_key, b.runtime.replay_key);
    assert_eq!(a.call_work, b.call_work);
}
