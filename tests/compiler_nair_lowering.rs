use nordoi_kernel::{
    compile_nair_lowering_boundary, CompilerError, Instruction, NairProgram, SemanticPlanForm,
    SourceId, SourceText, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("c04-{id}.noi"), text).unwrap()
}

#[test]
fn empty_plan_lowers_to_single_halt() {
    let artifact = compile_nair_lowering_boundary(&source(1, "module demo;")).unwrap();
    assert!(matches!(artifact.plan().form(), SemanticPlanForm::Empty));
    assert_eq!(artifact.semantic_work_item_count(), 0);
    assert_eq!(artifact.nair_instruction_count(), 1);
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
}

#[test]
fn entry_plan_lowers_to_single_halt() {
    let artifact = compile_nair_lowering_boundary(&source(2, "module demo; entry main;")).unwrap();
    assert_eq!(artifact.plan().entry().unwrap().name().as_str(), "main");
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
}

#[test]
fn lowered_program_is_valid_nair_0_6() {
    let artifact = compile_nair_lowering_boundary(&source(3, "entry main;")).unwrap();
    assert_eq!(NAIR_FORMAT_MAJOR, 0);
    assert_eq!(NAIR_FORMAT_MINOR, 6);
    artifact.program().validate().unwrap();
    let decoded = NairProgram::from_canonical_bytes(artifact.canonical_nair_bytes()).unwrap();
    assert_eq!(&decoded, artifact.program());
}

#[test]
fn empty_and_entry_have_same_operational_nair_bytes() {
    let empty = compile_nair_lowering_boundary(&source(4, "module demo;")).unwrap();
    let entry = compile_nair_lowering_boundary(&source(5, "module demo; entry main;")).unwrap();
    assert_eq!(empty.canonical_nair_bytes(), entry.canonical_nair_bytes());
}

#[test]
fn empty_and_entry_keep_distinct_c04_witnesses() {
    let empty = compile_nair_lowering_boundary(&source(6, "module demo;")).unwrap();
    let entry = compile_nair_lowering_boundary(&source(7, "module demo; entry main;")).unwrap();
    assert_ne!(empty.canonical_c04_bytes(), entry.canonical_c04_bytes());
}

#[test]
fn different_entry_names_keep_same_nair_but_distinct_lowering_witnesses() {
    let first = compile_nair_lowering_boundary(&source(8, "entry main;")).unwrap();
    let second = compile_nair_lowering_boundary(&source(9, "entry other;")).unwrap();
    assert_eq!(first.canonical_nair_bytes(), second.canonical_nair_bytes());
    assert_ne!(first.canonical_c04_bytes(), second.canonical_c04_bytes());
}

#[test]
fn comments_spacing_and_source_id_do_not_change_lowering_witness() {
    let first = source(10, "module demo; type A; entry main;");
    let second = source(
        11,
        "module demo /*m*/ ;\n type /*t*/ A ;\n entry /*e*/ main ; // tail\n",
    );
    assert_eq!(
        compile_nair_lowering_boundary(&first)
            .unwrap()
            .canonical_c04_bytes(),
        compile_nair_lowering_boundary(&second)
            .unwrap()
            .canonical_c04_bytes()
    );
}

#[test]
fn module_identity_changes_lowering_witness_but_not_zero_work_nair() {
    let first = compile_nair_lowering_boundary(&source(12, "module one; entry main;")).unwrap();
    let second = compile_nair_lowering_boundary(&source(13, "module two; entry main;")).unwrap();
    assert_eq!(first.canonical_nair_bytes(), second.canonical_nair_bytes());
    assert_ne!(first.canonical_c04_bytes(), second.canonical_c04_bytes());
}

#[test]
fn registry_identity_changes_lowering_witness_but_not_zero_work_nair() {
    let first = compile_nair_lowering_boundary(&source(14, "type A; entry main;")).unwrap();
    let second = compile_nair_lowering_boundary(&source(15, "type B; entry main;")).unwrap();
    assert_eq!(first.canonical_nair_bytes(), second.canonical_nair_bytes());
    assert_ne!(first.canonical_c04_bytes(), second.canonical_c04_bytes());
}

#[test]
fn declared_effect_does_not_become_nair_work_or_authority() {
    let artifact =
        compile_nair_lowering_boundary(&source(16, "effect Network; entry main;")).unwrap();
    assert!(artifact.plan().required_effects().is_empty());
    assert!(!artifact.requires_host_authority());
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
}

#[test]
fn c04_preserves_c03_plan_witness() {
    let artifact =
        compile_nair_lowering_boundary(&source(17, "module demo; type A; effect Net; entry main;"))
            .unwrap();
    let bytes = artifact.canonical_c04_bytes();
    assert!(bytes.starts_with(b"NORDOI-C0.4-NAIR-LOWERING\0"));
    assert!(bytes
        .windows(artifact.plan().canonical_c03_bytes().len())
        .any(|window| window == artifact.plan().canonical_c03_bytes()));
}

#[test]
fn c04_witness_contains_exact_canonical_nair_bytes() {
    let artifact = compile_nair_lowering_boundary(&source(18, "entry main;")).unwrap();
    assert!(artifact
        .canonical_c04_bytes()
        .ends_with(artifact.canonical_nair_bytes()));
}

#[test]
fn raw_nair_bytes_do_not_embed_entry_name() {
    let artifact = compile_nair_lowering_boundary(&source(19, "entry SecretEntry;")).unwrap();
    assert!(!artifact
        .canonical_nair_bytes()
        .windows(b"SecretEntry".len())
        .any(|window| window == b"SecretEntry"));
}

#[test]
fn unsupported_body_fails_before_nair_publication() {
    assert!(matches!(
        compile_nair_lowering_boundary(&source(20, "future_body")),
        Err(CompilerError::BodyFrontend(_))
    ));
}

#[test]
fn equal_sources_produce_byte_identical_nair_and_c04_witnesses() {
    let src = source(21, "module demo; entry main;");
    let first = compile_nair_lowering_boundary(&src).unwrap();
    let second = compile_nair_lowering_boundary(&src).unwrap();
    assert_eq!(first.canonical_nair_bytes(), second.canonical_nair_bytes());
    assert_eq!(first.canonical_c04_bytes(), second.canonical_c04_bytes());
}
