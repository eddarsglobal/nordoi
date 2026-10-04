use super::effects::SemanticEffectSet;
use super::error::{CompilerError, CompilerResult};
use super::hir::{HirUnit, SemanticName};
use super::lower::lower_type_effect_unit_to_hir;
use super::nsir::{validate_hir, NsirUnit};
use super::symbols::{resolve_effect_set, ResolvedEffectSet};
use crate::frontend::{
    analyze_minimal_body_unit, MinimalBodyForm, MinimalBodyUnit, SourceSpan, SourceText,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirEntryPoint {
    name: SemanticName,
    origin_span: SourceSpan,
}

impl HirEntryPoint {
    pub fn new(name: SemanticName, origin_span: SourceSpan) -> Self {
        Self { name, origin_span }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirMinimalBody {
    Empty,
    Entry(HirEntryPoint),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirBodyUnit {
    semantic: HirUnit,
    body_span: SourceSpan,
    body: HirMinimalBody,
}

impl HirBodyUnit {
    pub fn new(semantic: HirUnit, body_span: SourceSpan, body: HirMinimalBody) -> Self {
        Self {
            semantic,
            body_span,
            body,
        }
    }

    pub fn semantic(&self) -> &HirUnit {
        &self.semantic
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }

    pub fn body(&self) -> &HirMinimalBody {
        &self.body
    }

    pub fn entry(&self) -> Option<&HirEntryPoint> {
        match &self.body {
            HirMinimalBody::Empty => None,
            HirMinimalBody::Entry(entry) => Some(entry),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirEntryPoint {
    name: SemanticName,
    origin_span: SourceSpan,
    required_effects: ResolvedEffectSet,
}

impl NsirEntryPoint {
    pub(crate) fn new(
        name: SemanticName,
        origin_span: SourceSpan,
        required_effects: ResolvedEffectSet,
    ) -> Self {
        Self {
            name,
            origin_span,
            required_effects,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NsirMinimalBody {
    Empty,
    Entry(NsirEntryPoint),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirBodyUnit {
    semantic: NsirUnit,
    body: NsirMinimalBody,
}

impl NsirBodyUnit {
    pub(crate) fn new(semantic: NsirUnit, body: NsirMinimalBody) -> Self {
        Self { semantic, body }
    }

    pub fn semantic(&self) -> &NsirUnit {
        &self.semantic
    }

    pub fn body(&self) -> &NsirMinimalBody {
        &self.body
    }

    pub fn entry(&self) -> Option<&NsirEntryPoint> {
        match &self.body {
            NsirMinimalBody::Empty => None,
            NsirMinimalBody::Entry(entry) => Some(entry),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.body(), NsirMinimalBody::Empty)
    }

    /// L0.5 body witness. The certified C0.1, L0.4 and C0.2 witnesses remain unchanged.
    /// Source spans and trivia do not participate. Entry points are pure zero-work declarations.
    pub fn canonical_l05_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-L0.5-BODY\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let semantic = self.semantic.canonical_c02_bytes();
        bytes.extend_from_slice(&(semantic.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&semantic);

        match &self.body {
            NsirMinimalBody::Empty => bytes.push(0),
            NsirMinimalBody::Entry(entry) => {
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
            }
        }
        bytes
    }
}

pub fn lower_minimal_body_unit_to_hir(
    source: &SourceText,
    unit: &MinimalBodyUnit,
) -> CompilerResult<HirBodyUnit> {
    let semantic = lower_type_effect_unit_to_hir(source, unit.type_effect_unit())?;
    let body = match unit.form() {
        MinimalBodyForm::Empty => HirMinimalBody::Empty,
        MinimalBodyForm::Entry(entry) => HirMinimalBody::Entry(HirEntryPoint::new(
            SemanticName::new(entry.name().text(source)?.to_owned())?,
            entry.span(),
        )),
    };
    Ok(HirBodyUnit::new(semantic, unit.body_span(), body))
}

pub fn validate_body_hir(unit: HirBodyUnit) -> CompilerResult<NsirBodyUnit> {
    let HirBodyUnit {
        semantic,
        body_span,
        body,
    } = unit;

    let semantic_body = semantic.body_span();
    if semantic_body != body_span {
        return Err(CompilerError::BodyLayerSpanMismatch {
            semantic_body,
            body: body_span,
        });
    }

    if let HirMinimalBody::Entry(entry) = &body {
        if entry.origin_span().source() != body_span.source() {
            return Err(CompilerError::SourceMismatch {
                file: body_span,
                other: entry.origin_span(),
            });
        }
        if !contains(body_span, entry.origin_span()) {
            return Err(CompilerError::EntrySpanOutsideBody {
                body: body_span,
                entry: entry.origin_span(),
            });
        }
    }

    let semantic = validate_hir(semantic)?;
    let body = match body {
        HirMinimalBody::Empty => NsirMinimalBody::Empty,
        HirMinimalBody::Entry(entry) => {
            let required_effects =
                resolve_effect_set(semantic.registry(), &SemanticEffectSet::empty())?;
            NsirMinimalBody::Entry(NsirEntryPoint::new(
                entry.name,
                entry.origin_span,
                required_effects,
            ))
        }
    };

    Ok(NsirBodyUnit::new(semantic, body))
}

/// L0.5 minimal body boundary. It accepts only a fully understood empty body or one pure
/// `entry Name;` declaration. It performs no NSIR -> NAIR lowering and no runtime work.
pub fn compile_minimal_body_boundary(source: &SourceText) -> CompilerResult<NsirBodyUnit> {
    let unit = analyze_minimal_body_unit(source)?;
    let hir = lower_minimal_body_unit_to_hir(source, &unit)?;
    validate_body_hir(hir)
}

fn contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
