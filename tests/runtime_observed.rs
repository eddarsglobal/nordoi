use nordoi_kernel::{
    run_closed, run_closed_observed, InputBatch, Instruction, NairProgram, RegisterId, Value,
};

fn result_program(value: i64) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(value),
        },
        Instruction::Halt,
    ])
}

#[test]
fn closed_runtime_observation_exposes_final_register() {
    let report = run_closed_observed(&result_program(42), &InputBatch::default()).unwrap();
    assert_eq!(report.register(RegisterId(0)), Some(&Value::Int(42)));
    assert_eq!(report.final_registers().len(), 1);
}

#[test]
fn observed_runtime_keeps_legacy_runtime_report_identical() {
    let program = result_program(42);
    let input = InputBatch::default();
    let legacy = run_closed(&program, &input).unwrap();
    let observed = run_closed_observed(&program, &input).unwrap();
    assert_eq!(&legacy, observed.runtime());
}

#[test]
fn observed_runtime_is_quiescent_and_state_free_for_const_halt() {
    let report = run_closed_observed(&result_program(42), &InputBatch::default()).unwrap();
    let runtime = report.runtime();
    assert!(report.is_quiescent());
    assert_eq!(runtime.input_events, 0);
    assert!(runtime.final_atoms.is_empty());
    assert_eq!(runtime.execution.execution.created_domains, 0);
    assert_eq!(runtime.execution.execution.created_atoms, 0);
    assert!(runtime.execution.frames.is_empty());
    assert_eq!(runtime.execution.created_input_bridges, 0);
}

#[test]
fn observed_runtime_replay_is_deterministic() {
    let program = result_program(42);
    let first = run_closed_observed(&program, &InputBatch::default()).unwrap();
    let second = run_closed_observed(&program, &InputBatch::default()).unwrap();
    assert_eq!(first, second);
}

#[test]
fn different_const_values_change_runtime_replay_key() {
    let first = run_closed_observed(&result_program(1), &InputBatch::default()).unwrap();
    let second = run_closed_observed(&result_program(2), &InputBatch::default()).unwrap();
    assert_ne!(first.runtime().replay_key, second.runtime().replay_key);
}

#[test]
fn halt_only_observation_has_no_registers() {
    let program = NairProgram::from_instructions(vec![Instruction::Halt]);
    let report = run_closed_observed(&program, &InputBatch::default()).unwrap();
    assert!(report.final_registers().is_empty());
}
