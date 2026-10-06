use super::effects::SemanticEffectSet;
use super::error::CompilerError;
use super::hir::{HirUnit, SemanticName};
use super::lower::lower_type_effect_unit_to_hir;
use super::nsir::{validate_hir, NsirUnit};
use super::symbols::{resolve_effect_set, ResolvedEffectSet};
use crate::frontend::{
    analyze_pure_condition_unit, NameError, PureConditionBodyForm, PureConditionError,
    PureConditionUnit, SourceSpan, SourceText, SurfacePureComparator, SurfacePureConditionKind,
};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticPureComparator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl SemanticPureComparator {
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

    const fn canonical_tag(self) -> u8 {
        match self {
            Self::Eq => 1,
            Self::Ne => 2,
            Self::Lt => 3,
            Self::Le => 4,
            Self::Gt => 5,
            Self::Ge => 6,
        }
    }
}

impl From<SurfacePureComparator> for SemanticPureComparator {
    fn from(value: SurfacePureComparator) -> Self {
        match value {
            SurfacePureComparator::Eq => Self::Eq,
            SurfacePureComparator::Ne => Self::Ne,
            SurfacePureComparator::Lt => Self::Lt,
            SurfacePureComparator::Le => Self::Le,
            SurfacePureComparator::Gt => Self::Gt,
            SurfacePureComparator::Ge => Self::Ge,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPureConditionKind {
    Bool {
        value: bool,
        origin_span: SourceSpan,
    },
    IntCompare {
        lhs: i64,
        lhs_span: SourceSpan,
        comparator: SemanticPureComparator,
        comparator_span: SourceSpan,
        rhs: i64,
        rhs_span: SourceSpan,
        origin_span: SourceSpan,
    },
}

impl HirPureConditionKind {
    pub fn origin_span(&self) -> SourceSpan {
        match self {
            Self::Bool { origin_span, .. } | Self::IntCompare { origin_span, .. } => *origin_span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureConditionEntry {
    name: SemanticName,
    origin_span: SourceSpan,
    condition: Option<HirPureConditionKind>,
}

impl HirPureConditionEntry {
    pub fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        condition: Option<HirPureConditionKind>,
    ) -> Self {
        Self {
            name,
            origin_span,
            condition,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn condition(&self) -> Option<&HirPureConditionKind> {
        self.condition.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPureConditionForm {
    Empty,
    Entry(HirPureConditionEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureConditionUnit {
    semantic: HirUnit,
    body_span: SourceSpan,
    form: HirPureConditionForm,
}

impl HirPureConditionUnit {
    pub fn new(semantic: HirUnit, body_span: SourceSpan, form: HirPureConditionForm) -> Self {
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

    pub fn form(&self) -> &HirPureConditionForm {
        &self.form
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticPureCondition {
    Bool(bool),
    IntCompare {
        lhs: i64,
        comparator: SemanticPureComparator,
        rhs: i64,
    },
}

impl SemanticPureCondition {
    pub fn value(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::IntCompare {
                lhs,
                comparator,
                rhs,
            } => match comparator {
                SemanticPureComparator::Eq => lhs == rhs,
                SemanticPureComparator::Ne => lhs != rhs,
                SemanticPureComparator::Lt => lhs < rhs,
                SemanticPureComparator::Le => lhs <= rhs,
                SemanticPureComparator::Gt => lhs > rhs,
                SemanticPureComparator::Ge => lhs >= rhs,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureConditionEntry {
    name: SemanticName,
    origin_span: SourceSpan,
    required_effects: ResolvedEffectSet,
    condition: Option<SemanticPureCondition>,
}

impl NsirPureConditionEntry {
    pub(crate) fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        required_effects: ResolvedEffectSet,
        condition: Option<SemanticPureCondition>,
    ) -> Self {
        Self {
            name,
            origin_span,
            required_effects,
            condition,
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

    pub fn condition(&self) -> Option<&SemanticPureCondition> {
        self.condition.as_ref()
    }

    pub fn result_bool(&self) -> Option<bool> {
        self.condition.as_ref().map(SemanticPureCondition::value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NsirPureConditionForm {
    Empty,
    Entry(NsirPureConditionEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureConditionUnit {
    semantic: NsirUnit,
    form: NsirPureConditionForm,
}

impl NsirPureConditionUnit {
    pub(crate) fn new(semantic: NsirUnit, form: NsirPureConditionForm) -> Self {
        Self { semantic, form }
    }

    pub fn semantic(&self) -> &NsirUnit {
        &self.semantic
    }

    pub fn form(&self) -> &NsirPureConditionForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&NsirPureConditionEntry> {
        match &self.form {
            NsirPureConditionForm::Empty => None,
            NsirPureConditionForm::Entry(entry) => Some(entry),
        }
    }

    pub fn condition(&self) -> Option<&SemanticPureCondition> {
        self.entry().and_then(NsirPureConditionEntry::condition)
    }

    pub fn result_bool(&self) -> Option<bool> {
        self.entry().and_then(NsirPureConditionEntry::result_bool)
    }

    pub fn is_pure(&self) -> bool {
        match self.entry() {
            Some(entry) => entry.is_pure(),
            None => true,
        }
    }

    /// L0.9 pure-condition witness. Earlier certified witnesses remain unchanged.
    /// Source IDs, source spans, whitespace, comments, host authority and runtime state are excluded.
    pub fn canonical_l09_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-L0.9-PURE-CONDITION\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let semantic = self.semantic.canonical_c02_bytes();
        bytes.extend_from_slice(&(semantic.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&semantic);

        match &self.form {
            NsirPureConditionForm::Empty => bytes.push(0),
            NsirPureConditionForm::Entry(entry) => {
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

                match entry.condition() {
                    None => bytes.push(0),
                    Some(SemanticPureCondition::Bool(value)) => {
                        bytes.push(1);
                        bytes.push(u8::from(*value));
                        bytes.push(u8::from(*value));
                    }
                    Some(SemanticPureCondition::IntCompare {
                        lhs,
                        comparator,
                        rhs,
                    }) => {
                        bytes.push(2);
                        bytes.extend_from_slice(&lhs.to_be_bytes());
                        bytes.push(comparator.canonical_tag());
                        bytes.extend_from_slice(&rhs.to_be_bytes());
                        bytes.push(u8::from(entry.result_bool().expect("condition exists")));
                    }
                }
            }
        }
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureConditionCompilerError {
    Frontend(PureConditionError),
    Compiler(CompilerError),
    BodySpanMismatch {
        semantic_body: SourceSpan,
        body: SourceSpan,
    },
    EntrySpanOutsideBody {
        body: SourceSpan,
        entry: SourceSpan,
    },
    ConditionSpanOutsideEntry {
        entry: SourceSpan,
        condition: SourceSpan,
    },
    ConditionNodeSpanOutsideCondition {
        condition: SourceSpan,
        node: SourceSpan,
    },
    NegativeIntegerLiteral {
        span: SourceSpan,
    },
}

impl PureConditionCompilerError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Frontend(error) => error.primary_span(),
            Self::Compiler(error) => error.primary_span(),
            Self::BodySpanMismatch { body, .. } => Some(*body),
            Self::EntrySpanOutsideBody { entry, .. } => Some(*entry),
            Self::ConditionSpanOutsideEntry { condition, .. } => Some(*condition),
            Self::ConditionNodeSpanOutsideCondition { node, .. } => Some(*node),
            Self::NegativeIntegerLiteral { span } => Some(*span),
        }
    }
}

impl Display for PureConditionCompilerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Frontend(error) => Display::fmt(error, f),
            Self::Compiler(error) => Display::fmt(error, f),
            Self::BodySpanMismatch { .. } => {
                write!(f, "L0.9 body span does not match semantic body")
            }
            Self::EntrySpanOutsideBody { .. } => {
                write!(f, "L0.9 entry span lies outside the semantic body")
            }
            Self::ConditionSpanOutsideEntry { .. } => {
                write!(f, "L0.9 condition span lies outside its entry")
            }
            Self::ConditionNodeSpanOutsideCondition { .. } => {
                write!(f, "L0.9 condition node span lies outside the condition")
            }
            Self::NegativeIntegerLiteral { .. } => {
                write!(f, "L0.9 comparison operands must be non-negative integers")
            }
        }
    }
}

impl Error for PureConditionCompilerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frontend(error) => Some(error),
            Self::Compiler(error) => Some(error),
            _ => None,
        }
    }
}

impl From<PureConditionError> for PureConditionCompilerError {
    fn from(value: PureConditionError) -> Self {
        Self::Frontend(value)
    }
}

impl From<CompilerError> for PureConditionCompilerError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<NameError> for PureConditionCompilerError {
    fn from(value: NameError) -> Self {
        Self::Frontend(PureConditionError::Name(value))
    }
}

pub type PureConditionCompilerResult<T> = Result<T, PureConditionCompilerError>;

pub fn lower_pure_condition_unit_to_hir(
    source: &SourceText,
    unit: &PureConditionUnit,
) -> PureConditionCompilerResult<HirPureConditionUnit> {
    let semantic = lower_type_effect_unit_to_hir(source, unit.type_effect_unit())?;
    let form = match unit.form() {
        PureConditionBodyForm::Empty => HirPureConditionForm::Empty,
        PureConditionBodyForm::Entry(entry) => {
            HirPureConditionForm::Entry(HirPureConditionEntry::new(
                SemanticName::new(
                    entry
                        .name()
                        .text(source)
                        .map_err(CompilerError::from)?
                        .to_owned(),
                )?,
                entry.span(),
                None,
            ))
        }
        PureConditionBodyForm::EntryCondition(entry) => {
            let condition = match entry.condition().kind() {
                SurfacePureConditionKind::Bool { span, value, .. } => HirPureConditionKind::Bool {
                    value: *value,
                    origin_span: *span,
                },
                SurfacePureConditionKind::IntCompare {
                    span,
                    lhs_span,
                    lhs,
                    comparator_span,
                    comparator,
                    rhs_span,
                    rhs,
                    ..
                } => HirPureConditionKind::IntCompare {
                    lhs: *lhs,
                    lhs_span: *lhs_span,
                    comparator: (*comparator).into(),
                    comparator_span: *comparator_span,
                    rhs: *rhs,
                    rhs_span: *rhs_span,
                    origin_span: *span,
                },
            };
            HirPureConditionForm::Entry(HirPureConditionEntry::new(
                SemanticName::new(
                    entry
                        .name()
                        .text(source)
                        .map_err(CompilerError::from)?
                        .to_owned(),
                )?,
                entry.span(),
                Some(condition),
            ))
        }
    };
    Ok(HirPureConditionUnit::new(semantic, unit.body_span(), form))
}

pub fn validate_pure_condition_hir(
    unit: HirPureConditionUnit,
) -> PureConditionCompilerResult<NsirPureConditionUnit> {
    let HirPureConditionUnit {
        semantic,
        body_span,
        form,
    } = unit;

    let semantic_body = semantic.body_span();
    if semantic_body != body_span {
        return Err(PureConditionCompilerError::BodySpanMismatch {
            semantic_body,
            body: body_span,
        });
    }

    if let HirPureConditionForm::Entry(entry) = &form {
        if entry.origin_span().source() != body_span.source()
            || !contains(body_span, entry.origin_span())
        {
            return Err(PureConditionCompilerError::EntrySpanOutsideBody {
                body: body_span,
                entry: entry.origin_span(),
            });
        }
        if let Some(condition) = entry.condition() {
            validate_condition_spans(entry.origin_span(), condition)?;
        }
    }

    let semantic = validate_hir(semantic)?;
    let form = match form {
        HirPureConditionForm::Empty => NsirPureConditionForm::Empty,
        HirPureConditionForm::Entry(entry) => {
            let required_effects =
                resolve_effect_set(semantic.registry(), &SemanticEffectSet::empty())?;
            let condition = match entry.condition {
                None => None,
                Some(HirPureConditionKind::Bool { value, .. }) => {
                    Some(SemanticPureCondition::Bool(value))
                }
                Some(HirPureConditionKind::IntCompare {
                    lhs,
                    lhs_span,
                    comparator,
                    rhs,
                    rhs_span,
                    ..
                }) => {
                    if lhs < 0 {
                        return Err(PureConditionCompilerError::NegativeIntegerLiteral {
                            span: lhs_span,
                        });
                    }
                    if rhs < 0 {
                        return Err(PureConditionCompilerError::NegativeIntegerLiteral {
                            span: rhs_span,
                        });
                    }
                    Some(SemanticPureCondition::IntCompare {
                        lhs,
                        comparator,
                        rhs,
                    })
                }
            };
            NsirPureConditionForm::Entry(NsirPureConditionEntry::new(
                entry.name,
                entry.origin_span,
                required_effects,
                condition,
            ))
        }
    };

    Ok(NsirPureConditionUnit::new(semantic, form))
}

fn validate_condition_spans(
    entry_span: SourceSpan,
    condition: &HirPureConditionKind,
) -> PureConditionCompilerResult<()> {
    let condition_span = condition.origin_span();
    if condition_span.source() != entry_span.source() || !contains(entry_span, condition_span) {
        return Err(PureConditionCompilerError::ConditionSpanOutsideEntry {
            entry: entry_span,
            condition: condition_span,
        });
    }

    if let HirPureConditionKind::IntCompare {
        lhs_span,
        comparator_span,
        rhs_span,
        ..
    } = condition
    {
        for node in [*lhs_span, *comparator_span, *rhs_span] {
            if node.source() != condition_span.source() || !contains(condition_span, node) {
                return Err(
                    PureConditionCompilerError::ConditionNodeSpanOutsideCondition {
                        condition: condition_span,
                        node,
                    },
                );
            }
        }
    }
    Ok(())
}

/// L0.9 pure-condition boundary. It recognizes boolean literals and one integer comparison
/// using ==, !=, <, <=, >, or >=. It evaluates semantic truth only. It does not create an
/// execution plan, lower NAIR, invoke runtime work, allocate storage, perform I/O, dispatch
/// effects, consult capabilities, or grant host authority.
pub fn compile_pure_condition_boundary(
    source: &SourceText,
) -> PureConditionCompilerResult<NsirPureConditionUnit> {
    let unit = analyze_pure_condition_unit(source)?;
    let hir = lower_pure_condition_unit_to_hir(source, &unit)?;
    validate_pure_condition_hir(hir)
}

fn contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    outer.source() == inner.source()
        && outer.start().get() <= inner.start().get()
        && inner.end().get() <= outer.end().get()
}
