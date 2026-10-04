use super::error::CompilerResult;
use super::hir::{
    HirBodyState, HirDeclaration, HirUnit, SemanticDeclarationKind, SemanticModuleIdentity,
    SemanticName, SemanticPath,
};
use super::nsir::{validate_hir, NsirUnit};
use crate::frontend::{
    analyze_module_unit, analyze_type_effect_unit, ModuleUnit, SourceText, SurfaceDeclarationKind,
    TypeEffectUnit,
};

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

pub fn lower_type_effect_unit_to_hir(
    source: &SourceText,
    unit: &TypeEffectUnit,
) -> CompilerResult<HirUnit> {
    let base = lower_module_unit_to_hir(source, unit.module_unit())?;
    let mut declarations = Vec::with_capacity(unit.declarations().len());
    for declaration in unit.declarations() {
        let kind = match declaration.kind() {
            SurfaceDeclarationKind::Type => SemanticDeclarationKind::Type,
            SurfaceDeclarationKind::Effect => SemanticDeclarationKind::Effect,
        };
        declarations.push(HirDeclaration::new(
            kind,
            SemanticName::new(declaration.name().text(source)?.to_owned())?,
            declaration.span(),
        ));
    }

    Ok(HirUnit::new_with_declarations(
        base.module().clone(),
        base.file_span(),
        base.module_span(),
        declarations,
        unit.body_span(),
        HirBodyState::Unlowered,
    ))
}

/// Certified C0.1 semantic boundary. This deliberately remains module-only and treats the entire
/// post-module source as unlowered body, even after L0.4 is added.
pub fn compile_semantic_boundary(source: &SourceText) -> CompilerResult<NsirUnit> {
    let module_unit = analyze_module_unit(source)?;
    let hir = lower_module_unit_to_hir(source, &module_unit)?;
    validate_hir(hir)
}

/// L0.4 semantic boundary. Recognizes only the leading opaque-type/effect declaration prelude.
pub fn compile_type_effect_boundary(source: &SourceText) -> CompilerResult<NsirUnit> {
    let unit = analyze_type_effect_unit(source)?;
    let hir = lower_type_effect_unit_to_hir(source, &unit)?;
    validate_hir(hir)
}

/// C0.2 resolved semantic boundary. Builds the canonical type/effect registry after full HIR
/// validation. It still performs no NSIR -> NAIR lowering and grants no host authority.
pub fn compile_resolved_semantic_boundary(source: &SourceText) -> CompilerResult<NsirUnit> {
    compile_type_effect_boundary(source)
}
