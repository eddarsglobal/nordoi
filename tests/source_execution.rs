use nordoi_kernel::{
    compile_nair_lowering_boundary, execute_source_v01, run_closed, validate_v01_execution,
    InputBatch, Instruction, SourceExecutionError, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("v01-{id}.noi"), text).unwrap()
}

#[test]
fn entry_executes_end_to_end_as_one_halt() {
    let report = execute_source_v01(&source(1, "module demo; entry main;")).unwrap();
    assert_eq!(
        report.lowering().program().instructions(),
        &[Instruction::Halt]
    );
    assert_eq!(
        report.runtime().execution.execution.executed_instructions,
        1
    );
    assert!(report.is_quiescent());
}

#[test]
fn empty_body_executes_end_to_end_as_one_halt() {
    let report = execute_source_v01(&source(2, "module demo; // empty body\n")).unwrap();
    assert_eq!(
        report.lowering().program().instructions(),
        &[Instruction::Halt]
    );
    assert_eq!(
        report.runtime().execution.execution.executed_instructions,
        1
    );
}

#[test]
fn v01_execution_has_zero_input_state_and_render_work() {
    let report = execute_source_v01(&source(3, "entry main;")).unwrap();
    let runtime = report.runtime();
    let execution = &runtime.execution.execution;

    assert_eq!(runtime.input_events, 0);
    assert!(runtime.final_atoms.is_empty());
    assert_eq!(execution.created_domains, 0);
    assert_eq!(execution.created_atoms, 0);
    assert_eq!(execution.committed_transactions, 0);
    assert_eq!(execution.rolled_back_transactions, 0);
    assert_eq!(execution.scheduled_work, 0);
    assert!(runtime.execution.frames.is_empty());
    assert_eq!(runtime.execution.created_input_bridges, 0);
    assert!(runtime.execution.input_applications.is_empty());
}

#[test]
fn execution_is_deterministic_for_equal_source() {
    let src = source(4, "module demo; entry main;");
    let first = execute_source_v01(&src).unwrap();
    let second = execute_source_v01(&src).unwrap();
    assert_eq!(first.runtime(), second.runtime());
    assert_eq!(
        first.canonical_v01_receipt_bytes(),
        second.canonical_v01_receipt_bytes()
    );
}

#[test]
fn comments_spacing_and_source_id_do_not_change_execution_receipt() {
    let first = execute_source_v01(&source(5, "module demo; type A; entry main;")).unwrap();
    let second = execute_source_v01(&source(
        6,
        "module demo /*m*/ ;\n type /*t*/ A ;\n entry /*e*/ main ; // tail\n",
    ))
    .unwrap();
    assert_eq!(
        first.canonical_v01_receipt_bytes(),
        second.canonical_v01_receipt_bytes()
    );
}

#[test]
fn different_entry_names_share_runtime_replay_but_keep_distinct_receipts() {
    let first = execute_source_v01(&source(7, "entry main;")).unwrap();
    let second = execute_source_v01(&source(8, "entry other;")).unwrap();
    assert_eq!(first.runtime().replay_key, second.runtime().replay_key);
    assert_ne!(
        first.canonical_v01_receipt_bytes(),
        second.canonical_v01_receipt_bytes()
    );
}

#[test]
fn different_modules_share_halt_runtime_replay_but_keep_distinct_receipts() {
    let first = execute_source_v01(&source(9, "module one; entry main;")).unwrap();
    let second = execute_source_v01(&source(10, "module two; entry main;")).unwrap();
    assert_eq!(first.runtime().replay_key, second.runtime().replay_key);
    assert_ne!(
        first.canonical_v01_receipt_bytes(),
        second.canonical_v01_receipt_bytes()
    );
}

#[test]
fn declared_effect_does_not_become_runtime_work_or_authority() {
    let report = execute_source_v01(&source(11, "effect Network; entry main;")).unwrap();
    assert!(report.lowering().plan().required_effects().is_empty());
    assert!(!report.lowering().requires_host_authority());
    assert_eq!(report.runtime().execution.execution.scheduled_work, 0);
}

#[test]
fn receipt_has_explicit_v01_domain() {
    let report = execute_source_v01(&source(12, "entry main;")).unwrap();
    assert!(report
        .canonical_v01_receipt_bytes()
        .starts_with(b"NORDOI-V0.1-EXECUTION-RECEIPT\0"));
}

#[test]
fn receipt_commits_to_exact_c04_witness() {
    let report = execute_source_v01(&source(13, "entry main;")).unwrap();
    let c04 = report.lowering().canonical_c04_bytes();
    assert!(report
        .canonical_v01_receipt_bytes()
        .windows(c04.len())
        .any(|window| window == c04.as_slice()));
}

#[test]
fn receipt_commits_to_canonical_empty_input() {
    let report = execute_source_v01(&source(14, "entry main;")).unwrap();
    let input = InputBatch::default().canonical_bytes().unwrap();
    assert_eq!(report.canonical_input_bytes(), input.as_slice());
    assert!(report
        .canonical_v01_receipt_bytes()
        .windows(input.len())
        .any(|window| window == input.as_slice()));
}

#[test]
fn runtime_replay_matches_direct_closed_runtime_for_same_lowering() {
    let src = source(15, "module demo; entry main;");
    let report = execute_source_v01(&src).unwrap();
    let direct = run_closed(report.lowering().program(), &InputBatch::default()).unwrap();
    assert_eq!(report.runtime(), &direct);
}

#[test]
fn unsupported_body_fails_before_runtime_execution() {
    assert!(matches!(
        execute_source_v01(&source(16, "future_body")),
        Err(SourceExecutionError::Compiler(_))
    ));
}

#[test]
fn validation_rejects_runtime_report_claiming_input_events() {
    let lowering = compile_nair_lowering_boundary(&source(17, "entry main;")).unwrap();
    let mut runtime = run_closed(lowering.program(), &InputBatch::default()).unwrap();
    runtime.input_events = 1;
    assert!(matches!(
        validate_v01_execution(&lowering, &runtime),
        Err(SourceExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_runtime_report_claiming_residual_work() {
    let lowering = compile_nair_lowering_boundary(&source(18, "entry main;")).unwrap();
    let mut runtime = run_closed(lowering.program(), &InputBatch::default()).unwrap();
    runtime.execution.execution.scheduled_work = 1;
    assert!(matches!(
        validate_v01_execution(&lowering, &runtime),
        Err(SourceExecutionError::InvariantViolation { .. })
    ));
}
