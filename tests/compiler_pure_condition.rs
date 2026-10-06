use nordoi_kernel::{
    analyze_pure_condition_unit, compile_pure_condition_boundary, lower_pure_condition_unit_to_hir,
    validate_pure_condition_hir, ByteOffset, HirPureConditionEntry, HirPureConditionForm,
    HirPureConditionKind, HirPureConditionUnit, PureConditionCompilerError, SemanticName,
    SemanticPureComparator, SemanticPureCondition, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "condition.noi", text).unwrap()
}

#[test]
fn boolean_literals_have_real_bool_results() {
    let yes = compile_pure_condition_boundary(&source(1, "entry main returns true;")).unwrap();
    let no = compile_pure_condition_boundary(&source(2, "entry main returns false;")).unwrap();
    assert_eq!(yes.result_bool(), Some(true));
    assert_eq!(no.result_bool(), Some(false));
}

#[test]
fn all_comparators_evaluate_deterministically() {
    let cases = [
        ("1 == 1", true),
        ("1 != 1", false),
        ("1 < 2", true),
        ("2 <= 2", true),
        ("3 > 4", false),
        ("4 >= 4", true),
    ];
    for (index, (expr, expected)) in cases.into_iter().enumerate() {
        let text = format!("entry main returns {expr};");
        let unit = compile_pure_condition_boundary(&source(index as u32 + 1, &text)).unwrap();
        assert_eq!(unit.result_bool(), Some(expected), "expr={expr}");
    }
}

#[test]
fn semantic_condition_preserves_comparator_and_operands() {
    let unit = compile_pure_condition_boundary(&source(1, "entry main returns 20 <= 22;")).unwrap();
    assert!(matches!(
        unit.condition(),
        Some(SemanticPureCondition::IntCompare {
            lhs: 20,
            comparator: SemanticPureComparator::Le,
            rhs: 22
        })
    ));
}

#[test]
fn l09_witness_has_explicit_domain() {
    let unit = compile_pure_condition_boundary(&source(1, "entry main returns true;")).unwrap();
    assert!(unit
        .canonical_l09_bytes()
        .starts_with(b"NORDOI-L0.9-PURE-CONDITION\0"));
}

#[test]
fn equal_semantics_ignore_comments_spacing_and_source_id() {
    let a = compile_pure_condition_boundary(&source(
        1,
        "module demo; effect Net; entry main returns 20<=22;",
    ))
    .unwrap();
    let b = compile_pure_condition_boundary(&source(
        99,
        "module /*x*/ demo ; effect Net ; entry /*y*/ main returns 20 <= 22 ;",
    ))
    .unwrap();
    assert_eq!(a.canonical_l09_bytes(), b.canonical_l09_bytes());
}

#[test]
fn different_condition_forms_with_same_truth_keep_distinct_identity() {
    let literal = compile_pure_condition_boundary(&source(1, "entry main returns true;")).unwrap();
    let compare = compile_pure_condition_boundary(&source(2, "entry main returns 1 < 2;")).unwrap();
    assert_eq!(literal.result_bool(), compare.result_bool());
    assert_ne!(literal.canonical_l09_bytes(), compare.canonical_l09_bytes());
}

#[test]
fn different_comparators_keep_distinct_identity_even_if_truth_matches() {
    let lt = compile_pure_condition_boundary(&source(1, "entry main returns 1 < 2;")).unwrap();
    let ne = compile_pure_condition_boundary(&source(2, "entry main returns 1 != 2;")).unwrap();
    assert_eq!(lt.result_bool(), Some(true));
    assert_eq!(ne.result_bool(), Some(true));
    assert_ne!(lt.canonical_l09_bytes(), ne.canonical_l09_bytes());
}

#[test]
fn declared_effect_does_not_become_required_or_authority() {
    let unit =
        compile_pure_condition_boundary(&source(1, "effect Network; entry main returns 1 < 2;"))
            .unwrap();
    let entry = unit.entry().unwrap();
    assert!(entry.required_effects().is_pure());
    assert!(unit.is_pure());
}

#[test]
fn empty_and_plain_entry_remain_distinct_from_false() {
    let empty = compile_pure_condition_boundary(&source(1, "module demo;")).unwrap();
    let none = compile_pure_condition_boundary(&source(2, "entry main;")).unwrap();
    let no = compile_pure_condition_boundary(&source(3, "entry main returns false;")).unwrap();
    assert_eq!(empty.result_bool(), None);
    assert_eq!(none.result_bool(), None);
    assert_eq!(no.result_bool(), Some(false));
    assert_ne!(none.canonical_l09_bytes(), no.canonical_l09_bytes());
}

#[test]
fn module_and_entry_name_are_committed_to_l09_witness() {
    let a =
        compile_pure_condition_boundary(&source(1, "module a; entry main returns true;")).unwrap();
    let b =
        compile_pure_condition_boundary(&source(2, "module b; entry main returns true;")).unwrap();
    let c =
        compile_pure_condition_boundary(&source(3, "module a; entry other returns true;")).unwrap();
    assert_ne!(a.canonical_l09_bytes(), b.canonical_l09_bytes());
    assert_ne!(a.canonical_l09_bytes(), c.canonical_l09_bytes());
}

#[test]
fn older_expression_boundary_remains_frozen_for_boolean_literal() {
    let err =
        nordoi_kernel::compile_pure_expression_boundary(&source(1, "entry main returns true;"))
            .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::CompilerError::PureExpressionFrontend(_)
    ));
}

#[test]
fn older_binding_boundary_remains_frozen_for_comparison() {
    let err = nordoi_kernel::compile_pure_binding_boundary(&source(1, "entry main returns 1 < 2;"))
        .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::CompilerError::PureBindingFrontend(_)
    ));
}

#[test]
fn manual_negative_integer_is_rejected_fail_closed() {
    let src = source(1, "entry main returns 1 < 2;");
    let parsed = analyze_pure_condition_unit(&src).unwrap();
    let lowered = lower_pure_condition_unit_to_hir(&src, &parsed).unwrap();
    let entry = match lowered.form() {
        HirPureConditionForm::Entry(entry) => entry,
        _ => panic!("expected entry"),
    };
    let condition_span = entry.condition().unwrap().origin_span();
    let manual = HirPureConditionUnit::new(
        lowered.semantic().clone(),
        lowered.body_span(),
        HirPureConditionForm::Entry(HirPureConditionEntry::new(
            SemanticName::new("main").unwrap(),
            entry.origin_span(),
            Some(HirPureConditionKind::IntCompare {
                lhs: -1,
                lhs_span: condition_span,
                comparator: SemanticPureComparator::Lt,
                comparator_span: condition_span,
                rhs: 2,
                rhs_span: condition_span,
                origin_span: condition_span,
            }),
        )),
    );
    assert!(matches!(
        validate_pure_condition_hir(manual).unwrap_err(),
        PureConditionCompilerError::NegativeIntegerLiteral { .. }
    ));
}

#[test]
fn condition_outside_entry_is_rejected_fail_closed() {
    let src = source(1, "module demo; entry main returns true;");
    let parsed = analyze_pure_condition_unit(&src).unwrap();
    let lowered = lower_pure_condition_unit_to_hir(&src, &parsed).unwrap();
    let entry = match lowered.form() {
        HirPureConditionForm::Entry(entry) => entry,
        _ => panic!("expected entry"),
    };
    let outside = src.span(ByteOffset::new(0), ByteOffset::new(6)).unwrap();
    let manual = HirPureConditionUnit::new(
        lowered.semantic().clone(),
        lowered.body_span(),
        HirPureConditionForm::Entry(HirPureConditionEntry::new(
            SemanticName::new("main").unwrap(),
            entry.origin_span(),
            Some(HirPureConditionKind::Bool {
                value: true,
                origin_span: outside,
            }),
        )),
    );
    assert!(matches!(
        validate_pure_condition_hir(manual).unwrap_err(),
        PureConditionCompilerError::ConditionSpanOutsideEntry { .. }
    ));
}

#[test]
fn condition_is_semantic_only_and_does_not_touch_runtime_api() {
    let unit = compile_pure_condition_boundary(&source(1, "entry main returns 5 >= 5;")).unwrap();
    assert_eq!(unit.result_bool(), Some(true));
    assert!(unit.is_pure());
}

#[test]
fn bool_value_is_already_a_native_value_kind_without_runtime_use() {
    assert_eq!(
        nordoi_kernel::Value::Bool(true),
        nordoi_kernel::Value::Bool(true)
    );
}
