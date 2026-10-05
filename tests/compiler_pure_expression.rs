use nordoi_kernel::{
    analyze_pure_expression_unit, compile_pure_expression_boundary, compile_pure_result_boundary,
    lower_pure_expression_unit_to_hir, validate_pure_expression_hir, ByteOffset, CompilerError,
    HirPureExpression, HirPureExpressionEntry, HirPureExpressionForm, HirPureExpressionOp,
    HirPureExpressionUnit, NsirPureExpressionForm, SemanticName, SemanticPureExpressionOp,
    SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "expr.noi", text).unwrap()
}

#[test]
fn addition_evaluates_to_checked_integer_result() {
    let unit = compile_pure_expression_boundary(&source(1, "entry main returns 20 + 22;")).unwrap();
    assert_eq!(unit.result_i64(), Some(42));
    let expression = unit.expression().unwrap();
    assert_eq!(expression.node_count(), 3);
    assert_eq!(
        expression.ops(),
        &[
            SemanticPureExpressionOp::Int(20),
            SemanticPureExpressionOp::Int(22),
            SemanticPureExpressionOp::Add,
        ]
    );
}

#[test]
fn nested_parentheses_evaluate_correctly() {
    let unit =
        compile_pure_expression_boundary(&source(1, "entry main returns 1 + (2 + (3 + 4));"))
            .unwrap();
    assert_eq!(unit.result_i64(), Some(10));
}

#[test]
fn left_and_right_grouping_keep_distinct_l07_witnesses() {
    let left =
        compile_pure_expression_boundary(&source(1, "entry main returns (1 + 2) + 3;")).unwrap();
    let right =
        compile_pure_expression_boundary(&source(2, "entry main returns 1 + (2 + 3);")).unwrap();
    assert_eq!(left.result_i64(), Some(6));
    assert_eq!(right.result_i64(), Some(6));
    assert_ne!(left.canonical_l07_bytes(), right.canonical_l07_bytes());
}

#[test]
fn equal_expression_semantics_ignore_comments_spacing_and_source_id() {
    let a = compile_pure_expression_boundary(&source(
        1,
        "module demo; entry main returns 1 + (2 + 3);",
    ))
    .unwrap();
    let b = compile_pure_expression_boundary(&source(
        99,
        "module /*x*/ demo ; entry /*a*/ main returns 1/*b*/+(2 /*c*/ + 3);",
    ))
    .unwrap();
    assert_eq!(a.canonical_l07_bytes(), b.canonical_l07_bytes());
}

#[test]
fn expression_witness_has_explicit_domain() {
    let unit = compile_pure_expression_boundary(&source(1, "entry main returns 1 + 2;")).unwrap();
    assert!(unit
        .canonical_l07_bytes()
        .starts_with(b"NORDOI-L0.7-PURE-EXPRESSION\0"));
}

#[test]
fn changing_operand_changes_l07_witness() {
    let a = compile_pure_expression_boundary(&source(1, "entry main returns 1 + 2;")).unwrap();
    let b = compile_pure_expression_boundary(&source(2, "entry main returns 1 + 3;")).unwrap();
    assert_ne!(a.canonical_l07_bytes(), b.canonical_l07_bytes());
}

#[test]
fn entry_name_changes_l07_witness() {
    let a = compile_pure_expression_boundary(&source(1, "entry alpha returns 1 + 2;")).unwrap();
    let b = compile_pure_expression_boundary(&source(2, "entry beta returns 1 + 2;")).unwrap();
    assert_ne!(a.canonical_l07_bytes(), b.canonical_l07_bytes());
}

#[test]
fn module_identity_changes_l07_witness() {
    let a = compile_pure_expression_boundary(&source(1, "module a; entry main returns 1 + 2;"))
        .unwrap();
    let b = compile_pure_expression_boundary(&source(2, "module b; entry main returns 1 + 2;"))
        .unwrap();
    assert_ne!(a.canonical_l07_bytes(), b.canonical_l07_bytes());
}

#[test]
fn declared_effect_is_not_required_and_grants_no_authority() {
    let unit =
        compile_pure_expression_boundary(&source(1, "effect Network; entry main returns 1 + 2;"))
            .unwrap();
    let NsirPureExpressionForm::Entry(entry) = unit.form() else {
        panic!("expected entry");
    };
    assert!(entry.required_effects().is_pure());
    assert!(unit.is_pure());
}

#[test]
fn checked_addition_overflow_fails_closed() {
    let error =
        compile_pure_expression_boundary(&source(1, "entry main returns 9223372036854775807 + 1;"))
            .unwrap_err();
    assert!(matches!(
        error,
        CompilerError::PureExpressionIntegerOverflow { .. }
    ));
}

#[test]
fn max_value_without_addition_is_valid() {
    let unit =
        compile_pure_expression_boundary(&source(1, "entry main returns 9223372036854775807;"))
            .unwrap();
    assert_eq!(unit.result_i64(), Some(i64::MAX));
}

#[test]
fn zero_and_no_result_are_distinct_l07_semantics() {
    let none = compile_pure_expression_boundary(&source(1, "entry main;")).unwrap();
    let zero = compile_pure_expression_boundary(&source(2, "entry main returns 0;")).unwrap();
    assert_ne!(none.canonical_l07_bytes(), zero.canonical_l07_bytes());
}

#[test]
fn literal_l07_result_matches_l06_value_without_replacing_l06_witness() {
    let l06 = compile_pure_result_boundary(&source(1, "entry main returns 42;")).unwrap();
    let l07 = compile_pure_expression_boundary(&source(2, "entry main returns 42;")).unwrap();
    assert_eq!(l06.result_i64(), l07.result_i64());
    assert!(l06
        .canonical_l06_bytes()
        .starts_with(b"NORDOI-L0.6-PURE-RESULT\0"));
    assert!(l07
        .canonical_l07_bytes()
        .starts_with(b"NORDOI-L0.7-PURE-EXPRESSION\0"));
}

#[test]
fn l06_boundary_still_rejects_addition() {
    let error =
        compile_pure_result_boundary(&source(1, "entry main returns 20 + 22;")).unwrap_err();
    assert!(matches!(error, CompilerError::PureResultFrontend(_)));
}

#[test]
fn manual_expression_outside_entry_is_rejected() {
    let src = source(1, "module demo; entry main returns 1 + 2;");
    let parsed = analyze_pure_expression_unit(&src).unwrap();
    let lowered = lower_pure_expression_unit_to_hir(&src, &parsed).unwrap();
    let body = lowered.body_span();
    let entry_span = match lowered.form() {
        HirPureExpressionForm::Entry(entry) => entry.origin_span(),
        _ => panic!("expected entry"),
    };
    let outside = src.span(ByteOffset::new(0), ByteOffset::new(6)).unwrap();
    let manual = HirPureExpressionUnit::new(
        lowered.semantic().clone(),
        body,
        HirPureExpressionForm::Entry(HirPureExpressionEntry::new(
            SemanticName::new("main").unwrap(),
            entry_span,
            Some(HirPureExpression::new(
                outside,
                vec![HirPureExpressionOp::int(1, outside)],
            )),
        )),
    );
    let error = validate_pure_expression_hir(manual).unwrap_err();
    assert!(matches!(
        error,
        CompilerError::PureExpressionSpanOutsideEntry { .. }
    ));
}

#[test]
fn manual_node_outside_expression_is_rejected() {
    let src = source(1, "module demo; entry main returns 1 + 2;");
    let parsed = analyze_pure_expression_unit(&src).unwrap();
    let lowered = lower_pure_expression_unit_to_hir(&src, &parsed).unwrap();
    let body = lowered.body_span();
    let entry_span = match lowered.form() {
        HirPureExpressionForm::Entry(entry) => entry.origin_span(),
        _ => panic!("expected entry"),
    };
    let expression_span = match lowered.form() {
        HirPureExpressionForm::Entry(entry) => entry.expression().unwrap().origin_span(),
        _ => panic!("expected entry"),
    };
    let bad_node = src.span(ByteOffset::new(0), ByteOffset::new(6)).unwrap();
    let manual = HirPureExpressionUnit::new(
        lowered.semantic().clone(),
        body,
        HirPureExpressionForm::Entry(HirPureExpressionEntry::new(
            SemanticName::new("main").unwrap(),
            entry_span,
            Some(HirPureExpression::new(
                expression_span,
                vec![HirPureExpressionOp::int(1, bad_node)],
            )),
        )),
    );
    let error = validate_pure_expression_hir(manual).unwrap_err();
    assert!(matches!(
        error,
        CompilerError::PureExpressionNodeSpanOutsideExpression { .. }
    ));
}

#[test]
fn manual_negative_literal_is_rejected() {
    let src = source(1, "entry main returns 1;");
    let parsed = analyze_pure_expression_unit(&src).unwrap();
    let lowered = lower_pure_expression_unit_to_hir(&src, &parsed).unwrap();
    let body = lowered.body_span();
    let entry_span = match lowered.form() {
        HirPureExpressionForm::Entry(entry) => entry.origin_span(),
        _ => panic!("expected entry"),
    };
    let expression_span = match lowered.form() {
        HirPureExpressionForm::Entry(entry) => entry.expression().unwrap().origin_span(),
        _ => panic!("expected entry"),
    };
    let manual = HirPureExpressionUnit::new(
        lowered.semantic().clone(),
        body,
        HirPureExpressionForm::Entry(HirPureExpressionEntry::new(
            SemanticName::new("main").unwrap(),
            entry_span,
            Some(HirPureExpression::new(
                expression_span,
                vec![HirPureExpressionOp::int(-1, expression_span)],
            )),
        )),
    );
    let error = validate_pure_expression_hir(manual).unwrap_err();
    assert!(matches!(
        error,
        CompilerError::NegativePureExpressionLiteral { .. }
    ));
}

#[test]
fn manual_invalid_postfix_is_rejected() {
    let src = source(1, "entry main returns 1;");
    let parsed = analyze_pure_expression_unit(&src).unwrap();
    let lowered = lower_pure_expression_unit_to_hir(&src, &parsed).unwrap();
    let body = lowered.body_span();
    let entry_span = match lowered.form() {
        HirPureExpressionForm::Entry(entry) => entry.origin_span(),
        _ => panic!("expected entry"),
    };
    let expression_span = match lowered.form() {
        HirPureExpressionForm::Entry(entry) => entry.expression().unwrap().origin_span(),
        _ => panic!("expected entry"),
    };
    let manual = HirPureExpressionUnit::new(
        lowered.semantic().clone(),
        body,
        HirPureExpressionForm::Entry(HirPureExpressionEntry::new(
            SemanticName::new("main").unwrap(),
            entry_span,
            Some(HirPureExpression::new(
                expression_span,
                vec![HirPureExpressionOp::add(expression_span)],
            )),
        )),
    );
    let error = validate_pure_expression_hir(manual).unwrap_err();
    assert!(matches!(
        error,
        CompilerError::InvalidPureExpressionPostfix { .. }
    ));
}

#[test]
fn cross_source_expression_span_is_rejected() {
    let src = source(1, "entry main returns 1;");
    let other = source(2, "1");
    let parsed = analyze_pure_expression_unit(&src).unwrap();
    let lowered = lower_pure_expression_unit_to_hir(&src, &parsed).unwrap();
    let body = lowered.body_span();
    let entry_span = match lowered.form() {
        HirPureExpressionForm::Entry(entry) => entry.origin_span(),
        _ => panic!("expected entry"),
    };
    let other_span = other.full_span();
    let manual = HirPureExpressionUnit::new(
        lowered.semantic().clone(),
        body,
        HirPureExpressionForm::Entry(HirPureExpressionEntry::new(
            SemanticName::new("main").unwrap(),
            entry_span,
            Some(HirPureExpression::new(
                other_span,
                vec![HirPureExpressionOp::int(1, other_span)],
            )),
        )),
    );
    let error = validate_pure_expression_hir(manual).unwrap_err();
    assert!(matches!(error, CompilerError::SourceMismatch { .. }));
}
