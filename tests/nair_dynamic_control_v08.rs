use nordoi_kernel::{
    execute_nair_with_render_and_input_observed, run_closed_observed, AtomicKernel,
    AtomicRenderCore, InputBatch, InputDeviceId, InputEvent, InputPayload, InputSequence,
    InputSource, InputTarget, Instruction, NairError, NairProgram, RegisterId, Value,
    NAIR_DYNAMIC_BRANCH_MINOR, NAIR_LATEST_FORMAT_MINOR,
};

fn key_batch(code: u32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(8),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        }],
    }
}

fn branch_program() -> NairProgram {
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
        Instruction::BranchValue {
            dst: RegisterId(3),
            condition: RegisterId(2),
            then_value: Value::Int(100),
            else_value: Value::Int(200),
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_010_dynamic_branch_minor_remains_stable() {
    assert_eq!(NAIR_DYNAMIC_BRANCH_MINOR, 10);
    assert_eq!(NAIR_LATEST_FORMAT_MINOR, 14);
}

#[test]
fn dynamic_branch_requires_010_header_and_round_trips() {
    let program = branch_program();
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(program.required_format_minor(), 10);
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 10);
    assert_eq!(NairProgram::from_canonical_bytes(&bytes).unwrap(), program);
}

#[test]
fn dynamic_branch_true_selects_then_value() {
    let report = run_closed_observed(&branch_program(), &key_batch(41)).unwrap();
    assert_eq!(report.register(RegisterId(3)), Some(&Value::Int(100)));
    assert_eq!(report.runtime.execution.execution.runtime_branches, 1);
}

#[test]
fn dynamic_branch_false_selects_else_value() {
    let report = run_closed_observed(&branch_program(), &key_batch(39)).unwrap();
    assert_eq!(report.register(RegisterId(3)), Some(&Value::Int(200)));
    assert_eq!(report.runtime.execution.execution.runtime_branches, 1);
}

#[test]
fn dynamic_branch_condition_must_be_bool() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::BranchValue {
            dst: RegisterId(1),
            condition: RegisterId(0),
            then_value: Value::Int(10),
            else_value: Value::Int(20),
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        program.validate(),
        Err(NairError::BranchConditionNotBool(RegisterId(0)))
    );
}

#[test]
fn dynamic_branch_arms_must_have_same_kind() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(true),
        },
        Instruction::BranchValue {
            dst: RegisterId(1),
            condition: RegisterId(0),
            then_value: Value::Int(10),
            else_value: Value::Bool(false),
        },
        Instruction::Halt,
    ]);
    assert_eq!(program.validate(), Err(NairError::BranchArmKindMismatch));
}

#[test]
fn dynamic_branch_rejects_non_scalar_v08_arm_kind() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(true),
        },
        Instruction::BranchValue {
            dst: RegisterId(1),
            condition: RegisterId(0),
            then_value: Value::Text("yes".to_owned()),
            else_value: Value::Text("no".to_owned()),
        },
        Instruction::Halt,
    ]);
    assert_eq!(program.validate(), Err(NairError::BranchArmKindUnsupported));
}

#[test]
fn branch_observation_is_deterministic() {
    let program = branch_program();
    let a = run_closed_observed(&program, &key_batch(41)).unwrap();
    let b = run_closed_observed(&program, &key_batch(41)).unwrap();
    assert_eq!(a.runtime.replay_key, b.runtime.replay_key);
    assert_eq!(a.runtime.execution.execution.runtime_branches, 1);
    assert_eq!(b.runtime.execution.execution.runtime_branches, 1);

    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let observed = execute_nair_with_render_and_input_observed(
        &mut kernel,
        &mut render,
        &key_batch(39),
        &program,
    )
    .unwrap();
    assert_eq!(observed.execution.execution.runtime_branches, 1);
}
