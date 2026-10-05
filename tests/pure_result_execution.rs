use nordoi_kernel::{
    compile_pure_result_nair_boundary, execute_pure_result_source_v02, run_closed_observed,
    validate_v02_execution, InputBatch, PureResultExecutionError, RegisterId, SourceId, SourceText,
    Value,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("v02-{id}.noi"), text).unwrap()
}

#[test]
fn integer_result_executes_end_to_end_and_is_recovered_from_r0() {
    let report = execute_pure_result_source_v02(&source(1, "entry main returns 42;")).unwrap();
    assert_eq!(report.result_i64(), Some(42));
    assert_eq!(
        report.runtime().register(RegisterId(0)),
        Some(&Value::Int(42))
    );
    assert_eq!(
        report
            .runtime()
            .runtime()
            .execution
            .execution
            .executed_instructions,
        2
    );
}

#[test]
fn zero_result_is_distinct_from_no_result_after_execution() {
    let zero = execute_pure_result_source_v02(&source(2, "entry main returns 0;")).unwrap();
    let none = execute_pure_result_source_v02(&source(3, "entry main;")).unwrap();
    assert_eq!(zero.result_i64(), Some(0));
    assert_eq!(none.result_i64(), None);
    assert_eq!(zero.runtime().final_registers().len(), 1);
    assert!(none.runtime().final_registers().is_empty());
}

#[test]
fn entry_without_result_executes_as_halt_only() {
    let report = execute_pure_result_source_v02(&source(4, "entry main;")).unwrap();
    assert_eq!(report.result(), None);
    assert_eq!(
        report
            .runtime()
            .runtime()
            .execution
            .execution
            .executed_instructions,
        1
    );
}

#[test]
fn empty_body_executes_without_result() {
    let report = execute_pure_result_source_v02(&source(5, "module demo; // empty\n")).unwrap();
    assert_eq!(report.result(), None);
    assert!(report.runtime().final_registers().is_empty());
}

#[test]
fn int_max_round_trips_through_runtime_registers() {
    let report =
        execute_pure_result_source_v02(&source(6, "entry main returns 9223372036854775807;"))
            .unwrap();
    assert_eq!(report.result_i64(), Some(i64::MAX));
}

#[test]
fn execution_has_zero_external_state_and_authority() {
    let report =
        execute_pure_result_source_v02(&source(7, "effect Network; entry main returns 42;"))
            .unwrap();
    let lowering = report.lowering();
    let runtime = report.runtime().runtime();
    let execution = &runtime.execution.execution;

    assert_eq!(lowering.semantic_work_item_count(), 0);
    assert!(lowering.plan().required_effects().is_empty());
    assert!(!lowering.requires_host_authority());
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
    assert!(report.is_quiescent());
}

#[test]
fn repeated_execution_is_deterministic() {
    let src = source(8, "module demo; entry main returns 42;");
    let first = execute_pure_result_source_v02(&src).unwrap();
    let second = execute_pure_result_source_v02(&src).unwrap();
    assert_eq!(first.runtime(), second.runtime());
    assert_eq!(
        first.canonical_v02_receipt_bytes(),
        second.canonical_v02_receipt_bytes()
    );
}

#[test]
fn comments_spacing_and_source_id_do_not_change_v02_receipt() {
    let first =
        execute_pure_result_source_v02(&source(9, "module demo; type A; entry main returns 42;"))
            .unwrap();
    let second = execute_pure_result_source_v02(&source(
        99,
        "module /*m*/ demo ; type /*t*/ A ; entry /*e*/ main /*x*/ returns /*r*/ 42 /*z*/ ;",
    ))
    .unwrap();
    assert_eq!(
        first.canonical_v02_receipt_bytes(),
        second.canonical_v02_receipt_bytes()
    );
}

#[test]
fn entry_name_changes_receipt_but_not_runtime_replay_when_value_matches() {
    let first = execute_pure_result_source_v02(&source(10, "entry alpha returns 42;")).unwrap();
    let second = execute_pure_result_source_v02(&source(11, "entry beta returns 42;")).unwrap();
    assert_eq!(
        first.runtime().runtime().replay_key,
        second.runtime().runtime().replay_key
    );
    assert_ne!(
        first.canonical_v02_receipt_bytes(),
        second.canonical_v02_receipt_bytes()
    );
}

#[test]
fn module_identity_changes_receipt_but_not_runtime_replay() {
    let first =
        execute_pure_result_source_v02(&source(12, "module a; entry main returns 42;")).unwrap();
    let second =
        execute_pure_result_source_v02(&source(13, "module b; entry main returns 42;")).unwrap();
    assert_eq!(
        first.runtime().runtime().replay_key,
        second.runtime().runtime().replay_key
    );
    assert_ne!(
        first.canonical_v02_receipt_bytes(),
        second.canonical_v02_receipt_bytes()
    );
}

#[test]
fn different_result_changes_runtime_replay_and_receipt() {
    let first = execute_pure_result_source_v02(&source(14, "entry main returns 1;")).unwrap();
    let second = execute_pure_result_source_v02(&source(15, "entry main returns 2;")).unwrap();
    assert_ne!(
        first.runtime().runtime().replay_key,
        second.runtime().runtime().replay_key
    );
    assert_ne!(
        first.canonical_v02_receipt_bytes(),
        second.canonical_v02_receipt_bytes()
    );
}

#[test]
fn receipt_has_explicit_v02_domain() {
    let report = execute_pure_result_source_v02(&source(16, "entry main returns 42;")).unwrap();
    assert!(report
        .canonical_v02_receipt_bytes()
        .starts_with(b"NORDOI-V0.2-PURE-RESULT-EXECUTION-RECEIPT\0"));
}

#[test]
fn receipt_commits_to_exact_c06_witness() {
    let report = execute_pure_result_source_v02(&source(17, "entry main returns 42;")).unwrap();
    let c06 = report.lowering().canonical_c06_bytes();
    assert!(report
        .canonical_v02_receipt_bytes()
        .windows(c06.len())
        .any(|window| window == c06.as_slice()));
}

#[test]
fn receipt_commits_to_canonical_empty_input() {
    let report = execute_pure_result_source_v02(&source(18, "entry main returns 42;")).unwrap();
    let input = InputBatch::default().canonical_bytes().unwrap();
    assert_eq!(report.canonical_input_bytes(), input.as_slice());
}

#[test]
fn direct_observed_runtime_matches_v02_runtime() {
    let src = source(19, "entry main returns 42;");
    let report = execute_pure_result_source_v02(&src).unwrap();
    let direct = run_closed_observed(report.lowering().program(), &InputBatch::default()).unwrap();
    assert_eq!(report.runtime(), &direct);
}

#[test]
fn unsupported_or_noncanonical_result_fails_before_runtime() {
    assert!(matches!(
        execute_pure_result_source_v02(&source(20, "entry main returns 01;")),
        Err(PureResultExecutionError::Compiler(_))
    ));
}

#[test]
fn validation_rejects_forged_result_value() {
    let lowering =
        compile_pure_result_nair_boundary(&source(21, "entry main returns 42;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime
        .final_registers
        .insert(RegisterId(0), Value::Int(41));
    assert!(matches!(
        validate_v02_execution(&lowering, &runtime),
        Err(PureResultExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_extra_register_observation() {
    let lowering =
        compile_pure_result_nair_boundary(&source(22, "entry main returns 42;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime
        .final_registers
        .insert(RegisterId(1), Value::Int(99));
    assert!(matches!(
        validate_v02_execution(&lowering, &runtime),
        Err(PureResultExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_runtime_claiming_input_events() {
    let lowering =
        compile_pure_result_nair_boundary(&source(23, "entry main returns 42;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime.runtime.input_events = 1;
    assert!(matches!(
        validate_v02_execution(&lowering, &runtime),
        Err(PureResultExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_runtime_claiming_residual_work() {
    let lowering =
        compile_pure_result_nair_boundary(&source(24, "entry main returns 42;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime.runtime.execution.execution.scheduled_work = 1;
    assert!(matches!(
        validate_v02_execution(&lowering, &runtime),
        Err(PureResultExecutionError::InvariantViolation { .. })
    ));
}
