use nordoi_kernel::{
    run_closed_selective_observed, BranchExpr, InputBatch, InputDeviceId, InputEvent, InputPayload,
    InputSequence, InputSource, InputTarget, Instruction, NairError, NairProgram, RegisterId,
    RuntimeError, Value, NAIR_DYNAMIC_BRANCH_MINOR, NAIR_LATEST_FORMAT_MINOR,
    NAIR_SELECTIVE_BRANCH_MINOR,
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

fn add_input(value: i64) -> BranchExpr {
    BranchExpr::IntAddChecked {
        lhs: Box::new(BranchExpr::Register(RegisterId(0))),
        rhs: Box::new(BranchExpr::Value(Value::Int(value))),
    }
}

fn selective_program() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(40),
        },
        Instruction::IntGt {
            dst: RegisterId(2),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::BranchEval {
            dst: RegisterId(3),
            condition: RegisterId(2),
            then_expr: add_input(100),
            else_expr: add_input(200),
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_011_is_the_latest_supported_minor() {
    assert_eq!(NAIR_DYNAMIC_BRANCH_MINOR, 10);
    assert_eq!(NAIR_SELECTIVE_BRANCH_MINOR, 11);
    assert_eq!(NAIR_LATEST_FORMAT_MINOR, 11);
}

#[test]
fn selective_branch_requires_011_header_and_round_trips() {
    let program = selective_program();
    assert_eq!(program.required_format_minor(), 11);
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 11);
    assert_eq!(NairProgram::from_canonical_bytes(&bytes).unwrap(), program);
}

#[test]
fn selective_true_path_executes_only_then_expression() {
    let report = run_closed_selective_observed(&selective_program(), &key_batch(41)).unwrap();
    let execution = &report.runtime.execution.execution;
    assert_eq!(report.register(RegisterId(3)), Some(&Value::Int(141)));
    assert_eq!(execution.runtime_branches, 1);
    assert_eq!(report.branch_work.selected_branch_instructions, 3);
    assert_eq!(report.branch_work.discarded_branch_instructions, 0);
}

#[test]
fn selective_false_path_executes_only_else_expression() {
    let report = run_closed_selective_observed(&selective_program(), &key_batch(39)).unwrap();
    let execution = &report.runtime.execution.execution;
    assert_eq!(report.register(RegisterId(3)), Some(&Value::Int(239)));
    assert_eq!(execution.runtime_branches, 1);
    assert_eq!(report.branch_work.selected_branch_instructions, 3);
    assert_eq!(report.branch_work.discarded_branch_instructions, 0);
}

#[test]
fn unselected_overflowing_branch_expression_is_not_evaluated() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(40),
        },
        Instruction::IntGt {
            dst: RegisterId(2),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::BranchEval {
            dst: RegisterId(3),
            condition: RegisterId(2),
            then_expr: add_input(i64::MAX),
            else_expr: add_input(200),
        },
        Instruction::Halt,
    ]);
    let report = run_closed_selective_observed(&program, &key_batch(39)).unwrap();
    assert_eq!(report.register(RegisterId(3)), Some(&Value::Int(239)));
    assert_eq!(report.branch_work.discarded_branch_instructions, 0);
}

#[test]
fn selected_overflowing_branch_expression_fails_closed() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ReadInputKeyCode {
            dst: RegisterId(0),
            event_index: 0,
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(40),
        },
        Instruction::IntGt {
            dst: RegisterId(2),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::BranchEval {
            dst: RegisterId(3),
            condition: RegisterId(2),
            then_expr: add_input(i64::MAX),
            else_expr: add_input(200),
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        run_closed_selective_observed(&program, &key_batch(41)),
        Err(RuntimeError::Nair(
            NairError::BranchExpressionIntegerOverflow
        ))
    );
}

#[test]
fn selective_branch_arms_must_have_same_kind() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(true),
        },
        Instruction::BranchEval {
            dst: RegisterId(1),
            condition: RegisterId(0),
            then_expr: BranchExpr::Value(Value::Int(1)),
            else_expr: BranchExpr::Value(Value::Bool(false)),
        },
        Instruction::Halt,
    ]);
    assert_eq!(program.validate(), Err(NairError::BranchArmKindMismatch));
}

#[test]
fn selective_branch_observation_is_deterministic() {
    let program = selective_program();
    let a = run_closed_selective_observed(&program, &key_batch(41)).unwrap();
    let b = run_closed_selective_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(a.runtime.replay_key, b.runtime.replay_key);
    assert_eq!(
        a.branch_work.selected_branch_instructions,
        b.branch_work.selected_branch_instructions
    );
    assert_eq!(a.branch_work.discarded_branch_instructions, 0);
}
