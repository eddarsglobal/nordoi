use super::error::{CompilerError, CompilerResult};
use super::hir::{
    HirBodyState, HirUnit, SemanticDeclarationKind, SemanticModuleIdentity, SemanticName,
    MAX_SEMANTIC_DECLARATIONS,
};
use crate::frontend::SourceSpan;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NsirBodyState {
    Unlowered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirDeclaration {
    kind: SemanticDeclarationKind,
    name: SemanticName,
    origin_span: SourceSpan,
}

impl NsirDeclaration {
    pub(crate) fn new(
        kind: SemanticDeclarationKind,
        name: SemanticName,
        origin_span: SourceSpan,
    ) -> Self {
        Self {
            kind,
            name,
            origin_span,
        }
    }

    pub fn kind(&self) -> SemanticDeclarationKind {
        self.kind
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn origin_span(&self) -> SourceSpan {
        self.origin_span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirUnit {
    module: SemanticModuleIdentity,
    origin: NsirOrigin,
    declarations: Vec<NsirDeclaration>,
    body_state: NsirBodyState,
}

impl NsirUnit {
    pub(crate) fn new(
        module: SemanticModuleIdentity,
        origin: NsirOrigin,
        declarations: Vec<NsirDeclaration>,
        body_state: NsirBodyState,
    ) -> Self {
        Self {
            module,
            origin,
            declarations,
            body_state,
        }
    }

    pub fn module(&self) -> &SemanticModuleIdentity {
        &self.module
    }

    pub fn origin(&self) -> NsirOrigin {
        self.origin
    }

    pub fn declarations(&self) -> &[NsirDeclaration] {
        &self.declarations
    }

    pub fn type_declaration_count(&self) -> usize {
        self.declarations
            .iter()
            .filter(|declaration| declaration.kind() == SemanticDeclarationKind::Type)
            .count()
    }

    pub fn effect_declaration_count(&self) -> usize {
        self.declarations
            .iter()
            .filter(|declaration| declaration.kind() == SemanticDeclarationKind::Effect)
            .count()
    }

    pub fn body_state(&self) -> NsirBodyState {
        self.body_state
    }

    /// C0.1 module-only identity witness. L0.4 deliberately preserves this method unchanged.
    pub fn canonical_identity_bytes(&self) -> Vec<u8> {
        self.module.canonical_bytes()
    }

    /// L0.4 semantic witness for the module identity plus opaque type/effect declarations.
    /// Diagnostic spans, comments, whitespace and declaration order do not participate.
    pub fn canonical_semantic_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-L0.4-SEMANTIC\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let module = self.module.canonical_bytes();
        bytes.extend_from_slice(&(module.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&module);

        let mut declarations = self.declarations.iter().collect::<Vec<_>>();
        declarations.sort_by(|left, right| {
            left.kind()
                .cmp(&right.kind())
                .then_with(|| left.name().as_str().cmp(right.name().as_str()))
        });
        bytes.extend_from_slice(&(declarations.len() as u32).to_be_bytes());
        for declaration in declarations {
            bytes.push(declaration.kind().canonical_tag());
            let name = declaration.name().as_str().as_bytes();
            bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
            bytes.extend_from_slice(name);
        }
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NsirOrigin {
    file_span: SourceSpan,
    module_span: Option<SourceSpan>,
    body_span: SourceSpan,
}

impl NsirOrigin {
    pub fn file_span(self) -> SourceSpan {
        self.file_span
    }

    pub fn module_span(self) -> Option<SourceSpan> {
        self.module_span
    }

    pub fn body_span(self) -> SourceSpan {
        self.body_span
    }
}

pub fn validate_hir(unit: HirUnit) -> CompilerResult<NsirUnit> {
    validate_span_sources(&unit)?;
    validate_containment(&unit)?;
    validate_declarations(&unit)?;

    let body_state = match unit.body_state() {
        HirBodyState::Unlowered => NsirBodyState::Unlowered,
    };

    let origin = NsirOrigin {
        file_span: unit.file_span(),
        module_span: unit.module_span(),
        body_span: unit.body_span(),
    };
    let declarations = unit
        .declarations()
        .iter()
        .map(|declaration| {
            NsirDeclaration::new(
                declaration.kind(),
                declaration.name().clone(),
                declaration.span(),
            )
        })
        .collect();

    Ok(NsirUnit::new(
        unit.module().clone(),
        origin,
        declarations,
        body_state,
    ))
}

fn validate_span_sources(unit: &HirUnit) -> CompilerResult<()> {
    let file = unit.file_span();
    for other in [unit.module_span(), Some(unit.body_span())]
        .into_iter()
        .flatten()
        .chain(
            unit.declarations()
                .iter()
                .map(|declaration| declaration.span()),
        )
    {
        if other.source() != file.source() {
            return Err(CompilerError::SourceMismatch { file, other });
        }
    }
    Ok(())
}

fn validate_containment(unit: &HirUnit) -> CompilerResult<()> {
    let file = unit.file_span();
    if let Some(module) = unit.module_span() {
        if !contains(file, module) {
            return Err(CompilerError::ModuleSpanOutsideFile { file, module });
        }
    }

    let body = unit.body_span();
    if !contains(file, body) {
        return Err(CompilerError::BodySpanOutsideFile { file, body });
    }

    if let Some(module) = unit.module_span() {
        if body.start() < module.end() {
            return Err(CompilerError::BodyStartsBeforeModuleEnds { module, body });
        }
    }

    Ok(())
}

fn validate_declarations(unit: &HirUnit) -> CompilerResult<()> {
    let count = unit.declarations().len() as u32;
    if count > MAX_SEMANTIC_DECLARATIONS {
        return Err(CompilerError::TooManySemanticDeclarations {
            count,
            maximum: MAX_SEMANTIC_DECLARATIONS,
        });
    }

    let file = unit.file_span();
    let prelude_offset = unit
        .module_span()
        .map(|module| module.end())
        .unwrap_or(file.start());
    let prelude_start =
        SourceSpan::new_unchecked(file.source(), prelude_offset.get(), prelude_offset.get());
    let body = unit.body_span();
    let mut previous: Option<SourceSpan> = None;
    let mut seen: BTreeMap<(SemanticDeclarationKind, &str), SourceSpan> = BTreeMap::new();

    for declaration in unit.declarations() {
        let span = declaration.span();
        if !contains(file, span) {
            return Err(CompilerError::DeclarationSpanOutsideFile {
                file,
                declaration: span,
            });
        }
        if span.start() < prelude_offset {
            return Err(CompilerError::DeclarationStartsBeforePrelude {
                prelude_start,
                declaration: span,
            });
        }
        if let Some(previous_span) = previous {
            if span.start() < previous_span.end() {
                return Err(CompilerError::DeclarationOrderViolation {
                    previous: previous_span,
                    declaration: span,
                });
            }
        }
        if span.end() > body.start() {
            return Err(CompilerError::DeclarationOverlapsBody {
                declaration: span,
                body,
            });
        }

        let key = (declaration.kind(), declaration.name().as_str());
        if let Some(first) = seen.insert(key, span) {
            return Err(CompilerError::DuplicateSemanticDeclaration {
                kind: declaration.kind().label(),
                name: declaration.name().as_str().to_owned(),
                first,
                duplicate: span,
            });
        }
        previous = Some(span);
    }

    Ok(())
}

fn contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
