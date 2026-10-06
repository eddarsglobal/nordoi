use nordoi_kernel::{
    compile_pure_condition_nair_v05, compile_static_if_nair_v05, compile_static_if_plan_v05,
    execute_pure_condition_source_v05, execute_static_if_source_v05, SourceId, SourceText,
    StaticIfBranch, Value,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(505), "v05.noi", text).unwrap()
}

#[test]
fn bool_true_executes_as_base_nair_06() {
    let report = execute_pure_condition_source_v05(&source("entry main returns true;")).unwrap();
    assert_eq!(report.result_bool(), Some(true));
    assert_eq!(report.lowering().nair_format_minor(), 6);
    assert_eq!(report.lowering().nair_instruction_count(), 2);
    assert!(report.runtime().is_quiescent());
}

#[test]
fn bool_false_executes_as_false_not_none() {
    let report = execute_pure_condition_source_v05(&source("entry main returns false;")).unwrap();
    assert_eq!(report.result_bool(), Some(false));
    assert_eq!(
        report
            .runtime()
            .register(report.lowering().result_register().unwrap()),
        Some(&Value::Bool(false))
    );
}

#[test]
fn integer_comparison_executes_through_nair_08() {
    let report =
        execute_pure_condition_source_v05(&source("entry main returns 20 <= 22;")).unwrap();
    assert_eq!(report.result_bool(), Some(true));
    assert_eq!(report.lowering().nair_format_minor(), 8);
    assert_eq!(report.lowering().nair_instruction_count(), 4);
    assert_eq!(report.runtime().final_registers().len(), 3);
}

#[test]
fn all_comparators_execute_correctly() {
    for (expr, expected) in [
        ("1 == 1", true),
        ("1 != 2", true),
        ("1 < 2", true),
        ("2 <= 2", true),
        ("3 > 2", true),
        ("3 >= 3", true),
        ("3 < 2", false),
    ] {
        let text = format!("entry main returns {expr};");
        assert_eq!(
            execute_pure_condition_source_v05(&source(&text))
                .unwrap()
                .result_bool(),
            Some(expected),
            "expr={expr}"
        );
    }
}

#[test]
fn static_if_selects_then_branch_with_bindings() {
    let report = execute_static_if_source_v05(&source(
        "const value = 20; const limit = 22; entry main returns if value < limit { value + 22 } else { 0 };",
    ))
    .unwrap();
    assert_eq!(
        report.lowering().plan().selected_branch(),
        StaticIfBranch::Then
    );
    assert_eq!(report.result_i64(), Some(42));
    assert_eq!(report.lowering().dead_branch_instruction_count(), 0);
    assert_eq!(report.lowering().plan().runtime_branch_count(), 0);
    assert!(report.runtime().is_quiescent());
}

#[test]
fn static_if_selects_else_branch() {
    let report = execute_static_if_source_v05(&source(
        "const value = 30; const limit = 22; entry main returns if value < limit { 999 } else { value + 12 };",
    ))
    .unwrap();
    assert_eq!(
        report.lowering().plan().selected_branch(),
        StaticIfBranch::Else
    );
    assert_eq!(report.result_i64(), Some(42));
    assert_eq!(report.lowering().dead_branch_instruction_count(), 0);
}

#[test]
fn dead_branch_content_does_not_change_operational_nair() {
    let a = compile_static_if_nair_v05(&source("entry main returns if true { 42 } else { 1 };"))
        .unwrap();
    let b = compile_static_if_nair_v05(&source(
        "entry main returns if true { 42 } else { 999 + 1 };",
    ))
    .unwrap();
    assert_eq!(
        a.program().canonical_bytes().unwrap(),
        b.program().canonical_bytes().unwrap()
    );
    assert_ne!(
        a.canonical_v05_lowering_bytes(),
        b.canonical_v05_lowering_bytes()
    );
}

#[test]
fn dead_branch_is_still_semantically_validated() {
    let error = compile_static_if_plan_v05(&source(
        "entry main returns if true { 42 } else { missing + 1 };",
    ))
    .unwrap_err();
    assert!(error.is_frontend_failure());
}

#[test]
fn unknown_binding_in_condition_fails_before_runtime() {
    let error = compile_static_if_plan_v05(&source(
        "entry main returns if missing < 2 { 1 } else { 0 };",
    ))
    .unwrap_err();
    assert!(error.is_frontend_failure());
}

#[test]
fn literal_false_static_if_has_zero_runtime_branch_cost() {
    let artifact = compile_static_if_nair_v05(&source(
        "entry main returns if false { 999 + 1 } else { 42 };",
    ))
    .unwrap();
    assert_eq!(artifact.plan().selected_branch(), StaticIfBranch::Else);
    assert_eq!(artifact.dead_branch_instruction_count(), 0);
    assert_eq!(artifact.nair_instruction_count(), 2);
    assert_eq!(artifact.nair_format_minor(), 6);
}

#[test]
fn selected_addition_branch_reuses_nair_07_only() {
    let artifact = compile_static_if_nair_v05(&source(
        "entry main returns if 1 < 2 { 20 + 22 } else { 0 };",
    ))
    .unwrap();
    assert_eq!(artifact.nair_format_minor(), 7);
    assert_eq!(artifact.nair_instruction_count(), 4);
}

#[test]
fn direct_condition_and_static_if_witnesses_are_deterministic() {
    let condition_a =
        compile_pure_condition_nair_v05(&source("entry main returns 3 >= 2;")).unwrap();
    let condition_b =
        compile_pure_condition_nair_v05(&source("entry main returns 3 >= 2;")).unwrap();
    assert_eq!(
        condition_a.canonical_v05_condition_nair_bytes(),
        condition_b.canonical_v05_condition_nair_bytes()
    );

    let if_a =
        compile_static_if_nair_v05(&source("entry main returns if 3 >= 2 { 42 } else { 0 };"))
            .unwrap();
    let if_b =
        compile_static_if_nair_v05(&source("entry main returns if 3 >= 2 { 42 } else { 0 };"))
            .unwrap();
    assert_eq!(
        if_a.canonical_v05_lowering_bytes(),
        if_b.canonical_v05_lowering_bytes()
    );
}
