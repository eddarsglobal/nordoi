use nordoi_kernel::{
    analyze_pure_binding_unit, PureBindingBodyForm, PureBindingError, SourceId, SourceText,
    SurfacePureBindingExpressionOp, MAX_PURE_BINDINGS,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "bindings.noi", text).unwrap()
}

#[test]
fn two_bindings_and_entry_are_parsed() {
    let src = source(1, "const x = 20; const y = 22; entry main returns x + y;");
    let unit = analyze_pure_binding_unit(&src).unwrap();
    assert_eq!(unit.bindings().len(), 2);
    assert_eq!(unit.bindings()[0].name().text(&src).unwrap(), "x");
    assert_eq!(unit.bindings()[0].value(), 20);
    assert_eq!(unit.bindings()[1].name().text(&src).unwrap(), "y");
    assert_eq!(unit.bindings()[1].value(), 22);
    let PureBindingBodyForm::EntryExpression(entry) = unit.form() else {
        panic!("expected expression entry");
    };
    assert_eq!(entry.expression().node_count(), 3);
    assert!(matches!(
        entry.expression().ops().first(),
        Some(SurfacePureBindingExpressionOp::BindingRef { .. })
    ));
    assert!(matches!(
        entry.expression().ops().get(1),
        Some(SurfacePureBindingExpressionOp::BindingRef { .. })
    ));
    assert!(matches!(
        entry.expression().ops().get(2),
        Some(SurfacePureBindingExpressionOp::Add { .. })
    ));
}

#[test]
fn binding_prelude_can_exist_without_entry() {
    let unit = analyze_pure_binding_unit(&source(1, "const answer = 42;")).unwrap();
    assert_eq!(unit.bindings().len(), 1);
    assert!(matches!(unit.form(), PureBindingBodyForm::Empty));
}

#[test]
fn comments_and_spacing_are_trivia() {
    let src = source(
        1,
        "const /*a*/ x /*b*/ = /*c*/ 20 /*d*/; // e\n entry main returns x + 22;",
    );
    let unit = analyze_pure_binding_unit(&src).unwrap();
    assert_eq!(unit.bindings().len(), 1);
    let PureBindingBodyForm::EntryExpression(entry) = unit.form() else {
        panic!("expected expression entry");
    };
    assert_eq!(entry.expression().node_count(), 3);
}

#[test]
fn binding_reference_may_be_parenthesized() {
    let unit =
        analyze_pure_binding_unit(&source(1, "const x = 20; entry main returns 1 + (x + 21);"))
            .unwrap();
    let PureBindingBodyForm::EntryExpression(entry) = unit.form() else {
        panic!("expected expression entry");
    };
    assert_eq!(entry.expression().node_count(), 5);
}

#[test]
fn binding_initializer_must_be_literal_not_expression() {
    let error = analyze_pure_binding_unit(&source(1, "const x = 20 + 22; entry main returns x;"))
        .unwrap_err();
    assert!(matches!(
        error,
        PureBindingError::ExpectedBindingTerminator { .. }
    ));
}

#[test]
fn binding_initializer_leading_zero_is_rejected() {
    let error =
        analyze_pure_binding_unit(&source(1, "const x = 01; entry main returns x;")).unwrap_err();
    assert!(matches!(
        error,
        PureBindingError::InvalidBindingIntegerLiteral { .. }
    ));
}

#[test]
fn binding_initializer_out_of_range_is_rejected() {
    let error = analyze_pure_binding_unit(&source(
        1,
        "const x = 9223372036854775808; entry main returns x;",
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        PureBindingError::BindingIntegerLiteralOutOfRange { .. }
    ));
}

#[test]
fn declaration_after_entry_is_rejected() {
    let error =
        analyze_pure_binding_unit(&source(1, "entry main returns 1; const x = 2;")).unwrap_err();
    assert!(matches!(
        error,
        PureBindingError::UnexpectedAfterEntry { .. }
    ));
}

#[test]
fn unsupported_operator_remains_rejected() {
    let error = analyze_pure_binding_unit(&source(1, "const x = 6; entry main returns x * 7;"))
        .unwrap_err();
    assert!(matches!(error, PureBindingError::ExpectedPlusOrEnd { .. }));
}

#[test]
fn arbitrary_identifier_is_surface_reference_until_semantic_resolution() {
    let unit = analyze_pure_binding_unit(&source(1, "entry main returns unknown + 1;")).unwrap();
    let PureBindingBodyForm::EntryExpression(entry) = unit.form() else {
        panic!("expected expression entry");
    };
    assert!(matches!(
        entry.expression().ops().first(),
        Some(SurfacePureBindingExpressionOp::BindingRef { .. })
    ));
}

#[test]
fn exact_binding_limit_is_accepted() {
    let mut text = String::new();
    for index in 0..MAX_PURE_BINDINGS {
        text.push_str(&format!("const b{index} = {index};\n"));
    }
    text.push_str("entry main returns b0;");
    let unit = analyze_pure_binding_unit(&source(1, &text)).unwrap();
    assert_eq!(unit.bindings().len(), MAX_PURE_BINDINGS as usize);
}

#[test]
fn binding_limit_plus_one_is_rejected() {
    let mut text = String::new();
    for index in 0..=MAX_PURE_BINDINGS {
        text.push_str(&format!("const b{index} = {index};\n"));
    }
    let error = analyze_pure_binding_unit(&source(1, &text)).unwrap_err();
    assert!(matches!(error, PureBindingError::TooManyBindings { .. }));
}
