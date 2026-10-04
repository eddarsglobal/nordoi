use nordoi_kernel::{
    analyze_minimal_body_unit, lex, BodyError, MinimalBodyForm, SourceId, SourceText, TokenKind,
    MAX_NAME_BYTES,
};

fn source(id: u32, text: impl Into<String>) -> SourceText {
    SourceText::new(SourceId::new(id), "body.noi", text).expect("source must be valid")
}

#[test]
fn empty_source_is_a_fully_understood_empty_body() {
    let src = source(1, "");
    let unit = analyze_minimal_body_unit(&src).unwrap();
    assert!(matches!(unit.form(), MinimalBodyForm::Empty));
    assert_eq!(unit.body_span(), src.full_span());
}

#[test]
fn trivia_only_residual_body_is_empty() {
    let src = source(2, "module demo; type A; effect Net; /* body trivia */\n");
    let unit = analyze_minimal_body_unit(&src).unwrap();
    assert!(matches!(unit.form(), MinimalBodyForm::Empty));
}

#[test]
fn one_entry_declaration_is_recognized() {
    let src = source(3, "entry main;");
    let unit = analyze_minimal_body_unit(&src).unwrap();
    let entry = unit.entry().expect("entry must exist");
    assert_eq!(entry.name().text(&src).unwrap(), "main");
    assert_eq!(src.slice(entry.span()).unwrap(), "entry main;");
}

#[test]
fn entry_follows_module_and_type_effect_prelude() {
    let src = source(4, "module app.core; type User; effect Clock; entry main;");
    let unit = analyze_minimal_body_unit(&src).unwrap();
    assert_eq!(unit.type_effect_unit().declarations().len(), 2);
    assert_eq!(unit.entry().unwrap().name().text(&src).unwrap(), "main");
}

#[test]
fn entry_remains_a_lexical_identifier() {
    let src = source(5, "entry main;");
    let tokens = lex(&src).unwrap();
    assert_eq!(tokens[0].kind(), &TokenKind::Identifier);
}

#[test]
fn entry_keyword_is_exact_lowercase_contextual_text() {
    let src = source(6, "Entry main;");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::ExpectedEntryOrEnd { .. })
    ));
}

#[test]
fn trivia_is_allowed_inside_entry_declaration() {
    let src = source(7, "entry /*a*/ main //b\n ; /*tail*/");
    let unit = analyze_minimal_body_unit(&src).unwrap();
    assert_eq!(unit.entry().unwrap().name().text(&src).unwrap(), "main");
}

#[test]
fn entry_tokens_are_retained_losslessly() {
    let src = source(8, "entry main;");
    let unit = analyze_minimal_body_unit(&src).unwrap();
    let entry = unit.entry().unwrap();
    assert_eq!(src.slice(entry.keyword().span()).unwrap(), "entry");
    assert_eq!(entry.name().text(&src).unwrap(), "main");
    assert_eq!(src.slice(entry.terminator().span()).unwrap(), ";");
}

#[test]
fn entry_span_covers_keyword_through_terminator() {
    let src = source(9, "  entry /*x*/ main ;  ");
    let unit = analyze_minimal_body_unit(&src).unwrap();
    assert_eq!(
        src.slice(unit.entry().unwrap().span()).unwrap(),
        "entry /*x*/ main ;"
    );
}

#[test]
fn missing_entry_name_fails_closed() {
    let src = source(10, "entry");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::ExpectedEntryName { .. })
    ));
}

#[test]
fn numeric_entry_name_is_rejected() {
    let src = source(11, "entry 42;");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::ExpectedEntryName { .. })
    ));
}

#[test]
fn group_cannot_be_entry_name() {
    let src = source(12, "entry (main);");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::ExpectedEntryName { .. })
    ));
}

#[test]
fn entry_requires_semicolon() {
    let src = source(13, "entry main");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::ExpectedEntryTerminator { .. })
    ));
}

#[test]
fn non_semicolon_after_entry_name_is_rejected() {
    let src = source(14, "entry main {}");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::ExpectedEntryTerminator { .. })
    ));
}

#[test]
fn second_entry_is_rejected() {
    let src = source(15, "entry main; entry other;");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::UnexpectedAfterEntry { .. })
    ));
}

#[test]
fn arbitrary_body_syntax_is_rejected_instead_of_partially_interpreted() {
    let src = source(16, "future_body");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::ExpectedEntryOrEnd { .. })
    ));
}

#[test]
fn significant_element_after_entry_is_rejected() {
    let src = source(17, "entry main; future_body");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::UnexpectedAfterEntry { .. })
    ));
}

#[test]
fn structural_failure_anywhere_prevents_body_publication() {
    let src = source(18, "type A; entry main; (]");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::TypeEffect(_))
    ));
}

#[test]
fn unicode_executable_name_failure_is_inherited() {
    let src = source(19, "entry Entrée;");
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::TypeEffect(_))
    ));
}

#[test]
fn entry_name_length_limit_is_inherited() {
    let name = "a".repeat(MAX_NAME_BYTES as usize + 1);
    let src = source(20, format!("entry {name};"));
    assert!(matches!(
        analyze_minimal_body_unit(&src),
        Err(BodyError::Name(_))
    ));
}

#[test]
fn trailing_comments_after_entry_are_allowed() {
    let src = source(21, "entry main; //done\n /*done*/");
    assert!(analyze_minimal_body_unit(&src).is_ok());
}

#[test]
fn analyzer_is_deterministic_for_equal_source() {
    let src = source(22, "module stable; type A; effect Net; entry main;");
    assert_eq!(
        analyze_minimal_body_unit(&src).unwrap(),
        analyze_minimal_body_unit(&src).unwrap()
    );
}
