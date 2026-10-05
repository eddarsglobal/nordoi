use nordoi_kernel::{
    analyze_pure_result_unit, compile_execution_plan_boundary, compile_pure_result_boundary,
    lower_pure_result_unit_to_hir, validate_pure_result_hir, ByteOffset, CompilerError,
    HirPureIntResult, HirPureResultEntry, HirPureResultForm, HirPureResultUnit, NsirPureResultForm,
    SemanticName, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "result.noi", text).unwrap()
}

#[test]
fn pure_integer_result_compiles_to_nsir_l06() {
    let src = source(1, "module demo; type A; effect Net; entry main returns 42;");
    let unit = compile_pure_result_boundary(&src).unwrap();
    match unit.form() {
        NsirPureResultForm::Entry(entry) => {
            assert_eq!(entry.name().as_str(), "main");
            assert!(entry.is_pure());
            assert_eq!(entry.result_i64(), Some(42));
            assert!(entry.required_effects().effects().is_empty());
        }
        _ => panic!("expected entry"),
    }
}

#[test]
fn empty_body_remains_explicit() {
    let unit = compile_pure_result_boundary(&source(1, "module demo; //tail\n")).unwrap();
    assert!(matches!(unit.form(), NsirPureResultForm::Empty));
    assert_eq!(unit.result_i64(), None);
    assert!(unit.is_pure());
}

#[test]
fn l05_entry_without_result_remains_supported_only_as_no_result() {
    let unit = compile_pure_result_boundary(&source(1, "entry main;")).unwrap();
    let entry = unit.entry().unwrap();
    assert_eq!(entry.name().as_str(), "main");
    assert_eq!(entry.result_i64(), None);
}

#[test]
fn declared_effect_does_not_become_result_requirement_or_authority() {
    let unit =
        compile_pure_result_boundary(&source(1, "effect Network; entry main returns 7;")).unwrap();
    let entry = unit.entry().unwrap();
    assert!(entry.required_effects().effects().is_empty());
    assert!(entry.is_pure());
}

#[test]
fn changing_result_changes_l06_witness() {
    let a = compile_pure_result_boundary(&source(1, "entry main returns 1;")).unwrap();
    let b = compile_pure_result_boundary(&source(2, "entry main returns 2;")).unwrap();
    assert_ne!(a.canonical_l06_bytes(), b.canonical_l06_bytes());
}

#[test]
fn equal_semantics_ignore_comments_spacing_and_source_id() {
    let a =
        compile_pure_result_boundary(&source(1, "module demo; entry main returns 42;")).unwrap();
    let b = compile_pure_result_boundary(&source(
        99,
        "module /*x*/ demo ;\n entry /*a*/ main /*b*/ returns /*c*/ 42 /*d*/ ;",
    ))
    .unwrap();
    assert_eq!(a.canonical_l06_bytes(), b.canonical_l06_bytes());
}

#[test]
fn l06_witness_has_explicit_domain() {
    let unit = compile_pure_result_boundary(&source(1, "entry main returns 42;")).unwrap();
    assert!(unit
        .canonical_l06_bytes()
        .starts_with(b"NORDOI-L0.6-PURE-RESULT\0"));
}

#[test]
fn prior_c02_witness_is_preserved() {
    let no_result =
        compile_pure_result_boundary(&source(1, "module demo; type A; entry main;")).unwrap();
    let result =
        compile_pure_result_boundary(&source(2, "module demo; type A; entry main returns 42;"))
            .unwrap();
    assert_eq!(
        no_result.semantic().canonical_c02_bytes(),
        result.semantic().canonical_c02_bytes()
    );
}

#[test]
fn entry_name_changes_l06_witness() {
    let a = compile_pure_result_boundary(&source(1, "entry alpha returns 42;")).unwrap();
    let b = compile_pure_result_boundary(&source(2, "entry beta returns 42;")).unwrap();
    assert_ne!(a.canonical_l06_bytes(), b.canonical_l06_bytes());
}

#[test]
fn module_identity_changes_l06_witness() {
    let a = compile_pure_result_boundary(&source(1, "module a; entry main returns 42;")).unwrap();
    let b = compile_pure_result_boundary(&source(2, "module b; entry main returns 42;")).unwrap();
    assert_ne!(a.canonical_l06_bytes(), b.canonical_l06_bytes());
}

#[test]
fn zero_and_no_result_are_distinct_semantics() {
    let none = compile_pure_result_boundary(&source(1, "entry main;")).unwrap();
    let zero = compile_pure_result_boundary(&source(2, "entry main returns 0;")).unwrap();
    assert_ne!(none.canonical_l06_bytes(), zero.canonical_l06_bytes());
}

#[test]
fn result_source_does_not_silently_enter_c03_plan() {
    let error = compile_execution_plan_boundary(&source(1, "entry main returns 42;")).unwrap_err();
    assert!(matches!(error, CompilerError::BodyFrontend(_)));
}

#[test]
fn manual_entry_outside_body_is_rejected() {
    let src = source(1, "module demo; entry main returns 42;");
    let parsed = analyze_pure_result_unit(&src).unwrap();
    let lowered = lower_pure_result_unit_to_hir(&src, &parsed).unwrap();
    let body = lowered.body_span();
    let outside = src.span(ByteOffset::new(0), ByteOffset::new(6)).unwrap();
    let manual = HirPureResultUnit::new(
        lowered.semantic().clone(),
        body,
        HirPureResultForm::Entry(HirPureResultEntry::new(
            SemanticName::new("main".to_owned()).unwrap(),
            outside,
            None,
        )),
    );
    let error = validate_pure_result_hir(manual).unwrap_err();
    assert!(matches!(
        error,
        CompilerError::PureResultEntrySpanOutsideBody { .. }
    ));
}

#[test]
fn manual_result_span_outside_entry_is_rejected() {
    let src = source(1, "module demo; entry main returns 42;");
    let parsed = analyze_pure_result_unit(&src).unwrap();
    let lowered = lower_pure_result_unit_to_hir(&src, &parsed).unwrap();
    let body = lowered.body_span();
    let entry_span = src
        .span(ByteOffset::new(13), ByteOffset::new(src.len().get()))
        .unwrap();
    let bad_result_span = src.span(ByteOffset::new(0), ByteOffset::new(6)).unwrap();
    let manual = HirPureResultUnit::new(
        lowered.semantic().clone(),
        body,
        HirPureResultForm::Entry(HirPureResultEntry::new(
            SemanticName::new("main".to_owned()).unwrap(),
            entry_span,
            Some(HirPureIntResult::new(42, bad_result_span)),
        )),
    );
    let error = validate_pure_result_hir(manual).unwrap_err();
    assert!(matches!(
        error,
        CompilerError::PureResultValueSpanOutsideEntry { .. }
    ));
}

#[test]
fn cross_source_result_span_is_rejected() {
    let src = source(1, "module demo; entry main returns 42;");
    let other = source(2, "42");
    let parsed = analyze_pure_result_unit(&src).unwrap();
    let lowered = lower_pure_result_unit_to_hir(&src, &parsed).unwrap();
    let entry_span = match lowered.form() {
        HirPureResultForm::Entry(entry) => entry.origin_span(),
        _ => panic!("expected entry"),
    };
    let result_span = other.full_span();
    let manual = HirPureResultUnit::new(
        lowered.semantic().clone(),
        lowered.body_span(),
        HirPureResultForm::Entry(HirPureResultEntry::new(
            SemanticName::new("main".to_owned()).unwrap(),
            entry_span,
            Some(HirPureIntResult::new(42, result_span)),
        )),
    );
    let error = validate_pure_result_hir(manual).unwrap_err();
    assert!(matches!(error, CompilerError::SourceMismatch { .. }));
}

#[test]
fn int_max_is_preserved_exactly() {
    let unit = compile_pure_result_boundary(&source(1, "entry main returns 9223372036854775807;"))
        .unwrap();
    assert_eq!(unit.result_i64(), Some(i64::MAX));
}
