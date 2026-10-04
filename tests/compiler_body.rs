use nordoi_kernel::{
    analyze_minimal_body_unit, analyze_type_effect_unit, compile_minimal_body_boundary,
    compile_resolved_semantic_boundary, lower_minimal_body_unit_to_hir,
    lower_type_effect_unit_to_hir, validate_body_hir, CompilerError, HirBodyUnit, HirEntryPoint,
    HirMinimalBody, NsirBodyState, NsirMinimalBody, SemanticName, SourceId, SourceText,
};

fn source(id: u32, text: impl Into<String>) -> SourceText {
    SourceText::new(SourceId::new(id), "compiler-body.noi", text).expect("source must be valid")
}

#[test]
fn empty_body_compiles_as_fully_understood_empty_form() {
    let unit = compile_minimal_body_boundary(&source(1, "module demo; type A;")).unwrap();
    assert!(unit.is_empty());
    assert!(matches!(unit.body(), NsirMinimalBody::Empty));
}

#[test]
fn entry_compiles_to_pure_zero_work_nsir_form() {
    let unit = compile_minimal_body_boundary(&source(2, "entry main;")).unwrap();
    let entry = unit.entry().expect("entry must exist");
    assert_eq!(entry.name().as_str(), "main");
    assert!(entry.is_pure());
    assert!(entry.required_effects().effects().is_empty());
}

#[test]
fn l05_preserves_c02_registry_and_witness() {
    let src = source(3, "type Z; type A; effect Net; entry main;");
    let c02 = compile_resolved_semantic_boundary(&src).unwrap();
    let l05 = compile_minimal_body_boundary(&src).unwrap();
    assert_eq!(
        c02.canonical_c02_bytes(),
        l05.semantic().canonical_c02_bytes()
    );
    assert_eq!(
        l05.semantic().registry().resolve_type("A").unwrap().get(),
        1
    );
    assert_eq!(
        l05.semantic().registry().resolve_type("Z").unwrap().get(),
        2
    );
}

#[test]
fn c02_boundary_remains_explicitly_unlowered_for_l05_source() {
    let unit = compile_resolved_semantic_boundary(&source(4, "entry main;")).unwrap();
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
}

#[test]
fn l05_witness_is_stable_across_spacing_comments_and_source_id() {
    let first =
        compile_minimal_body_boundary(&source(5, "module demo; type A; effect Net; entry main;"))
            .unwrap();
    let second = compile_minimal_body_boundary(&source(
        999,
        "module /*m*/ demo /*x*/ ;\n type A ; effect Net ;\n entry /*e*/ main ; //tail\n",
    ))
    .unwrap();
    assert_eq!(first.canonical_l05_bytes(), second.canonical_l05_bytes());
}

#[test]
fn changing_entry_name_changes_l05_witness() {
    let first = compile_minimal_body_boundary(&source(6, "entry main;")).unwrap();
    let second = compile_minimal_body_boundary(&source(7, "entry other;")).unwrap();
    assert_ne!(first.canonical_l05_bytes(), second.canonical_l05_bytes());
}

#[test]
fn empty_and_entry_bodies_have_different_l05_witnesses() {
    let empty = compile_minimal_body_boundary(&source(8, "")).unwrap();
    let entry = compile_minimal_body_boundary(&source(9, "entry main;")).unwrap();
    assert_ne!(empty.canonical_l05_bytes(), entry.canonical_l05_bytes());
}

#[test]
fn l05_witness_is_distinct_from_c02_witness() {
    let unit = compile_minimal_body_boundary(&source(10, "entry main;")).unwrap();
    assert_ne!(
        unit.canonical_l05_bytes(),
        unit.semantic().canonical_c02_bytes()
    );
}

#[test]
fn declared_effects_do_not_become_entry_requirements() {
    let unit = compile_minimal_body_boundary(&source(11, "effect Network; entry main;")).unwrap();
    assert_eq!(unit.semantic().registry().effects().len(), 1);
    assert!(unit.entry().unwrap().required_effects().is_pure());
}

#[test]
fn entry_name_can_share_spelling_with_type_and_effect_namespaces() {
    let unit = compile_minimal_body_boundary(&source(12, "type State; effect State; entry State;"))
        .unwrap();
    assert_eq!(unit.entry().unwrap().name().as_str(), "State");
    assert!(unit.semantic().registry().resolve_type("State").is_some());
    assert!(unit.semantic().registry().resolve_effect("State").is_some());
}

#[test]
fn anonymous_module_can_publish_a_minimal_entry() {
    let unit = compile_minimal_body_boundary(&source(13, "entry main;")).unwrap();
    assert!(unit.semantic().module().canonical_text().is_none());
}

#[test]
fn body_lowerer_retains_entry_diagnostic_span() {
    let src = source(14, "type A;\nentry main;\n");
    let surface = analyze_minimal_body_unit(&src).unwrap();
    let hir = lower_minimal_body_unit_to_hir(&src, &surface).unwrap();
    assert_eq!(
        src.slice(hir.entry().unwrap().origin_span()).unwrap(),
        "entry main;"
    );
}

#[test]
fn manual_body_span_mismatch_is_rejected() {
    let src = source(15, "entry main;");
    let type_effect = analyze_type_effect_unit(&src).unwrap();
    let semantic = lower_type_effect_unit_to_hir(&src, &type_effect).unwrap();
    let wrong = src
        .span(src.full_span().start(), src.full_span().start())
        .unwrap();
    let hir = HirBodyUnit::new(semantic, wrong, HirMinimalBody::Empty);
    assert!(matches!(
        validate_body_hir(hir),
        Err(CompilerError::BodyLayerSpanMismatch { .. })
    ));
}

#[test]
fn manual_entry_outside_body_is_rejected() {
    let src = source(16, "type A; entry main;");
    let type_effect = analyze_type_effect_unit(&src).unwrap();
    let semantic = lower_type_effect_unit_to_hir(&src, &type_effect).unwrap();
    let prelude_span = type_effect.declarations()[0].span();
    let entry = HirEntryPoint::new(SemanticName::new("main").unwrap(), prelude_span);
    let body_span = type_effect.body_span();
    let hir = HirBodyUnit::new(semantic, body_span, HirMinimalBody::Entry(entry));
    assert!(matches!(
        validate_body_hir(hir),
        Err(CompilerError::EntrySpanOutsideBody { .. })
    ));
}

#[test]
fn cross_source_entry_span_is_rejected() {
    let first = source(17, "entry main;");
    let second = source(18, "entry other;");
    let type_effect = analyze_type_effect_unit(&first).unwrap();
    let semantic = lower_type_effect_unit_to_hir(&first, &type_effect).unwrap();
    let entry = HirEntryPoint::new(SemanticName::new("main").unwrap(), second.full_span());
    let hir = HirBodyUnit::new(
        semantic,
        type_effect.body_span(),
        HirMinimalBody::Entry(entry),
    );
    assert!(matches!(
        validate_body_hir(hir),
        Err(CompilerError::SourceMismatch { .. })
    ));
}

#[test]
fn unsupported_future_body_syntax_fails_closed_at_l05_boundary() {
    assert!(matches!(
        compile_minimal_body_boundary(&source(19, "future + syntax")),
        Err(CompilerError::BodyFrontend(_))
    ));
}

#[test]
fn entry_does_not_modify_prior_semantic_witnesses() {
    let src = source(20, "module demo; type A; effect Net; entry main;");
    let c02 = compile_resolved_semantic_boundary(&src).unwrap();
    let l05 = compile_minimal_body_boundary(&src).unwrap();
    assert_eq!(
        c02.canonical_identity_bytes(),
        l05.semantic().canonical_identity_bytes()
    );
    assert_eq!(
        c02.canonical_semantic_bytes(),
        l05.semantic().canonical_semantic_bytes()
    );
    assert_eq!(
        c02.canonical_c02_bytes(),
        l05.semantic().canonical_c02_bytes()
    );
}

#[test]
fn lowered_body_form_is_entry_not_unlowered_claim() {
    let unit = compile_minimal_body_boundary(&source(21, "entry main;")).unwrap();
    assert!(matches!(unit.body(), NsirMinimalBody::Entry(_)));
    assert_eq!(unit.semantic().body_state(), NsirBodyState::Unlowered);
}

#[test]
fn equal_sources_produce_equal_l05_witnesses() {
    let src = source(22, "module demo; entry main;");
    assert_eq!(
        compile_minimal_body_boundary(&src)
            .unwrap()
            .canonical_l05_bytes(),
        compile_minimal_body_boundary(&src)
            .unwrap()
            .canonical_l05_bytes()
    );
}
