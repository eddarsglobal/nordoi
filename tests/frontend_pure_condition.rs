use nordoi_kernel::{
    analyze_pure_condition_unit, PureConditionBodyForm, PureConditionError, SourceId, SourceText,
    SurfacePureComparator, SurfacePureConditionKind,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "condition.noi", text).unwrap()
}

fn condition(text: &str) -> SurfacePureConditionKind {
    let unit = analyze_pure_condition_unit(&source(1, text)).unwrap();
    let PureConditionBodyForm::EntryCondition(entry) = unit.form() else {
        panic!("expected condition entry");
    };
    entry.condition().kind().clone()
}

#[test]
fn empty_body_and_plain_entry_remain_supported() {
    assert!(matches!(
        analyze_pure_condition_unit(&source(1, "module demo;"))
            .unwrap()
            .form(),
        PureConditionBodyForm::Empty
    ));
    assert!(matches!(
        analyze_pure_condition_unit(&source(2, "entry main;"))
            .unwrap()
            .form(),
        PureConditionBodyForm::Entry(_)
    ));
}

#[test]
fn true_and_false_are_contextual_boolean_literals() {
    assert!(matches!(
        condition("entry main returns true;"),
        SurfacePureConditionKind::Bool { value: true, .. }
    ));
    assert!(matches!(
        condition("entry main returns false;"),
        SurfacePureConditionKind::Bool { value: false, .. }
    ));
}

#[test]
fn all_six_integer_comparators_are_recognized() {
    let cases = [
        ("1==1", SurfacePureComparator::Eq),
        ("1!=2", SurfacePureComparator::Ne),
        ("1<2", SurfacePureComparator::Lt),
        ("1<=2", SurfacePureComparator::Le),
        ("2>1", SurfacePureComparator::Gt),
        ("2>=1", SurfacePureComparator::Ge),
    ];
    for (expr, expected) in cases {
        let text = format!("entry main returns {expr};");
        let SurfacePureConditionKind::IntCompare { comparator, .. } = condition(&text) else {
            panic!("expected integer comparison: {expr}");
        };
        assert_eq!(comparator, expected, "expr={expr}");
    }
}

#[test]
fn whitespace_around_comparator_is_allowed() {
    let SurfacePureConditionKind::IntCompare { lhs, rhs, .. } =
        condition("entry main returns 20 <= 22;")
    else {
        panic!("expected comparison");
    };
    assert_eq!((lhs, rhs), (20, 22));
}

#[test]
fn trivia_inside_two_character_comparator_is_rejected() {
    let error = analyze_pure_condition_unit(&source(
        1,
        "entry main returns 20 < /*not canonical*/ = 22;",
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        PureConditionError::InvalidIntegerLiteral { .. }
            | PureConditionError::InvalidComparator { .. }
    ));
}

#[test]
fn leading_zero_is_rejected() {
    let error = analyze_pure_condition_unit(&source(1, "entry main returns 01 < 2;")).unwrap_err();
    assert!(matches!(
        error,
        PureConditionError::InvalidIntegerLiteral { .. }
    ));
}

#[test]
fn oversized_integer_is_rejected() {
    let error =
        analyze_pure_condition_unit(&source(1, "entry main returns 9223372036854775808 < 2;"))
            .unwrap_err();
    assert!(matches!(
        error,
        PureConditionError::IntegerLiteralOutOfRange { .. }
    ));
}

#[test]
fn negative_integer_is_not_in_l09() {
    let error = analyze_pure_condition_unit(&source(1, "entry main returns -1 < 0;")).unwrap_err();
    assert!(matches!(
        error,
        PureConditionError::InvalidIntegerLiteral { .. }
            | PureConditionError::InvalidComparator { .. }
    ));
}

#[test]
fn lone_equal_and_bang_are_rejected() {
    for expr in ["1 = 1", "1 ! 1"] {
        let text = format!("entry main returns {expr};");
        let error = analyze_pure_condition_unit(&source(1, &text)).unwrap_err();
        assert!(matches!(
            error,
            PureConditionError::InvalidComparator { .. }
        ));
    }
}

#[test]
fn arithmetic_is_not_silently_accepted_as_condition() {
    let error = analyze_pure_condition_unit(&source(1, "entry main returns 1 + 2;")).unwrap_err();
    assert!(matches!(
        error,
        PureConditionError::InvalidComparator { .. }
            | PureConditionError::InvalidIntegerLiteral { .. }
    ));
}

#[test]
fn identifiers_other_than_boolean_literals_are_rejected() {
    let error = analyze_pure_condition_unit(&source(1, "entry main returns maybe;")).unwrap_err();
    assert!(matches!(
        error,
        PureConditionError::InvalidBooleanOrComparison { .. }
    ));
}

#[test]
fn extra_comparison_chain_is_rejected() {
    let error =
        analyze_pure_condition_unit(&source(1, "entry main returns 1 < 2 < 3;")).unwrap_err();
    assert!(matches!(
        error,
        PureConditionError::UnexpectedAfterCondition { .. }
    ));
}

#[test]
fn missing_right_operand_is_rejected() {
    let error = analyze_pure_condition_unit(&source(1, "entry main returns 1 <;")).unwrap_err();
    assert!(matches!(
        error,
        PureConditionError::MissingRightOperand { .. }
    ));
}

#[test]
fn missing_condition_and_missing_terminator_fail_closed() {
    assert!(matches!(
        analyze_pure_condition_unit(&source(1, "entry main returns ;")).unwrap_err(),
        PureConditionError::ExpectedCondition { .. }
    ));
    assert!(matches!(
        analyze_pure_condition_unit(&source(2, "entry main returns true")).unwrap_err(),
        PureConditionError::ExpectedConditionTerminator { .. }
    ));
}
