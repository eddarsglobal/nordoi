use nordoi_kernel::{
    analyze_module_unit, compile_semantic_boundary, lower_module_unit_to_hir, validate_hir,
    CompilerError, HirBodyState, HirUnit, NsirBodyState, SemanticModuleIdentity, SemanticName,
    SemanticPath, SourceId, SourceText, MAX_SEMANTIC_NAME_BYTES, MAX_SEMANTIC_PATH_SEGMENTS,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("c01-{id}.noi"), text)
        .expect("fixture source must be valid")
}

#[test]
fn named_module_lowers_to_validated_nsir_identity() {
    let source = source(1, "module alpha.beta;\nbody\n");
    let unit = compile_semantic_boundary(&source).expect("semantic boundary must compile");
    assert_eq!(
        unit.module().canonical_text().as_deref(),
        Some("alpha.beta")
    );
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
}

#[test]
fn anonymous_unit_remains_explicitly_anonymous() {
    let source = source(2, "alpha\n");
    let unit = compile_semantic_boundary(&source).expect("anonymous unit must compile");
    assert_eq!(unit.module(), &SemanticModuleIdentity::Anonymous);
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
}

#[test]
fn comments_and_spacing_do_not_change_semantic_module_identity() {
    let compact = source(3, "module alpha.beta;\n");
    let spaced = source(4, "module alpha /* x */ . beta ;\n");
    let compact = compile_semantic_boundary(&compact).expect("compact unit must compile");
    let spaced = compile_semantic_boundary(&spaced).expect("spaced unit must compile");
    assert_eq!(
        compact.canonical_identity_bytes(),
        spaced.canonical_identity_bytes()
    );
}

#[test]
fn different_module_identity_changes_canonical_bytes() {
    let alpha = source(5, "module alpha;\n");
    let beta = source(6, "module beta;\n");
    let alpha = compile_semantic_boundary(&alpha).expect("alpha must compile");
    let beta = compile_semantic_boundary(&beta).expect("beta must compile");
    assert_ne!(
        alpha.canonical_identity_bytes(),
        beta.canonical_identity_bytes()
    );
}

#[test]
fn source_ids_and_spans_do_not_change_canonical_identity_bytes() {
    let first = source(7, "module stable.identity;\n");
    let second = source(999, "module stable.identity;\n");
    let first = compile_semantic_boundary(&first).expect("first must compile");
    let second = compile_semantic_boundary(&second).expect("second must compile");
    assert_ne!(
        first.origin().file_span().source(),
        second.origin().file_span().source()
    );
    assert_eq!(
        first.canonical_identity_bytes(),
        second.canonical_identity_bytes()
    );
}

#[test]
fn canonical_identity_has_explicit_domain_and_variant() {
    let named = source(8, "module domain.test;\n");
    let anonymous = source(9, "body\n");
    let named = compile_semantic_boundary(&named).expect("named must compile");
    let anonymous = compile_semantic_boundary(&anonymous).expect("anonymous must compile");
    assert!(named
        .canonical_identity_bytes()
        .starts_with(b"NORDOI-C0.1-MODULE-ID\0\x01"));
    assert!(anonymous
        .canonical_identity_bytes()
        .starts_with(b"NORDOI-C0.1-MODULE-ID\0\x00"));
}

#[test]
fn hir_retains_diagnostic_spans_but_marks_body_unlowered() {
    let source = source(10, "module alpha;\n(beta)\n");
    let module = analyze_module_unit(&source).expect("module analysis must succeed");
    let hir = lower_module_unit_to_hir(&source, &module).expect("HIR lowering must succeed");
    assert_eq!(hir.body_state(), HirBodyState::Unlowered);
    assert_eq!(hir.file_span(), source.full_span());
    assert!(hir.module_span().is_some());
    assert_eq!(hir.body_span().end(), source.len());
}

#[test]
fn named_body_begins_after_module_declaration() {
    let source = source(11, "module alpha;\nbody\n");
    let module = analyze_module_unit(&source).expect("module analysis must succeed");
    let hir = lower_module_unit_to_hir(&source, &module).expect("HIR lowering must succeed");
    let module_span = hir.module_span().expect("named unit has module span");
    assert_eq!(hir.body_span().start(), module_span.end());
}

#[test]
fn anonymous_body_is_the_full_source_span() {
    let source = source(12, "body\n");
    let module = analyze_module_unit(&source).expect("module analysis must succeed");
    let hir = lower_module_unit_to_hir(&source, &module).expect("HIR lowering must succeed");
    assert_eq!(hir.body_span(), source.full_span());
}

#[test]
fn compiler_propagates_frontend_failure_without_partial_nsir() {
    let source = source(13, "module alpha beta;");
    let error = compile_semantic_boundary(&source).expect_err("malformed header must fail");
    assert!(matches!(error, CompilerError::Frontend(_)));
}

#[test]
fn semantic_name_enforces_ascii_identifier_profile() {
    assert!(SemanticName::new("alpha_2").is_ok());
    assert!(matches!(
        SemanticName::new("2alpha"),
        Err(CompilerError::InvalidSemanticName { .. })
    ));
    assert!(matches!(
        SemanticName::new("alpha-beta"),
        Err(CompilerError::InvalidSemanticName { .. })
    ));
}

#[test]
fn semantic_name_rejects_empty_value() {
    assert_eq!(
        SemanticName::new("").expect_err("empty name must fail"),
        CompilerError::EmptySemanticName
    );
}

#[test]
fn semantic_name_limit_is_bounded() {
    let exact = "a".repeat(MAX_SEMANTIC_NAME_BYTES as usize);
    let over = "a".repeat(MAX_SEMANTIC_NAME_BYTES as usize + 1);
    assert!(SemanticName::new(exact).is_ok());
    assert!(matches!(
        SemanticName::new(over),
        Err(CompilerError::SemanticNameTooLong { .. })
    ));
}

#[test]
fn semantic_path_requires_at_least_one_segment() {
    assert_eq!(
        SemanticPath::new(Vec::new()).expect_err("empty path must fail"),
        CompilerError::EmptySemanticPath
    );
}

#[test]
fn semantic_path_segment_count_is_bounded() {
    let segment = SemanticName::new("x").expect("name must be valid");
    let exact = vec![segment.clone(); MAX_SEMANTIC_PATH_SEGMENTS as usize];
    let over = vec![segment; MAX_SEMANTIC_PATH_SEGMENTS as usize + 1];
    assert!(SemanticPath::new(exact).is_ok());
    assert!(matches!(
        SemanticPath::new(over),
        Err(CompilerError::TooManySemanticSegments { .. })
    ));
}

#[test]
fn validation_rejects_cross_source_body_span() {
    let a = source(14, "alpha");
    let b = source(15, "beta");
    let hir = HirUnit::new(
        SemanticModuleIdentity::Anonymous,
        a.full_span(),
        None,
        b.full_span(),
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::SourceMismatch { .. })
    ));
}

#[test]
fn validation_rejects_body_outside_file_span() {
    let source = source(16, "abcdef");
    let file = source
        .span(
            nordoi_kernel::ByteOffset::new(1),
            nordoi_kernel::ByteOffset::new(5),
        )
        .expect("file fixture span must be valid");
    let hir = HirUnit::new(
        SemanticModuleIdentity::Anonymous,
        file,
        None,
        source.full_span(),
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::BodySpanOutsideFile { .. })
    ));
}

#[test]
fn validation_rejects_module_span_outside_file_span() {
    let source = source(17, "abcdef");
    let file = source
        .span(
            nordoi_kernel::ByteOffset::new(1),
            nordoi_kernel::ByteOffset::new(5),
        )
        .expect("file fixture span must be valid");
    let module = source.full_span();
    let body = source
        .span(
            nordoi_kernel::ByteOffset::new(5),
            nordoi_kernel::ByteOffset::new(5),
        )
        .expect("body fixture span must be valid");
    let hir = HirUnit::new(
        SemanticModuleIdentity::Anonymous,
        file,
        Some(module),
        body,
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::ModuleSpanOutsideFile { .. })
    ));
}

#[test]
fn validation_rejects_body_that_overlaps_module_header() {
    let source = source(18, "abcdef");
    let module = source
        .span(
            nordoi_kernel::ByteOffset::new(0),
            nordoi_kernel::ByteOffset::new(3),
        )
        .expect("module fixture span must be valid");
    let body = source
        .span(
            nordoi_kernel::ByteOffset::new(2),
            nordoi_kernel::ByteOffset::new(6),
        )
        .expect("body fixture span must be valid");
    let hir = HirUnit::new(
        SemanticModuleIdentity::Anonymous,
        source.full_span(),
        Some(module),
        body,
        HirBodyState::Unlowered,
    );
    assert!(matches!(
        validate_hir(hir),
        Err(CompilerError::BodyStartsBeforeModuleEnds { .. })
    ));
}

#[test]
fn validated_nsir_is_deterministic_for_equal_source() {
    let source = source(19, "module deterministic.unit;\n(alpha)\n");
    let first = compile_semantic_boundary(&source).expect("first compile must succeed");
    let second = compile_semantic_boundary(&source).expect("second compile must succeed");
    assert_eq!(first, second);
    assert_eq!(
        first.canonical_identity_bytes(),
        second.canonical_identity_bytes()
    );
}

#[test]
fn c01_does_not_claim_body_semantics() {
    let source = source(20, "module alpha;\nanything + at + all\n");
    let unit = compile_semantic_boundary(&source).expect("structural body must remain opaque");
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
}
