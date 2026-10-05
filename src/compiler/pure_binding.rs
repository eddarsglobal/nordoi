use super::effects::SemanticEffectSet;
use super::error::{CompilerError, CompilerResult};
use super::hir::{HirUnit, SemanticName};
use super::lower::lower_type_effect_unit_to_hir;
use super::nsir::{validate_hir, NsirUnit};
use super::symbols::{resolve_effect_set, ResolvedEffectSet};
use crate::frontend::{
    analyze_pure_binding_unit, PureBindingBodyForm, PureBindingUnit, SourceSpan, SourceText,
    SurfacePureBindingExpressionOp, MAX_PURE_BINDINGS, MAX_PURE_EXPRESSION_NODES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticPureBindingId(u32);

impl SemanticPureBindingId {
    fn new(raw: u32) -> Self {
        debug_assert!(raw != 0);
        Self(raw)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureBinding {
    name: SemanticName,
    origin_span: SourceSpan,
    value_span: SourceSpan,
    value: i64,
}

impl HirPureBinding {
    pub fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        value_span: SourceSpan,
        value: i64,
    ) -> Self {
        Self {
            name,
            origin_span,
            value_span,
            value,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn value_span(&self) -> SourceSpan {
        self.value_span
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPureBindingExpressionOp {
    Int {
        value: i64,
        origin_span: SourceSpan,
    },
    BindingRef {
        name: SemanticName,
        origin_span: SourceSpan,
    },
    Add {
        origin_span: SourceSpan,
    },
}

impl HirPureBindingExpressionOp {
    pub fn origin_span(&self) -> SourceSpan {
        match self {
            Self::Int { origin_span, .. }
            | Self::BindingRef { origin_span, .. }
            | Self::Add { origin_span } => *origin_span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureBindingExpression {
    origin_span: SourceSpan,
    ops: Vec<HirPureBindingExpressionOp>,
}

impl HirPureBindingExpression {
    pub fn new(origin_span: SourceSpan, ops: Vec<HirPureBindingExpressionOp>) -> Self {
        Self { origin_span, ops }
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn ops(&self) -> &[HirPureBindingExpressionOp] {
        &self.ops
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureBindingEntry {
    name: SemanticName,
    origin_span: SourceSpan,
    expression: Option<HirPureBindingExpression>,
}

impl HirPureBindingEntry {
    pub fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        expression: Option<HirPureBindingExpression>,
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

    pub fn expression(&self) -> Option<&HirPureBindingExpression> {
        self.expression.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPureBindingForm {
    Empty,
    Entry(HirPureBindingEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureBindingUnit {
    semantic: HirUnit,
    body_span: SourceSpan,
    bindings: Vec<HirPureBinding>,
    form: HirPureBindingForm,
}

impl HirPureBindingUnit {
    pub fn new(
        semantic: HirUnit,
        body_span: SourceSpan,
        bindings: Vec<HirPureBinding>,
        form: HirPureBindingForm,
    ) -> Self {
        Self {
            semantic,
            body_span,
            bindings,
            form,
        }
    }

    pub fn semantic(&self) -> &HirUnit {
        &self.semantic
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }

    pub fn bindings(&self) -> &[HirPureBinding] {
        &self.bindings
    }

    pub fn form(&self) -> &HirPureBindingForm {
        &self.form
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureBindingSymbol {
    id: SemanticPureBindingId,
    name: SemanticName,
    value: i64,
    origin_span: SourceSpan,
}

impl NsirPureBindingSymbol {
    fn new(
        id: SemanticPureBindingId,
        name: SemanticName,
        value: i64,
        origin_span: SourceSpan,
    ) -> Self {
        Self {
            id,
            name,
            value,
            origin_span,
        }
    }

    pub fn id(&self) -> SemanticPureBindingId {
        self.id
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureBindingRegistry {
    bindings: Vec<NsirPureBindingSymbol>,
}

impl PureBindingRegistry {
    fn from_hir(bindings: &[HirPureBinding]) -> CompilerResult<Self> {
        if bindings.len() > MAX_PURE_BINDINGS as usize {
            let span = bindings
                .last()
                .map(HirPureBinding::origin_span)
                .expect("oversized binding set must be non-empty");
            return Err(CompilerError::TooManyPureBindings {
                count: bindings.len() as u32,
                maximum: MAX_PURE_BINDINGS,
                span,
            });
        }

        let mut ordered = bindings.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.name().cmp(right.name()));
        for pair in ordered.windows(2) {
            if pair[0].name() == pair[1].name() {
                return Err(CompilerError::DuplicatePureBinding {
                    name: pair[1].name().as_str().to_owned(),
                    first: pair[0].origin_span(),
                    duplicate: pair[1].origin_span(),
                });
            }
        }

        let bindings = ordered
            .into_iter()
            .enumerate()
            .map(|(index, binding)| {
                NsirPureBindingSymbol::new(
                    SemanticPureBindingId::new(index as u32 + 1),
                    binding.name().clone(),
                    binding.value(),
                    binding.origin_span(),
                )
            })
            .collect();
        Ok(Self { bindings })
    }

    pub fn bindings(&self) -> &[NsirPureBindingSymbol] {
        &self.bindings
    }

    pub fn resolve(&self, name: &str) -> Option<&NsirPureBindingSymbol> {
        self.bindings
            .binary_search_by(|binding| binding.name().as_str().cmp(name))
            .ok()
            .map(|index| &self.bindings[index])
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-L0.8-PURE-BINDING-REGISTRY\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);
        bytes.extend_from_slice(&(self.bindings.len() as u32).to_be_bytes());
        for binding in &self.bindings {
            bytes.extend_from_slice(&binding.id().get().to_be_bytes());
            let name = binding.name().as_str().as_bytes();
            bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
            bytes.extend_from_slice(name);
            bytes.extend_from_slice(&binding.value().to_be_bytes());
        }
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticPureBindingExpressionOp {
    Int(i64),
    Binding(SemanticPureBindingId),
    Add,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureBindingExpression {
    origin_span: SourceSpan,
    ops: Vec<SemanticPureBindingExpressionOp>,
    value: i64,
}

impl NsirPureBindingExpression {
    fn new(origin_span: SourceSpan, ops: Vec<SemanticPureBindingExpressionOp>, value: i64) -> Self {
        Self {
            origin_span,
            ops,
            value,
        }
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn ops(&self) -> &[SemanticPureBindingExpressionOp] {
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
pub struct NsirPureBindingEntry {
    name: SemanticName,
    origin_span: SourceSpan,
    required_effects: ResolvedEffectSet,
    expression: Option<NsirPureBindingExpression>,
}

impl NsirPureBindingEntry {
    fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        required_effects: ResolvedEffectSet,
        expression: Option<NsirPureBindingExpression>,
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

    pub fn expression(&self) -> Option<&NsirPureBindingExpression> {
        self.expression.as_ref()
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.expression
            .as_ref()
            .map(NsirPureBindingExpression::value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NsirPureBindingForm {
    Empty,
    Entry(NsirPureBindingEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureBindingUnit {
    semantic: NsirUnit,
    bindings: PureBindingRegistry,
    form: NsirPureBindingForm,
}

impl NsirPureBindingUnit {
    fn new(semantic: NsirUnit, bindings: PureBindingRegistry, form: NsirPureBindingForm) -> Self {
        Self {
            semantic,
            bindings,
            form,
        }
    }

    pub fn semantic(&self) -> &NsirUnit {
        &self.semantic
    }

    pub fn bindings(&self) -> &PureBindingRegistry {
        &self.bindings
    }

    pub fn form(&self) -> &NsirPureBindingForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&NsirPureBindingEntry> {
        match &self.form {
            NsirPureBindingForm::Empty => None,
            NsirPureBindingForm::Entry(entry) => Some(entry),
        }
    }

    pub fn expression(&self) -> Option<&NsirPureBindingExpression> {
        self.entry().and_then(NsirPureBindingEntry::expression)
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.entry().and_then(NsirPureBindingEntry::result_i64)
    }

    pub fn is_pure(&self) -> bool {
        match self.entry() {
            Some(entry) => entry.required_effects().is_pure(),
            None => true,
        }
    }

    pub fn canonical_l08_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-L0.8-PURE-BINDINGS\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let semantic = self.semantic.canonical_c02_bytes();
        bytes.extend_from_slice(&(semantic.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&semantic);

        let bindings = self.bindings.canonical_bytes();
        bytes.extend_from_slice(&(bindings.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&bindings);

        match &self.form {
            NsirPureBindingForm::Empty => bytes.push(0),
            NsirPureBindingForm::Entry(entry) => {
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
                                SemanticPureBindingExpressionOp::Int(value) => {
                                    bytes.push(1);
                                    bytes.extend_from_slice(&value.to_be_bytes());
                                }
                                SemanticPureBindingExpressionOp::Binding(id) => {
                                    bytes.push(2);
                                    bytes.extend_from_slice(&id.get().to_be_bytes());
                                }
                                SemanticPureBindingExpressionOp::Add => bytes.push(3),
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

pub fn lower_pure_binding_unit_to_hir(
    source: &SourceText,
    unit: &PureBindingUnit,
) -> CompilerResult<HirPureBindingUnit> {
    let semantic = lower_type_effect_unit_to_hir(source, unit.type_effect_unit())?;
    let bindings = unit
        .bindings()
        .iter()
        .map(|binding| {
            Ok(HirPureBinding::new(
                SemanticName::new(binding.name().text(source)?.to_owned())?,
                binding.span(),
                binding.value_token().span(),
                binding.value(),
            ))
        })
        .collect::<CompilerResult<Vec<_>>>()?;

    let form = match unit.form() {
        PureBindingBodyForm::Empty => HirPureBindingForm::Empty,
        PureBindingBodyForm::Entry(entry) => HirPureBindingForm::Entry(HirPureBindingEntry::new(
            SemanticName::new(entry.name().text(source)?.to_owned())?,
            entry.span(),
            None,
        )),
        PureBindingBodyForm::EntryExpression(entry) => {
            let mut ops = Vec::with_capacity(entry.expression().ops().len());
            for op in entry.expression().ops() {
                match op {
                    SurfacePureBindingExpressionOp::Int { span, value, .. } => {
                        ops.push(HirPureBindingExpressionOp::Int {
                            value: *value,
                            origin_span: *span,
                        });
                    }
                    SurfacePureBindingExpressionOp::BindingRef { span, name, .. } => {
                        ops.push(HirPureBindingExpressionOp::BindingRef {
                            name: SemanticName::new(name.text(source)?.to_owned())?,
                            origin_span: *span,
                        });
                    }
                    SurfacePureBindingExpressionOp::Add { span, .. } => {
                        ops.push(HirPureBindingExpressionOp::Add { origin_span: *span });
                    }
                }
            }
            HirPureBindingForm::Entry(HirPureBindingEntry::new(
                SemanticName::new(entry.name().text(source)?.to_owned())?,
                entry.span(),
                Some(HirPureBindingExpression::new(
                    entry.expression().span(),
                    ops,
                )),
            ))
        }
    };

    Ok(HirPureBindingUnit::new(
        semantic,
        unit.body_span(),
        bindings,
        form,
    ))
}

pub fn validate_pure_binding_hir(unit: HirPureBindingUnit) -> CompilerResult<NsirPureBindingUnit> {
    let HirPureBindingUnit {
        semantic,
        body_span,
        bindings,
        form,
    } = unit;

    let semantic_body = semantic.body_span();
    if semantic_body != body_span {
        return Err(CompilerError::PureBindingBodySpanMismatch {
            semantic_body,
            body: body_span,
        });
    }

    for binding in &bindings {
        if binding.origin_span().source() != body_span.source() {
            return Err(CompilerError::SourceMismatch {
                file: body_span,
                other: binding.origin_span(),
            });
        }
        if !contains(body_span, binding.origin_span()) {
            return Err(CompilerError::PureBindingDeclarationSpanOutsideBody {
                body: body_span,
                binding: binding.origin_span(),
            });
        }
        if binding.value_span().source() != binding.origin_span().source() {
            return Err(CompilerError::SourceMismatch {
                file: binding.origin_span(),
                other: binding.value_span(),
            });
        }
        if !contains(binding.origin_span(), binding.value_span()) {
            return Err(CompilerError::PureBindingValueSpanOutsideDeclaration {
                binding: binding.origin_span(),
                value: binding.value_span(),
            });
        }
        if binding.value() < 0 {
            return Err(CompilerError::NegativePureBindingLiteral {
                span: binding.value_span(),
            });
        }
    }

    if let HirPureBindingForm::Entry(entry) = &form {
        if entry.origin_span().source() != body_span.source() {
            return Err(CompilerError::SourceMismatch {
                file: body_span,
                other: entry.origin_span(),
            });
        }
        if !contains(body_span, entry.origin_span()) {
            return Err(CompilerError::PureBindingEntrySpanOutsideBody {
                body: body_span,
                entry: entry.origin_span(),
            });
        }
        if let Some(expression) = entry.expression() {
            validate_expression_spans(entry.origin_span(), expression)?;
        }
    }

    let semantic = validate_hir(semantic)?;
    let registry = PureBindingRegistry::from_hir(&bindings)?;
    let form = match form {
        HirPureBindingForm::Empty => NsirPureBindingForm::Empty,
        HirPureBindingForm::Entry(entry) => {
            let required_effects =
                resolve_effect_set(semantic.registry(), &SemanticEffectSet::empty())?;
            let expression = match entry.expression {
                Some(expression) => Some(validate_expression(expression, &registry)?),
                None => None,
            };
            NsirPureBindingForm::Entry(NsirPureBindingEntry::new(
                entry.name,
                entry.origin_span,
                required_effects,
                expression,
            ))
        }
    };

    Ok(NsirPureBindingUnit::new(semantic, registry, form))
}

fn validate_expression_spans(
    entry_span: SourceSpan,
    expression: &HirPureBindingExpression,
) -> CompilerResult<()> {
    if expression.origin_span().source() != entry_span.source() {
        return Err(CompilerError::SourceMismatch {
            file: entry_span,
            other: expression.origin_span(),
        });
    }
    if !contains(entry_span, expression.origin_span()) {
        return Err(CompilerError::PureBindingExpressionSpanOutsideEntry {
            entry: entry_span,
            expression: expression.origin_span(),
        });
    }
    if expression.ops().len() > MAX_PURE_EXPRESSION_NODES as usize {
        return Err(CompilerError::TooManyPureBindingExpressionNodes {
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
            return Err(
                CompilerError::PureBindingExpressionNodeSpanOutsideExpression {
                    expression: expression.origin_span(),
                    node: span,
                },
            );
        }
    }
    Ok(())
}

fn validate_expression(
    expression: HirPureBindingExpression,
    registry: &PureBindingRegistry,
) -> CompilerResult<NsirPureBindingExpression> {
    if expression.ops.is_empty() {
        return Err(CompilerError::InvalidPureBindingExpressionPostfix {
            span: expression.origin_span,
        });
    }

    let mut stack = Vec::new();
    let mut semantic_ops = Vec::with_capacity(expression.ops.len());
    for op in expression.ops {
        match op {
            HirPureBindingExpressionOp::Int { value, origin_span } => {
                if value < 0 {
                    return Err(CompilerError::NegativePureBindingLiteral { span: origin_span });
                }
                stack.push(value);
                semantic_ops.push(SemanticPureBindingExpressionOp::Int(value));
            }
            HirPureBindingExpressionOp::BindingRef { name, origin_span } => {
                let Some(binding) = registry.resolve(name.as_str()) else {
                    return Err(CompilerError::UnknownPureBinding {
                        name: name.as_str().to_owned(),
                        span: origin_span,
                    });
                };
                stack.push(binding.value());
                semantic_ops.push(SemanticPureBindingExpressionOp::Binding(binding.id()));
            }
            HirPureBindingExpressionOp::Add { origin_span } => {
                if stack.len() < 2 {
                    return Err(CompilerError::InvalidPureBindingExpressionPostfix {
                        span: origin_span,
                    });
                }
                let right = stack
                    .pop()
                    .expect("validated L0.8 postfix add requires right operand");
                let left = stack
                    .pop()
                    .expect("validated L0.8 postfix add requires left operand");
                let value = left
                    .checked_add(right)
                    .ok_or(CompilerError::PureBindingIntegerOverflow { span: origin_span })?;
                stack.push(value);
                semantic_ops.push(SemanticPureBindingExpressionOp::Add);
            }
        }
    }

    if stack.len() != 1 {
        return Err(CompilerError::InvalidPureBindingExpressionPostfix {
            span: expression.origin_span,
        });
    }
    let value = stack
        .pop()
        .expect("validated L0.8 expression must leave exactly one value");
    Ok(NsirPureBindingExpression::new(
        expression.origin_span,
        semantic_ops,
        value,
    ))
}

/// L0.8 pure named-binding boundary. Contextual `const Name = <canonical-int>;` declarations
/// are immutable compile-time semantic bindings. Expression references resolve to canonical
/// binding IDs and values without defining storage, mutation, runtime work, authority or NAIR.
pub fn compile_pure_binding_boundary(source: &SourceText) -> CompilerResult<NsirPureBindingUnit> {
    let unit = analyze_pure_binding_unit(source)?;
    let hir = lower_pure_binding_unit_to_hir(source, &unit)?;
    validate_pure_binding_hir(hir)
}

fn contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
