use nordoi_kernel::{
    compile_resolved_semantic_boundary, compile_type_effect_boundary, resolve_effect_set,
    CompilerError, NsirBodyState, SemanticEffectSet, SemanticName, SourceId, SourceText,
};

fn source(id: u32, text: impl Into<String>) -> SourceText {
    SourceText::new(SourceId::new(id), "compiler-registry.noi", text).expect("source must be valid")
}

#[test]
fn c02_builds_separate_typed_symbol_tables() {
    let unit = compile_resolved_semantic_boundary(&source(
        1,
        "module demo; type UserId; type Account; effect Network; effect Clock; body",
    ))
    .unwrap();
    assert_eq!(unit.registry().types().len(), 2);
    assert_eq!(unit.registry().effects().len(), 2);
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
}

#[test]
fn symbol_ids_are_canonical_by_name_not_source_order() {
    let first =
        compile_resolved_semantic_boundary(&source(2, "type Z; type A; effect Net; effect Clock;"))
            .unwrap();
    let second =
        compile_resolved_semantic_boundary(&source(3, "effect Clock; type A; effect Net; type Z;"))
            .unwrap();
    assert_eq!(first.registry().resolve_type("A").unwrap().get(), 1);
    assert_eq!(first.registry().resolve_type("Z").unwrap().get(), 2);
    assert_eq!(first.registry().resolve_effect("Clock").unwrap().get(), 1);
    assert_eq!(first.registry().resolve_effect("Net").unwrap().get(), 2);
    assert_eq!(
        first.registry().canonical_bytes(),
        second.registry().canonical_bytes()
    );
}

#[test]
fn type_and_effect_namespaces_can_share_spelling() {
    let unit = compile_resolved_semantic_boundary(&source(4, "type State; effect State;")).unwrap();
    assert_eq!(unit.registry().resolve_type("State").unwrap().get(), 1);
    assert_eq!(unit.registry().resolve_effect("State").unwrap().get(), 1);
}

#[test]
fn unknown_symbol_resolution_is_explicit_none() {
    let unit = compile_resolved_semantic_boundary(&source(5, "type A; effect Net;")).unwrap();
    assert!(unit.registry().resolve_type("Missing").is_none());
    assert!(unit.registry().resolve_effect("Missing").is_none());
}

#[test]
fn published_symbol_ids_are_never_zero() {
    let unit = compile_resolved_semantic_boundary(&source(6, "type A; effect Net;")).unwrap();
    assert_ne!(unit.registry().types()[0].id().get(), 0);
    assert_ne!(unit.registry().effects()[0].id().get(), 0);
}

#[test]
fn registry_preserves_origin_spans_for_diagnostics() {
    let src = source(7, "type UserId; effect Network;");
    let unit = compile_resolved_semantic_boundary(&src).unwrap();
    assert_eq!(
        src.slice(unit.registry().types()[0].origin_span()).unwrap(),
        "type UserId;"
    );
    assert_eq!(
        src.slice(unit.registry().effects()[0].origin_span())
            .unwrap(),
        "effect Network;"
    );
}

#[test]
fn spans_comments_and_source_id_do_not_change_c02_identity() {
    let first =
        compile_resolved_semantic_boundary(&source(8, "module demo; type A; effect Net;")).unwrap();
    let second = compile_resolved_semantic_boundary(&source(
        999,
        "module /*m*/ demo /*x*/ ;\n effect /*e*/ Net ;\n type A ;",
    ))
    .unwrap();
    assert_eq!(first.canonical_c02_bytes(), second.canonical_c02_bytes());
}

#[test]
fn changing_type_symbol_changes_c02_identity() {
    let first = compile_resolved_semantic_boundary(&source(9, "type A;")).unwrap();
    let second = compile_resolved_semantic_boundary(&source(10, "type B;")).unwrap();
    assert_ne!(first.canonical_c02_bytes(), second.canonical_c02_bytes());
}

#[test]
fn changing_module_changes_c02_identity() {
    let first = compile_resolved_semantic_boundary(&source(11, "module a; type T;")).unwrap();
    let second = compile_resolved_semantic_boundary(&source(12, "module b; type T;")).unwrap();
    assert_ne!(first.canonical_c02_bytes(), second.canonical_c02_bytes());
}

#[test]
fn c01_and_l04_witnesses_remain_available_and_distinct() {
    let unit = compile_resolved_semantic_boundary(&source(13, "module demo; type A; effect Net;"))
        .unwrap();
    assert_ne!(
        unit.canonical_identity_bytes(),
        unit.canonical_semantic_bytes()
    );
    assert_ne!(unit.canonical_semantic_bytes(), unit.canonical_c02_bytes());
}

#[test]
fn c02_entrypoint_preserves_l04_semantic_witness() {
    let src = source(14, "module demo; type A; effect Net;");
    let l04 = compile_type_effect_boundary(&src).unwrap();
    let c02 = compile_resolved_semantic_boundary(&src).unwrap();
    assert_eq!(
        l04.canonical_semantic_bytes(),
        c02.canonical_semantic_bytes()
    );
}

#[test]
fn declared_effect_requirements_resolve_to_typed_ids() {
    let unit =
        compile_resolved_semantic_boundary(&source(15, "effect Network; effect Clock;")).unwrap();
    let requirements = SemanticEffectSet::new(vec![
        SemanticName::new("Network").unwrap(),
        SemanticName::new("Clock").unwrap(),
    ])
    .unwrap();
    let resolved = resolve_effect_set(unit.registry(), &requirements).unwrap();
    assert_eq!(resolved.effects().len(), 2);
    assert_eq!(resolved.effects()[0].get(), 1);
    assert_eq!(resolved.effects()[1].get(), 2);
}

#[test]
fn effect_requirement_order_does_not_change_resolved_ids() {
    let unit =
        compile_resolved_semantic_boundary(&source(16, "effect Network; effect Clock;")).unwrap();
    let first = SemanticEffectSet::new(vec![
        SemanticName::new("Network").unwrap(),
        SemanticName::new("Clock").unwrap(),
    ])
    .unwrap();
    let second = SemanticEffectSet::new(vec![
        SemanticName::new("Clock").unwrap(),
        SemanticName::new("Network").unwrap(),
    ])
    .unwrap();
    assert_eq!(
        resolve_effect_set(unit.registry(), &first).unwrap(),
        resolve_effect_set(unit.registry(), &second).unwrap()
    );
}

#[test]
fn empty_effect_requirement_set_resolves_as_pure() {
    let unit = compile_resolved_semantic_boundary(&source(17, "effect Network;")).unwrap();
    let resolved = resolve_effect_set(unit.registry(), &SemanticEffectSet::empty()).unwrap();
    assert!(resolved.is_pure());
    assert!(resolved.effects().is_empty());
}

#[test]
fn undeclared_effect_requirement_fails_closed() {
    let unit = compile_resolved_semantic_boundary(&source(18, "effect Clock;")).unwrap();
    let requirements = SemanticEffectSet::new(vec![SemanticName::new("Network").unwrap()]).unwrap();
    assert!(matches!(
        resolve_effect_set(unit.registry(), &requirements),
        Err(CompilerError::UnknownEffectRequirement { name }) if name == "Network"
    ));
}

#[test]
fn registry_resolution_is_case_sensitive() {
    let unit =
        compile_resolved_semantic_boundary(&source(19, "type State; effect Network;")).unwrap();
    assert!(unit.registry().resolve_type("state").is_none());
    assert!(unit.registry().resolve_effect("network").is_none());
}

#[test]
fn declaration_source_order_is_retained_separately_from_canonical_registry() {
    let unit = compile_resolved_semantic_boundary(&source(20, "type Z; type A;")).unwrap();
    assert_eq!(unit.declarations()[0].name().as_str(), "Z");
    assert_eq!(unit.declarations()[1].name().as_str(), "A");
    assert_eq!(unit.registry().types()[0].name().as_str(), "A");
    assert_eq!(unit.registry().types()[1].name().as_str(), "Z");
}

#[test]
fn registry_construction_does_not_claim_body_semantics() {
    let unit =
        compile_resolved_semantic_boundary(&source(21, "type A; opaque + future(body)")).unwrap();
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
}

#[test]
fn effect_resolution_does_not_grant_authority_or_execute_runtime_work() {
    let unit = compile_resolved_semantic_boundary(&source(22, "effect Network;")).unwrap();
    let requirements = SemanticEffectSet::new(vec![SemanticName::new("Network").unwrap()]).unwrap();
    let resolved = resolve_effect_set(unit.registry(), &requirements).unwrap();
    assert_eq!(
        resolved.effects()[0],
        unit.registry().resolve_effect("Network").unwrap()
    );
    assert_eq!(unit.body_state(), NsirBodyState::Unlowered);
}
