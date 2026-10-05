use nordoi_kernel::{
    compile_pure_expression_boundary, compile_pure_expression_execution_plan_boundary,
    CompilerError, PureExpressionPlanForm, SemanticPureExpressionOp, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("c07-{id}.noi"), text).unwrap()
}

#[test]
fn empty_body_plans_zero_work_without_authority() {
    let plan = compile_pure_expression_execution_plan_boundary(&source(1, "")).unwrap();
    assert!(matches!(plan.form(), PureExpressionPlanForm::Empty));
    assert!(plan.expression().is_none());
    assert_eq!(plan.result_i64(), None);
    assert_eq!(plan.work_item_count(), 0);
    assert!(plan.required_effects().is_empty());
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn named_entry_without_expression_is_preserved() {
    let plan = compile_pure_expression_execution_plan_boundary(&source(2, "entry main;")).unwrap();
    let entry = plan.entry().unwrap();
    assert_eq!(entry.name().as_str(), "main");
    assert!(entry.expression().is_none());
    assert_eq!(entry.result_i64(), None);
    assert_eq!(plan.result_i64(), None);
}

#[test]
fn literal_expression_is_preserved_as_one_postfix_node() {
    let plan =
        compile_pure_expression_execution_plan_boundary(&source(3, "entry main returns 42;"))
            .unwrap();
    let expression = plan.expression().unwrap();
    assert_eq!(expression.ops(), &[SemanticPureExpressionOp::Int(42)]);
    assert_eq!(expression.node_count(), 1);
    assert_eq!(expression.value(), 42);
    assert_eq!(plan.result_i64(), Some(42));
}

#[test]
fn addition_postfix_order_is_preserved() {
    let plan =
        compile_pure_expression_execution_plan_boundary(&source(4, "entry main returns 20 + 22;"))
            .unwrap();
    assert_eq!(
        plan.expression().unwrap().ops(),
        &[
            SemanticPureExpressionOp::Int(20),
            SemanticPureExpressionOp::Int(22),
            SemanticPureExpressionOp::Add,
        ]
    );
    assert_eq!(plan.result_i64(), Some(42));
}

#[test]
fn parenthesized_order_is_preserved_exactly() {
    let left = compile_pure_expression_execution_plan_boundary(&source(
        5,
        "entry main returns (1 + 2) + 3;",
    ))
    .unwrap();
    let right = compile_pure_expression_execution_plan_boundary(&source(
        6,
        "entry main returns 1 + (2 + 3);",
    ))
    .unwrap();

    assert_eq!(
        left.expression().unwrap().ops(),
        &[
            SemanticPureExpressionOp::Int(1),
            SemanticPureExpressionOp::Int(2),
            SemanticPureExpressionOp::Add,
            SemanticPureExpressionOp::Int(3),
            SemanticPureExpressionOp::Add,
        ]
    );
    assert_eq!(
        right.expression().unwrap().ops(),
        &[
            SemanticPureExpressionOp::Int(1),
            SemanticPureExpressionOp::Int(2),
            SemanticPureExpressionOp::Int(3),
            SemanticPureExpressionOp::Add,
            SemanticPureExpressionOp::Add,
        ]
    );
    assert_eq!(left.result_i64(), Some(6));
    assert_eq!(right.result_i64(), Some(6));
    assert_ne!(left.canonical_c07_bytes(), right.canonical_c07_bytes());
}

#[test]
fn unparenthesized_addition_remains_left_associative() {
    let plan = compile_pure_expression_execution_plan_boundary(&source(
        7,
        "entry main returns 1 + 2 + 3;",
    ))
    .unwrap();
    assert_eq!(
        plan.expression().unwrap().ops(),
        &[
            SemanticPureExpressionOp::Int(1),
            SemanticPureExpressionOp::Int(2),
            SemanticPureExpressionOp::Add,
            SemanticPureExpressionOp::Int(3),
            SemanticPureExpressionOp::Add,
        ]
    );
}

#[test]
fn plan_remains_zero_work_pure_and_authority_free() {
    let plan = compile_pure_expression_execution_plan_boundary(&source(
        8,
        "effect Network; entry main returns 20 + 22;",
    ))
    .unwrap();
    assert_eq!(plan.work_item_count(), 0);
    assert!(plan.required_effects().is_empty());
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn c07_witness_is_deterministic_for_equal_semantics() {
    let a = compile_pure_expression_execution_plan_boundary(&source(
        9,
        "module demo; entry main returns 20 + 22;",
    ))
    .unwrap();
    let b = compile_pure_expression_execution_plan_boundary(&source(
        10,
        "module demo; entry main returns 20 + 22;",
    ))
    .unwrap();
    assert_eq!(a.canonical_c07_bytes(), b.canonical_c07_bytes());
}

#[test]
fn whitespace_and_comments_do_not_change_c07_witness() {
    let a = compile_pure_expression_execution_plan_boundary(&source(
        11,
        "module demo; entry main returns 20 + 22;",
    ))
    .unwrap();
    let b = compile_pure_expression_execution_plan_boundary(&source(
        12,
        "module demo; /*x*/ entry /*a*/ main /*b*/ returns (20) + /*c*/ 22 ;",
    ))
    .unwrap();
    assert_eq!(a.canonical_c07_bytes(), b.canonical_c07_bytes());
}

#[test]
fn changing_entry_name_changes_c07_witness() {
    let a =
        compile_pure_expression_execution_plan_boundary(&source(13, "entry main returns 20 + 22;"))
            .unwrap();
    let b = compile_pure_expression_execution_plan_boundary(&source(
        14,
        "entry other returns 20 + 22;",
    ))
    .unwrap();
    assert_ne!(a.canonical_c07_bytes(), b.canonical_c07_bytes());
}

#[test]
fn changing_expression_value_changes_c07_witness() {
    let a =
        compile_pure_expression_execution_plan_boundary(&source(15, "entry main returns 20 + 22;"))
            .unwrap();
    let b =
        compile_pure_expression_execution_plan_boundary(&source(16, "entry main returns 20 + 23;"))
            .unwrap();
    assert_ne!(a.canonical_c07_bytes(), b.canonical_c07_bytes());
}

#[test]
fn l07_witness_is_preserved_by_c07_boundary() {
    let src = source(17, "module demo; entry main returns 20 + 22;");
    let l07 = compile_pure_expression_boundary(&src).unwrap();
    let c07 = compile_pure_expression_execution_plan_boundary(&src).unwrap();
    assert_eq!(
        l07.canonical_l07_bytes(),
        c07.expression_semantics().canonical_l07_bytes()
    );
}

#[test]
fn source_spans_are_not_part_of_c07_witness() {
    let a =
        compile_pure_expression_execution_plan_boundary(&source(18, "entry main returns 1 + 2;"))
            .unwrap();
    let b = compile_pure_expression_execution_plan_boundary(&source(
        19,
        "\n\nentry main returns 1 + 2;\n",
    ))
    .unwrap();
    assert_eq!(a.canonical_c07_bytes(), b.canonical_c07_bytes());
}

#[test]
fn zero_result_is_distinct_from_no_expression() {
    let zero =
        compile_pure_expression_execution_plan_boundary(&source(20, "entry main returns 0;"))
            .unwrap();
    let none = compile_pure_expression_execution_plan_boundary(&source(21, "entry main;")).unwrap();
    assert_eq!(zero.result_i64(), Some(0));
    assert_eq!(none.result_i64(), None);
    assert_ne!(zero.canonical_c07_bytes(), none.canonical_c07_bytes());
}

#[test]
fn node_count_is_semantic_postfix_count() {
    let plan = compile_pure_expression_execution_plan_boundary(&source(
        22,
        "entry main returns 1 + (2 + 3);",
    ))
    .unwrap();
    assert_eq!(plan.expression().unwrap().node_count(), 5);
}

#[test]
fn overflow_fails_before_plan_publication() {
    let error = compile_pure_expression_execution_plan_boundary(&source(
        23,
        "entry main returns 9223372036854775807 + 1;",
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        CompilerError::PureExpressionIntegerOverflow { .. }
            | CompilerError::PureExpressionFrontend(_)
    ));
}

#[test]
fn unsupported_operator_fails_before_plan_publication() {
    let error =
        compile_pure_expression_execution_plan_boundary(&source(24, "entry main returns 6 * 7;"))
            .unwrap_err();
    assert!(matches!(error, CompilerError::PureExpressionFrontend(_)));
}

#[test]
fn c07_plan_does_not_change_certified_l07_semantics() {
    let src = source(25, "entry main returns 4 + 5;");
    let before = compile_pure_expression_boundary(&src).unwrap();
    let plan = compile_pure_expression_execution_plan_boundary(&src).unwrap();
    assert_eq!(before.result_i64(), Some(9));
    assert_eq!(plan.result_i64(), Some(9));
    assert_eq!(
        before.canonical_l07_bytes(),
        plan.expression_semantics().canonical_l07_bytes()
    );
}
