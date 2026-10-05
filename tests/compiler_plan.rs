use nordoi_kernel::{
    compile_execution_plan_boundary, compile_minimal_body_boundary, CompilerError,
    SemanticPlanForm, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("c03-{id}.noi"), text).unwrap()
}

#[test]
fn empty_body_compiles_to_zero_work_plan() {
    let plan =
        compile_execution_plan_boundary(&source(1, "module demo; type A; // tail\n")).unwrap();
    assert!(matches!(plan.form(), SemanticPlanForm::Empty));
    assert_eq!(plan.work_item_count(), 0);
    assert!(plan.required_effects().is_empty());
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn entry_body_compiles_to_named_zero_work_plan() {
    let plan = compile_execution_plan_boundary(&source(2, "module demo; entry main;")).unwrap();
    let entry = plan.entry().unwrap();
    assert_eq!(entry.name().as_str(), "main");
    assert!(entry.required_effects().is_pure());
    assert_eq!(plan.work_item_count(), 0);
}

#[test]
fn declared_effect_does_not_become_plan_requirement_or_authority() {
    let plan =
        compile_execution_plan_boundary(&source(3, "module demo; effect Network; entry main;"))
            .unwrap();
    assert!(plan.required_effects().is_empty());
    assert!(!plan.requires_host_authority());
    assert!(plan
        .body_semantics()
        .semantic()
        .registry()
        .resolve_effect("Network")
        .is_some());
}

#[test]
fn c03_witness_is_deterministic_for_equal_source() {
    let src = source(4, "module demo; type A; entry main;");
    assert_eq!(
        compile_execution_plan_boundary(&src)
            .unwrap()
            .canonical_c03_bytes(),
        compile_execution_plan_boundary(&src)
            .unwrap()
            .canonical_c03_bytes()
    );
}

#[test]
fn comments_spacing_and_source_id_do_not_change_plan_identity() {
    let first = source(5, "module demo; type A; entry main;");
    let second = source(
        6,
        "module demo /*m*/ ;\n type /*t*/ A ;\n entry /*e*/ main ; // tail\n",
    );
    assert_eq!(
        compile_execution_plan_boundary(&first)
            .unwrap()
            .canonical_c03_bytes(),
        compile_execution_plan_boundary(&second)
            .unwrap()
            .canonical_c03_bytes()
    );
}

#[test]
fn entry_name_changes_plan_identity() {
    let first = compile_execution_plan_boundary(&source(7, "entry main;")).unwrap();
    let second = compile_execution_plan_boundary(&source(8, "entry other;")).unwrap();
    assert_ne!(first.canonical_c03_bytes(), second.canonical_c03_bytes());
}

#[test]
fn module_identity_changes_plan_identity() {
    let first = compile_execution_plan_boundary(&source(9, "module one; entry main;")).unwrap();
    let second = compile_execution_plan_boundary(&source(10, "module two; entry main;")).unwrap();
    assert_ne!(first.canonical_c03_bytes(), second.canonical_c03_bytes());
}

#[test]
fn registry_identity_changes_plan_identity() {
    let first = compile_execution_plan_boundary(&source(11, "type A; entry main;")).unwrap();
    let second = compile_execution_plan_boundary(&source(12, "type B; entry main;")).unwrap();
    assert_ne!(first.canonical_c03_bytes(), second.canonical_c03_bytes());
}

#[test]
fn declaration_source_order_does_not_change_plan_identity() {
    let first = source(13, "type Z; effect Net; type A; effect Clock; entry main;");
    let second = source(14, "effect Clock; type A; effect Net; type Z; entry main;");
    assert_eq!(
        compile_execution_plan_boundary(&first)
            .unwrap()
            .canonical_c03_bytes(),
        compile_execution_plan_boundary(&second)
            .unwrap()
            .canonical_c03_bytes()
    );
}

#[test]
fn empty_and_entry_plans_have_distinct_identity() {
    let empty = compile_execution_plan_boundary(&source(15, "module demo;")).unwrap();
    let entry = compile_execution_plan_boundary(&source(16, "module demo; entry main;")).unwrap();
    assert_ne!(empty.canonical_c03_bytes(), entry.canonical_c03_bytes());
}

#[test]
fn c03_preserves_certified_l05_witness() {
    let src = source(17, "module demo; type A; effect Net; entry main;");
    let l05 = compile_minimal_body_boundary(&src).unwrap();
    let c03 = compile_execution_plan_boundary(&src).unwrap();
    assert_eq!(
        l05.canonical_l05_bytes(),
        c03.body_semantics().canonical_l05_bytes()
    );
}

#[test]
fn c03_preserves_c02_semantic_registry_witness() {
    let src = source(18, "module demo; type A; effect Net; entry main;");
    let l05 = compile_minimal_body_boundary(&src).unwrap();
    let c03 = compile_execution_plan_boundary(&src).unwrap();
    assert_eq!(
        l05.semantic().canonical_c02_bytes(),
        c03.body_semantics().semantic().canonical_c02_bytes()
    );
}

#[test]
fn unsupported_body_fails_before_plan_publication() {
    assert!(matches!(
        compile_execution_plan_boundary(&source(19, "future_body")),
        Err(CompilerError::BodyFrontend(_))
    ));
}

#[test]
fn anonymous_module_can_have_entry_plan() {
    let plan = compile_execution_plan_boundary(&source(20, "entry main;")).unwrap();
    assert!(plan
        .body_semantics()
        .semantic()
        .module()
        .canonical_text()
        .is_none());
    assert_eq!(plan.entry().unwrap().name().as_str(), "main");
}

#[test]
fn c03_domain_is_explicit() {
    let bytes = compile_execution_plan_boundary(&source(21, "entry main;"))
        .unwrap()
        .canonical_c03_bytes();
    assert!(bytes.starts_with(b"NORDOI-C0.3-PLAN\0"));
}
