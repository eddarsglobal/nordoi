use super::effects::SemanticEffectSet;
use super::error::{CompilerError, CompilerResult};
use super::hir::{HirUnit, SemanticName};
use super::lower::lower_type_effect_unit_to_hir;
use super::nsir::{validate_hir, NsirUnit};
use super::symbols::{resolve_effect_set, ResolvedEffectSet};
use crate::frontend::{
    analyze_pure_result_unit, PureResultBodyForm, PureResultUnit, SourceSpan, SourceText,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureIntResult {
    value: i64,
    origin_span: SourceSpan,
}

impl HirPureIntResult {
    pub fn new(value: i64, origin_span: SourceSpan) -> Self {
        Self { value, origin_span }
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureResultEntry {
    name: SemanticName,
    origin_span: SourceSpan,
    result: Option<HirPureIntResult>,
}

impl HirPureResultEntry {
    pub fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        result: Option<HirPureIntResult>,
    ) -> Self {
        Self {
            name,
            origin_span,
            result,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }

    pub fn result(&self) -> Option<&HirPureIntResult> {
        self.result.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPureResultForm {
    Empty,
    Entry(HirPureResultEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPureResultUnit {
    semantic: HirUnit,
    body_span: SourceSpan,
    form: HirPureResultForm,
}

impl HirPureResultUnit {
    pub fn new(semantic: HirUnit, body_span: SourceSpan, form: HirPureResultForm) -> Self {
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

    pub fn form(&self) -> &HirPureResultForm {
        &self.form
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureIntResult {
    value: i64,
    origin_span: SourceSpan,
}

impl NsirPureIntResult {
    pub(crate) fn new(value: i64, origin_span: SourceSpan) -> Self {
        Self { value, origin_span }
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureResultEntry {
    name: SemanticName,
    origin_span: SourceSpan,
    required_effects: ResolvedEffectSet,
    result: Option<NsirPureIntResult>,
}

impl NsirPureResultEntry {
    pub(crate) fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        required_effects: ResolvedEffectSet,
        result: Option<NsirPureIntResult>,
    ) -> Self {
        Self {
            name,
            origin_span,
            required_effects,
            result,
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

    pub fn result(&self) -> Option<&NsirPureIntResult> {
        self.result.as_ref()
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.result.as_ref().map(NsirPureIntResult::value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NsirPureResultForm {
    Empty,
    Entry(NsirPureResultEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirPureResultUnit {
    semantic: NsirUnit,
    form: NsirPureResultForm,
}

impl NsirPureResultUnit {
    pub(crate) fn new(semantic: NsirUnit, form: NsirPureResultForm) -> Self {
        Self { semantic, form }
    }

    pub fn semantic(&self) -> &NsirUnit {
        &self.semantic
    }

    pub fn form(&self) -> &NsirPureResultForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&NsirPureResultEntry> {
        match &self.form {
            NsirPureResultForm::Empty => None,
            NsirPureResultForm::Entry(entry) => Some(entry),
        }
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.entry().and_then(NsirPureResultEntry::result_i64)
    }

    pub fn is_pure(&self) -> bool {
        match self.entry() {
            Some(entry) => entry.is_pure(),
            None => true,
        }
    }

    /// L0.6 pure-result witness. This is additive and does not replace any earlier witness.
    /// Source spans, comments, whitespace and source IDs are excluded.
    pub fn canonical_l06_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-L0.6-PURE-RESULT\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let semantic = self.semantic.canonical_c02_bytes();
        bytes.extend_from_slice(&(semantic.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&semantic);

        match &self.form {
            NsirPureResultForm::Empty => bytes.push(0),
            NsirPureResultForm::Entry(entry) => {
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
                match entry.result() {
                    None => bytes.push(0),
                    Some(result) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&result.value().to_be_bytes());
                    }
                }
            }
        }

        bytes
    }
}

pub fn lower_pure_result_unit_to_hir(
    source: &SourceText,
    unit: &PureResultUnit,
) -> CompilerResult<HirPureResultUnit> {
    let semantic = lower_type_effect_unit_to_hir(source, unit.type_effect_unit())?;
    let form = match unit.form() {
        PureResultBodyForm::Empty => HirPureResultForm::Empty,
        PureResultBodyForm::Entry(entry) => HirPureResultForm::Entry(HirPureResultEntry::new(
            SemanticName::new(entry.name().text(source)?.to_owned())?,
            entry.span(),
            None,
        )),
        PureResultBodyForm::EntryInt(entry) => HirPureResultForm::Entry(HirPureResultEntry::new(
            SemanticName::new(entry.name().text(source)?.to_owned())?,
            entry.span(),
            Some(HirPureIntResult::new(
                entry.result().value(),
                entry.result().span(),
            )),
        )),
    };

    Ok(HirPureResultUnit::new(semantic, unit.body_span(), form))
}

pub fn validate_pure_result_hir(unit: HirPureResultUnit) -> CompilerResult<NsirPureResultUnit> {
    let HirPureResultUnit {
        semantic,
        body_span,
        form,
    } = unit;

    let semantic_body = semantic.body_span();
    if semantic_body != body_span {
        return Err(CompilerError::PureResultBodySpanMismatch {
            semantic_body,
            body: body_span,
        });
    }

    if let HirPureResultForm::Entry(entry) = &form {
        if entry.origin_span().source() != body_span.source() {
            return Err(CompilerError::SourceMismatch {
                file: body_span,
                other: entry.origin_span(),
            });
        }
        if !contains(body_span, entry.origin_span()) {
            return Err(CompilerError::PureResultEntrySpanOutsideBody {
                body: body_span,
                entry: entry.origin_span(),
            });
        }
        if let Some(result) = entry.result() {
            if result.origin_span().source() != entry.origin_span().source() {
                return Err(CompilerError::SourceMismatch {
                    file: entry.origin_span(),
                    other: result.origin_span(),
                });
            }
            if !contains(entry.origin_span(), result.origin_span()) {
                return Err(CompilerError::PureResultValueSpanOutsideEntry {
                    entry: entry.origin_span(),
                    result: result.origin_span(),
                });
            }
        }
    }

    let semantic = validate_hir(semantic)?;
    let form = match form {
        HirPureResultForm::Empty => NsirPureResultForm::Empty,
        HirPureResultForm::Entry(entry) => {
            let required_effects =
                resolve_effect_set(semantic.registry(), &SemanticEffectSet::empty())?;
            let result = entry
                .result
                .map(|result| NsirPureIntResult::new(result.value, result.origin_span));
            NsirPureResultForm::Entry(NsirPureResultEntry::new(
                entry.name,
                entry.origin_span,
                required_effects,
                result,
            ))
        }
    };

    Ok(NsirPureResultUnit::new(semantic, form))
}

/// L0.6 pure-result boundary. It understands empty bodies, certified L0.5 entries, and the new
/// pure contextual form `entry Name returns <canonical-decimal-i64>;`. It does not create a C0.3
/// execution plan, lower to NAIR, invoke the runtime, perform I/O, or grant authority.
pub fn compile_pure_result_boundary(source: &SourceText) -> CompilerResult<NsirPureResultUnit> {
    let unit = analyze_pure_result_unit(source)?;
    let hir = lower_pure_result_unit_to_hir(source, &unit)?;
    validate_pure_result_hir(hir)
}

fn contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
