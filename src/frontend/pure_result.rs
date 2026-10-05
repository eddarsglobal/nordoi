use super::ast::AstElement;
use super::body::SurfaceEntry;
use super::lexer::{Token, TokenKind};
use super::name::Name;
use super::pure_result_error::{PureResultError, PureResultResult};
use super::source::{SourceSpan, SourceText};
use super::type_effect::{analyze_type_effect_unit, TypeEffectUnit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceIntResult {
    span: SourceSpan,
    token: Token,
    value: i64,
}

impl SurfaceIntResult {
    pub(crate) fn new(span: SourceSpan, token: Token, value: i64) -> Self {
        Self { span, token, value }
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceResultEntry {
    span: SourceSpan,
    keyword: Token,
    name: Name,
    returns_keyword: Token,
    result: SurfaceIntResult,
    terminator: Token,
}

impl SurfaceResultEntry {
    pub(crate) fn new(
        span: SourceSpan,
        keyword: Token,
        name: Name,
        returns_keyword: Token,
        result: SurfaceIntResult,
        terminator: Token,
    ) -> Self {
        Self {
            span,
            keyword,
            name,
            returns_keyword,
            result,
            terminator,
        }
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn keyword(&self) -> &Token {
        &self.keyword
    }

    pub fn name(&self) -> Name {
        self.name
    }

    pub fn returns_keyword(&self) -> &Token {
        &self.returns_keyword
    }

    pub fn result(&self) -> &SurfaceIntResult {
        &self.result
    }

    pub fn terminator(&self) -> &Token {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureResultBodyForm {
    Empty,
    Entry(SurfaceEntry),
    EntryInt(SurfaceResultEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureResultUnit {
    type_effect_unit: TypeEffectUnit,
    form: PureResultBodyForm,
    body_span: SourceSpan,
}

impl PureResultUnit {
    pub(crate) fn new(
        type_effect_unit: TypeEffectUnit,
        form: PureResultBodyForm,
        body_span: SourceSpan,
    ) -> Self {
        Self {
            type_effect_unit,
            form,
            body_span,
        }
    }

    pub fn type_effect_unit(&self) -> &TypeEffectUnit {
        &self.type_effect_unit
    }

    pub fn form(&self) -> &PureResultBodyForm {
        &self.form
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }

    pub fn result_i64(&self) -> Option<i64> {
        match &self.form {
            PureResultBodyForm::EntryInt(entry) => Some(entry.result().value()),
            PureResultBodyForm::Empty | PureResultBodyForm::Entry(_) => None,
        }
    }
}

pub fn analyze_pure_result_unit(source: &SourceText) -> PureResultResult<PureResultUnit> {
    let type_effect_unit = analyze_type_effect_unit(source)?;
    PureResultAnalyzer::new(source, type_effect_unit).analyze()
}

#[derive(Debug)]
pub struct PureResultAnalyzer<'a> {
    source: &'a SourceText,
    type_effect_unit: TypeEffectUnit,
}

impl<'a> PureResultAnalyzer<'a> {
    pub fn new(source: &'a SourceText, type_effect_unit: TypeEffectUnit) -> Self {
        Self {
            source,
            type_effect_unit,
        }
    }

    pub fn analyze(self) -> PureResultResult<PureResultUnit> {
        let body_span = self.type_effect_unit.body_span();
        let mut cursor = ElementCursor::after_offset(
            self.type_effect_unit.module_unit().file().elements(),
            body_span.start().get(),
        );

        let Some(first) = cursor.next_significant() else {
            return Ok(PureResultUnit::new(
                self.type_effect_unit,
                PureResultBodyForm::Empty,
                body_span,
            ));
        };

        let keyword = self.entry_keyword(first)?.clone();
        let name_element = cursor
            .next_significant()
            .ok_or(PureResultError::ExpectedEntryName {
                span: self.type_effect_unit.module_unit().file().eof().span(),
            })?;
        let name = self.entry_name(name_element)?;
        let continuation =
            cursor
                .next_significant()
                .ok_or(PureResultError::ExpectedTerminatorOrReturns {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;

        if let Some(terminator) = self.try_terminator(continuation) {
            if let Some(extra) = cursor.next_significant() {
                return Err(PureResultError::UnexpectedAfterEntry { span: extra.span() });
            }
            let terminator = terminator.clone();
            let entry_span = self
                .source
                .span(keyword.span().start(), terminator.span().end())?;
            let entry = SurfaceEntry::new(entry_span, keyword, name, terminator);
            return Ok(PureResultUnit::new(
                self.type_effect_unit,
                PureResultBodyForm::Entry(entry),
                body_span,
            ));
        }

        let returns_keyword = self.returns_keyword(continuation)?.clone();
        let result_element =
            cursor
                .next_significant()
                .ok_or(PureResultError::ExpectedResultLiteral {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let result = self.result_literal(result_element)?;
        let terminator_element =
            cursor
                .next_significant()
                .ok_or(PureResultError::ExpectedResultTerminator {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let terminator = self.result_terminator(terminator_element)?.clone();

        if let Some(extra) = cursor.next_significant() {
            return Err(PureResultError::UnexpectedAfterEntry { span: extra.span() });
        }

        let entry_span = self
            .source
            .span(keyword.span().start(), terminator.span().end())?;
        let entry = SurfaceResultEntry::new(
            entry_span,
            keyword,
            name,
            returns_keyword,
            result,
            terminator,
        );
        Ok(PureResultUnit::new(
            self.type_effect_unit,
            PureResultBodyForm::EntryInt(entry),
            body_span,
        ))
    }

    fn entry_keyword<'b>(&self, element: &'b AstElement) -> PureResultResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureResultError::ExpectedEntryOrEnd {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier || self.source.slice(token.span())? != "entry" {
            return Err(PureResultError::ExpectedEntryOrEnd { span: token.span() });
        }
        Ok(token)
    }

    fn entry_name(&self, element: &AstElement) -> PureResultResult<Name> {
        let Some(token) = element.as_token() else {
            return Err(PureResultError::ExpectedEntryName {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier {
            return Err(PureResultError::ExpectedEntryName { span: token.span() });
        }
        Ok(Name::from_token(self.source, token)?)
    }

    fn try_terminator<'b>(&self, element: &'b AstElement) -> Option<&'b Token> {
        let token = element.as_token()?;
        (token.kind() == &TokenKind::Punctuation(';')).then_some(token)
    }

    fn returns_keyword<'b>(&self, element: &'b AstElement) -> PureResultResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureResultError::ExpectedTerminatorOrReturns {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier || self.source.slice(token.span())? != "returns" {
            return Err(PureResultError::ExpectedTerminatorOrReturns { span: token.span() });
        }
        Ok(token)
    }

    fn result_literal(&self, element: &AstElement) -> PureResultResult<SurfaceIntResult> {
        let Some(token) = element.as_token() else {
            return Err(PureResultError::ExpectedResultLiteral {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::NumericCandidate {
            return Err(PureResultError::ExpectedResultLiteral { span: token.span() });
        }
        let text = self.source.slice(token.span())?;
        if !is_canonical_non_negative_decimal(text) {
            return Err(PureResultError::InvalidResultLiteral { span: token.span() });
        }
        let value = text
            .parse::<i64>()
            .map_err(|_| PureResultError::ResultLiteralOutOfRange { span: token.span() })?;
        Ok(SurfaceIntResult::new(token.span(), token.clone(), value))
    }

    fn result_terminator<'b>(&self, element: &'b AstElement) -> PureResultResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureResultError::ExpectedResultTerminator {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Punctuation(';') {
            return Err(PureResultError::ExpectedResultTerminator { span: token.span() });
        }
        Ok(token)
    }
}

fn is_canonical_non_negative_decimal(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes == b"0" {
        return true;
    }
    if bytes.is_empty() || !(b'1'..=b'9').contains(&bytes[0]) {
        return false;
    }
    bytes[1..].iter().all(|byte| byte.is_ascii_digit())
}

#[derive(Debug)]
struct ElementCursor<'a> {
    elements: &'a [AstElement],
    index: usize,
}

impl<'a> ElementCursor<'a> {
    fn after_offset(elements: &'a [AstElement], offset: u32) -> Self {
        let mut index = 0usize;
        while index < elements.len() && elements[index].span().end().get() <= offset {
            index += 1;
        }
        Self { elements, index }
    }

    fn next_significant(&mut self) -> Option<&'a AstElement> {
        while self.index < self.elements.len() {
            let element = &self.elements[self.index];
            self.index += 1;
            match element {
                AstElement::Token(token) if token.is_trivia() => continue,
                _ => return Some(element),
            }
        }
        None
    }
}
