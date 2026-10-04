use nordoi_kernel::{
    parse, AstElement, AstFile, Delimiter, LexError, ParseError, Parser, SourceId, SourceText,
    Token, TokenKind, MAX_PARSE_NESTING,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(7), "parser.noi", text).unwrap()
}

fn collect_tokens<'a>(elements: &'a [AstElement], output: &mut Vec<&'a Token>) {
    for element in elements {
        match element {
            AstElement::Token(token) => output.push(token),
            AstElement::Group(group) => {
                output.push(group.open());
                collect_tokens(group.elements(), output);
                output.push(group.close());
            }
        }
    }
}

fn reconstruct(source: &SourceText, file: &AstFile) -> String {
    let mut tokens = Vec::new();
    collect_tokens(file.elements(), &mut tokens);
    tokens.into_iter().fold(String::new(), |mut output, token| {
        output.push_str(source.slice(token.span()).unwrap());
        output
    })
}

#[test]
fn empty_source_parses_to_empty_file_and_exact_eof() {
    let source = source("");
    let file = parse(&source).unwrap();
    assert!(file.elements().is_empty());
    assert_eq!(file.source(), source.id());
    assert_eq!(file.span(), source.full_span());
    assert_eq!(file.eof().kind(), &TokenKind::Eof);
    assert_eq!(file.eof().span().start(), source.len());
    assert_eq!(file.eof().span().end(), source.len());
}

#[test]
fn parser_is_lossless_across_nested_structure_and_trivia() {
    let source = source("alpha /*x*/ ( beta [\"{not group}\"] //y\n {42} ) tail");
    let file = parse(&source).unwrap();
    assert_eq!(reconstruct(&source, &file), source.text());
}

#[test]
fn all_three_delimiter_families_form_groups() {
    let source = source("() [] {}");
    let file = parse(&source).unwrap();
    let groups: Vec<_> = file
        .elements()
        .iter()
        .filter_map(AstElement::as_group)
        .collect();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].delimiter(), Delimiter::Parenthesis);
    assert_eq!(groups[1].delimiter(), Delimiter::Bracket);
    assert_eq!(groups[2].delimiter(), Delimiter::Brace);
}

#[test]
fn nested_groups_preserve_parent_child_structure() {
    let source = source("({[x]})");
    let file = parse(&source).unwrap();
    let paren = file.elements()[0].as_group().unwrap();
    let brace = paren.elements()[0].as_group().unwrap();
    let bracket = brace.elements()[0].as_group().unwrap();
    assert_eq!(paren.delimiter(), Delimiter::Parenthesis);
    assert_eq!(brace.delimiter(), Delimiter::Brace);
    assert_eq!(bracket.delimiter(), Delimiter::Bracket);
    assert_eq!(source.slice(bracket.span()).unwrap(), "[x]");
}

#[test]
fn group_span_includes_both_delimiters() {
    let source = source("before ( x ) after");
    let file = parse(&source).unwrap();
    let group = file
        .elements()
        .iter()
        .find_map(AstElement::as_group)
        .unwrap();
    assert_eq!(source.slice(group.span()).unwrap(), "( x )");
    assert_eq!(source.slice(group.open().span()).unwrap(), "(");
    assert_eq!(source.slice(group.close().span()).unwrap(), ")");
}

#[test]
fn trivia_is_retained_inside_groups() {
    let source = source("(/* keep */\n  )");
    let file = parse(&source).unwrap();
    let group = file.elements()[0].as_group().unwrap();
    assert!(group
        .elements()
        .iter()
        .filter_map(AstElement::as_token)
        .all(Token::is_trivia));
    assert_eq!(reconstruct(&source, &file), source.text());
}

#[test]
fn non_delimiter_punctuation_remains_plain_tokens() {
    let source = source("a->b == c && d; e:f");
    let file = parse(&source).unwrap();
    assert!(file
        .elements()
        .iter()
        .all(|element| element.as_group().is_none()));
    assert_eq!(reconstruct(&source, &file), source.text());
}

#[test]
fn identifiers_are_not_reclassified_as_keywords() {
    let source = source("if let module effect capability fn");
    let file = parse(&source).unwrap();
    let significant: Vec<_> = file
        .elements()
        .iter()
        .filter_map(AstElement::as_token)
        .filter(|token| !token.is_trivia())
        .collect();
    assert_eq!(significant.len(), 6);
    assert!(significant
        .iter()
        .all(|token| token.kind() == &TokenKind::Identifier));
}

#[test]
fn delimiter_text_inside_quoted_text_is_opaque() {
    let source = source("\"([{}])\"");
    let file = parse(&source).unwrap();
    assert_eq!(file.elements().len(), 1);
    let token = file.elements()[0].as_token().unwrap();
    assert_eq!(token.kind(), &TokenKind::QuotedText);
}

#[test]
fn delimiter_text_inside_comments_is_opaque() {
    let source = source("// ([{}])\n/* ({[]}) */");
    let file = parse(&source).unwrap();
    assert!(file
        .elements()
        .iter()
        .all(|element| element.as_group().is_none()));
    assert_eq!(reconstruct(&source, &file), source.text());
}

#[test]
fn unexpected_root_closer_fails_closed() {
    let source = source(")");
    assert!(matches!(
        parse(&source),
        Err(ParseError::UnexpectedClosingDelimiter {
            found: Delimiter::Parenthesis,
            ..
        })
    ));
}

#[test]
fn mismatched_closer_reports_expected_and_found() {
    let source = source("([)]");
    assert!(matches!(
        parse(&source),
        Err(ParseError::MismatchedClosingDelimiter {
            expected: Delimiter::Bracket,
            found: Delimiter::Parenthesis,
            ..
        })
    ));
}

#[test]
fn mismatched_closer_primary_span_is_the_closer() {
    let source = source("[)");
    let error = parse(&source).unwrap_err();
    let span = error.primary_span().unwrap();
    assert_eq!(source.slice(span).unwrap(), ")");
}

#[test]
fn unclosed_group_reports_opening_delimiter() {
    let source = source("a { b");
    let error = parse(&source).unwrap_err();
    match error {
        ParseError::UnclosedDelimiter {
            delimiter,
            open_span,
            eof_span,
        } => {
            assert_eq!(delimiter, Delimiter::Brace);
            assert_eq!(source.slice(open_span).unwrap(), "{");
            assert_eq!(eof_span.start(), source.len());
            assert_eq!(eof_span.end(), source.len());
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn deepest_unclosed_group_is_reported_first() {
    let source = source("({[");
    assert!(matches!(
        parse(&source),
        Err(ParseError::UnclosedDelimiter {
            delimiter: Delimiter::Bracket,
            ..
        })
    ));
}

#[test]
fn lexical_failure_is_propagated_without_partial_ast() {
    let source = source("ok ( café )");
    assert!(matches!(
        parse(&source),
        Err(ParseError::Lex(LexError::UnsupportedIdentifierCharacter {
            character: 'é',
            ..
        }))
    ));
}

#[test]
fn bidi_failure_is_propagated_before_structural_parsing() {
    let source = source("(a \u{202E} b)");
    assert!(matches!(
        parse(&source),
        Err(ParseError::Lex(LexError::BidirectionalControl {
            character: '\u{202E}',
            ..
        }))
    ));
}

#[test]
fn maximum_structural_depth_is_accepted() {
    let depth = MAX_PARSE_NESTING as usize;
    let text = format!("{}x{}", "(".repeat(depth), ")".repeat(depth));
    let source = source(&text);
    assert!(parse(&source).is_ok());
}

#[test]
fn structural_depth_above_limit_fails_before_unbounded_stack_growth() {
    let depth = MAX_PARSE_NESTING as usize + 1;
    let text = format!("{}x{}", "(".repeat(depth), ")".repeat(depth));
    let source = source(&text);
    assert!(matches!(
        parse(&source),
        Err(ParseError::NestingLimitExceeded {
            maximum: MAX_PARSE_NESTING,
            ..
        })
    ));
}

#[test]
fn nesting_limit_error_points_at_first_disallowed_opener() {
    let depth = MAX_PARSE_NESTING as usize + 1;
    let text = "(".repeat(depth);
    let source = source(&text);
    let error = parse(&source).unwrap_err();
    let span = error.primary_span().unwrap();
    assert_eq!(span.start().get(), MAX_PARSE_NESTING);
    assert_eq!(source.slice(span).unwrap(), "(");
}

#[test]
fn parse_is_deterministic_for_equal_source() {
    let source = source("alpha({beta:[42]}) // end");
    let first = parse(&source).unwrap();
    let second = parse(&source).unwrap();
    assert_eq!(first, second);
}

#[test]
fn parser_object_and_parse_function_are_equivalent() {
    let source = source("a(b[c]{d})");
    assert_eq!(
        Parser::new(&source).unwrap().parse().unwrap(),
        parse(&source).unwrap()
    );
}

#[test]
fn ast_element_spans_slice_exact_original_source() {
    let source = source("a (b) c");
    let file = parse(&source).unwrap();
    for element in file.elements() {
        let slice = source.slice(element.span()).unwrap();
        assert!(!slice.is_empty());
    }
}

#[test]
fn groups_do_not_claim_call_index_block_or_collection_semantics() {
    let source = source("name(x) name[x] name{x}");
    let file = parse(&source).unwrap();
    let groups: Vec<_> = file
        .elements()
        .iter()
        .filter_map(AstElement::as_group)
        .collect();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].delimiter(), Delimiter::Parenthesis);
    assert_eq!(groups[1].delimiter(), Delimiter::Bracket);
    assert_eq!(groups[2].delimiter(), Delimiter::Brace);
}

#[test]
fn punctuation_sequences_are_not_merged_into_operators() {
    let source = source("a=>b::c??d");
    let file = parse(&source).unwrap();
    let punctuation: Vec<char> = file
        .elements()
        .iter()
        .filter_map(AstElement::as_token)
        .filter_map(|token| match token.kind() {
            TokenKind::Punctuation(character) => Some(*character),
            _ => None,
        })
        .collect();
    assert_eq!(punctuation, vec!['=', '>', ':', ':', '?', '?']);
}
