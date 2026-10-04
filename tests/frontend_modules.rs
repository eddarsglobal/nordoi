use nordoi_kernel::{
    analyze_module_unit, lex, ModuleAnalyzer, ModuleError, Name, NameError, SourceId, SourceText,
    TokenKind, MAX_MODULE_SEGMENTS, MAX_NAME_BYTES,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(11), "modules.noi", text).unwrap()
}

#[test]
fn empty_file_is_anonymous_module_unit() {
    let source = source("");
    let unit = analyze_module_unit(&source).unwrap();
    assert!(unit.is_anonymous());
    assert!(unit.module().is_none());
}

#[test]
fn non_module_leading_identifier_keeps_unit_anonymous() {
    let source = source("alpha module beta;");
    let unit = analyze_module_unit(&source).unwrap();
    assert!(unit.is_anonymous());
}

#[test]
fn contextual_module_keyword_is_exact_and_lowercase() {
    let source = source("Module alpha;");
    let unit = analyze_module_unit(&source).unwrap();
    assert!(unit.is_anonymous());
}

#[test]
fn module_remains_an_identifier_at_lexical_layer() {
    let source = source("module alpha;");
    let tokens = lex(&source).unwrap();
    assert_eq!(tokens[0].kind(), &TokenKind::Identifier);
}

#[test]
fn simple_module_header_is_recognized() {
    let source = source("module alpha;");
    let unit = analyze_module_unit(&source).unwrap();
    let declaration = unit.module().unwrap();
    assert_eq!(source.slice(declaration.span()).unwrap(), "module alpha;");
    assert_eq!(declaration.path().canonical_text(&source).unwrap(), "alpha");
}

#[test]
fn qualified_module_path_is_recognized() {
    let source = source("module alpha.beta.gamma;");
    let unit = analyze_module_unit(&source).unwrap();
    let path = unit.module().unwrap().path();
    assert_eq!(path.segments().len(), 3);
    assert_eq!(path.canonical_text(&source).unwrap(), "alpha.beta.gamma");
}

#[test]
fn trivia_is_allowed_inside_module_header() {
    let source = source("/*lead*/ module /*a*/ alpha /*b*/ . /*c*/ beta /*d*/ ; tail");
    let unit = analyze_module_unit(&source).unwrap();
    let declaration = unit.module().unwrap();
    assert_eq!(
        declaration.path().canonical_text(&source).unwrap(),
        "alpha.beta"
    );
    assert_eq!(
        source.slice(declaration.span()).unwrap(),
        "module /*a*/ alpha /*b*/ . /*c*/ beta /*d*/ ;"
    );
}

#[test]
fn module_path_span_covers_original_source_between_first_and_last_segment() {
    let source = source("module alpha /*x*/ . beta;");
    let unit = analyze_module_unit(&source).unwrap();
    let path = unit.module().unwrap().path();
    assert_eq!(source.slice(path.span()).unwrap(), "alpha /*x*/ . beta");
}

#[test]
fn module_keyword_token_is_retained() {
    let source = source("module alpha;");
    let unit = analyze_module_unit(&source).unwrap();
    let keyword = unit.module().unwrap().keyword();
    assert_eq!(keyword.kind(), &TokenKind::Identifier);
    assert_eq!(source.slice(keyword.span()).unwrap(), "module");
}

#[test]
fn module_terminator_token_is_retained() {
    let source = source("module alpha;");
    let unit = analyze_module_unit(&source).unwrap();
    let terminator = unit.module().unwrap().terminator();
    assert_eq!(terminator.kind(), &TokenKind::Punctuation(';'));
    assert_eq!(source.slice(terminator.span()).unwrap(), ";");
}

#[test]
fn body_remains_in_original_structural_ast() {
    let source = source("module alpha; { body }");
    let unit = analyze_module_unit(&source).unwrap();
    assert_eq!(unit.file().span(), source.full_span());
    assert!(unit
        .file()
        .elements()
        .iter()
        .any(|element| element.as_group().is_some()));
}

#[test]
fn module_without_name_fails_closed() {
    let source = source("module;");
    assert!(matches!(
        analyze_module_unit(&source),
        Err(ModuleError::ExpectedModuleName { .. })
    ));
}

#[test]
fn numeric_module_name_is_rejected() {
    let source = source("module 42;");
    assert!(matches!(
        analyze_module_unit(&source),
        Err(ModuleError::ExpectedModuleName { .. })
    ));
}

#[test]
fn group_cannot_be_module_name() {
    let source = source("module (alpha);");
    assert!(matches!(
        analyze_module_unit(&source),
        Err(ModuleError::ExpectedModuleName { .. })
    ));
}

#[test]
fn missing_module_terminator_fails_at_eof() {
    let source = source("module alpha");
    let error = analyze_module_unit(&source).unwrap_err();
    assert!(matches!(error, ModuleError::MissingModuleTerminator { .. }));
    let span = error.primary_span().unwrap();
    assert_eq!(span.start(), source.len());
    assert_eq!(span.end(), source.len());
}

#[test]
fn missing_separator_between_segments_is_rejected() {
    let source = source("module alpha beta;");
    let error = analyze_module_unit(&source).unwrap_err();
    assert!(matches!(
        error,
        ModuleError::ExpectedModulePathSeparatorOrTerminator { .. }
    ));
    assert_eq!(source.slice(error.primary_span().unwrap()).unwrap(), "beta");
}

#[test]
fn missing_name_after_dot_is_rejected() {
    let source = source("module alpha.;");
    let error = analyze_module_unit(&source).unwrap_err();
    assert!(matches!(
        error,
        ModuleError::ExpectedModuleNameAfterSeparator { .. }
    ));
    assert_eq!(source.slice(error.primary_span().unwrap()).unwrap(), ";");
}

#[test]
fn repeated_dot_is_rejected() {
    let source = source("module alpha..beta;");
    assert!(matches!(
        analyze_module_unit(&source),
        Err(ModuleError::ExpectedModuleNameAfterSeparator { .. })
    ));
}

#[test]
fn group_after_dot_is_rejected() {
    let source = source("module alpha.(beta);");
    let error = analyze_module_unit(&source).unwrap_err();
    assert!(matches!(
        error,
        ModuleError::ExpectedModuleNameAfterSeparator { .. }
    ));
    assert_eq!(
        source.slice(error.primary_span().unwrap()).unwrap(),
        "(beta)"
    );
}

#[test]
fn exactly_maximum_name_length_is_accepted() {
    let name = "a".repeat(MAX_NAME_BYTES as usize);
    let source = source(&format!("module {name};"));
    let unit = analyze_module_unit(&source).unwrap();
    assert_eq!(
        unit.module()
            .unwrap()
            .path()
            .canonical_text(&source)
            .unwrap(),
        name
    );
}

#[test]
fn overlong_module_name_is_rejected() {
    let name = "a".repeat(MAX_NAME_BYTES as usize + 1);
    let source = source(&format!("module {name};"));
    assert!(matches!(
        analyze_module_unit(&source),
        Err(ModuleError::Name(NameError::TooLong {
            maximum: MAX_NAME_BYTES,
            ..
        }))
    ));
}

#[test]
fn exactly_maximum_module_segments_are_accepted() {
    let path = (0..MAX_MODULE_SEGMENTS)
        .map(|index| format!("m{index}"))
        .collect::<Vec<_>>()
        .join(".");
    let source = source(&format!("module {path};"));
    let unit = analyze_module_unit(&source).unwrap();
    assert_eq!(
        unit.module().unwrap().path().segments().len(),
        MAX_MODULE_SEGMENTS as usize
    );
}

#[test]
fn module_segment_limit_fails_before_unbounded_growth() {
    let path = (0..=MAX_MODULE_SEGMENTS)
        .map(|index| format!("m{index}"))
        .collect::<Vec<_>>()
        .join(".");
    let source = source(&format!("module {path};"));
    assert!(matches!(
        analyze_module_unit(&source),
        Err(ModuleError::TooManyModuleSegments {
            maximum: MAX_MODULE_SEGMENTS,
            ..
        })
    ));
}

#[test]
fn underscore_names_follow_existing_ascii_identifier_profile() {
    let source = source("module _alpha.beta_2;");
    let unit = analyze_module_unit(&source).unwrap();
    assert_eq!(
        unit.module()
            .unwrap()
            .path()
            .canonical_text(&source)
            .unwrap(),
        "_alpha.beta_2"
    );
}

#[test]
fn unicode_identifier_failure_is_inherited_from_lexer() {
    let source = source("module café;");
    assert!(matches!(
        analyze_module_unit(&source),
        Err(ModuleError::Parse(_))
    ));
}

#[test]
fn structural_failure_anywhere_prevents_module_unit_publication() {
    let source = source("module alpha; (]");
    assert!(matches!(
        analyze_module_unit(&source),
        Err(ModuleError::Parse(_))
    ));
}

#[test]
fn module_word_after_header_is_not_reinterpreted_by_name_layer() {
    let source = source("module alpha; module beta;");
    let unit = analyze_module_unit(&source).unwrap();
    assert_eq!(
        unit.module()
            .unwrap()
            .path()
            .canonical_text(&source)
            .unwrap(),
        "alpha"
    );
}

#[test]
fn analyzer_object_and_function_are_equivalent() {
    let source = source("module alpha.beta; tail");
    assert_eq!(
        ModuleAnalyzer::new(&source).unwrap().analyze().unwrap(),
        analyze_module_unit(&source).unwrap()
    );
}

#[test]
fn name_type_preserves_exact_span_identity() {
    let source = source("alpha");
    let tokens = lex(&source).unwrap();
    let name = Name::from_token(&source, &tokens[0]).unwrap();
    assert_eq!(name.text(&source).unwrap(), "alpha");
    assert_eq!(source.slice(name.span()).unwrap(), "alpha");
}

#[test]
fn name_type_rejects_non_identifier_token() {
    let source = source("42");
    let tokens = lex(&source).unwrap();
    assert!(matches!(
        Name::from_token(&source, &tokens[0]),
        Err(NameError::ExpectedIdentifier { .. })
    ));
}
