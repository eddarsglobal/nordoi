use nordoi_kernel::{
    compile_pure_binding_nair_boundary, execute_pure_binding_source_v04,
    execute_pure_expression_source_v03, run_closed_observed, validate_v04_execution, InputBatch,
    PureBindingExecutionError, RegisterId, SourceId, SourceText, Value,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("v04-{id}.noi"), text).unwrap()
}

#[test]
fn binding_addition_executes_end_to_end_without_runtime_binding_state() {
    let report = execute_pure_binding_source_v04(&source(
        1,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
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
    assert_eq!(report.binding_runtime_storage_item_count(), 0);
    assert_eq!(report.binding_runtime_lookup_count(), 0);
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
fn grouped_binding_expression_executes_in_certified_postfix_order() {
    let report = execute_pure_binding_source_v04(&source(
        2,
        "const x = 1; const y = 2; const z = 3; entry main returns x + (y + z);",
    ))
    .unwrap();
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
fn declaration_order_does_not_change_runtime_replay_or_receipt() {
    let a = execute_pure_binding_source_v04(&source(
        3,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
    let b = execute_pure_binding_source_v04(&source(
        4,
        "const y = 22; const x = 20; entry main returns x + y;",
    ))
    .unwrap();
    assert_eq!(
        a.runtime().runtime().replay_key,
        b.runtime().runtime().replay_key
    );
    assert_eq!(
        a.canonical_v04_receipt_bytes(),
        b.canonical_v04_receipt_bytes()
    );
}

#[test]
fn unused_binding_has_zero_runtime_cost_vs_literal_program() {
    let binding =
        execute_pure_binding_source_v04(&source(5, "const unused = 999; entry main returns 42;"))
            .unwrap();
    let literal = execute_pure_expression_source_v03(&source(6, "entry main returns 42;")).unwrap();
    assert_eq!(binding.result_i64(), Some(42));
    assert_eq!(
        binding.runtime().runtime().replay_key,
        literal.runtime().runtime().replay_key
    );
    assert_eq!(
        binding
            .runtime()
            .runtime()
            .execution
            .execution
            .executed_instructions,
        2
    );
    assert_eq!(binding.runtime().final_registers().len(), 1);
}

#[test]
fn binding_reference_and_equivalent_literal_share_runtime_replay_but_not_v04_receipt() {
    let by_binding =
        execute_pure_binding_source_v04(&source(7, "const x = 42; entry main returns x;")).unwrap();
    let by_literal =
        execute_pure_binding_source_v04(&source(8, "const x = 42; entry main returns 42;"))
            .unwrap();
    assert_eq!(
        by_binding.runtime().runtime().replay_key,
        by_literal.runtime().runtime().replay_key
    );
    assert_ne!(
        by_binding.canonical_v04_receipt_bytes(),
        by_literal.canonical_v04_receipt_bytes()
    );
}

#[test]
fn single_binding_reference_remains_nair_06() {
    let report = execute_pure_binding_source_v04(&source(
        9,
        "const answer = 42; entry main returns answer;",
    ))
    .unwrap();
    assert_eq!(report.lowering().nair_format_minor(), 6);
    assert_eq!(report.result_i64(), Some(42));
    assert_eq!(report.runtime().final_registers().len(), 1);
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
fn entry_without_expression_executes_halt_only_with_zero_binding_state() {
    let report =
        execute_pure_binding_source_v04(&source(10, "const x = 20; const y = 22; entry main;"))
            .unwrap();
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
    assert_eq!(report.binding_runtime_storage_item_count(), 0);
}

#[test]
fn empty_body_executes_halt_only() {
    let report = execute_pure_binding_source_v04(&source(11, "module demo; // tail\n")).unwrap();
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
    assert!(report.runtime().is_quiescent());
}

#[test]
fn closed_binding_execution_creates_no_external_runtime_state() {
    let report =
        execute_pure_binding_source_v04(&source(12, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let runtime = report.runtime().runtime();
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
    assert!(report.runtime().is_quiescent());
}

#[test]
fn repeated_execution_is_deterministic() {
    let src = source(13, "module demo; const x = 20; entry main returns x + 22;");
    let a = execute_pure_binding_source_v04(&src).unwrap();
    let b = execute_pure_binding_source_v04(&src).unwrap();
    assert_eq!(a.runtime(), b.runtime());
    assert_eq!(
        a.canonical_v04_receipt_bytes(),
        b.canonical_v04_receipt_bytes()
    );
}

#[test]
fn source_id_and_trivia_do_not_change_v04_receipt() {
    let a = execute_pure_binding_source_v04(&source(
        14,
        "module demo; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let b = execute_pure_binding_source_v04(&source(
        99,
        "module /*m*/ demo; const /*c*/ x = 20; entry main returns (x) + /*v*/ 22;",
    ))
    .unwrap();
    assert_eq!(
        a.canonical_v04_receipt_bytes(),
        b.canonical_v04_receipt_bytes()
    );
}

#[test]
fn module_identity_changes_receipt_but_not_runtime_replay() {
    let a = execute_pure_binding_source_v04(&source(
        15,
        "module a; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let b = execute_pure_binding_source_v04(&source(
        16,
        "module b; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    assert_eq!(
        a.runtime().runtime().replay_key,
        b.runtime().runtime().replay_key
    );
    assert_ne!(
        a.canonical_v04_receipt_bytes(),
        b.canonical_v04_receipt_bytes()
    );
}

#[test]
fn receipt_commits_to_exact_c010_witness() {
    let report =
        execute_pure_binding_source_v04(&source(17, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let c010 = report.lowering().canonical_c010_bytes();
    assert!(report
        .canonical_v04_receipt_bytes()
        .windows(c010.len())
        .any(|window| window == c010.as_slice()));
}

#[test]
fn receipt_commits_to_canonical_empty_input() {
    let report =
        execute_pure_binding_source_v04(&source(18, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let input = InputBatch::default().canonical_bytes().unwrap();
    assert_eq!(report.canonical_input_bytes(), input.as_slice());
}

#[test]
fn direct_observed_runtime_matches_v04_runtime() {
    let src = source(19, "const x = 20; entry main returns x + 22;");
    let report = execute_pure_binding_source_v04(&src).unwrap();
    let direct = run_closed_observed(report.lowering().program(), &InputBatch::default()).unwrap();
    assert_eq!(report.runtime(), &direct);
}

#[test]
fn overflow_fails_closed_before_runtime_publication() {
    let result = execute_pure_binding_source_v04(&source(
        20,
        "const max = 9223372036854775807; entry main returns max + 1;",
    ));
    assert!(matches!(
        result,
        Err(PureBindingExecutionError::Compiler(_))
    ));
}

#[test]
fn validation_rejects_forged_final_result_register_value() {
    let lowering =
        compile_pure_binding_nair_boundary(&source(21, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime
        .final_registers
        .insert(RegisterId(2), Value::Int(41));
    assert!(matches!(
        validate_v04_execution(&lowering, &runtime),
        Err(PureBindingExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_forged_intermediate_binding_register_value() {
    let lowering =
        compile_pure_binding_nair_boundary(&source(22, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime
        .final_registers
        .insert(RegisterId(0), Value::Int(19));
    assert!(matches!(
        validate_v04_execution(&lowering, &runtime),
        Err(PureBindingExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_extra_register_observation() {
    let lowering =
        compile_pure_binding_nair_boundary(&source(23, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime
        .final_registers
        .insert(RegisterId(3), Value::Int(99));
    assert!(matches!(
        validate_v04_execution(&lowering, &runtime),
        Err(PureBindingExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_runtime_claiming_input_events() {
    let lowering =
        compile_pure_binding_nair_boundary(&source(24, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime.runtime.input_events = 1;
    assert!(matches!(
        validate_v04_execution(&lowering, &runtime),
        Err(PureBindingExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn validation_rejects_runtime_claiming_residual_work() {
    let lowering =
        compile_pure_binding_nair_boundary(&source(25, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let mut runtime = run_closed_observed(lowering.program(), &InputBatch::default()).unwrap();
    runtime.runtime.execution.execution.scheduled_work = 1;
    assert!(matches!(
        validate_v04_execution(&lowering, &runtime),
        Err(PureBindingExecutionError::InvariantViolation { .. })
    ));
}

#[test]
fn different_grouping_same_value_has_different_replay_and_receipt() {
    let left = execute_pure_binding_source_v04(&source(
        26,
        "const x = 1; const y = 2; const z = 3; entry main returns (x + y) + z;",
    ))
    .unwrap();
    let right = execute_pure_binding_source_v04(&source(
        27,
        "const x = 1; const y = 2; const z = 3; entry main returns x + (y + z);",
    ))
    .unwrap();
    assert_eq!(left.result_i64(), Some(6));
    assert_eq!(right.result_i64(), Some(6));
    assert_ne!(
        left.runtime().runtime().replay_key,
        right.runtime().runtime().replay_key
    );
    assert_ne!(
        left.canonical_v04_receipt_bytes(),
        right.canonical_v04_receipt_bytes()
    );
}
