use nordoi_kernel::{
    analyze_type_effect_unit, lex, SourceId, SourceText, SurfaceDeclarationKind, TokenKind,
    TypeEffectError, MAX_NAME_BYTES, MAX_TYPE_EFFECT_DECLARATIONS,
};

fn source(id: u32, text: impl Into<String>) -> SourceText {
    SourceText::new(SourceId::new(id), "types-effects.noi", text).expect("source must be valid")
}

#[test]
fn empty_file_has_no_declarations_and_full_residual_body() {
    let source = source(1, "");
    let unit = analyze_type_effect_unit(&source).expect("empty source must analyze");
    assert!(unit.declarations().is_empty());
    assert_eq!(unit.body_span(), source.full_span());
}

#[test]
fn opaque_type_and_effect_declarations_are_recognized() {
    let source = source(2, "type UserId; effect Network;\nbody\n");
    let unit = analyze_type_effect_unit(&source).expect("prelude must analyze");
    assert_eq!(unit.declarations().len(), 2);
    assert_eq!(unit.declarations()[0].kind(), SurfaceDeclarationKind::Type);
    assert_eq!(
        unit.declarations()[1].kind(),
        SurfaceDeclarationKind::Effect
    );
    assert_eq!(
        unit.declarations()[0].name().text(&source).unwrap(),
        "UserId"
    );
    assert_eq!(
        unit.declarations()[1].name().text(&source).unwrap(),
        "Network"
    );
}

#[test]
fn declarations_follow_optional_module_header() {
    let source = source(
        3,
        "module demo.core;\n type UserId;\n effect Network;\nbody\n",
    );
    let unit = analyze_type_effect_unit(&source).expect("module prelude must analyze");
    assert_eq!(
        unit.module_unit()
            .module()
            .expect("module must exist")
            .path()
            .canonical_text(&source)
            .unwrap(),
        "demo.core"
    );
    assert_eq!(unit.declarations().len(), 2);
}

#[test]
fn type_and_effect_remain_lexical_identifiers() {
    let source = source(4, "type effect");
    let tokens = lex(&source).expect("lexing must succeed");
    assert_eq!(tokens[0].kind(), &TokenKind::Identifier);
    assert_eq!(tokens[2].kind(), &TokenKind::Identifier);
}

#[test]
fn declaration_keywords_are_exact_lowercase_contextual_words() {
    let source = source(5, "Type UserId;\n");
    let unit = analyze_type_effect_unit(&source).expect("non-contextual spelling is body");
    assert!(unit.declarations().is_empty());
    assert_eq!(unit.body_span(), source.full_span());
}

#[test]
fn trivia_is_allowed_inside_declarations() {
    let source = source(
        6,
        "module demo;\ntype /*a*/ UserId /*b*/ ;\neffect //x\n Network ;\n",
    );
    let unit = analyze_type_effect_unit(&source).expect("trivia must be accepted");
    assert_eq!(unit.declarations().len(), 2);
}

#[test]
fn body_begins_after_last_declaration_terminator() {
    let source = source(7, "module demo;\ntype UserId;\neffect Network;\n\nbody\n");
    let unit = analyze_type_effect_unit(&source).expect("prelude must analyze");
    let last_end = unit.declarations().last().unwrap().span().end();
    assert_eq!(unit.body_span().start(), last_end);
    assert_eq!(source.slice(unit.body_span()).unwrap(), "\n\nbody\n");
}

#[test]
fn analysis_stops_at_first_non_declaration_body_element() {
    let source = source(8, "type A;\n(body)\ntype Later;\n");
    let unit = analyze_type_effect_unit(&source).expect("body remains opaque");
    assert_eq!(unit.declarations().len(), 1);
    assert!(source
        .slice(unit.body_span())
        .unwrap()
        .contains("type Later;"));
}

#[test]
fn type_word_after_body_start_is_not_reinterpreted() {
    let source = source(9, "body\ntype Later;\n");
    let unit = analyze_type_effect_unit(&source).expect("whole source is body");
    assert!(unit.declarations().is_empty());
    assert_eq!(unit.body_span(), source.full_span());
}

#[test]
fn missing_type_name_fails_closed() {
    let source = source(10, "type");
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::ExpectedTypeName { .. })
    ));
}

#[test]
fn missing_effect_name_fails_closed() {
    let source = source(11, "effect;");
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::ExpectedEffectName { .. })
    ));
}

#[test]
fn numeric_type_name_is_rejected() {
    let source = source(12, "type 42;");
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::ExpectedTypeName { .. })
    ));
}

#[test]
fn group_cannot_be_effect_name() {
    let source = source(13, "effect (Network);");
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::ExpectedEffectName { .. })
    ));
}

#[test]
fn type_declaration_requires_semicolon() {
    let source = source(14, "type UserId");
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::ExpectedTypeTerminator { .. })
    ));
}

#[test]
fn effect_declaration_requires_semicolon() {
    let source = source(15, "effect Network type A;");
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::ExpectedEffectTerminator { .. })
    ));
}

#[test]
fn maximum_declaration_count_is_accepted() {
    let mut text = String::new();
    for index in 0..MAX_TYPE_EFFECT_DECLARATIONS {
        text.push_str(&format!("type T{index};\n"));
    }
    let source = source(16, text);
    let unit = analyze_type_effect_unit(&source).expect("maximum prelude must be accepted");
    assert_eq!(
        unit.declarations().len() as u32,
        MAX_TYPE_EFFECT_DECLARATIONS
    );
}

#[test]
fn declaration_above_limit_fails_before_unbounded_growth() {
    let mut text = String::new();
    for index in 0..=MAX_TYPE_EFFECT_DECLARATIONS {
        text.push_str(&format!("effect E{index};\n"));
    }
    let source = source(17, text);
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::TooManyDeclarations { .. })
    ));
}

#[test]
fn name_length_limit_is_inherited() {
    let long = "a".repeat(MAX_NAME_BYTES as usize + 1);
    let source = source(18, format!("type {long};"));
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::Name(_))
    ));
}

#[test]
fn unicode_executable_name_failure_is_inherited_from_lexer() {
    let source = source(19, "type Élément;");
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::Module(_))
    ));
}

#[test]
fn structural_failure_anywhere_prevents_publication() {
    let source = source(20, "type A;\n(body]\n");
    assert!(matches!(
        analyze_type_effect_unit(&source),
        Err(TypeEffectError::Module(_))
    ));
}

#[test]
fn keyword_name_and_terminator_tokens_are_retained() {
    let source = source(21, "type Value;");
    let unit = analyze_type_effect_unit(&source).expect("declaration must analyze");
    let declaration = &unit.declarations()[0];
    assert_eq!(source.slice(declaration.keyword().span()).unwrap(), "type");
    assert_eq!(declaration.name().text(&source).unwrap(), "Value");
    assert_eq!(source.slice(declaration.terminator().span()).unwrap(), ";");
}

#[test]
fn analyzer_is_deterministic_for_equal_source() {
    let source = source(22, "module stable;\ntype A; effect B;\nbody");
    let first = analyze_type_effect_unit(&source).expect("first analysis must succeed");
    let second = analyze_type_effect_unit(&source).expect("second analysis must succeed");
    assert_eq!(first, second);
}
