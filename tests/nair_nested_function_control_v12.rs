use nordoi_kernel::{
    runtime::run_closed_call_observed, CallExpr, InputBatch, InputDeviceId, InputEvent,
    InputPayload, InputSequence, InputSource, InputTarget, Instruction, NairError, NairProgram,
    RegisterId, RuntimeError, Value, NAIR_ACYCLIC_CALL_GRAPH_MINOR, NAIR_LATEST_FORMAT_MINOR,
    NAIR_RUNTIME_CALL_MINOR, NAIR_STRUCTURED_CALL_CONTROL_MINOR,
};

fn key_batch(code: u32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(12),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        }],
    }
}

fn selective_body() -> CallExpr {
    CallExpr::IfElse {
        condition: Box::new(CallExpr::IntGt {
            lhs: Box::new(CallExpr::Parameter(0)),
            rhs: Box::new(CallExpr::Value(Value::Int(40))),
        }),
        then_expr: Box::new(CallExpr::IntAddChecked {
            lhs: Box::new(CallExpr::Parameter(0)),
            rhs: Box::new(CallExpr::Value(Value::Int(100))),
        }),
        else_expr: Box::new(CallExpr::IntAddChecked {
            lhs: Box::new(CallExpr::Parameter(0)),
            rhs: Box::new(CallExpr::Value(Value::Int(200))),
        }),
    }
}

fn selective_program() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 0,
            args: vec![RegisterId(0)],
            body: selective_body(),
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_014_is_latest_and_prior_call_minors_remain_stable() {
    assert_eq!(NAIR_RUNTIME_CALL_MINOR, 12);
    assert_eq!(NAIR_ACYCLIC_CALL_GRAPH_MINOR, 13);
    assert_eq!(NAIR_STRUCTURED_CALL_CONTROL_MINOR, 14);
    assert_eq!(NAIR_LATEST_FORMAT_MINOR, 14);
}

#[test]
fn structured_call_control_requires_014_and_round_trips() {
    let program = selective_program();
    assert_eq!(program.required_format_minor(), 14);
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 14);
    assert_eq!(NairProgram::from_canonical_bytes(&bytes).unwrap(), program);
}

#[test]
fn true_path_executes_only_selected_function_arm() {
    let report = run_closed_call_observed(&selective_program(), &key_batch(41)).unwrap();
    assert_eq!(report.register(RegisterId(1)), Some(&Value::Int(141)));
    assert_eq!(report.call_work.runtime_calls, 1);
    assert_eq!(report.call_work.max_call_depth, 1);
    assert_eq!(report.runtime.execution.execution.runtime_branches, 1);
    assert_eq!(report.call_work.call_body_instructions, 7);
}

#[test]
fn false_path_executes_only_selected_function_arm() {
    let report = run_closed_call_observed(&selective_program(), &key_batch(39)).unwrap();
    assert_eq!(report.register(RegisterId(1)), Some(&Value::Int(239)));
    assert_eq!(report.call_work.runtime_calls, 1);
    assert_eq!(report.runtime.execution.execution.runtime_branches, 1);
    assert_eq!(report.call_work.call_body_instructions, 7);
}

#[test]
fn unselected_overflowing_function_arm_is_not_evaluated() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 0,
            args: vec![RegisterId(0)],
            body: CallExpr::IfElse {
                condition: Box::new(CallExpr::IntGt {
                    lhs: Box::new(CallExpr::Parameter(0)),
                    rhs: Box::new(CallExpr::Value(Value::Int(40))),
                }),
                then_expr: Box::new(CallExpr::IntAddChecked {
                    lhs: Box::new(CallExpr::Parameter(0)),
                    rhs: Box::new(CallExpr::Value(Value::Int(100))),
                }),
                else_expr: Box::new(CallExpr::IntAddChecked {
                    lhs: Box::new(CallExpr::Parameter(0)),
                    rhs: Box::new(CallExpr::Value(Value::Int(i64::MAX))),
                }),
            },
        },
        Instruction::Halt,
    ]);
    let report = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(report.register(RegisterId(1)), Some(&Value::Int(141)));
    assert_eq!(report.runtime.execution.execution.runtime_branches, 1);
}

#[test]
fn selected_overflowing_function_arm_fails_closed() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 0,
            args: vec![RegisterId(0)],
            body: CallExpr::IfElse {
                condition: Box::new(CallExpr::IntGt {
                    lhs: Box::new(CallExpr::Parameter(0)),
                    rhs: Box::new(CallExpr::Value(Value::Int(40))),
                }),
                then_expr: Box::new(CallExpr::IntAddChecked {
                    lhs: Box::new(CallExpr::Parameter(0)),
                    rhs: Box::new(CallExpr::Value(Value::Int(100))),
                }),
                else_expr: Box::new(CallExpr::IntAddChecked {
                    lhs: Box::new(CallExpr::Parameter(0)),
                    rhs: Box::new(CallExpr::Value(Value::Int(i64::MAX))),
                }),
            },
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        run_closed_call_observed(&program, &key_batch(39)),
        Err(RuntimeError::Nair(NairError::CallExpressionIntegerOverflow))
    );
}

#[test]
fn structured_call_control_arms_must_have_same_kind() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(41),
        },
        Instruction::CallEval {
            dst: RegisterId(1),
            function_id: 0,
            args: vec![RegisterId(0)],
            body: CallExpr::IfElse {
                condition: Box::new(CallExpr::IntGt {
                    lhs: Box::new(CallExpr::Parameter(0)),
                    rhs: Box::new(CallExpr::Value(Value::Int(40))),
                }),
                then_expr: Box::new(CallExpr::Value(Value::Int(1))),
                else_expr: Box::new(CallExpr::Value(Value::Bool(false))),
            },
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        program.validate(),
        Err(NairError::CallBranchArmKindMismatch)
    );
}

#[test]
fn structured_call_control_observation_is_deterministic() {
    let program = selective_program();
    let a = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    let b = run_closed_call_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(a.runtime.replay_key, b.runtime.replay_key);
    assert_eq!(a.call_work, b.call_work);
    assert_eq!(a.final_registers, b.final_registers);
}
