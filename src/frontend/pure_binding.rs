use super::ast::{AstElement, Delimiter};
use super::body::SurfaceEntry;
use super::lexer::{Token, TokenKind};
use super::name::Name;
use super::pure_binding_error::{PureBindingError, PureBindingResult};
use super::source::{SourceSpan, SourceText};
use super::type_effect::{analyze_type_effect_unit, TypeEffectUnit};
use super::MAX_PURE_EXPRESSION_NODES;

pub const MAX_PURE_BINDINGS: u32 = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePureBinding {
    span: SourceSpan,
    keyword: Token,
    name: Name,
    equals: Token,
    value_token: Token,
    value: i64,
    terminator: Token,
}

impl SurfacePureBinding {
    pub(crate) fn new(
        span: SourceSpan,
        keyword: Token,
        name: Name,
        equals: Token,
        value_token: Token,
        value: i64,
        terminator: Token,
    ) -> Self {
        Self {
            span,
            keyword,
            name,
            equals,
            value_token,
            value,
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

    pub fn equals(&self) -> &Token {
        &self.equals
    }

    pub fn value_token(&self) -> &Token {
        &self.value_token
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn terminator(&self) -> &Token {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfacePureBindingExpressionOp {
    Int {
        span: SourceSpan,
        token: Token,
        value: i64,
    },
    BindingRef {
        span: SourceSpan,
        token: Token,
        name: Name,
    },
    Add {
        span: SourceSpan,
        token: Token,
    },
}

impl SurfacePureBindingExpressionOp {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Int { span, .. } | Self::BindingRef { span, .. } | Self::Add { span, .. } => {
                *span
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePureBindingExpression {
    span: SourceSpan,
    ops: Vec<SurfacePureBindingExpressionOp>,
}

impl SurfacePureBindingExpression {
    pub(crate) fn new(span: SourceSpan, ops: Vec<SurfacePureBindingExpressionOp>) -> Self {
        Self { span, ops }
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn ops(&self) -> &[SurfacePureBindingExpressionOp] {
        &self.ops
    }

    pub fn node_count(&self) -> usize {
        self.ops.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePureBindingEntry {
    span: SourceSpan,
    keyword: Token,
    name: Name,
    returns_keyword: Token,
    expression: SurfacePureBindingExpression,
    terminator: Token,
}

impl SurfacePureBindingEntry {
    pub(crate) fn new(
        span: SourceSpan,
        keyword: Token,
        name: Name,
        returns_keyword: Token,
        expression: SurfacePureBindingExpression,
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

    pub fn expression(&self) -> &SurfacePureBindingExpression {
        &self.expression
    }

    pub fn terminator(&self) -> &Token {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureBindingBodyForm {
    Empty,
    Entry(SurfaceEntry),
    EntryExpression(SurfacePureBindingEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureBindingUnit {
    type_effect_unit: TypeEffectUnit,
    bindings: Vec<SurfacePureBinding>,
    form: PureBindingBodyForm,
    body_span: SourceSpan,
}

impl PureBindingUnit {
    pub(crate) fn new(
        type_effect_unit: TypeEffectUnit,
        bindings: Vec<SurfacePureBinding>,
        form: PureBindingBodyForm,
        body_span: SourceSpan,
    ) -> Self {
        Self {
            type_effect_unit,
            bindings,
            form,
            body_span,
        }
    }

    pub fn type_effect_unit(&self) -> &TypeEffectUnit {
        &self.type_effect_unit
    }

    pub fn bindings(&self) -> &[SurfacePureBinding] {
        &self.bindings
    }

    pub fn form(&self) -> &PureBindingBodyForm {
        &self.form
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }
}

pub fn analyze_pure_binding_unit(source: &SourceText) -> PureBindingResult<PureBindingUnit> {
    let type_effect_unit = analyze_type_effect_unit(source)?;
    PureBindingAnalyzer::new(source, type_effect_unit).analyze()
}

#[derive(Debug)]
pub struct PureBindingAnalyzer<'a> {
    source: &'a SourceText,
    type_effect_unit: TypeEffectUnit,
}

impl<'a> PureBindingAnalyzer<'a> {
    pub fn new(source: &'a SourceText, type_effect_unit: TypeEffectUnit) -> Self {
        Self {
            source,
            type_effect_unit,
        }
    }

    pub fn analyze(self) -> PureBindingResult<PureBindingUnit> {
        let body_span = self.type_effect_unit.body_span();
        let mut cursor = ElementCursor::after_offset(
            self.type_effect_unit.module_unit().file().elements(),
            body_span.start().get(),
        );
        let mut bindings = Vec::new();

        loop {
            let Some(first) = cursor.next_significant() else {
                return Ok(PureBindingUnit::new(
                    self.type_effect_unit,
                    bindings,
                    PureBindingBodyForm::Empty,
                    body_span,
                ));
            };

            if self.is_keyword(first, "const")? {
                if bindings.len() >= MAX_PURE_BINDINGS as usize {
                    return Err(PureBindingError::TooManyBindings {
                        maximum: MAX_PURE_BINDINGS,
                        span: first.span(),
                    });
                }
                bindings.push(self.parse_binding(first, &mut cursor)?);
                continue;
            }

            if !self.is_keyword(first, "entry")? {
                return Err(PureBindingError::ExpectedConstOrEntryOrEnd { span: first.span() });
            }

            let form = self.parse_entry(first, &mut cursor)?;
            return Ok(PureBindingUnit::new(
                self.type_effect_unit,
                bindings,
                form,
                body_span,
            ));
        }
    }

    fn parse_binding(
        &self,
        first: &AstElement,
        cursor: &mut ElementCursor<'_>,
    ) -> PureBindingResult<SurfacePureBinding> {
        let keyword = first
            .as_token()
            .expect("contextual const must be an identifier token")
            .clone();
        let name_element =
            cursor
                .next_significant()
                .ok_or(PureBindingError::ExpectedBindingName {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let name = self.binding_name(name_element)?;

        let equals_element =
            cursor
                .next_significant()
                .ok_or(PureBindingError::ExpectedBindingEquals {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let equals = self.punctuation(
            equals_element,
            '=',
            PureBindingError::ExpectedBindingEquals {
                span: equals_element.span(),
            },
        )?;

        let value_element =
            cursor
                .next_significant()
                .ok_or(PureBindingError::ExpectedBindingValue {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let value_token =
            value_element
                .as_token()
                .ok_or(PureBindingError::ExpectedBindingValue {
                    span: value_element.span(),
                })?;
        if value_token.kind() != &TokenKind::NumericCandidate {
            return Err(PureBindingError::ExpectedBindingValue {
                span: value_token.span(),
            });
        }
        let text = self.source.slice(value_token.span())?;
        if !is_canonical_non_negative_decimal(text) {
            return Err(PureBindingError::InvalidBindingIntegerLiteral {
                span: value_token.span(),
            });
        }
        let value =
            text.parse::<i64>()
                .map_err(|_| PureBindingError::BindingIntegerLiteralOutOfRange {
                    span: value_token.span(),
                })?;

        let terminator_element =
            cursor
                .next_significant()
                .ok_or(PureBindingError::ExpectedBindingTerminator {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let terminator = self.punctuation(
            terminator_element,
            ';',
            PureBindingError::ExpectedBindingTerminator {
                span: terminator_element.span(),
            },
        )?;
        let span = self
            .source
            .span(keyword.span().start(), terminator.span().end())?;

        Ok(SurfacePureBinding::new(
            span,
            keyword,
            name,
            equals.clone(),
            value_token.clone(),
            value,
            terminator.clone(),
        ))
    }

    fn parse_entry(
        &self,
        first: &AstElement,
        cursor: &mut ElementCursor<'_>,
    ) -> PureBindingResult<PureBindingBodyForm> {
        let keyword = first
            .as_token()
            .expect("contextual entry must be an identifier token")
            .clone();
        let name_element =
            cursor
                .next_significant()
                .ok_or(PureBindingError::ExpectedEntryName {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;
        let name = self.entry_name(name_element)?;
        let continuation =
            cursor
                .next_significant()
                .ok_or(PureBindingError::ExpectedTerminatorOrReturns {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                })?;

        if let Some(terminator) = self.try_terminator(continuation) {
            if let Some(extra) = cursor.next_significant() {
                return Err(PureBindingError::UnexpectedAfterEntry { span: extra.span() });
            }
            let terminator = terminator.clone();
            let entry_span = self
                .source
                .span(keyword.span().start(), terminator.span().end())?;
            return Ok(PureBindingBodyForm::Entry(SurfaceEntry::new(
                entry_span, keyword, name, terminator,
            )));
        }

        let returns_keyword = self.returns_keyword(continuation)?.clone();
        let mut expression_elements = Vec::new();
        let terminator = loop {
            let element = cursor.next_significant().ok_or(
                PureBindingError::ExpectedExpressionTerminator {
                    span: self.type_effect_unit.module_unit().file().eof().span(),
                },
            )?;
            if let Some(terminator) = self.try_terminator(element) {
                break terminator.clone();
            }
            expression_elements.push(element);
        };

        if expression_elements.is_empty() {
            return Err(PureBindingError::ExpectedExpression {
                span: terminator.span(),
            });
        }

        if let Some(extra) = cursor.next_significant() {
            return Err(PureBindingError::UnexpectedAfterEntry { span: extra.span() });
        }

        let expression = self.parse_expression(&expression_elements)?;
        let entry_span = self
            .source
            .span(keyword.span().start(), terminator.span().end())?;
        Ok(PureBindingBodyForm::EntryExpression(
            SurfacePureBindingEntry::new(
                entry_span,
                keyword,
                name,
                returns_keyword,
                expression,
                terminator,
            ),
        ))
    }

    fn parse_expression(
        &self,
        elements: &[&AstElement],
    ) -> PureBindingResult<SurfacePureBindingExpression> {
        let first = elements
            .first()
            .expect("pure binding expression parser requires non-empty elements");
        let last = elements
            .last()
            .expect("pure binding expression parser requires non-empty elements");
        let span = self.source.span(first.span().start(), last.span().end())?;
        let mut ops = Vec::new();
        self.parse_expression_sequence(elements, &mut ops)?;
        Ok(SurfacePureBindingExpression::new(span, ops))
    }

    fn parse_expression_sequence(
        &self,
        elements: &[&AstElement],
        ops: &mut Vec<SurfacePureBindingExpressionOp>,
    ) -> PureBindingResult<()> {
        let mut index = 0usize;
        self.parse_operand(elements, &mut index, ops)?;

        while index < elements.len() {
            let operator = elements[index];
            index += 1;
            let token = operator
                .as_token()
                .ok_or(PureBindingError::ExpectedPlusOrEnd {
                    span: operator.span(),
                })?;
            if token.kind() != &TokenKind::Punctuation('+') {
                return Err(PureBindingError::ExpectedPlusOrEnd { span: token.span() });
            }

            self.parse_operand(elements, &mut index, ops)?;
            self.push_op(
                ops,
                SurfacePureBindingExpressionOp::Add {
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
        ops: &mut Vec<SurfacePureBindingExpressionOp>,
    ) -> PureBindingResult<()> {
        let Some(element) = elements.get(*index).copied() else {
            let span = elements
                .last()
                .map(|element| element.span())
                .unwrap_or_else(|| self.type_effect_unit.module_unit().file().eof().span());
            return Err(PureBindingError::ExpectedOperand { span });
        };
        *index += 1;

        if let Some(token) = element.as_token() {
            match token.kind() {
                TokenKind::NumericCandidate => {
                    let text = self.source.slice(token.span())?;
                    if !is_canonical_non_negative_decimal(text) {
                        return Err(PureBindingError::InvalidIntegerLiteral { span: token.span() });
                    }
                    let value = text.parse::<i64>().map_err(|_| {
                        PureBindingError::IntegerLiteralOutOfRange { span: token.span() }
                    })?;
                    return self.push_op(
                        ops,
                        SurfacePureBindingExpressionOp::Int {
                            span: token.span(),
                            token: token.clone(),
                            value,
                        },
                        token.span(),
                    );
                }
                TokenKind::Identifier => {
                    let name = Name::from_token(self.source, token)?;
                    return self.push_op(
                        ops,
                        SurfacePureBindingExpressionOp::BindingRef {
                            span: token.span(),
                            token: token.clone(),
                            name,
                        },
                        token.span(),
                    );
                }
                _ => {
                    return Err(PureBindingError::ExpectedOperand { span: token.span() });
                }
            }
        }

        let group = element
            .as_group()
            .expect("AST element must be either token or group");
        if group.delimiter() != Delimiter::Parenthesis {
            return Err(PureBindingError::UnsupportedGroup { span: group.span() });
        }
        let inner = significant_elements(group.elements());
        if inner.is_empty() {
            return Err(PureBindingError::EmptyParenthesizedExpression { span: group.span() });
        }
        self.parse_expression_sequence(&inner, ops)
    }

    fn push_op(
        &self,
        ops: &mut Vec<SurfacePureBindingExpressionOp>,
        op: SurfacePureBindingExpressionOp,
        span: SourceSpan,
    ) -> PureBindingResult<()> {
        if ops.len() >= MAX_PURE_EXPRESSION_NODES as usize {
            return Err(PureBindingError::TooManyExpressionNodes {
                maximum: MAX_PURE_EXPRESSION_NODES,
                span,
            });
        }
        ops.push(op);
        Ok(())
    }

    fn is_keyword(&self, element: &AstElement, expected: &str) -> PureBindingResult<bool> {
        let Some(token) = element.as_token() else {
            return Ok(false);
        };
        if token.kind() != &TokenKind::Identifier {
            return Ok(false);
        }
        Ok(self.source.slice(token.span())? == expected)
    }

    fn binding_name(&self, element: &AstElement) -> PureBindingResult<Name> {
        let Some(token) = element.as_token() else {
            return Err(PureBindingError::ExpectedBindingName {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier {
            return Err(PureBindingError::ExpectedBindingName { span: token.span() });
        }
        Ok(Name::from_token(self.source, token)?)
    }

    fn entry_name(&self, element: &AstElement) -> PureBindingResult<Name> {
        let Some(token) = element.as_token() else {
            return Err(PureBindingError::ExpectedEntryName {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier {
            return Err(PureBindingError::ExpectedEntryName { span: token.span() });
        }
        Ok(Name::from_token(self.source, token)?)
    }

    fn punctuation<'b>(
        &self,
        element: &'b AstElement,
        expected: char,
        error: PureBindingError,
    ) -> PureBindingResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(error);
        };
        if token.kind() != &TokenKind::Punctuation(expected) {
            return Err(error);
        }
        Ok(token)
    }

    fn try_terminator<'b>(&self, element: &'b AstElement) -> Option<&'b Token> {
        let token = element.as_token()?;
        (token.kind() == &TokenKind::Punctuation(';')).then_some(token)
    }

    fn returns_keyword<'b>(&self, element: &'b AstElement) -> PureBindingResult<&'b Token> {
        let Some(token) = element.as_token() else {
            return Err(PureBindingError::ExpectedTerminatorOrReturns {
                span: element.span(),
            });
        };
        if token.kind() != &TokenKind::Identifier || self.source.slice(token.span())? != "returns" {
            return Err(PureBindingError::ExpectedTerminatorOrReturns { span: token.span() });
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
