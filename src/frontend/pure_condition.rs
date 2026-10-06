use super::ast::AstElement;
use super::body::SurfaceEntry;
use super::lexer::{Token, TokenKind};
use super::name::Name;
use super::pure_condition_error::{PureConditionError, PureConditionResult};
use super::source::{SourceSpan, SourceText};
use super::type_effect::{analyze_type_effect_unit, TypeEffectUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfacePureComparator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl SurfacePureComparator {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfacePureConditionKind {
    Bool {
        span: SourceSpan,
        token: Token,
        value: bool,
    },
    IntCompare {
        span: SourceSpan,
        lhs_span: SourceSpan,
        lhs_token: Token,
        lhs: i64,
        comparator_span: SourceSpan,
        comparator: SurfacePureComparator,
        rhs_span: SourceSpan,
        rhs_token: Token,
        rhs: i64,
    },
}

impl SurfacePureConditionKind {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Bool { span, .. } | Self::IntCompare { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePureCondition {
    kind: SurfacePureConditionKind,
}

impl SurfacePureCondition {
    pub(crate) fn new(kind: SurfacePureConditionKind) -> Self {
        Self { kind }
    }

    pub fn span(&self) -> SourceSpan {
        self.kind.span()
    }

    pub fn kind(&self) -> &SurfacePureConditionKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceConditionEntry {
    span: SourceSpan,
    keyword: Token,
    name: Name,
    returns_keyword: Token,
    condition: SurfacePureCondition,
    terminator: Token,
}

impl SurfaceConditionEntry {
    pub(crate) fn new(
        span: SourceSpan,
        keyword: Token,
        name: Name,
        returns_keyword: Token,
        condition: SurfacePureCondition,
        terminator: Token,
    ) -> Self {
        Self {
            span,
            keyword,
            name,
            returns_keyword,
            condition,
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

    pub fn condition(&self) -> &SurfacePureCondition {
        &self.condition
    }

    pub fn terminator(&self) -> &Token {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureConditionBodyForm {
    Empty,
    Entry(SurfaceEntry),
    EntryCondition(SurfaceConditionEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureConditionUnit {
    type_effect_unit: TypeEffectUnit,
    form: PureConditionBodyForm,
    body_span: SourceSpan,
}

impl PureConditionUnit {
    pub(crate) fn new(
        type_effect_unit: TypeEffectUnit,
        form: PureConditionBodyForm,
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

    pub fn form(&self) -> &PureConditionBodyForm {
        &self.form
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }
}

pub fn analyze_pure_condition_unit(source: &SourceText) -> PureConditionResult<PureConditionUnit> {
    let type_effect_unit = analyze_type_effect_unit(source)?;
    PureConditionAnalyzer::new(source, type_effect_unit).analyze()
}

#[derive(Debug)]
pub struct PureConditionAnalyzer<'a> {
    source: &'a SourceText,
    type_effect_unit: TypeEffectUnit,
}

impl<'a> PureConditionAnalyzer<'a> {
    pub fn new(source: &'a SourceText, type_effect_unit: TypeEffectUnit) -> Self {
        Self {
            source,
            type_effect_unit,
        }
    }

    pub fn analyze(self) -> PureConditionResult<PureConditionUnit> {
        let body_span = self.type_effect_unit.body_span();
        let mut cursor = ElementCursor::after_offset(
            self.type_effect_unit.module_unit().file().elements(),
            body_span.start().get(),
        );

        let Some(first) = cursor.next_significant() else {
            return Ok(PureConditionUnit::new(
                self.type_effect_unit,
                PureConditionBodyForm::Empty,
                body_span,
            ));
        };

        let keyword = self.entry_keyword(first)?.clone();
        let name_element =
            cursor
                .next_significant()
                .ok_or(PureConditionError::ExpectedEntryName {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let name = self.entry_name(name_element)?;
        let continuation =
            cursor
                .next_significant()
                .ok_or(PureConditionError::ExpectedTerminatorOrReturns {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;

        if let Some(terminator) = self.try_terminator(continuation) {
            if let Some(extra) = cursor.next_significant() {
                return Err(PureConditionError::UnexpectedAfterEntry { span: extra.span() });
            }
            let terminator = terminator.clone();
            let entry_span = self
                .source
                .span(keyword.span().start(), terminator.span().end())?;
            let entry = SurfaceEntry::new(entry_span, keyword, name, terminator);
            return Ok(PureConditionUnit::new(
                self.type_effect_unit,
                PureConditionBodyForm::Entry(entry),
                body_span,
            ));
        }

        let returns_keyword = self.returns_keyword(continuation)?.clone();
        let mut condition_elements = Vec::new();
        let terminator = loop {
            let element = cursor.next_significant().ok_or(
                PureConditionError::ExpectedConditionTerminator {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                },
            )?;
            if let Some(terminator) = self.try_terminator(element) {
                break terminator.clone();
            }
            condition_elements.push(element);
        };

        if condition_elements.is_empty() {
            return Err(PureConditionError::ExpectedCondition {
                span: terminator.span(),
            });
        }
        if let Some(extra) = cursor.next_significant() {
            return Err(PureConditionError::UnexpectedAfterEntry { span: extra.span() });
        }

        let condition = self.parse_condition(&condition_elements)?;
        let entry_span = self
            .source
            .span(keyword.span().start(), terminator.span().end())?;
        let entry = SurfaceConditionEntry::new(
            entry_span,
            keyword,
            name,
            returns_keyword,
            condition,
            terminator,
        );

        Ok(PureConditionUnit::new(
            self.type_effect_unit,
            PureConditionBodyForm::EntryCondition(entry),
            body_span,
        ))
    }

    fn parse_condition(
        &self,
        elements: &[&AstElement],
    ) -> PureConditionResult<SurfacePureCondition> {
        let first = elements
            .first()
            .expect("pure condition parser requires non-empty elements");
        let last = elements
            .last()
            .expect("pure condition parser requires non-empty elements");
        let full_span = self.source.span(first.span().start(), last.span().end())?;

        if elements.len() == 1 {
            let Some(token) = elements[0].as_token() else {
                return Err(PureConditionError::InvalidBooleanOrComparison {
                    span: elements[0].span(),
                });
            };
            if token.kind() == &TokenKind::Identifier {
                match self.source.slice(token.span())? {
                    "true" => {
                        return Ok(SurfacePureCondition::new(SurfacePureConditionKind::Bool {
                            span: token.span(),
                            token: token.clone(),
                            value: true,
                        }));
                    }
                    "false" => {
                        return Ok(SurfacePureCondition::new(SurfacePureConditionKind::Bool {
                            span: token.span(),
                            token: token.clone(),
                            value: false,
                        }));
                    }
                    _ => {}
                }
            }
            return Err(PureConditionError::InvalidBooleanOrComparison { span: token.span() });
        }

        let lhs_token = self.integer_token(elements[0])?.clone();
        let lhs = self.integer_value(&lhs_token)?;
        let mut index = 1usize;
        let (comparator, comparator_span) = self.comparator(elements, &mut index)?;
        let Some(rhs_element) = elements.get(index).copied() else {
            return Err(PureConditionError::MissingRightOperand {
                span: comparator_span,
            });
        };
        let rhs_token = self.integer_token(rhs_element)?.clone();
        let rhs = self.integer_value(&rhs_token)?;
        index += 1;
        if let Some(extra) = elements.get(index) {
            return Err(PureConditionError::UnexpectedAfterCondition { span: extra.span() });
        }

        Ok(SurfacePureCondition::new(
            SurfacePureConditionKind::IntCompare {
                span: full_span,
                lhs_span: lhs_token.span(),
                lhs_token,
                lhs,
                comparator_span,
                comparator,
                rhs_span: rhs_token.span(),
                rhs_token,
                rhs,
            },
        ))
    }

    fn comparator(
        &self,
        elements: &[&AstElement],
        index: &mut usize,
    ) -> PureConditionResult<(SurfacePureComparator, SourceSpan)> {
        let Some(first_element) = elements.get(*index).copied() else {
            return Err(PureConditionError::InvalidComparator {
                span: elements[0].span(),
            });
        };
        let Some(first) = first_element.as_token() else {
            return Err(PureConditionError::InvalidComparator {
                span: first_element.span(),
            });
        };
        let TokenKind::Punctuation(first_char) = first.kind() else {
            return Err(PureConditionError::InvalidComparator { span: first.span() });
        };
        *index += 1;

        let second = elements.get(*index).and_then(|element| element.as_token());
        let adjacent_equals = second.filter(|token| {
            token.kind() == &TokenKind::Punctuation('=')
                && first.span().end().get() == token.span().start().get()
        });

        let (comparator, end) = match *first_char {
            '=' => {
                let Some(eq) = adjacent_equals else {
                    return Err(PureConditionError::InvalidComparator { span: first.span() });
                };
                *index += 1;
                (SurfacePureComparator::Eq, eq.span().end())
            }
            '!' => {
                let Some(eq) = adjacent_equals else {
                    return Err(PureConditionError::InvalidComparator { span: first.span() });
                };
                *index += 1;
                (SurfacePureComparator::Ne, eq.span().end())
            }
            '<' => match adjacent_equals {
                Some(eq) => {
                    *index += 1;
                    (SurfacePureComparator::Le, eq.span().end())
                }
                None => (SurfacePureComparator::Lt, first.span().end()),
            },
            '>' => match adjacent_equals {
                Some(eq) => {
                    *index += 1;
                    (SurfacePureComparator::Ge, eq.span().end())
                }
                None => (SurfacePureComparator::Gt, first.span().end()),
            },
            _ => return Err(PureConditionError::InvalidComparator { span: first.span() }),
        };

        let span = self.source.span(first.span().start(), end)?;
        Ok((comparator, span))
    }

    fn integer_token<'b>(&self, element: &'b AstElement) -> PureConditionResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureConditionError::InvalidIntegerLiteral {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::NumericCandidate {
            return Err(PureConditionError::InvalidIntegerLiteral { span: token.span() });
        }
        Ok(token)
    }

    fn integer_value(&self, token: &Token) -> PureConditionResult<i64> {
        let text = self.source.slice(token.span())?;
        if !is_canonical_non_negative_decimal(text) {
            return Err(PureConditionError::InvalidIntegerLiteral { span: token.span() });
        }
        text.parse::<i64>()
            .map_err(|_| PureConditionError::IntegerLiteralOutOfRange { span: token.span() })
    }

    fn entry_keyword<'b>(&self, element: &'b AstElement) -> PureConditionResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureConditionError::ExpectedEntryOrEnd {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier || self.source.slice(token.span())? != "entry" {
            return Err(PureConditionError::ExpectedEntryOrEnd { span: token.span() });
        }
        Ok(token)
    }

    fn entry_name(&self, element: &AstElement) -> PureConditionResult<Name> {
        let Some(token) = element.as_token() else {
            return Err(PureConditionError::ExpectedEntryName {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier {
            return Err(PureConditionError::ExpectedEntryName { span: token.span() });
        }
        Ok(Name::from_token(self.source, token)?)
    }

    fn returns_keyword<'b>(&self, element: &'b AstElement) -> PureConditionResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureConditionError::ExpectedTerminatorOrReturns {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier || self.source.slice(token.span())? != "returns" {
            return Err(PureConditionError::ExpectedTerminatorOrReturns { span: token.span() });
        }
        Ok(token)
    }

    fn try_terminator<'b>(&self, element: &'b AstElement) -> Option<&'b Token> {
        let token = element.as_token()?;
        (token.kind() == &TokenKind::Punctuation(';')).then_some(token)
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
