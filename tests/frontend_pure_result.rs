use nordoi_kernel::{
    analyze_pure_result_unit, PureResultBodyForm, PureResultError, SourceId, SourceText,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(41), "pure.noi", text).unwrap()
}

#[test]
fn empty_body_is_supported() {
    let unit = analyze_pure_result_unit(&source("module demo; // tail\n")).unwrap();
    assert!(matches!(unit.form(), PureResultBodyForm::Empty));
    assert_eq!(unit.result_i64(), None);
}

#[test]
fn certified_l05_entry_remains_supported_at_l06_boundary() {
    let src = source("module demo; entry main;");
    let unit = analyze_pure_result_unit(&src).unwrap();
    match unit.form() {
        PureResultBodyForm::Entry(entry) => assert_eq!(entry.name().text(&src).unwrap(), "main"),
        _ => panic!("expected entry"),
    }
    assert_eq!(unit.result_i64(), None);
}

#[test]
fn entry_can_return_zero() {
    let src = source("module demo; entry main returns 0;");
    let unit = analyze_pure_result_unit(&src).unwrap();
    match unit.form() {
        PureResultBodyForm::EntryInt(entry) => {
            assert_eq!(entry.name().text(&src).unwrap(), "main");
            assert_eq!(entry.result().value(), 0);
            assert_eq!(
                src.slice(entry.returns_keyword().span()).unwrap(),
                "returns"
            );
        }
        _ => panic!("expected result entry"),
    }
}

#[test]
fn entry_can_return_i64_max() {
    let src = source("entry main returns 9223372036854775807;");
    let unit = analyze_pure_result_unit(&src).unwrap();
    assert_eq!(unit.result_i64(), Some(i64::MAX));
}

#[test]
fn comments_and_trivia_are_allowed_between_parts() {
    let src = source("entry /*a*/ main //b\n returns /*c*/ 42 /*d*/ ;");
    let unit = analyze_pure_result_unit(&src).unwrap();
    assert_eq!(unit.result_i64(), Some(42));
}

#[test]
fn leading_zero_is_rejected_as_noncanonical() {
    let error = analyze_pure_result_unit(&source("entry main returns 01;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::InvalidResultLiteral { .. }
    ));
}

#[test]
fn underscore_is_rejected() {
    let error = analyze_pure_result_unit(&source("entry main returns 1_000;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::InvalidResultLiteral { .. }
    ));
}

#[test]
fn suffix_is_rejected() {
    let error = analyze_pure_result_unit(&source("entry main returns 42i64;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::InvalidResultLiteral { .. }
    ));
}

#[test]
fn hexadecimal_spelling_is_rejected() {
    let error = analyze_pure_result_unit(&source("entry main returns 0x2a;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::InvalidResultLiteral { .. }
    ));
}

#[test]
fn out_of_range_integer_is_rejected() {
    let error =
        analyze_pure_result_unit(&source("entry main returns 9223372036854775808;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::ResultLiteralOutOfRange { .. }
    ));
}

#[test]
fn negative_spelling_is_not_defined() {
    let error = analyze_pure_result_unit(&source("entry main returns -1;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::ExpectedResultLiteral { .. }
    ));
}

#[test]
fn plus_spelling_is_not_defined() {
    let error = analyze_pure_result_unit(&source("entry main returns +1;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::ExpectedResultLiteral { .. }
    ));
}

#[test]
fn missing_result_literal_fails_closed() {
    let error = analyze_pure_result_unit(&source("entry main returns ;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::ExpectedResultLiteral { .. }
    ));
}

#[test]
fn missing_result_terminator_fails_closed() {
    let error = analyze_pure_result_unit(&source("entry main returns 42")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::ExpectedResultTerminator { .. }
    ));
}

#[test]
fn unexpected_continuation_after_entry_name_fails_closed() {
    let error = analyze_pure_result_unit(&source("entry main gives 42;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::ExpectedTerminatorOrReturns { .. }
    ));
}

#[test]
fn extra_significant_token_after_result_fails_closed() {
    let error = analyze_pure_result_unit(&source("entry main returns 42; other")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::UnexpectedAfterEntry { .. }
    ));
}

#[test]
fn second_entry_fails_closed() {
    let error =
        analyze_pure_result_unit(&source("entry main returns 42; entry other;")).unwrap_err();
    assert!(matches!(
        error,
        PureResultError::UnexpectedAfterEntry { .. }
    ));
}

#[test]
fn returns_is_contextual_not_a_global_keyword() {
    let unit = analyze_pure_result_unit(&source("entry returns returns 7;")).unwrap();
    match unit.form() {
        PureResultBodyForm::EntryInt(entry) => assert_eq!(entry.result().value(), 7),
        _ => panic!("expected result entry"),
    }
}

#[test]
fn arbitrary_body_still_fails_closed() {
    let error = analyze_pure_result_unit(&source("future_body")).unwrap_err();
    assert!(matches!(error, PureResultError::ExpectedEntryOrEnd { .. }));
}
