use nordoi_kernel::{
    execute_nair_with_render_and_input, execute_nair_with_render_and_input_observed, AtomicKernel,
    AtomicRenderCore, InputBatch, Instruction, NairProgram, RegisterId, Value,
};

#[test]
fn observed_execution_captures_final_const_register() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(42),
        },
        Instruction::Halt,
    ]);
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render_and_input_observed(
        &mut kernel,
        &mut render,
        &InputBatch::default(),
        &program,
    )
    .unwrap();

    assert_eq!(report.register(RegisterId(0)), Some(&Value::Int(42)));
    assert_eq!(report.final_registers.len(), 1);
    assert_eq!(report.execution.execution.executed_instructions, 2);
}

#[test]
fn observed_halt_only_execution_has_no_registers() {
    let program = NairProgram::from_instructions(vec![Instruction::Halt]);
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render_and_input_observed(
        &mut kernel,
        &mut render,
        &InputBatch::default(),
        &program,
    )
    .unwrap();

    assert!(report.final_registers.is_empty());
    assert_eq!(report.execution.execution.executed_instructions, 1);
}

#[test]
fn observation_does_not_create_kernel_or_render_state() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(7),
        },
        Instruction::Halt,
    ]);
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let report = execute_nair_with_render_and_input_observed(
        &mut kernel,
        &mut render,
        &InputBatch::default(),
        &program,
    )
    .unwrap();

    assert_eq!(report.execution.execution.created_domains, 0);
    assert_eq!(report.execution.execution.created_atoms, 0);
    assert!(report.execution.render_bindings.is_empty());
    assert!(report.execution.frames.is_empty());
}

#[test]
fn legacy_interactive_report_remains_register_free_and_equivalent() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(9),
        },
        Instruction::Halt,
    ]);
    let input = InputBatch::default();
    let mut legacy_kernel = AtomicKernel::new();
    let mut legacy_render = AtomicRenderCore::new();
    let legacy = execute_nair_with_render_and_input(
        &mut legacy_kernel,
        &mut legacy_render,
        &input,
        &program,
    )
    .unwrap();

    let mut observed_kernel = AtomicKernel::new();
    let mut observed_render = AtomicRenderCore::new();
    let observed = execute_nair_with_render_and_input_observed(
        &mut observed_kernel,
        &mut observed_render,
        &input,
        &program,
    )
    .unwrap();

    assert_eq!(legacy, observed.execution);
}

#[test]
fn register_observation_is_deterministic() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(i64::MAX),
        },
        Instruction::Halt,
    ]);
    let input = InputBatch::default();

    let run = || {
        let mut kernel = AtomicKernel::new();
        let mut render = AtomicRenderCore::new();
        execute_nair_with_render_and_input_observed(&mut kernel, &mut render, &input, &program)
            .unwrap()
    };

    assert_eq!(run(), run());
}
