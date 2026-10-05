use super::ast::{AstElement, Delimiter};
use super::body::SurfaceEntry;
use super::lexer::{Token, TokenKind};
use super::name::Name;
use super::pure_expression_error::{PureExpressionError, PureExpressionResult};
use super::source::{SourceSpan, SourceText};
use super::type_effect::{analyze_type_effect_unit, TypeEffectUnit};

pub const MAX_PURE_EXPRESSION_NODES: u32 = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfacePureExpressionOp {
    Int {
        span: SourceSpan,
        token: Token,
        value: i64,
    },
    Add {
        span: SourceSpan,
        token: Token,
    },
}

impl SurfacePureExpressionOp {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Int { span, .. } | Self::Add { span, .. } => *span,
        }
    }

    pub fn int_value(&self) -> Option<i64> {
        match self {
            Self::Int { value, .. } => Some(*value),
            Self::Add { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePureExpression {
    span: SourceSpan,
    ops: Vec<SurfacePureExpressionOp>,
}

impl SurfacePureExpression {
    pub(crate) fn new(span: SourceSpan, ops: Vec<SurfacePureExpressionOp>) -> Self {
        Self { span, ops }
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn ops(&self) -> &[SurfacePureExpressionOp] {
        &self.ops
    }

    pub fn node_count(&self) -> usize {
        self.ops.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceExpressionEntry {
    span: SourceSpan,
    keyword: Token,
    name: Name,
    returns_keyword: Token,
    expression: SurfacePureExpression,
    terminator: Token,
}

impl SurfaceExpressionEntry {
    pub(crate) fn new(
        span: SourceSpan,
        keyword: Token,
        name: Name,
        returns_keyword: Token,
        expression: SurfacePureExpression,
        terminator: Token,
    ) -> Self {
        Self {
            span,
            keyword,
            name,
            returns_keyword,
            expression,
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

    pub fn expression(&self) -> &SurfacePureExpression {
        &self.expression
    }

    pub fn terminator(&self) -> &Token {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureExpressionBodyForm {
    Empty,
    Entry(SurfaceEntry),
    EntryExpression(SurfaceExpressionEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureExpressionUnit {
    type_effect_unit: TypeEffectUnit,
    form: PureExpressionBodyForm,
    body_span: SourceSpan,
}

impl PureExpressionUnit {
    pub(crate) fn new(
        type_effect_unit: TypeEffectUnit,
        form: PureExpressionBodyForm,
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

    pub fn form(&self) -> &PureExpressionBodyForm {
        &self.form
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }
}

pub fn analyze_pure_expression_unit(
    source: &SourceText,
) -> PureExpressionResult<PureExpressionUnit> {
    let type_effect_unit = analyze_type_effect_unit(source)?;
    PureExpressionAnalyzer::new(source, type_effect_unit).analyze()
}

#[derive(Debug)]
pub struct PureExpressionAnalyzer<'a> {
    source: &'a SourceText,
    type_effect_unit: TypeEffectUnit,
}

impl<'a> PureExpressionAnalyzer<'a> {
    pub fn new(source: &'a SourceText, type_effect_unit: TypeEffectUnit) -> Self {
        Self {
            source,
            type_effect_unit,
        }
    }

    pub fn analyze(self) -> PureExpressionResult<PureExpressionUnit> {
        let body_span = self.type_effect_unit.body_span();
        let mut cursor = ElementCursor::after_offset(
            self.type_effect_unit.module_unit().file().elements(),
            body_span.start().get(),
        );

        let Some(first) = cursor.next_significant() else {
            return Ok(PureExpressionUnit::new(
                self.type_effect_unit,
                PureExpressionBodyForm::Empty,
                body_span,
            ));
        };

        let keyword = self.entry_keyword(first)?.clone();
        let name_element =
            cursor
                .next_significant()
                .ok_or(PureExpressionError::ExpectedEntryName {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let name = self.entry_name(name_element)?;
        let continuation =
            cursor
                .next_significant()
                .ok_or(PureExpressionError::ExpectedTerminatorOrReturns {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;

        if let Some(terminator) = self.try_terminator(continuation) {
            if let Some(extra) = cursor.next_significant() {
                return Err(PureExpressionError::UnexpectedAfterEntry { span: extra.span() });
            }
            let terminator = terminator.clone();
            let entry_span = self
                .source
                .span(keyword.span().start(), terminator.span().end())?;
            let entry = SurfaceEntry::new(entry_span, keyword, name, terminator);
            return Ok(PureExpressionUnit::new(
                self.type_effect_unit,
                PureExpressionBodyForm::Entry(entry),
                body_span,
            ));
        }

        let returns_keyword = self.returns_keyword(continuation)?.clone();
        let mut expression_elements = Vec::new();
        let terminator = loop {
            let element = cursor.next_significant().ok_or(
                PureExpressionError::ExpectedExpressionTerminator {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                },
            )?;
            if let Some(terminator) = self.try_terminator(element) {
                break terminator.clone();
            }
            expression_elements.push(element);
        };

        if expression_elements.is_empty() {
            return Err(PureExpressionError::ExpectedExpression {
                span: terminator.span(),
            });
        }

        if let Some(extra) = cursor.next_significant() {
            return Err(PureExpressionError::UnexpectedAfterEntry { span: extra.span() });
        }

        let expression = self.parse_expression(&expression_elements)?;
        let entry_span = self
            .source
            .span(keyword.span().start(), terminator.span().end())?;
        let entry = SurfaceExpressionEntry::new(
            entry_span,
            keyword,
            name,
            returns_keyword,
            expression,
            terminator,
        );

        Ok(PureExpressionUnit::new(
            self.type_effect_unit,
            PureExpressionBodyForm::EntryExpression(entry),
            body_span,
        ))
    }

    fn parse_expression(
        &self,
        elements: &[&AstElement],
    ) -> PureExpressionResult<SurfacePureExpression> {
        let first = elements
            .first()
            .expect("pure expression parser requires a non-empty element sequence");
        let last = elements
            .last()
            .expect("pure expression parser requires a non-empty element sequence");
        let span = self.source.span(first.span().start(), last.span().end())?;
        let mut ops = Vec::new();
        self.parse_expression_sequence(elements, &mut ops)?;
        Ok(SurfacePureExpression::new(span, ops))
    }

    fn parse_expression_sequence(
        &self,
        elements: &[&AstElement],
        ops: &mut Vec<SurfacePureExpressionOp>,
    ) -> PureExpressionResult<()> {
        let mut index = 0usize;
        self.parse_operand(elements, &mut index, ops)?;

        while index < elements.len() {
            let operator = elements[index];
            index += 1;
            let token = operator
                .as_token()
                .ok_or(PureExpressionError::ExpectedPlusOrEnd {
                    span: operator.span(),
                })?;
            if token.kind() != &TokenKind::Punctuation('+') {
                return Err(PureExpressionError::ExpectedPlusOrEnd { span: token.span() });
            }

            self.parse_operand(elements, &mut index, ops)?;
            self.push_op(
                ops,
                SurfacePureExpressionOp::Add {
                    span: token.span(),
                    token: token.clone(),
                },
                token.span(),
            )?;
        }

        Ok(())
    }

    fn parse_operand(
        &self,
        elements: &[&AstElement],
        index: &mut usize,
        ops: &mut Vec<SurfacePureExpressionOp>,
    ) -> PureExpressionResult<()> {
        let Some(element) = elements.get(*index).copied() else {
            let span = elements
                .last()
                .map(|element| element.span())
                .unwrap_or_else(|| self.type_effect_unit.module_unit().file().eof().span());
            return Err(PureExpressionError::ExpectedOperand { span });
        };
        *index += 1;

        if let Some(token) = element.as_token() {
            if token.kind() != &TokenKind::NumericCandidate {
                return Err(PureExpressionError::ExpectedOperand { span: token.span() });
            }
            let text = self.source.slice(token.span())?;
            if !is_canonical_non_negative_decimal(text) {
                return Err(PureExpressionError::InvalidIntegerLiteral { span: token.span() });
            }
            let value =
                text.parse::<i64>()
                    .map_err(|_| PureExpressionError::IntegerLiteralOutOfRange {
                        span: token.span(),
                    })?;
            return self.push_op(
                ops,
                SurfacePureExpressionOp::Int {
                    span: token.span(),
                    token: token.clone(),
                    value,
                },
                token.span(),
            );
        }

        let group = element
            .as_group()
            .expect("AST element must be either token or group");
        if group.delimiter() != Delimiter::Parenthesis {
            return Err(PureExpressionError::UnsupportedGroup { span: group.span() });
        }
        let inner = significant_elements(group.elements());
        if inner.is_empty() {
            return Err(PureExpressionError::EmptyParenthesizedExpression { span: group.span() });
        }
        self.parse_expression_sequence(&inner, ops)
    }

    fn push_op(
        &self,
        ops: &mut Vec<SurfacePureExpressionOp>,
        op: SurfacePureExpressionOp,
        span: SourceSpan,
    ) -> PureExpressionResult<()> {
        if ops.len() >= MAX_PURE_EXPRESSION_NODES as usize {
            return Err(PureExpressionError::TooManyExpressionNodes {
                maximum: MAX_PURE_EXPRESSION_NODES,
                span,
            });
        }
        ops.push(op);
        Ok(())
    }

    fn entry_keyword<'b>(&self, element: &'b AstElement) -> PureExpressionResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureExpressionError::ExpectedEntryOrEnd {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier || self.source.slice(token.span())? != "entry" {
            return Err(PureExpressionError::ExpectedEntryOrEnd { span: token.span() });
        }
        Ok(token)
    }

    fn entry_name(&self, element: &AstElement) -> PureExpressionResult<Name> {
        let Some(token) = element.as_token() else {
            return Err(PureExpressionError::ExpectedEntryName {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier {
            return Err(PureExpressionError::ExpectedEntryName { span: token.span() });
        }
        Ok(Name::from_token(self.source, token)?)
    }

    fn try_terminator<'b>(&self, element: &'b AstElement) -> Option<&'b Token> {
        let token = element.as_token()?;
        (token.kind() == &TokenKind::Punctuation(';')).then_some(token)
    }

    fn returns_keyword<'b>(&self, element: &'b AstElement) -> PureExpressionResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureExpressionError::ExpectedTerminatorOrReturns {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier || self.source.slice(token.span())? != "returns" {
            return Err(PureExpressionError::ExpectedTerminatorOrReturns { span: token.span() });
        }
        Ok(token)
    }
}

fn significant_elements(elements: &[AstElement]) -> Vec<&AstElement> {
    elements
        .iter()
        .filter(|element| !matches!(element, AstElement::Token(token) if token.is_trivia()))
        .collect()
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
