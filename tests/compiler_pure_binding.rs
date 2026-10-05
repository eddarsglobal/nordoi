use nordoi_kernel::{
    compile_pure_binding_boundary, compile_pure_expression_boundary, CompilerError,
    NsirPureBindingForm, SemanticPureBindingExpressionOp, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("binding-{id}.noi"), text).unwrap()
}

#[test]
fn binding_references_resolve_and_evaluate() {
    let unit = compile_pure_binding_boundary(&source(
        1,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
    assert_eq!(unit.result_i64(), Some(42));
    assert_eq!(unit.bindings().bindings().len(), 2);
    assert_eq!(unit.bindings().bindings()[0].id().get(), 1);
    assert_eq!(unit.bindings().bindings()[0].name().as_str(), "x");
    assert_eq!(unit.bindings().bindings()[1].id().get(), 2);
    assert_eq!(unit.bindings().bindings()[1].name().as_str(), "y");
    assert_eq!(
        unit.expression().unwrap().ops(),
        &[
            SemanticPureBindingExpressionOp::Binding(unit.bindings().bindings()[0].id()),
            SemanticPureBindingExpressionOp::Binding(unit.bindings().bindings()[1].id()),
            SemanticPureBindingExpressionOp::Add,
        ]
    );
}

#[test]
fn registry_ids_are_canonical_by_name_not_declaration_order() {
    let unit = compile_pure_binding_boundary(&source(
        1,
        "const z = 3; const a = 1; const m = 2; entry main returns a + m + z;",
    ))
    .unwrap();
    let names: Vec<_> = unit
        .bindings()
        .bindings()
        .iter()
        .map(|binding| (binding.id().get(), binding.name().as_str()))
        .collect();
    assert_eq!(names, vec![(1, "a"), (2, "m"), (3, "z")]);
}

#[test]
fn declaration_order_does_not_change_l08_witness() {
    let a = compile_pure_binding_boundary(&source(
        1,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
    let b = compile_pure_binding_boundary(&source(
        2,
        "const y = 22; const x = 20; entry main returns x + y;",
    ))
    .unwrap();
    assert_eq!(a.canonical_l08_bytes(), b.canonical_l08_bytes());
}

#[test]
fn comments_spacing_and_source_id_do_not_change_l08_witness() {
    let a = compile_pure_binding_boundary(&source(
        1,
        "module demo; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let b = compile_pure_binding_boundary(&source(
        99,
        "module /*m*/ demo ; const /*c*/ x=20 ; entry main returns (x) + 22 ;",
    ))
    .unwrap();
    assert_eq!(a.canonical_l08_bytes(), b.canonical_l08_bytes());
}

#[test]
fn binding_name_is_semantic_even_when_values_match() {
    let x = compile_pure_binding_boundary(&source(1, "const x = 20; entry main returns x + 22;"))
        .unwrap();
    let y = compile_pure_binding_boundary(&source(2, "const y = 20; entry main returns y + 22;"))
        .unwrap();
    assert_eq!(x.result_i64(), y.result_i64());
    assert_ne!(x.canonical_l08_bytes(), y.canonical_l08_bytes());
}

#[test]
fn duplicate_binding_is_rejected() {
    let error = compile_pure_binding_boundary(&source(
        1,
        "const x = 1; const x = 2; entry main returns x;",
    ))
    .unwrap_err();
    assert!(matches!(error, CompilerError::DuplicatePureBinding { .. }));
}

#[test]
fn unknown_binding_is_rejected() {
    let error =
        compile_pure_binding_boundary(&source(1, "entry main returns missing + 1;")).unwrap_err();
    assert!(matches!(error, CompilerError::UnknownPureBinding { .. }));
}

#[test]
fn binding_value_participates_in_checked_overflow() {
    let error = compile_pure_binding_boundary(&source(
        1,
        "const max = 9223372036854775807; entry main returns max + 1;",
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        CompilerError::PureBindingIntegerOverflow { .. }
    ));
}

#[test]
fn parenthesized_binding_references_preserve_postfix_structure() {
    let left = compile_pure_binding_boundary(&source(
        1,
        "const x = 1; const y = 2; const z = 3; entry main returns (x + y) + z;",
    ))
    .unwrap();
    let right = compile_pure_binding_boundary(&source(
        2,
        "const x = 1; const y = 2; const z = 3; entry main returns x + (y + z);",
    ))
    .unwrap();
    assert_eq!(left.result_i64(), Some(6));
    assert_eq!(right.result_i64(), Some(6));
    assert_ne!(
        left.expression().unwrap().ops(),
        right.expression().unwrap().ops()
    );
    assert_ne!(left.canonical_l08_bytes(), right.canonical_l08_bytes());
}

#[test]
fn zero_binding_result_is_not_no_result() {
    let none = compile_pure_binding_boundary(&source(1, "const z = 0; entry main;")).unwrap();
    let zero =
        compile_pure_binding_boundary(&source(2, "const z = 0; entry main returns z;")).unwrap();
    assert_eq!(none.result_i64(), None);
    assert_eq!(zero.result_i64(), Some(0));
    assert_ne!(none.canonical_l08_bytes(), zero.canonical_l08_bytes());
}

#[test]
fn declared_effect_remains_unused_and_grants_no_semantic_requirement() {
    let unit = compile_pure_binding_boundary(&source(
        1,
        "effect Network; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let NsirPureBindingForm::Entry(entry) = unit.form() else {
        panic!("expected entry");
    };
    assert!(entry.required_effects().is_pure());
    assert!(unit.is_pure());
}

#[test]
fn l08_witness_has_explicit_domain() {
    let unit =
        compile_pure_binding_boundary(&source(1, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    assert!(unit
        .canonical_l08_bytes()
        .starts_with(b"NORDOI-L0.8-PURE-BINDINGS\0"));
}

#[test]
fn old_l07_boundary_remains_frozen_and_rejects_const_prelude() {
    let error =
        compile_pure_expression_boundary(&source(1, "const x = 20; entry main returns x + 22;"))
            .unwrap_err();
    assert!(matches!(error, CompilerError::PureExpressionFrontend(_)));
}

#[test]
fn l08_without_bindings_accepts_existing_l07_expression_semantics_additively() {
    let unit = compile_pure_binding_boundary(&source(1, "entry main returns 20 + 22;")).unwrap();
    assert_eq!(unit.result_i64(), Some(42));
    assert!(unit.bindings().bindings().is_empty());
}

#[test]
fn forged_negative_binding_value_is_rejected_before_nsir_publication() {
    use nordoi_kernel::{
        analyze_pure_binding_unit, lower_pure_binding_unit_to_hir, validate_pure_binding_hir,
        HirPureBinding, HirPureBindingUnit,
    };
    let src = source(1, "const x = 1; entry main returns x;");
    let parsed = analyze_pure_binding_unit(&src).unwrap();
    let lowered = lower_pure_binding_unit_to_hir(&src, &parsed).unwrap();
    let original = &lowered.bindings()[0];
    let forged = HirPureBinding::new(
        original.name().clone(),
        original.origin_span(),
        original.value_span(),
        -1,
    );
    let manual = HirPureBindingUnit::new(
        lowered.semantic().clone(),
        lowered.body_span(),
        vec![forged],
        lowered.form().clone(),
    );
    let error = validate_pure_binding_hir(manual).unwrap_err();
    assert!(matches!(
        error,
        CompilerError::NegativePureBindingLiteral { .. }
    ));
}

#[test]
fn forged_cross_source_binding_value_span_is_rejected() {
    use nordoi_kernel::{
        analyze_pure_binding_unit, lower_pure_binding_unit_to_hir, validate_pure_binding_hir,
        HirPureBinding, HirPureBindingUnit,
    };
    let src = source(1, "const x = 1; entry main returns x;");
    let other = source(2, "1");
    let parsed = analyze_pure_binding_unit(&src).unwrap();
    let lowered = lower_pure_binding_unit_to_hir(&src, &parsed).unwrap();
    let original = &lowered.bindings()[0];
    let forged = HirPureBinding::new(
        original.name().clone(),
        original.origin_span(),
        other.full_span(),
        1,
    );
    let manual = HirPureBindingUnit::new(
        lowered.semantic().clone(),
        lowered.body_span(),
        vec![forged],
        lowered.form().clone(),
    );
    let error = validate_pure_binding_hir(manual).unwrap_err();
    assert!(matches!(error, CompilerError::SourceMismatch { .. }));
}

#[test]
fn forged_invalid_binding_postfix_is_rejected() {
    use nordoi_kernel::{
        analyze_pure_binding_unit, lower_pure_binding_unit_to_hir, validate_pure_binding_hir,
        HirPureBindingEntry, HirPureBindingExpression, HirPureBindingExpressionOp,
        HirPureBindingForm, HirPureBindingUnit,
    };
    let src = source(1, "const x = 1; entry main returns x;");
    let parsed = analyze_pure_binding_unit(&src).unwrap();
    let lowered = lower_pure_binding_unit_to_hir(&src, &parsed).unwrap();
    let HirPureBindingForm::Entry(entry) = lowered.form() else {
        panic!("expected entry");
    };
    let expression_span = entry.expression().unwrap().origin_span();
    let manual = HirPureBindingUnit::new(
        lowered.semantic().clone(),
        lowered.body_span(),
        lowered.bindings().to_vec(),
        HirPureBindingForm::Entry(HirPureBindingEntry::new(
            entry.name().clone(),
            entry.origin_span(),
            Some(HirPureBindingExpression::new(
                expression_span,
                vec![HirPureBindingExpressionOp::Add {
                    origin_span: expression_span,
                }],
            )),
        )),
    );
    let error = validate_pure_binding_hir(manual).unwrap_err();
    assert!(matches!(
        error,
        CompilerError::InvalidPureBindingExpressionPostfix { .. }
    ));
}

#[test]
fn type_or_effect_name_does_not_implicitly_resolve_as_value_binding() {
    for text in [
        "type x; entry main returns x;",
        "effect x; entry main returns x;",
    ] {
        let error = compile_pure_binding_boundary(&source(1, text)).unwrap_err();
        assert!(matches!(error, CompilerError::UnknownPureBinding { .. }));
    }
}

#[test]
fn const_is_contextual_not_globally_reserved() {
    let unit = compile_pure_binding_boundary(&source(
        1,
        "const const = 20; entry main returns const + 22;",
    ))
    .unwrap();
    assert_eq!(unit.result_i64(), Some(42));
    assert_eq!(unit.bindings().bindings()[0].name().as_str(), "const");
}
