use nordoi_kernel::{
    compile_pure_binding_boundary, compile_pure_binding_execution_plan_boundary, CompilerError,
    PureBindingPlanForm, SemanticPureBindingExpressionOp, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("c09-{id}.noi"), text).unwrap()
}

#[test]
fn empty_body_plans_zero_work_zero_storage_and_no_authority() {
    let plan = compile_pure_binding_execution_plan_boundary(&source(1, "")).unwrap();
    assert!(matches!(plan.form(), PureBindingPlanForm::Empty));
    assert_eq!(plan.binding_count(), 0);
    assert!(plan.expression().is_none());
    assert_eq!(plan.result_i64(), None);
    assert_eq!(plan.work_item_count(), 0);
    assert_eq!(plan.runtime_storage_item_count(), 0);
    assert!(plan.required_effects().is_empty());
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn entry_without_expression_preserves_bindings_without_runtime_storage() {
    let plan =
        compile_pure_binding_execution_plan_boundary(&source(2, "const x = 20; entry main;"))
            .unwrap();
    assert_eq!(plan.binding_count(), 1);
    assert_eq!(plan.entry().unwrap().name().as_str(), "main");
    assert!(plan.expression().is_none());
    assert_eq!(plan.result_i64(), None);
    assert_eq!(plan.runtime_storage_item_count(), 0);
}

#[test]
fn binding_references_are_preserved_in_plan_not_rewritten_to_literals() {
    let plan = compile_pure_binding_execution_plan_boundary(&source(
        3,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
    let ids = plan.binding_semantics().bindings().bindings();
    assert_eq!(
        plan.expression().unwrap().ops(),
        &[
            SemanticPureBindingExpressionOp::Binding(ids[0].id()),
            SemanticPureBindingExpressionOp::Binding(ids[1].id()),
            SemanticPureBindingExpressionOp::Add,
        ]
    );
    assert_eq!(plan.result_i64(), Some(42));
}

#[test]
fn registry_ids_remain_canonical_by_name() {
    let plan = compile_pure_binding_execution_plan_boundary(&source(
        4,
        "const z = 3; const a = 1; const m = 2; entry main returns a + m + z;",
    ))
    .unwrap();
    let names: Vec<_> = plan
        .binding_semantics()
        .bindings()
        .bindings()
        .iter()
        .map(|binding| (binding.id().get(), binding.name().as_str()))
        .collect();
    assert_eq!(names, vec![(1, "a"), (2, "m"), (3, "z")]);
}

#[test]
fn declaration_order_does_not_change_c09_witness() {
    let a = compile_pure_binding_execution_plan_boundary(&source(
        5,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
    let b = compile_pure_binding_execution_plan_boundary(&source(
        6,
        "const y = 22; const x = 20; entry main returns x + y;",
    ))
    .unwrap();
    assert_eq!(a.canonical_c09_bytes(), b.canonical_c09_bytes());
}

#[test]
fn whitespace_comments_and_source_id_do_not_change_c09_witness() {
    let a = compile_pure_binding_execution_plan_boundary(&source(
        7,
        "module demo; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let b = compile_pure_binding_execution_plan_boundary(&source(
        8,
        "module /*m*/ demo; const /*c*/ x = 20; entry main returns (x) + /*v*/ 22;",
    ))
    .unwrap();
    assert_eq!(a.canonical_c09_bytes(), b.canonical_c09_bytes());
}

#[test]
fn grouping_is_preserved_in_plan_witness() {
    let left = compile_pure_binding_execution_plan_boundary(&source(
        9,
        "const x = 1; const y = 2; const z = 3; entry main returns (x + y) + z;",
    ))
    .unwrap();
    let right = compile_pure_binding_execution_plan_boundary(&source(
        10,
        "const x = 1; const y = 2; const z = 3; entry main returns x + (y + z);",
    ))
    .unwrap();
    assert_eq!(left.result_i64(), Some(6));
    assert_eq!(right.result_i64(), Some(6));
    assert_ne!(
        left.expression().unwrap().ops(),
        right.expression().unwrap().ops()
    );
    assert_ne!(left.canonical_c09_bytes(), right.canonical_c09_bytes());
}

#[test]
fn same_value_from_different_binding_identity_remains_distinct() {
    let plan_x = compile_pure_binding_execution_plan_boundary(&source(
        11,
        "const x = 42; const y = 42; entry main returns x;",
    ))
    .unwrap();
    let plan_y = compile_pure_binding_execution_plan_boundary(&source(
        12,
        "const x = 42; const y = 42; entry main returns y;",
    ))
    .unwrap();
    assert_eq!(plan_x.result_i64(), plan_y.result_i64());
    assert_ne!(
        plan_x.expression().unwrap().ops(),
        plan_y.expression().unwrap().ops()
    );
    assert_ne!(plan_x.canonical_c09_bytes(), plan_y.canonical_c09_bytes());
}

#[test]
fn binding_reference_and_equal_literal_remain_distinct() {
    let by_binding = compile_pure_binding_execution_plan_boundary(&source(
        13,
        "const x = 42; entry main returns x;",
    ))
    .unwrap();
    let by_literal = compile_pure_binding_execution_plan_boundary(&source(
        14,
        "const x = 42; entry main returns 42;",
    ))
    .unwrap();
    assert_eq!(by_binding.result_i64(), by_literal.result_i64());
    assert_ne!(
        by_binding.expression().unwrap().ops(),
        by_literal.expression().unwrap().ops()
    );
    assert_ne!(
        by_binding.canonical_c09_bytes(),
        by_literal.canonical_c09_bytes()
    );
}

#[test]
fn changing_binding_value_changes_c09_witness() {
    let a = compile_pure_binding_execution_plan_boundary(&source(
        15,
        "const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let b = compile_pure_binding_execution_plan_boundary(&source(
        16,
        "const x = 21; entry main returns x + 22;",
    ))
    .unwrap();
    assert_ne!(a.canonical_c09_bytes(), b.canonical_c09_bytes());
}

#[test]
fn l08_witness_is_preserved_by_c09_boundary() {
    let src = source(17, "module demo; const x = 20; entry main returns x + 22;");
    let l08 = compile_pure_binding_boundary(&src).unwrap();
    let c09 = compile_pure_binding_execution_plan_boundary(&src).unwrap();
    assert_eq!(
        l08.canonical_l08_bytes(),
        c09.binding_semantics().canonical_l08_bytes()
    );
}

#[test]
fn plan_is_zero_work_zero_storage_pure_and_authority_free() {
    let plan = compile_pure_binding_execution_plan_boundary(&source(
        18,
        "effect Network; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    assert_eq!(plan.work_item_count(), 0);
    assert_eq!(plan.runtime_storage_item_count(), 0);
    assert!(plan.required_effects().is_empty());
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn zero_result_is_distinct_from_no_expression() {
    let zero = compile_pure_binding_execution_plan_boundary(&source(
        19,
        "const z = 0; entry main returns z;",
    ))
    .unwrap();
    let none =
        compile_pure_binding_execution_plan_boundary(&source(20, "const z = 0; entry main;"))
            .unwrap();
    assert_eq!(zero.result_i64(), Some(0));
    assert_eq!(none.result_i64(), None);
    assert_ne!(zero.canonical_c09_bytes(), none.canonical_c09_bytes());
}

#[test]
fn c09_witness_has_explicit_domain() {
    let plan = compile_pure_binding_execution_plan_boundary(&source(
        21,
        "const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    assert!(plan
        .canonical_c09_bytes()
        .starts_with(b"NORDOI-C0.9-PURE-BINDING-PLAN\0"));
}

#[test]
fn unknown_binding_fails_before_plan_publication() {
    let error = compile_pure_binding_execution_plan_boundary(&source(
        22,
        "entry main returns missing + 1;",
    ))
    .unwrap_err();
    assert!(matches!(error, CompilerError::UnknownPureBinding { .. }));
}

#[test]
fn duplicate_binding_fails_before_plan_publication() {
    let error = compile_pure_binding_execution_plan_boundary(&source(
        23,
        "const x = 1; const x = 2; entry main returns x;",
    ))
    .unwrap_err();
    assert!(matches!(error, CompilerError::DuplicatePureBinding { .. }));
}

#[test]
fn overflow_fails_before_plan_publication() {
    let error = compile_pure_binding_execution_plan_boundary(&source(
        24,
        "const max = 9223372036854775807; entry main returns max + 1;",
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        CompilerError::PureBindingIntegerOverflow { .. }
    ));
}

#[test]
fn l07_expression_plan_boundary_stays_frozen_and_rejects_const_prelude() {
    use nordoi_kernel::compile_pure_expression_execution_plan_boundary;
    let error = compile_pure_expression_execution_plan_boundary(&source(
        25,
        "const x = 20; entry main returns x + 22;",
    ))
    .unwrap_err();
    assert!(matches!(error, CompilerError::PureExpressionFrontend(_)));
}

#[test]
fn c09_without_bindings_remains_additive_over_l07_shape() {
    let plan =
        compile_pure_binding_execution_plan_boundary(&source(26, "entry main returns 20 + 22;"))
            .unwrap();
    assert_eq!(plan.binding_count(), 0);
    assert_eq!(plan.result_i64(), Some(42));
    assert_eq!(
        plan.expression().unwrap().ops(),
        &[
            SemanticPureBindingExpressionOp::Int(20),
            SemanticPureBindingExpressionOp::Int(22),
            SemanticPureBindingExpressionOp::Add,
        ]
    );
}
