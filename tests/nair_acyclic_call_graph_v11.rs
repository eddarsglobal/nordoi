use nordoi_kernel::{
    runtime::run_closed_call_observed, CallExpr, InputBatch, InputDeviceId, InputEvent,
    InputPayload, InputSequence, InputSource, InputTarget, Instruction, NairError, NairProgram,
    RegisterId, Value, MAX_NAIR_CALL_GRAPH_DEPTH, NAIR_ACYCLIC_CALL_GRAPH_MINOR,
    NAIR_LATEST_FORMAT_MINOR, NAIR_RUNTIME_CALL_MINOR,
};

fn key_batch(code: u32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(11),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        }],
    }
}

fn add_100_body() -> CallExpr {
    CallExpr::IntAddChecked {
        lhs: Box::new(CallExpr::Parameter(0)),
        rhs: Box::new(CallExpr::Value(Value::Int(100))),
    }
}

fn two_level_body() -> CallExpr {
    CallExpr::IntAddChecked {
        lhs: Box::new(CallExpr::DirectCall {
            function_id: 0,
            args: vec![CallExpr::Parameter(0)],
            body: Box::new(add_100_body()),
        }),
        rhs: Box::new(CallExpr::Value(Value::Int(100))),
    }
}

fn two_level_program() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 1,
            args: vec![RegisterId(0)],
            body: two_level_body(),
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_013_is_latest_and_012_remains_runtime_call_minor() {
    assert_eq!(NAIR_RUNTIME_CALL_MINOR, 12);
    assert_eq!(NAIR_ACYCLIC_CALL_GRAPH_MINOR, 13);
    assert_eq!(NAIR_LATEST_FORMAT_MINOR, 13);
}

#[test]
fn acyclic_call_graph_requires_013_and_round_trips() {
    let program = two_level_program();
    assert_eq!(program.required_format_minor(), 13);
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 13);
    assert_eq!(NairProgram::from_canonical_bytes(&bytes).unwrap(), program);
}

#[test]
fn two_level_call_graph_executes_exactly_two_calls() {
    let report = run_closed_call_observed(&two_level_program(), &key_batch(41)).unwrap();
    assert_eq!(report.register(RegisterId(1)), Some(&Value::Int(241)));
    assert_eq!(report.call_work.runtime_calls, 2);
    assert_eq!(report.call_work.max_call_depth, 2);
    assert_eq!(report.runtime.execution.execution.runtime_branches, 0);
}

#[test]
fn simple_v10_call_stays_on_012() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(41),
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 0,
            args: vec![RegisterId(0)],
            body: add_100_body(),
        },
        Instruction::Halt,
    ]);
    assert_eq!(program.required_format_minor(), 12);
}

#[test]
fn nested_call_can_return_bool() {
    let body = CallExpr::IntGt {
        lhs: Box::new(CallExpr::DirectCall {
            function_id: 0,
            args: vec![CallExpr::Parameter(0)],
            body: Box::new(add_100_body()),
        }),
        rhs: Box::new(CallExpr::Value(Value::Int(140))),
    };
    let program = NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 1,
            args: vec![RegisterId(0)],
            body,
        },
        Instruction::Halt,
    ]);
    let report = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(report.register(RegisterId(1)), Some(&Value::Bool(true)));
    assert_eq!(report.call_work.runtime_calls, 2);
    assert_eq!(report.call_work.max_call_depth, 2);
}

fn nested_chain(depth: usize) -> CallExpr {
    if depth <= 1 {
        return add_100_body();
    }
    CallExpr::DirectCall {
        function_id: depth as u32,
        args: vec![CallExpr::Parameter(0)],
        body: Box::new(nested_chain(depth - 1)),
    }
}

#[test]
fn nair_call_depth_is_bounded() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 99,
            args: vec![RegisterId(0)],
            body: nested_chain(MAX_NAIR_CALL_GRAPH_DEPTH + 1),
        },
        Instruction::Halt,
    ]);
    assert!(matches!(
        program.validate(),
        Err(NairError::CallDepthExceeded(_))
    ));
}

#[test]
fn acyclic_call_graph_observation_is_deterministic() {
    let program = two_level_program();
    let a = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    let b = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(a.runtime.replay_key, b.runtime.replay_key);
    assert_eq!(a.call_work, b.call_work);
    assert_eq!(a.final_registers, b.final_registers);
}

#[test]
fn nested_call_body_work_is_accounted() {
    let report = run_closed_call_observed(&two_level_program(), &key_batch(41)).unwrap();
    assert!(report.call_work.call_body_instructions >= 7);
    assert_eq!(report.call_work.runtime_calls, 2);
}
