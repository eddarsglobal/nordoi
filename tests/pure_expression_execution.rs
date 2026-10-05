use nordoi_kernel::{
    compile_pure_expression_nair_boundary, execute_pure_expression_source_v03, run_closed_observed,
    validate_v03_execution, InputBatch, PureExpressionExecutionError, RegisterId, SourceId,
    SourceText, Value,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("v03-{id}.noi"), text).unwrap()
}

#[test]
fn addition_executes_end_to_end_and_validates_every_ssa_register() {
    let report =
        execute_pure_expression_source_v03(&source(1, "entry main returns 20 + 22;")).unwrap();
    assert_eq!(report.result_i64(), Some(42));
    assert_eq!(
        report.runtime().register(RegisterId(0)),
        Some(&Value::Int(20))
    );
    assert_eq!(
        report.runtime().register(RegisterId(1)),
        Some(&Value::Int(22))
    );
    assert_eq!(
        report.runtime().register(RegisterId(2)),
        Some(&Value::Int(42))
    );
    assert_eq!(report.runtime().final_registers().len(), 3);
    assert_eq!(
        report
            .runtime()
            .runtime()
            .execution
            .execution
            .executed_instructions,
        4
    );
}

#[test]
fn grouped_expression_executes_in_certified_postfix_order() {
    let report =
        execute_pure_expression_source_v03(&source(2, "entry main returns 1 + (2 + 3);")).unwrap();
    assert_eq!(report.result_i64(), Some(6));
    assert_eq!(
        report.runtime().register(RegisterId(0)),
        Some(&Value::Int(1))
    );
    assert_eq!(
        report.runtime().register(RegisterId(1)),
        Some(&Value::Int(2))
    );
    assert_eq!(
        report.runtime().register(RegisterId(2)),
        Some(&Value::Int(3))
    );
    assert_eq!(
        report.runtime().register(RegisterId(3)),
        Some(&Value::Int(5))
    );
    assert_eq!(
        report.runtime().register(RegisterId(4)),
        Some(&Value::Int(6))
    );
    assert_eq!(
        report
            .runtime()
            .runtime()
            .execution
            .execution
            .executed_instructions,
        6
    );
}

#[test]
fn different_grouping_same_value_has_different_runtime_replay_and_receipt() {
    let left =
        execute_pure_expression_source_v03(&source(3, "entry main returns (1 + 2) + 3;")).unwrap();
    let right =
        execute_pure_expression_source_v03(&source(4, "entry main returns 1 + (2 + 3);")).unwrap();
    assert_eq!(left.result_i64(), Some(6));
    assert_eq!(right.result_i64(), Some(6));
    assert_ne!(
        left.runtime().runtime().replay_key,
        right.runtime().runtime().replay_key
    );
    assert_ne!(
        left.canonical_v03_receipt_bytes(),
        right.canonical_v03_receipt_bytes()
    );
}

#[test]
fn literal_only_execution_remains_nair_06() {
    let report = execute_pure_expression_source_v03(&source(5, "entry main returns 42;")).unwrap();
    assert_eq!(report.lowering().nair_format_minor(), 6);
    assert_eq!(report.result_i64(), Some(42));
    assert_eq!(report.runtime().final_registers().len(), 1);
}

#[test]
fn entry_without_expression_executes_halt_only() {
    let report = execute_pure_expression_source_v03(&source(6, "entry main;")).unwrap();
    assert_eq!(report.result(), None);
    assert_eq!(report.lowering().nair_format_minor(), 6);
    assert!(report.runtime().final_registers().is_empty());
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
fn empty_body_executes_without_expression() {
    let report = execute_pure_expression_source_v03(&source(7, "module demo; // empty\n")).unwrap();
    assert_eq!(report.result(), None);
    assert!(report.runtime().final_registers().is_empty());
}

#[test]
fn zero_expression_is_a_real_result() {
    let report =
        execute_pure_expression_source_v03(&source(8, "entry main returns 0 + 0;")).unwrap();
    assert_eq!(report.result_i64(), Some(0));
    assert_eq!(
        report.runtime().register(RegisterId(2)),
        Some(&Value::Int(0))
    );
}

#[test]
fn largest_safe_addition_executes_exactly() {
    let report = execute_pure_expression_source_v03(&source(
        9,
        "entry main returns 9223372036854775806 + 1;",
    ))
    .unwrap();
    assert_eq!(report.result_i64(), Some(i64::MAX));
}

#[test]
fn overflow_fails_before_runtime() {
    assert!(matches!(
        execute_pure_expression_source_v03(&source(
            10,
            "entry main returns 9223372036854775807 + 1;",
        )),
        Err(PureExpressionExecutionError::Compiler(_))
    ));
}

#[test]
fn execution_has_zero_external_state_effects_and_authority() {
    let report = execute_pure_expression_source_v03(&source(
        11,
        "effect Network; entry main returns 20 + 22;",
    ))
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
    let src = source(12, "module demo; entry main returns 20 + 22;");
    let first = execute_pure_expression_source_v03(&src).unwrap();
    let second = execute_pure_expression_source_v03(&src).unwrap();
    assert_eq!(first.runtime(), second.runtime());
    assert_eq!(
        first.canonical_v03_receipt_bytes(),
        second.canonical_v03_receipt_bytes()
    );
}

#[test]
fn comments_spacing_and_source_id_do_not_change_v03_receipt() {
    let first = execute_pure_expression_source_v03(&source(
        13,
        "module demo; type A; entry main returns 20 + 22;",
    ))
    .unwrap();
    let second = execute_pure_expression_source_v03(&source(
        99,
        "module /*m*/ demo ; type /*t*/ A ; entry /*e*/ main /*x*/ returns /*a*/ 20 /*b*/ + /*c*/ 22 /*z*/ ;",
    ))
    .unwrap();
    assert_eq!(
        first.canonical_v03_receipt_bytes(),
        second.canonical_v03_receipt_bytes()
    );
}

#[test]
fn entry_name_changes_receipt_but_not_runtime_replay_when_expression_matches() {
    let first =
        execute_pure_expression_source_v03(&source(14, "entry alpha returns 20 + 22;")).unwrap();
    let second =
        execute_pure_expression_source_v03(&source(15, "entry beta returns 20 + 22;")).unwrap();
    assert_eq!(
        first.runtime().runtime().replay_key,
        second.runtime().runtime().replay_key
    );
    assert_ne!(
        first.canonical_v03_receipt_bytes(),
        second.canonical_v03_receipt_bytes()
    );
}

#[test]
fn module_identity_changes_receipt_but_not_runtime_replay() {
    let first =
        execute_pure_expression_source_v03(&source(16, "module a; entry main returns 20 + 22;"))
            .unwrap();
    let second =
        execute_pure_expression_source_v03(&source(17, "module b; entry main returns 20 + 22;"))
            .unwrap();
    assert_eq!(
        first.runtime().runtime().replay_key,
        second.runtime().runtime().replay_key
    );
    assert_ne!(
        first.canonical_v03_receipt_bytes(),
        second.canonical_v03_receipt_bytes()
    );
}

#[test]
fn different_expression_changes_runtime_replay_and_receipt() {
    let first =
        execute_pure_expression_source_v03(&source(18, "entry main returns 20 + 22;")).unwrap();
    let second =
        execute_pure_expression_source_v03(&source(19, "entry main returns 20 + 23;")).unwrap();
    assert_ne!(
        first.runtime().runtime().replay_key,
        second.runtime().runtime().replay_key
    );
    assert_ne!(
        first.canonical_v03_receipt_bytes(),
        second.canonical_v03_receipt_bytes()
    );
}

#[test]
fn receipt_has_explicit_v03_domain() {
    let report =
        execute_pure_expression_source_v03(&source(20, "entry main returns 20 + 22;")).unwrap();
    assert!(report
        .canonical_v03_receipt_bytes()
        .starts_with(b"NORDOI-V0.3-PURE-EXPRESSION-EXECUTION-RECEIPT\0"));
}

#[test]
fn receipt_commits_to_exact_c08_witness() {
    let report =
        execute_pure_expression_source_v03(&source(21, "entry main returns 20 + 22;")).unwrap();
    let c08 = report.lowering().canonical_c08_bytes();
    assert!(report
        .canonical_v03_receipt_bytes()
        .windows(c08.len())
        .any(|window| window == c08.as_slice()));
}

#[test]
fn receipt_commits_to_canonical_empty_input() {
    let report =
        execute_pure_expression_source_v03(&source(22, "entry main returns 20 + 22;")).unwrap();
    let input = InputBatch::default().canonical_bytes().unwrap();
    assert_eq!(report.canonical_input_bytes(), input.as_slice());
}

#[test]
fn direct_observed_runtime_matches_v03_runtime() {
    let src = source(23, "entry main returns 20 + 22;");
    let report = execute_pure_expression_source_v03(&src).unwrap();
    let direct = run_closed_observed(report.lowering().program(), &InputBatch::default()).unwrap();
    assert_eq!(report.runtime(), &direct);
}

#[test]
fn validation_rejects_forged_final_result_register_value() {
    let lowering =
        compile_pure_expression_nair_boundary(&source(24, "entry main returns 20 + 22;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime
        .final_registers
        .insert(RegisterId(2), Value::Int(41));
    assert!(matches!(
        validate_v03_execution(&lowering, &runtime),
        Err(PureExpressionExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_forged_intermediate_register_value() {
    let lowering =
        compile_pure_expression_nair_boundary(&source(25, "entry main returns 20 + 22;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime
        .final_registers
        .insert(RegisterId(1), Value::Int(21));
    assert!(matches!(
        validate_v03_execution(&lowering, &runtime),
        Err(PureExpressionExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_extra_register_observation() {
    let lowering =
        compile_pure_expression_nair_boundary(&source(26, "entry main returns 20 + 22;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime
        .final_registers
        .insert(RegisterId(3), Value::Int(99));
    assert!(matches!(
        validate_v03_execution(&lowering, &runtime),
        Err(PureExpressionExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_runtime_claiming_input_events() {
    let lowering =
        compile_pure_expression_nair_boundary(&source(27, "entry main returns 20 + 22;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime.runtime.input_events = 1;
    assert!(matches!(
        validate_v03_execution(&lowering, &runtime),
        Err(PureExpressionExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_runtime_claiming_residual_work() {
    let lowering =
        compile_pure_expression_nair_boundary(&source(28, "entry main returns 20 + 22;")).unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime.runtime.execution.execution.scheduled_work = 1;
    assert!(matches!(
        validate_v03_execution(&lowering, &runtime),
        Err(PureExpressionExecutionError::InvariantViolation { .. })
    ));
}
