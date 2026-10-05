use nordoi_kernel::{
    analyze_pure_expression_unit, PureExpressionBodyForm, PureExpressionError, SourceId,
    SourceText, SurfacePureExpressionOp, MAX_PURE_EXPRESSION_NODES,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "expr.noi", text).unwrap()
}

#[test]
fn empty_body_is_supported() {
    let unit = analyze_pure_expression_unit(&source(1, "module demo;")).unwrap();
    assert!(matches!(unit.form(), PureExpressionBodyForm::Empty));
}

#[test]
fn l05_entry_is_supported_without_expression() {
    let unit = analyze_pure_expression_unit(&source(1, "entry main;")).unwrap();
    assert!(matches!(unit.form(), PureExpressionBodyForm::Entry(_)));
}

#[test]
fn single_literal_is_one_postfix_node() {
    let unit = analyze_pure_expression_unit(&source(1, "entry main returns 42;")).unwrap();
    let PureExpressionBodyForm::EntryExpression(entry) = unit.form() else {
        panic!("expected expression entry");
    };
    assert_eq!(entry.expression().node_count(), 1);
    assert_eq!(entry.expression().ops()[0].int_value(), Some(42));
}

#[test]
fn addition_is_lowered_to_left_associative_postfix() {
    let unit = analyze_pure_expression_unit(&source(1, "entry main returns 1 + 2 + 3;")).unwrap();
    let PureExpressionBodyForm::EntryExpression(entry) = unit.form() else {
        panic!("expected expression entry");
    };
    let ops = entry.expression().ops();
    assert_eq!(ops.len(), 5);
    assert!(matches!(
        ops.first(),
        Some(SurfacePureExpressionOp::Int { value: 1, .. })
    ));
    assert!(matches!(
        ops.get(1),
        Some(SurfacePureExpressionOp::Int { value: 2, .. })
    ));
    assert!(matches!(
        ops.get(2),
        Some(SurfacePureExpressionOp::Add { .. })
    ));
    assert!(matches!(
        ops.get(3),
        Some(SurfacePureExpressionOp::Int { value: 3, .. })
    ));
    assert!(matches!(
        ops.get(4),
        Some(SurfacePureExpressionOp::Add { .. })
    ));
}

#[test]
fn parentheses_preserve_evaluation_structure() {
    let left = analyze_pure_expression_unit(&source(1, "entry main returns (1 + 2) + 3;")).unwrap();
    let right =
        analyze_pure_expression_unit(&source(2, "entry main returns 1 + (2 + 3);")).unwrap();
    let PureExpressionBodyForm::EntryExpression(left) = left.form() else {
        panic!("expected expression entry");
    };
    let PureExpressionBodyForm::EntryExpression(right) = right.form() else {
        panic!("expected expression entry");
    };
    assert_ne!(left.expression().ops(), right.expression().ops());
}

#[test]
fn comments_are_trivia_inside_expression_and_groups() {
    let unit = analyze_pure_expression_unit(&source(
        1,
        "entry main returns 1 /*a*/ + ( /*b*/ 2 + 3 /*c*/ );",
    ))
    .unwrap();
    let PureExpressionBodyForm::EntryExpression(entry) = unit.form() else {
        panic!("expected expression entry");
    };
    assert_eq!(entry.expression().node_count(), 5);
}

#[test]
fn zero_literal_is_canonical() {
    let unit = analyze_pure_expression_unit(&source(1, "entry main returns 0 + 0;")).unwrap();
    assert!(matches!(
        unit.form(),
        PureExpressionBodyForm::EntryExpression(_)
    ));
}

#[test]
fn leading_zero_is_rejected() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns 01 + 2;")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::InvalidIntegerLiteral { .. }
    ));
}

#[test]
fn oversized_literal_is_rejected() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns 9223372036854775808;"))
        .unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::IntegerLiteralOutOfRange { .. }
    ));
}

#[test]
fn negative_literal_is_not_part_of_l07() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns -1;")).unwrap_err();
    assert!(matches!(error, PureExpressionError::ExpectedOperand { .. }));
}

#[test]
fn unary_plus_is_not_part_of_l07() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns +1;")).unwrap_err();
    assert!(matches!(error, PureExpressionError::ExpectedOperand { .. }));
}

#[test]
fn multiplication_is_not_part_of_l07() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns 2 * 3;")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::ExpectedPlusOrEnd { .. }
    ));
}

#[test]
fn subtraction_is_not_part_of_l07() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns 2 - 1;")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::ExpectedPlusOrEnd { .. }
    ));
}

#[test]
fn bracket_group_is_rejected() {
    let error =
        analyze_pure_expression_unit(&source(1, "entry main returns [1 + 2];")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::UnsupportedGroup { .. }
    ));
}

#[test]
fn brace_group_is_rejected() {
    let error =
        analyze_pure_expression_unit(&source(1, "entry main returns {1 + 2};")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::UnsupportedGroup { .. }
    ));
}

#[test]
fn empty_parentheses_are_rejected() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns ();")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::EmptyParenthesizedExpression { .. }
    ));
}

#[test]
fn consecutive_plus_is_rejected() {
    let error =
        analyze_pure_expression_unit(&source(1, "entry main returns 1 + + 2;")).unwrap_err();
    assert!(matches!(error, PureExpressionError::ExpectedOperand { .. }));
}

#[test]
fn trailing_plus_is_rejected() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns 1 +;")).unwrap_err();
    assert!(matches!(error, PureExpressionError::ExpectedOperand { .. }));
}

#[test]
fn adjacent_literals_without_operator_are_rejected() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns 1 2;")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::ExpectedPlusOrEnd { .. }
    ));
}

#[test]
fn missing_expression_is_rejected() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns ;")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::ExpectedExpression { .. }
    ));
}

#[test]
fn missing_terminator_is_rejected() {
    let error = analyze_pure_expression_unit(&source(1, "entry main returns 1 + 2")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::ExpectedExpressionTerminator { .. }
    ));
}

#[test]
fn significant_element_after_entry_is_rejected() {
    let error =
        analyze_pure_expression_unit(&source(1, "entry main returns 1 + 2; other")).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::UnexpectedAfterEntry { .. }
    ));
}

#[test]
fn expression_node_limit_is_enforced() {
    let terms = (0..((MAX_PURE_EXPRESSION_NODES / 2) + 2))
        .map(|_| "1")
        .collect::<Vec<_>>()
        .join(" + ");
    let text = format!("entry main returns {terms};");
    let error = analyze_pure_expression_unit(&source(1, &text)).unwrap_err();
    assert!(matches!(
        error,
        PureExpressionError::TooManyExpressionNodes { .. }
    ));
}
