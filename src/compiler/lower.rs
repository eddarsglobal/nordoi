use super::error::CompilerResult;
use super::hir::{HirBodyState, HirUnit, SemanticModuleIdentity, SemanticName, SemanticPath};
use super::nsir::{validate_hir, NsirUnit};
use crate::frontend::{analyze_module_unit, ModuleUnit, SourceText};

pub fn lower_module_unit_to_hir(source: &SourceText, unit: &ModuleUnit) -> CompilerResult<HirUnit> {
    let file_span = unit.file().span();
    let (identity, module_span, body_start) = match unit.module() {
        Some(declaration) => {
            let mut segments = Vec::with_capacity(declaration.path().segments().len());
            for segment in declaration.path().segments() {
                segments.push(SemanticName::new(segment.text(source)?.to_owned())?);
            }
            (
                SemanticModuleIdentity::Named(SemanticPath::new(segments)?),
                Some(declaration.span()),
                declaration.span().end(),
            )
        }
        None => (SemanticModuleIdentity::Anonymous, None, file_span.start()),
    };

    let body_span = source.span(body_start, file_span.end())?;
    Ok(HirUnit::new(
        identity,
        file_span,
        module_span,
        body_span,
        HirBodyState::Unlowered,
    ))
}

pub fn compile_semantic_boundary(source: &SourceText) -> CompilerResult<NsirUnit> {
    let module_unit = analyze_module_unit(source)?;
    let hir = lower_module_unit_to_hir(source, &module_unit)?;
    validate_hir(hir)
}
