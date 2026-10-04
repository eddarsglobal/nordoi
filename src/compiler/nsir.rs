use super::error::{CompilerError, CompilerResult};
use super::hir::{HirBodyState, HirUnit, SemanticModuleIdentity};
use crate::frontend::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NsirBodyState {
    Unlowered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsirUnit {
    module: SemanticModuleIdentity,
    origin: NsirOrigin,
    body_state: NsirBodyState,
}

impl NsirUnit {
    pub(crate) fn new(
        module: SemanticModuleIdentity,
        origin: NsirOrigin,
        body_state: NsirBodyState,
    ) -> Self {
        Self {
            module,
            origin,
            body_state,
        }
    }

    pub const fn module(&self) -> &SemanticModuleIdentity {
        &self.module
    }

    pub const fn origin(&self) -> NsirOrigin {
        self.origin
    }

    pub const fn body_state(&self) -> NsirBodyState {
        self.body_state
    }

    pub fn canonical_identity_bytes(&self) -> Vec<u8> {
        self.module.canonical_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NsirOrigin {
    file_span: SourceSpan,
    module_span: Option<SourceSpan>,
    body_span: SourceSpan,
}

impl NsirOrigin {
    pub const fn file_span(self) -> SourceSpan {
        self.file_span
    }

    pub const fn module_span(self) -> Option<SourceSpan> {
        self.module_span
    }

    pub const fn body_span(self) -> SourceSpan {
        self.body_span
    }
}

pub fn validate_hir(unit: HirUnit) -> CompilerResult<NsirUnit> {
    validate_span_sources(&unit)?;
    validate_containment(&unit)?;

    let body_state = match unit.body_state() {
        HirBodyState::Unlowered => NsirBodyState::Unlowered,
    };

    let origin = NsirOrigin {
        file_span: unit.file_span(),
        module_span: unit.module_span(),
        body_span: unit.body_span(),
    };

    Ok(NsirUnit::new(unit.module().clone(), origin, body_state))
}

fn validate_span_sources(unit: &HirUnit) -> CompilerResult<()> {
    let file = unit.file_span();
    for other in [unit.module_span(), Some(unit.body_span())]
        .into_iter()
        .flatten()
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

fn contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
