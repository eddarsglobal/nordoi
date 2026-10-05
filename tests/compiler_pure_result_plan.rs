use nordoi_kernel::{
    compile_execution_plan_boundary, compile_nair_lowering_boundary, compile_pure_result_boundary,
    compile_pure_result_execution_plan_boundary, execute_source_v01, CompilerError,
    PureResultPlanForm, PureResultPlanValue, SourceExecutionError, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "result-plan.noi", text).unwrap()
}

#[test]
fn integer_result_compiles_to_zero_work_pure_result_plan() {
    let plan = compile_pure_result_execution_plan_boundary(&source(
        1,
        "module demo; type A; effect Net; entry main returns 42;",
    ))
    .unwrap();

    match plan.form() {
        PureResultPlanForm::Entry(entry) => {
            assert_eq!(entry.name().as_str(), "main");
            assert_eq!(entry.result_i64(), Some(42));
            assert!(entry.required_effects().is_pure());
        }
        _ => panic!("expected entry"),
    }
    assert_eq!(plan.work_item_count(), 0);
    assert!(plan.required_effects().is_empty());
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn c05_witness_has_explicit_domain() {
    let plan =
        compile_pure_result_execution_plan_boundary(&source(1, "entry main returns 42;")).unwrap();
    assert!(plan
        .canonical_c05_bytes()
        .starts_with(b"NORDOI-C0.5-PURE-RESULT-PLAN\0"));
}

#[test]
fn c05_preserves_exact_l06_witness() {
    let src = source(1, "module demo; entry main returns 42;");
    let l06 = compile_pure_result_boundary(&src).unwrap();
    let c05 = compile_pure_result_execution_plan_boundary(&src).unwrap();
    assert_eq!(
        l06.canonical_l06_bytes(),
        c05.result_semantics().canonical_l06_bytes()
    );
}

#[test]
fn c05_preserves_c02_registry_identity() {
    let src = source(1, "module demo; type Z; type A; entry main returns 42;");
    let l06 = compile_pure_result_boundary(&src).unwrap();
    let c05 = compile_pure_result_execution_plan_boundary(&src).unwrap();
    assert_eq!(
        l06.semantic().canonical_c02_bytes(),
        c05.result_semantics().semantic().canonical_c02_bytes()
    );
}

#[test]
fn changing_result_changes_c05_witness() {
    let a =
        compile_pure_result_execution_plan_boundary(&source(1, "entry main returns 1;")).unwrap();
    let b =
        compile_pure_result_execution_plan_boundary(&source(2, "entry main returns 2;")).unwrap();
    assert_ne!(a.canonical_c05_bytes(), b.canonical_c05_bytes());
}

#[test]
fn equal_semantics_ignore_comments_spacing_and_source_id() {
    let a = compile_pure_result_execution_plan_boundary(&source(
        1,
        "module demo; entry main returns 42;",
    ))
    .unwrap();
    let b = compile_pure_result_execution_plan_boundary(&source(
        99,
        "module /*x*/ demo ;\n entry /*a*/ main /*b*/ returns /*c*/ 42 /*d*/ ;",
    ))
    .unwrap();
    assert_eq!(a.canonical_c05_bytes(), b.canonical_c05_bytes());
}

#[test]
fn declaration_source_order_does_not_change_c05_identity() {
    let a = compile_pure_result_execution_plan_boundary(&source(
        1,
        "type Z; type A; effect Y; effect B; entry main returns 42;",
    ))
    .unwrap();
    let b = compile_pure_result_execution_plan_boundary(&source(
        2,
        "effect B; type A; effect Y; type Z; entry main returns 42;",
    ))
    .unwrap();
    assert_eq!(a.canonical_c05_bytes(), b.canonical_c05_bytes());
}

#[test]
fn declared_effect_does_not_become_requirement_or_authority() {
    let plan = compile_pure_result_execution_plan_boundary(&source(
        1,
        "effect Network; entry main returns 7;",
    ))
    .unwrap();
    assert!(plan.required_effects().is_empty());
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn entry_without_result_is_supported_as_none() {
    let plan = compile_pure_result_execution_plan_boundary(&source(1, "entry main;")).unwrap();
    let entry = plan.entry().unwrap();
    assert_eq!(entry.name().as_str(), "main");
    assert_eq!(entry.result(), None);
    assert_eq!(plan.result_i64(), None);
}

#[test]
fn empty_body_is_supported_as_empty_plan() {
    let plan =
        compile_pure_result_execution_plan_boundary(&source(1, "module demo; // tail\n")).unwrap();
    assert!(matches!(plan.form(), PureResultPlanForm::Empty));
    assert_eq!(plan.result(), None);
    assert_eq!(plan.work_item_count(), 0);
}

#[test]
fn zero_result_is_distinct_from_no_result() {
    let none = compile_pure_result_execution_plan_boundary(&source(1, "entry main;")).unwrap();
    let zero =
        compile_pure_result_execution_plan_boundary(&source(2, "entry main returns 0;")).unwrap();
    assert_ne!(none.canonical_c05_bytes(), zero.canonical_c05_bytes());
    assert_eq!(zero.result(), Some(PureResultPlanValue::Int(0)));
}

#[test]
fn int_max_is_preserved_exactly() {
    let plan = compile_pure_result_execution_plan_boundary(&source(
        1,
        "entry main returns 9223372036854775807;",
    ))
    .unwrap();
    assert_eq!(plan.result_i64(), Some(i64::MAX));
}

#[test]
fn entry_name_changes_c05_witness() {
    let a =
        compile_pure_result_execution_plan_boundary(&source(1, "entry alpha returns 42;")).unwrap();
    let b =
        compile_pure_result_execution_plan_boundary(&source(2, "entry beta returns 42;")).unwrap();
    assert_ne!(a.canonical_c05_bytes(), b.canonical_c05_bytes());
}

#[test]
fn module_identity_changes_c05_witness() {
    let a =
        compile_pure_result_execution_plan_boundary(&source(1, "module a; entry main returns 42;"))
            .unwrap();
    let b =
        compile_pure_result_execution_plan_boundary(&source(2, "module b; entry main returns 42;"))
            .unwrap();
    assert_ne!(a.canonical_c05_bytes(), b.canonical_c05_bytes());
}

#[test]
fn c03_plan_remains_frozen_and_rejects_result_source() {
    let error = compile_execution_plan_boundary(&source(1, "entry main returns 42;")).unwrap_err();
    assert!(matches!(error, CompilerError::BodyFrontend(_)));
}

#[test]
fn c04_lowering_does_not_silently_accept_result_source() {
    let error = compile_nair_lowering_boundary(&source(1, "entry main returns 42;")).unwrap_err();
    assert!(matches!(error, CompilerError::BodyFrontend(_)));
}

#[test]
fn v01_execution_does_not_silently_accept_result_source() {
    let error = execute_source_v01(&source(1, "entry main returns 42;")).unwrap_err();
    assert!(matches!(
        error,
        SourceExecutionError::Compiler(CompilerError::BodyFrontend(_))
    ));
}

#[test]
fn result_payload_is_semantic_not_operational_work() {
    let plan =
        compile_pure_result_execution_plan_boundary(&source(1, "entry main returns 42;")).unwrap();
    assert_eq!(plan.result_i64(), Some(42));
    assert_eq!(plan.work_item_count(), 0);
    assert!(plan.required_effects().is_empty());
}
