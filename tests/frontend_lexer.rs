use nordoi_kernel::{
    lex, ByteOffset, LexError, Lexer, SourceError, SourceId, SourceText, Token, TokenKind,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(7), "test.noi", text).expect("valid test source")
}

fn lexemes<'a>(source: &'a SourceText, tokens: &[Token]) -> Vec<(&'a str, TokenKind)> {
    tokens
        .iter()
        .filter(|token| !matches!(token.kind(), TokenKind::Eof))
        .map(|token| {
            (
                source.slice(token.span()).expect("token span must resolve"),
                token.kind().clone(),
            )
        })
        .collect()
}

#[test]
fn source_text_preserves_identity_name_and_utf8_text() {
    let source = SourceText::new(SourceId::new(42), "hello.noi", "héllo").unwrap();
    assert_eq!(source.id(), SourceId::new(42));
    assert_eq!(source.name(), "hello.noi");
    assert_eq!(source.text(), "héllo");
    assert_eq!(source.len(), ByteOffset::new(6));
}

#[test]
fn physical_lines_support_lf_crlf_and_cr_without_normalizing_source() {
    let source = source("a\r\nβ\rc\nd");
    assert_eq!(source.line_count(), 4);
    assert_eq!(source.position(ByteOffset::new(0)).unwrap().line(), 1);
    assert_eq!(source.position(ByteOffset::new(3)).unwrap().line(), 2);
    assert_eq!(source.position(ByteOffset::new(6)).unwrap().line(), 3);
    assert_eq!(source.position(ByteOffset::new(8)).unwrap().line(), 4);
    assert_eq!(source.text(), "a\r\nβ\rc\nd");
}

#[test]
fn source_columns_count_unicode_scalars_not_utf8_bytes() {
    let source = source("éx");
    let position = source.position(ByteOffset::new(2)).unwrap();
    assert_eq!(position.line(), 1);
    assert_eq!(position.column(), 2);
}

#[test]
fn source_span_slices_exact_utf8_bytes() {
    let source = source("aéz");
    let span = source.span(ByteOffset::new(1), ByteOffset::new(3)).unwrap();
    assert_eq!(source.slice(span).unwrap(), "é");
    assert_eq!(span.len(), 2);
}

#[test]
fn source_rejects_offsets_inside_utf8_scalars() {
    let source = source("é");
    assert_eq!(
        source.position(ByteOffset::new(1)).unwrap_err(),
        SourceError::OffsetNotCharBoundary {
            offset: ByteOffset::new(1)
        }
    );
}

#[test]
fn source_rejects_cross_source_spans() {
    let first = SourceText::new(SourceId::new(1), "a.noi", "abc").unwrap();
    let second = SourceText::new(SourceId::new(2), "b.noi", "abc").unwrap();
    assert_eq!(
        second.slice(first.full_span()).unwrap_err(),
        SourceError::SourceMismatch {
            expected: SourceId::new(2),
            actual: SourceId::new(1),
        }
    );
}

#[test]
fn source_rejects_reversed_spans() {
    let source = source("abc");
    assert_eq!(
        source
            .span(ByteOffset::new(2), ByteOffset::new(1))
            .unwrap_err(),
        SourceError::InvalidSpanOrder {
            start: ByteOffset::new(2),
            end: ByteOffset::new(1),
        }
    );
}

#[test]
fn source_rejects_out_of_bounds_offsets() {
    let source = source("abc");
    assert_eq!(
        source.position(ByteOffset::new(4)).unwrap_err(),
        SourceError::OffsetOutOfBounds {
            offset: ByteOffset::new(4),
            source_len: ByteOffset::new(3),
        }
    );
}

#[test]
fn empty_source_emits_only_zero_width_eof() {
    let source = source("");
    let tokens = lex(&source).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind(), &TokenKind::Eof);
    assert!(tokens[0].span().is_empty());
    assert_eq!(tokens[0].span().start(), ByteOffset::new(0));
}

#[test]
fn lexer_is_lossless_for_every_accepted_source_byte() {
    let source = source("alpha  12x // café\r\n/* nested /* ok */ yes */ \"hé\\\"llo\" + beta");
    let tokens = lex(&source).unwrap();
    let reconstructed = tokens
        .iter()
        .filter(|token| !matches!(token.kind(), TokenKind::Eof))
        .map(|token| source.slice(token.span()).unwrap())
        .collect::<String>();
    assert_eq!(reconstructed, source.text());
}

#[test]
fn identifiers_remain_surface_neutral_and_are_not_keyword_classified() {
    let source = source("if let effect capability hello_world _x9");
    let tokens = lex(&source).unwrap();
    for (_, kind) in lexemes(&source, &tokens)
        .into_iter()
        .filter(|(_, kind)| !kind.is_trivia())
    {
        assert_eq!(kind, TokenKind::Identifier);
    }
}

#[test]
fn numeric_candidates_do_not_claim_literal_semantics() {
    let source = source("42 0xff 12_345 1e9");
    let tokens = lex(&source).unwrap();
    let significant: Vec<_> = lexemes(&source, &tokens)
        .into_iter()
        .filter(|(_, kind)| !kind.is_trivia())
        .collect();
    assert_eq!(
        significant,
        vec![
            ("42", TokenKind::NumericCandidate),
            ("0xff", TokenKind::NumericCandidate),
            ("12_345", TokenKind::NumericCandidate),
            ("1e9", TokenKind::NumericCandidate),
        ]
    );
}

#[test]
fn punctuation_is_single_character_to_avoid_operator_freeze() {
    let source = source("-> == && :: +");
    let tokens = lex(&source).unwrap();
    let significant: Vec<_> = lexemes(&source, &tokens)
        .into_iter()
        .filter(|(_, kind)| !kind.is_trivia())
        .collect();
    assert_eq!(
        significant,
        vec![
            ("-", TokenKind::Punctuation('-')),
            (">", TokenKind::Punctuation('>')),
            ("=", TokenKind::Punctuation('=')),
            ("=", TokenKind::Punctuation('=')),
            ("&", TokenKind::Punctuation('&')),
            ("&", TokenKind::Punctuation('&')),
            (":", TokenKind::Punctuation(':')),
            (":", TokenKind::Punctuation(':')),
            ("+", TokenKind::Punctuation('+')),
        ]
    );
}

#[test]
fn whitespace_is_preserved_as_trivia() {
    let source = source("a \t\r\nb");
    let tokens = lex(&source).unwrap();
    let items = lexemes(&source, &tokens);
    assert_eq!(items[1], (" \t\r\n", TokenKind::Whitespace));
    assert!(tokens[1].is_trivia());
}

#[test]
fn line_comment_is_lossless_and_excludes_line_break() {
    let source = source("a// hello\nb");
    let tokens = lex(&source).unwrap();
    assert_eq!(
        lexemes(&source, &tokens),
        vec![
            ("a", TokenKind::Identifier),
            ("// hello", TokenKind::LineComment),
            ("\n", TokenKind::Whitespace),
            ("b", TokenKind::Identifier),
        ]
    );
}

#[test]
fn nested_block_comments_are_one_lossless_trivia_token() {
    let source = source("a /* outer /* inner */ tail */ b");
    let tokens = lex(&source).unwrap();
    let items = lexemes(&source, &tokens);
    assert!(items.contains(&("/* outer /* inner */ tail */", TokenKind::BlockComment)));
}

#[test]
fn unterminated_block_comment_fails_closed() {
    let source = source("/* never closed");
    assert!(matches!(
        lex(&source),
        Err(LexError::UnterminatedBlockComment { .. })
    ));
}

#[test]
fn quoted_text_preserves_escapes_without_interpreting_them() {
    let source = source("\"a\\q\\\"b\"");
    let tokens = lex(&source).unwrap();
    assert_eq!(
        lexemes(&source, &tokens),
        vec![("\"a\\q\\\"b\"", TokenKind::QuotedText)]
    );
}

#[test]
fn raw_newline_inside_quoted_text_is_rejected() {
    let source = source("\"a\nb\"");
    assert!(matches!(
        lex(&source),
        Err(LexError::NewlineInQuotedText { .. })
    ));
}

#[test]
fn escaped_raw_newline_inside_quoted_text_is_also_rejected() {
    let source = source("\"a\\\nb\"");
    assert!(matches!(
        lex(&source),
        Err(LexError::NewlineInQuotedText { .. })
    ));
}

#[test]
fn unterminated_quoted_text_fails_closed() {
    let source = source("\"never closed");
    assert!(matches!(
        lex(&source),
        Err(LexError::UnterminatedQuotedText { .. })
    ));
}

#[test]
fn unicode_is_allowed_inside_quoted_text() {
    let source = source("\"مرحبا 世界 café\"");
    let tokens = lex(&source).unwrap();
    assert_eq!(tokens[0].kind(), &TokenKind::QuotedText);
}

#[test]
fn unicode_is_allowed_inside_comments() {
    let source = source("// مرحبا 世界 café");
    let tokens = lex(&source).unwrap();
    assert_eq!(tokens[0].kind(), &TokenKind::LineComment);
}

#[test]
fn non_ascii_identifier_characters_are_rejected_until_a_profile_is_certified() {
    let source = source("café");
    let mut lexer = Lexer::new(&source);
    assert_eq!(lexer.next_token().unwrap().kind(), &TokenKind::Identifier);
    assert!(matches!(
        lexer.next_token(),
        Err(LexError::UnsupportedIdentifierCharacter {
            character: 'é', ..
        })
    ));
}

#[test]
fn non_ascii_whitespace_is_rejected_in_code() {
    let source = source("a\u{00A0}b");
    assert!(matches!(
        lex(&source),
        Err(LexError::UnsupportedWhitespace {
            character: '\u{00A0}',
            ..
        })
    ));
}

#[test]
fn bidi_controls_are_rejected_in_executable_code() {
    let source = source("a \u{202D} b");
    assert!(matches!(
        lex(&source),
        Err(LexError::BidirectionalControl {
            character: '\u{202D}',
            ..
        })
    ));
}

#[test]
fn bidi_controls_are_rejected_even_inside_comments() {
    let source = source("// safe? \u{202E} hidden");
    assert!(matches!(
        lex(&source),
        Err(LexError::BidirectionalControl {
            character: '\u{202E}',
            ..
        })
    ));
}

#[test]
fn bidi_controls_are_rejected_even_inside_quoted_text() {
    let source = source("\"safe? \u{2066} hidden\"");
    assert!(matches!(
        lex(&source),
        Err(LexError::BidirectionalControl {
            character: '\u{2066}',
            ..
        })
    ));
}

#[test]
fn nul_is_rejected_everywhere() {
    let source = source("\"a\0b\"");
    assert!(matches!(lex(&source), Err(LexError::NulCharacter { .. })));
}

#[test]
fn unicode_symbol_outside_literal_or_comment_is_rejected() {
    let source = source("x 😀 y");
    assert!(matches!(
        lex(&source),
        Err(LexError::UnexpectedCharacter {
            character: '😀',
            ..
        })
    ));
}

#[test]
fn slash_is_plain_punctuation_when_it_does_not_start_comment() {
    let source = source("a/b");
    let tokens = lex(&source).unwrap();
    assert_eq!(
        lexemes(&source, &tokens),
        vec![
            ("a", TokenKind::Identifier),
            ("/", TokenKind::Punctuation('/')),
            ("b", TokenKind::Identifier),
        ]
    );
}

#[test]
fn comment_markers_inside_quoted_text_are_not_comments() {
    let source = source("\"// not a comment /* still text */\"");
    let tokens = lex(&source).unwrap();
    assert_eq!(tokens[0].kind(), &TokenKind::QuotedText);
}

#[test]
fn eof_span_is_exactly_at_source_end() {
    let source = source("abc");
    let tokens = lex(&source).unwrap();
    let eof = tokens.last().unwrap();
    assert_eq!(eof.kind(), &TokenKind::Eof);
    assert_eq!(eof.span().start(), source.len());
    assert_eq!(eof.span().end(), source.len());
}
