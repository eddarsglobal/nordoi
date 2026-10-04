use super::ast::AstElement;
use super::body_error::{BodyError, BodyResult};
use super::lexer::{Token, TokenKind};
use super::name::Name;
use super::source::{SourceSpan, SourceText};
use super::type_effect::{analyze_type_effect_unit, TypeEffectUnit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceEntry {
    span: SourceSpan,
    keyword: Token,
    name: Name,
    terminator: Token,
}

impl SurfaceEntry {
    pub(crate) fn new(span: SourceSpan, keyword: Token, name: Name, terminator: Token) -> Self {
        Self {
            span,
            keyword,
            name,
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

    pub fn terminator(&self) -> &Token {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinimalBodyForm {
    Empty,
    Entry(SurfaceEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimalBodyUnit {
    type_effect_unit: TypeEffectUnit,
    form: MinimalBodyForm,
    body_span: SourceSpan,
}

impl MinimalBodyUnit {
    pub(crate) fn new(
        type_effect_unit: TypeEffectUnit,
        form: MinimalBodyForm,
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

    pub fn form(&self) -> &MinimalBodyForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&SurfaceEntry> {
        match &self.form {
            MinimalBodyForm::Empty => None,
            MinimalBodyForm::Entry(entry) => Some(entry),
        }
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }
}

pub fn analyze_minimal_body_unit(source: &SourceText) -> BodyResult<MinimalBodyUnit> {
    let type_effect_unit = analyze_type_effect_unit(source)?;
    MinimalBodyAnalyzer::new(source, type_effect_unit).analyze()
}

#[derive(Debug)]
pub struct MinimalBodyAnalyzer<'a> {
    source: &'a SourceText,
    type_effect_unit: TypeEffectUnit,
}

impl<'a> MinimalBodyAnalyzer<'a> {
    pub fn new(source: &'a SourceText, type_effect_unit: TypeEffectUnit) -> Self {
        Self {
            source,
            type_effect_unit,
        }
    }

    pub fn analyze(self) -> BodyResult<MinimalBodyUnit> {
        let body_span = self.type_effect_unit.body_span();
        let mut cursor = ElementCursor::after_offset(
            self.type_effect_unit.module_unit().file().elements(),
            body_span.start().get(),
        );

        let Some(first) = cursor.next_significant() else {
            return Ok(MinimalBodyUnit::new(
                self.type_effect_unit,
                MinimalBodyForm::Empty,
                body_span,
            ));
        };

        let keyword = self.entry_keyword(first)?.clone();
        let name_element = cursor
            .next_significant()
            .ok_or(BodyError::ExpectedEntryName {
                span: self.type_effect_unit.module_unit().file().eof().span(),
            })?;
        let name = self.entry_name(name_element)?;
        let terminator_element =
            cursor
                .next_significant()
                .ok_or(BodyError::ExpectedEntryTerminator {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let terminator = self.entry_terminator(terminator_element)?.clone();

        if let Some(extra) = cursor.next_significant() {
            return Err(BodyError::UnexpectedAfterEntry { span: extra.span() });
        }

        let entry_span = self
            .source
            .span(keyword.span().start(), terminator.span().end())?;
        let entry = SurfaceEntry::new(entry_span, keyword, name, terminator);
        Ok(MinimalBodyUnit::new(
            self.type_effect_unit,
            MinimalBodyForm::Entry(entry),
            body_span,
        ))
    }

    fn entry_keyword<'b>(&self, element: &'b AstElement) -> BodyResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(BodyError::ExpectedEntryOrEnd {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier || self.source.slice(token.span())? != "entry" {
            return Err(BodyError::ExpectedEntryOrEnd { span: token.span() });
        }
        Ok(token)
    }

    fn entry_name(&self, element: &AstElement) -> BodyResult<Name> {
        let Some(token) = element.as_token() else {
            return Err(BodyError::ExpectedEntryName {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier {
            return Err(BodyError::ExpectedEntryName { span: token.span() });
        }
        Ok(Name::from_token(self.source, token)?)
    }

    fn entry_terminator<'b>(&self, element: &'b AstElement) -> BodyResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(BodyError::ExpectedEntryTerminator {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Punctuation(';') {
            return Err(BodyError::ExpectedEntryTerminator { span: token.span() });
        }
        Ok(token)
    }
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
