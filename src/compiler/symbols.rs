use super::effects::SemanticEffectSet;
use super::error::{CompilerError, CompilerResult};
use super::hir::{HirDeclaration, SemanticDeclarationKind, SemanticName};
use crate::frontend::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticTypeId(u32);

impl SemanticTypeId {
    pub(crate) fn new(raw: u32) -> Self {
        debug_assert!(raw != 0);
        Self(raw)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticEffectId(u32);

impl SemanticEffectId {
    pub(crate) fn new(raw: u32) -> Self {
        debug_assert!(raw != 0);
        Self(raw)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirTypeSymbol {
    id: SemanticTypeId,
    name: SemanticName,
    origin_span: SourceSpan,
}

impl NsirTypeSymbol {
    pub(crate) fn new(id: SemanticTypeId, name: SemanticName, origin_span: SourceSpan) -> Self {
        Self {
            id,
            name,
            origin_span,
        }
    }

    pub fn id(&self) -> SemanticTypeId {
        self.id
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirEffectSymbol {
    id: SemanticEffectId,
    name: SemanticName,
    origin_span: SourceSpan,
}

impl NsirEffectSymbol {
    pub(crate) fn new(id: SemanticEffectId, name: SemanticName, origin_span: SourceSpan) -> Self {
        Self {
            id,
            name,
            origin_span,
        }
    }

    pub fn id(&self) -> SemanticEffectId {
        self.id
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticRegistry {
    types: Vec<NsirTypeSymbol>,
    effects: Vec<NsirEffectSymbol>,
}

impl SemanticRegistry {
    pub(crate) fn from_hir(declarations: &[HirDeclaration]) -> Self {
        let mut types = declarations
            .iter()
            .filter(|declaration| declaration.kind() == SemanticDeclarationKind::Type)
            .map(|declaration| (declaration.name().clone(), declaration.span()))
            .collect::<Vec<_>>();
        types.sort_by_key(|(name, _)| name.clone());

        let types = types
            .into_iter()
            .enumerate()
            .map(|(index, (name, span))| {
                NsirTypeSymbol::new(SemanticTypeId::new(index as u32 + 1), name, span)
            })
            .collect();

        let mut effects = declarations
            .iter()
            .filter(|declaration| declaration.kind() == SemanticDeclarationKind::Effect)
            .map(|declaration| (declaration.name().clone(), declaration.span()))
            .collect::<Vec<_>>();
        effects.sort_by_key(|(name, _)| name.clone());

        let effects = effects
            .into_iter()
            .enumerate()
            .map(|(index, (name, span))| {
                NsirEffectSymbol::new(SemanticEffectId::new(index as u32 + 1), name, span)
            })
            .collect();

        Self { types, effects }
    }

    pub fn types(&self) -> &[NsirTypeSymbol] {
        &self.types
    }

    pub fn effects(&self) -> &[NsirEffectSymbol] {
        &self.effects
    }

    pub fn resolve_type(&self, name: &str) -> Option<SemanticTypeId> {
        self.types
            .binary_search_by(|symbol| symbol.name().as_str().cmp(name))
            .ok()
            .map(|index| self.types[index].id())
    }

    pub fn resolve_effect(&self, name: &str) -> Option<SemanticEffectId> {
        self.effects
            .binary_search_by(|symbol| symbol.name().as_str().cmp(name))
            .ok()
            .map(|index| self.effects[index].id())
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.2-REGISTRY\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);
        append_symbol_names(&mut bytes, self.types.iter().map(NsirTypeSymbol::name));
        append_symbol_names(&mut bytes, self.effects.iter().map(NsirEffectSymbol::name));
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEffectSet {
    effects: Vec<SemanticEffectId>,
}

impl ResolvedEffectSet {
    pub fn effects(&self) -> &[SemanticEffectId] {
        &self.effects
    }

    pub fn is_pure(&self) -> bool {
        self.effects.is_empty()
    }
}

pub fn resolve_effect_set(
    registry: &SemanticRegistry,
    requirements: &SemanticEffectSet,
) -> CompilerResult<ResolvedEffectSet> {
    let mut effects = Vec::with_capacity(requirements.effects().len());
    for requirement in requirements.effects() {
        let Some(id) = registry.resolve_effect(requirement.as_str()) else {
            return Err(CompilerError::UnknownEffectRequirement {
                name: requirement.as_str().to_owned(),
            });
        };
        effects.push(id);
    }
    effects.sort_unstable();
    Ok(ResolvedEffectSet { effects })
}

fn append_symbol_names<'a>(
    bytes: &mut Vec<u8>,
    symbols: impl ExactSizeIterator<Item = &'a SemanticName>,
) {
    bytes.extend_from_slice(&(symbols.len() as u32).to_be_bytes());
    for name in symbols {
        let raw = name.as_str().as_bytes();
        bytes.extend_from_slice(&(raw.len() as u32).to_be_bytes());
        bytes.extend_from_slice(raw);
    }
}
