use nordoi_kernel::{
    analyze_module_unit, analyze_type_effect_unit, compile_semantic_boundary,
    compile_type_effect_boundary, lower_type_effect_unit_to_hir, validate_hir, ByteOffset,
    CompilerError, HirBodyState, HirDeclaration, HirUnit, NsirBodyState, SemanticDeclarationKind,
    SemanticEffectSet, SemanticModuleIdentity, SemanticName, SourceId, SourceText,
    MAX_SEMANTIC_DECLARATIONS, MAX_SEMANTIC_EFFECT_REQUIREMENTS,
};

fn source(id: u32, text: impl Into<String>) -> SourceText {
    SourceText::new(SourceId::new(id), "compiler-types-effects.noi", text)
        .expect("source must be valid")
}

#[test]
fn l04_compiles_opaque_types_and_effect_identities() {
    let source = source(1, "module demo;\ntype UserId;\neffect Network;\nbody\n");
    let unit = compile_type_effect_boundary(&source).expect("L0.4 compile must succeed");
    assert_eq!(unit.type_declaration_count(), 1);
    assert_eq!(unit.effect_declaration_count(), 1);
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
    assert_eq!(unit.declarations()[0].name().as_str(), "UserId");
    assert_eq!(unit.declarations()[1].name().as_str(), "Network");
}

#[test]
fn residual_body_remains_explicitly_unlowered() {
    let source = source(2, "type A;\nanything + still + opaque\n");
    let unit = compile_type_effect_boundary(&source).expect("compile must succeed");
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
    assert_eq!(
        source.slice(unit.origin().body_span()).unwrap(),
        "\nanything + still + opaque\n"
    );
}

#[test]
fn c01_boundary_remains_module_only_for_compatibility() {
    let source = source(3, "type");
    let old = compile_semantic_boundary(&source).expect("C0.1 body remains opaque");
    assert!(old.declarations().is_empty());
    assert_eq!(old.origin().body_span(), source.full_span());
    assert!(compile_type_effect_boundary(&source).is_err());
}

#[test]
fn comments_and_spacing_do_not_change_l04_semantic_identity() {
    let first = source(4, "module demo; type A; effect Net;");
    let second = source(
        5,
        "module demo /*x*/ ;\n type /*a*/ A /*b*/ ;\n effect Net ;\n",
    );
    let first = compile_type_effect_boundary(&first).unwrap();
    let second = compile_type_effect_boundary(&second).unwrap();
    assert_eq!(
        first.canonical_semantic_bytes(),
        second.canonical_semantic_bytes()
    );
}

#[test]
fn declaration_order_is_not_semantic_for_opaque_l04_declarations() {
    let first = source(6, "type A; effect Net;");
    let second = source(7, "effect Net; type A;");
    let first = compile_type_effect_boundary(&first).unwrap();
    let second = compile_type_effect_boundary(&second).unwrap();
    assert_eq!(
        first.canonical_semantic_bytes(),
        second.canonical_semantic_bytes()
    );
}

#[test]
fn source_ids_and_spans_do_not_change_l04_semantic_identity() {
    let first = source(8, "module demo;\ntype A;\neffect Net;");
    let second = source(999, "module demo; type A; effect Net;");
    let first = compile_type_effect_boundary(&first).unwrap();
    let second = compile_type_effect_boundary(&second).unwrap();
    assert_eq!(
        first.canonical_semantic_bytes(),
        second.canonical_semantic_bytes()
    );
}

#[test]
fn module_only_c01_identity_method_is_preserved() {
    let source = source(9, "module demo; type A; effect Net;");
    let l04 = compile_type_effect_boundary(&source).unwrap();
    let c01 = compile_semantic_boundary(&source).unwrap();
    assert_eq!(
        l04.canonical_identity_bytes(),
        c01.canonical_identity_bytes()
    );
    assert_ne!(
        l04.canonical_semantic_bytes(),
        l04.canonical_identity_bytes()
    );
}

#[test]
fn changing_type_identity_changes_semantic_bytes() {
    let first = compile_type_effect_boundary(&source(10, "type A;")).unwrap();
    let second = compile_type_effect_boundary(&source(11, "type B;")).unwrap();
    assert_ne!(
        first.canonical_semantic_bytes(),
        second.canonical_semantic_bytes()
    );
}

#[test]
fn type_and_effect_with_same_name_are_distinct_namespaces() {
    let source = source(12, "type State; effect State;");
    let unit = compile_type_effect_boundary(&source).expect("cross-kind names are unambiguous");
    assert_eq!(unit.type_declaration_count(), 1);
    assert_eq!(unit.effect_declaration_count(), 1);
    assert_ne!(unit.declarations()[0].kind(), unit.declarations()[1].kind());
}

#[test]
fn duplicate_type_declaration_is_rejected_before_nsir_publication() {
    let source = source(13, "type A; type A;");
    assert!(matches!(
        compile_type_effect_boundary(&source),
        Err(CompilerError::DuplicateSemanticDeclaration { kind: "type", .. })
    ));
}

#[test]
fn duplicate_effect_declaration_is_rejected_before_nsir_publication() {
    let source = source(14, "effect Net; effect Net;");
    assert!(matches!(
        compile_type_effect_boundary(&source),
        Err(CompilerError::DuplicateSemanticDeclaration { kind: "effect", .. })
    ));
}

#[test]
fn lowerer_retains_declaration_diagnostic_spans() {
    let source = source(15, "type A; effect Net; body");
    let surface = analyze_type_effect_unit(&source).unwrap();
    let hir = lower_type_effect_unit_to_hir(&source, &surface).unwrap();
    assert_eq!(hir.declarations().len(), 2);
    assert_eq!(
        source.slice(hir.declarations()[0].span()).unwrap(),
        "type A;"
    );
    assert_eq!(hir.body_state(), HirBodyState::Unlowered);
}

#[test]
fn declaration_origin_spans_do_not_enter_canonical_bytes() {
    let source = source(16, "type A; effect Net;");
    let unit = compile_type_effect_boundary(&source).unwrap();
    assert_eq!(
        source.slice(unit.declarations()[0].origin_span()).unwrap(),
        "type A;"
    );
    assert!(!unit.canonical_semantic_bytes().is_empty());
}

#[test]
fn declaration_source_mismatch_is_rejected() {
    let file_source = source(17, "body");
    let other_source = source(18, "type A;");
    let declaration = HirDeclaration::new(
        SemanticDeclarationKind::Type,
        SemanticName::new("A").unwrap(),
        other_source.full_span(),
    );
    let hir = HirUnit::new_with_declarations(
        SemanticModuleIdentity::Anonymous,
        file_source.full_span(),
        None,
        vec![declaration],
        file_source.full_span(),
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::SourceMismatch { .. })
    ));
}

#[test]
fn declaration_outside_file_span_is_rejected() {
    let source = source(19, "abcdefghij");
    let file = source
        .span(ByteOffset::new(2), ByteOffset::new(10))
        .unwrap();
    let declaration_span = source.span(ByteOffset::new(0), ByteOffset::new(1)).unwrap();
    let body = source
        .span(ByteOffset::new(2), ByteOffset::new(10))
        .unwrap();
    let hir = HirUnit::new_with_declarations(
        SemanticModuleIdentity::Anonymous,
        file,
        None,
        vec![HirDeclaration::new(
            SemanticDeclarationKind::Type,
            SemanticName::new("A").unwrap(),
            declaration_span,
        )],
        body,
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::DeclarationSpanOutsideFile { .. })
    ));
}

#[test]
fn declaration_overlap_with_body_is_rejected() {
    let source = source(20, "0123456789");
    let declaration = source.span(ByteOffset::new(1), ByteOffset::new(6)).unwrap();
    let body = source
        .span(ByteOffset::new(5), ByteOffset::new(10))
        .unwrap();
    let hir = HirUnit::new_with_declarations(
        SemanticModuleIdentity::Anonymous,
        source.full_span(),
        None,
        vec![HirDeclaration::new(
            SemanticDeclarationKind::Effect,
            SemanticName::new("Net").unwrap(),
            declaration,
        )],
        body,
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::DeclarationOverlapsBody { .. })
    ));
}

#[test]
fn declaration_order_violation_is_rejected() {
    let source = source(21, "01234567890123456789");
    let first = source
        .span(ByteOffset::new(8), ByteOffset::new(10))
        .unwrap();
    let second = source.span(ByteOffset::new(4), ByteOffset::new(6)).unwrap();
    let body = source
        .span(ByteOffset::new(12), ByteOffset::new(20))
        .unwrap();
    let hir = HirUnit::new_with_declarations(
        SemanticModuleIdentity::Anonymous,
        source.full_span(),
        None,
        vec![
            HirDeclaration::new(
                SemanticDeclarationKind::Type,
                SemanticName::new("A").unwrap(),
                first,
            ),
            HirDeclaration::new(
                SemanticDeclarationKind::Type,
                SemanticName::new("B").unwrap(),
                second,
            ),
        ],
        body,
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::DeclarationOrderViolation { .. })
    ));
}

#[test]
fn semantic_declaration_count_is_bounded_even_for_manual_hir() {
    let source = source(22, "x");
    let span = source.span(ByteOffset::new(0), ByteOffset::new(0)).unwrap();
    let mut declarations = Vec::new();
    for index in 0..=MAX_SEMANTIC_DECLARATIONS {
        declarations.push(HirDeclaration::new(
            SemanticDeclarationKind::Type,
            SemanticName::new(format!("T{index}")).unwrap(),
            span,
        ));
    }
    let hir = HirUnit::new_with_declarations(
        SemanticModuleIdentity::Anonymous,
        source.full_span(),
        None,
        declarations,
        source.full_span(),
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::TooManySemanticDeclarations { .. })
    ));
}

#[test]
fn empty_effect_set_is_explicitly_pure() {
    let effects = SemanticEffectSet::empty();
    assert!(effects.is_pure());
    assert!(effects.effects().is_empty());
}

#[test]
fn semantic_effect_set_is_canonical_and_order_independent() {
    let first = SemanticEffectSet::new(vec![
        SemanticName::new("Network").unwrap(),
        SemanticName::new("Clock").unwrap(),
    ])
    .unwrap();
    let second = SemanticEffectSet::new(vec![
        SemanticName::new("Clock").unwrap(),
        SemanticName::new("Network").unwrap(),
    ])
    .unwrap();
    assert_eq!(first.effects(), second.effects());
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
}

#[test]
fn duplicate_effect_requirement_is_rejected() {
    let result = SemanticEffectSet::new(vec![
        SemanticName::new("Network").unwrap(),
        SemanticName::new("Network").unwrap(),
    ]);
    assert!(matches!(
        result,
        Err(CompilerError::DuplicateEffectRequirement { .. })
    ));
}

#[test]
fn semantic_effect_requirement_count_is_bounded() {
    let mut effects = Vec::new();
    for index in 0..=MAX_SEMANTIC_EFFECT_REQUIREMENTS {
        effects.push(SemanticName::new(format!("E{index}")).unwrap());
    }
    assert!(matches!(
        SemanticEffectSet::new(effects),
        Err(CompilerError::TooManyEffectRequirements { .. })
    ));
}

#[test]
fn effect_declaration_does_not_grant_host_authority_or_execute_work() {
    let source = source(23, "effect Network;");
    let unit = compile_type_effect_boundary(&source).unwrap();
    assert_eq!(unit.effect_declaration_count(), 1);
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
    assert!(unit.origin().body_span().is_empty());
}

#[test]
fn module_analysis_still_precedes_l04_declaration_analysis() {
    let source = source(24, "module demo; type A;");
    let module = analyze_module_unit(&source).unwrap();
    assert!(module.module().is_some());
    let unit = compile_type_effect_boundary(&source).unwrap();
    assert_eq!(unit.type_declaration_count(), 1);
}
