use super::effects::SemanticEffectSet;
use super::error::{CompilerError, CompilerResult};
use super::hir::{HirUnit, SemanticName};
use super::lower::lower_type_effect_unit_to_hir;
use super::nsir::{validate_hir, NsirUnit};
use super::symbols::{resolve_effect_set, ResolvedEffectSet};
use crate::frontend::{
    analyze_pure_expression_unit, PureExpressionBodyForm, PureExpressionUnit, SourceSpan,
    SourceText, SurfacePureExpressionOp, MAX_PURE_EXPRESSION_NODES,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPureExpressionOp {
    Int { value: i64, origin_span: SourceSpan },
    Add { origin_span: SourceSpan },
}

impl HirPureExpressionOp {
    pub fn int(value: i64, origin_span: SourceSpan) -> Self {
        Self::Int { value, origin_span }
    }

    pub fn add(origin_span: SourceSpan) -> Self {
        Self::Add { origin_span }
    }

    pub fn origin_span(&self) -> SourceSpan {
        match self {
            Self::Int { origin_span, .. } | Self::Add { origin_span } => *origin_span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureExpression {
    origin_span: SourceSpan,
    ops: Vec<HirPureExpressionOp>,
}

impl HirPureExpression {
    pub fn new(origin_span: SourceSpan, ops: Vec<HirPureExpressionOp>) -> Self {
        Self { origin_span, ops }
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn ops(&self) -> &[HirPureExpressionOp] {
        &self.ops
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureExpressionEntry {
    name: SemanticName,
    origin_span: SourceSpan,
    expression: Option<HirPureExpression>,
}

impl HirPureExpressionEntry {
    pub fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        expression: Option<HirPureExpression>,
    ) -> Self {
        Self {
            name,
            origin_span,
            expression,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn expression(&self) -> Option<&HirPureExpression> {
        self.expression.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPureExpressionForm {
    Empty,
    Entry(HirPureExpressionEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureExpressionUnit {
    semantic: HirUnit,
    body_span: SourceSpan,
    form: HirPureExpressionForm,
}

impl HirPureExpressionUnit {
    pub fn new(semantic: HirUnit, body_span: SourceSpan, form: HirPureExpressionForm) -> Self {
        Self {
            semantic,
            body_span,
            form,
        }
    }

    pub fn semantic(&self) -> &HirUnit {
        &self.semantic
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }

    pub fn form(&self) -> &HirPureExpressionForm {
        &self.form
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticPureExpressionOp {
    Int(i64),
    Add,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureExpression {
    origin_span: SourceSpan,
    ops: Vec<SemanticPureExpressionOp>,
    value: i64,
}

impl NsirPureExpression {
    pub(crate) fn new(
        origin_span: SourceSpan,
        ops: Vec<SemanticPureExpressionOp>,
        value: i64,
    ) -> Self {
        Self {
            origin_span,
            ops,
            value,
        }
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn ops(&self) -> &[SemanticPureExpressionOp] {
        &self.ops
    }

    pub fn node_count(&self) -> usize {
        self.ops.len()
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureExpressionEntry {
    name: SemanticName,
    origin_span: SourceSpan,
    required_effects: ResolvedEffectSet,
    expression: Option<NsirPureExpression>,
}

impl NsirPureExpressionEntry {
    pub(crate) fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        required_effects: ResolvedEffectSet,
        expression: Option<NsirPureExpression>,
    ) -> Self {
        Self {
            name,
            origin_span,
            required_effects,
            expression,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn required_effects(&self) -> &ResolvedEffectSet {
        &self.required_effects
    }

    pub fn is_pure(&self) -> bool {
        self.required_effects.is_pure()
    }

    pub fn expression(&self) -> Option<&NsirPureExpression> {
        self.expression.as_ref()
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.expression.as_ref().map(NsirPureExpression::value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NsirPureExpressionForm {
    Empty,
    Entry(NsirPureExpressionEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureExpressionUnit {
    semantic: NsirUnit,
    form: NsirPureExpressionForm,
}

impl NsirPureExpressionUnit {
    pub(crate) fn new(semantic: NsirUnit, form: NsirPureExpressionForm) -> Self {
        Self { semantic, form }
    }

    pub fn semantic(&self) -> &NsirUnit {
        &self.semantic
    }

    pub fn form(&self) -> &NsirPureExpressionForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&NsirPureExpressionEntry> {
        match &self.form {
            NsirPureExpressionForm::Empty => None,
            NsirPureExpressionForm::Entry(entry) => Some(entry),
        }
    }

    pub fn expression(&self) -> Option<&NsirPureExpression> {
        self.entry().and_then(NsirPureExpressionEntry::expression)
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.entry().and_then(NsirPureExpressionEntry::result_i64)
    }

    pub fn is_pure(&self) -> bool {
        match self.entry() {
            Some(entry) => entry.is_pure(),
            None => true,
        }
    }

    /// L0.7 pure-expression witness. Earlier witnesses remain unchanged.
    /// Source spans, comments, whitespace, source IDs and host authority are excluded.
    /// Expression evaluation order is retained through canonical postfix operations.
    pub fn canonical_l07_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-L0.7-PURE-EXPRESSION\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let semantic = self.semantic.canonical_c02_bytes();
        bytes.extend_from_slice(&(semantic.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&semantic);

        match &self.form {
            NsirPureExpressionForm::Empty => bytes.push(0),
            NsirPureExpressionForm::Entry(entry) => {
                bytes.push(1);
                let name = entry.name().as_str().as_bytes();
                bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
                bytes.extend_from_slice(name);
                bytes.extend_from_slice(
                    &(entry.required_effects().effects().len() as u32).to_be_bytes(),
                );
                for effect in entry.required_effects().effects() {
                    bytes.extend_from_slice(&effect.get().to_be_bytes());
                }

                match entry.expression() {
                    None => bytes.push(0),
                    Some(expression) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&(expression.ops().len() as u32).to_be_bytes());
                        for op in expression.ops() {
                            match op {
                                SemanticPureExpressionOp::Int(value) => {
                                    bytes.push(1);
                                    bytes.extend_from_slice(&value.to_be_bytes());
                                }
                                SemanticPureExpressionOp::Add => bytes.push(2),
                            }
                        }
                        bytes.extend_from_slice(&expression.value().to_be_bytes());
                    }
                }
            }
        }

        bytes
    }
}

pub fn lower_pure_expression_unit_to_hir(
    source: &SourceText,
    unit: &PureExpressionUnit,
) -> CompilerResult<HirPureExpressionUnit> {
    let semantic = lower_type_effect_unit_to_hir(source, unit.type_effect_unit())?;
    let form = match unit.form() {
        PureExpressionBodyForm::Empty => HirPureExpressionForm::Empty,
        PureExpressionBodyForm::Entry(entry) => {
            HirPureExpressionForm::Entry(HirPureExpressionEntry::new(
                SemanticName::new(entry.name().text(source)?.to_owned())?,
                entry.span(),
                None,
            ))
        }
        PureExpressionBodyForm::EntryExpression(entry) => {
            let mut ops = Vec::with_capacity(entry.expression().ops().len());
            for op in entry.expression().ops() {
                match op {
                    SurfacePureExpressionOp::Int { span, value, .. } => {
                        ops.push(HirPureExpressionOp::int(*value, *span));
                    }
                    SurfacePureExpressionOp::Add { span, .. } => {
                        ops.push(HirPureExpressionOp::add(*span));
                    }
                }
            }
            HirPureExpressionForm::Entry(HirPureExpressionEntry::new(
                SemanticName::new(entry.name().text(source)?.to_owned())?,
                entry.span(),
                Some(HirPureExpression::new(entry.expression().span(), ops)),
            ))
        }
    };

    Ok(HirPureExpressionUnit::new(semantic, unit.body_span(), form))
}

pub fn validate_pure_expression_hir(
    unit: HirPureExpressionUnit,
) -> CompilerResult<NsirPureExpressionUnit> {
    let HirPureExpressionUnit {
        semantic,
        body_span,
        form,
    } = unit;

    let semantic_body = semantic.body_span();
    if semantic_body != body_span {
        return Err(CompilerError::PureExpressionBodySpanMismatch {
            semantic_body,
            body: body_span,
        });
    }

    if let HirPureExpressionForm::Entry(entry) = &form {
        if entry.origin_span().source() != body_span.source() {
            return Err(CompilerError::SourceMismatch {
                file: body_span,
                other: entry.origin_span(),
            });
        }
        if !contains(body_span, entry.origin_span()) {
            return Err(CompilerError::PureExpressionEntrySpanOutsideBody {
                body: body_span,
                entry: entry.origin_span(),
            });
        }
        if let Some(expression) = entry.expression() {
            validate_expression_spans(entry.origin_span(), expression)?;
        }
    }

    let semantic = validate_hir(semantic)?;
    let form = match form {
        HirPureExpressionForm::Empty => NsirPureExpressionForm::Empty,
        HirPureExpressionForm::Entry(entry) => {
            let required_effects =
                resolve_effect_set(semantic.registry(), &SemanticEffectSet::empty())?;
            let expression = match entry.expression {
                Some(expression) => Some(validate_expression(expression)?),
                None => None,
            };
            NsirPureExpressionForm::Entry(NsirPureExpressionEntry::new(
                entry.name,
                entry.origin_span,
                required_effects,
                expression,
            ))
        }
    };

    Ok(NsirPureExpressionUnit::new(semantic, form))
}

fn validate_expression_spans(
    entry_span: SourceSpan,
    expression: &HirPureExpression,
) -> CompilerResult<()> {
    if expression.origin_span().source() != entry_span.source() {
        return Err(CompilerError::SourceMismatch {
            file: entry_span,
            other: expression.origin_span(),
        });
    }
    if !contains(entry_span, expression.origin_span()) {
        return Err(CompilerError::PureExpressionSpanOutsideEntry {
            entry: entry_span,
            expression: expression.origin_span(),
        });
    }
    if expression.ops().len() > MAX_PURE_EXPRESSION_NODES as usize {
        return Err(CompilerError::TooManyPureExpressionNodes {
            count: expression.ops().len() as u32,
            maximum: MAX_PURE_EXPRESSION_NODES,
            span: expression.origin_span(),
        });
    }
    for op in expression.ops() {
        let span = op.origin_span();
        if span.source() != expression.origin_span().source() {
            return Err(CompilerError::SourceMismatch {
                file: expression.origin_span(),
                other: span,
            });
        }
        if !contains(expression.origin_span(), span) {
            return Err(CompilerError::PureExpressionNodeSpanOutsideExpression {
                expression: expression.origin_span(),
                node: span,
            });
        }
    }
    Ok(())
}

fn validate_expression(expression: HirPureExpression) -> CompilerResult<NsirPureExpression> {
    if expression.ops.is_empty() {
        return Err(CompilerError::InvalidPureExpressionPostfix {
            span: expression.origin_span,
        });
    }

    let mut stack = Vec::new();
    let mut semantic_ops = Vec::with_capacity(expression.ops.len());

    for op in expression.ops {
        match op {
            HirPureExpressionOp::Int { value, origin_span } => {
                if value < 0 {
                    return Err(CompilerError::NegativePureExpressionLiteral { span: origin_span });
                }
                stack.push(value);
                semantic_ops.push(SemanticPureExpressionOp::Int(value));
            }
            HirPureExpressionOp::Add { origin_span } => {
                if stack.len() < 2 {
                    return Err(CompilerError::InvalidPureExpressionPostfix { span: origin_span });
                }
                let right = stack
                    .pop()
                    .expect("validated postfix add requires right operand");
                let left = stack
                    .pop()
                    .expect("validated postfix add requires left operand");
                let value = left
                    .checked_add(right)
                    .ok_or(CompilerError::PureExpressionIntegerOverflow { span: origin_span })?;
                stack.push(value);
                semantic_ops.push(SemanticPureExpressionOp::Add);
            }
        }
    }

    if stack.len() != 1 {
        return Err(CompilerError::InvalidPureExpressionPostfix {
            span: expression.origin_span,
        });
    }
    let value = stack
        .pop()
        .expect("validated pure expression must leave exactly one value");

    Ok(NsirPureExpression::new(
        expression.origin_span,
        semantic_ops,
        value,
    ))
}

/// L0.7 pure-expression boundary. It understands empty bodies, certified L0.5 entries,
/// L0.6 literal results, and the new pure `+` expression subset. It evaluates only checked,
/// non-negative i64 addition. It does not create an execution plan, lower NAIR, invoke the
/// runtime, perform I/O, dispatch effects, consult capabilities, or grant host authority.
pub fn compile_pure_expression_boundary(
    source: &SourceText,
) -> CompilerResult<NsirPureExpressionUnit> {
    let unit = analyze_pure_expression_unit(source)?;
    let hir = lower_pure_expression_unit_to_hir(source, &unit)?;
    validate_pure_expression_hir(hir)
}

fn contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
