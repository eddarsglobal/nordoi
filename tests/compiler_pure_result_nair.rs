use nordoi_kernel::{
    compile_nair_lowering_boundary, compile_pure_result_execution_plan_boundary,
    compile_pure_result_nair_boundary, execute_source_v01, Instruction, NairProgram,
    PureResultPlanForm, RegisterId, SourceExecutionError, SourceId, SourceText, Value,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "result-nair.noi", text).unwrap()
}

#[test]
fn integer_result_lowers_to_const_r0_then_halt() {
    let artifact = compile_pure_result_nair_boundary(&source(1, "entry main returns 42;")).unwrap();
    assert_eq!(artifact.result_register(), Some(RegisterId(0)));
    assert_eq!(artifact.result_i64(), Some(42));
    assert_eq!(
        artifact.program().instructions(),
        &[
            Instruction::Const {
                dst: RegisterId(0),
                value: Value::Int(42),
            },
            Instruction::Halt,
        ]
    );
}

#[test]
fn entry_without_result_remains_halt_only() {
    let artifact = compile_pure_result_nair_boundary(&source(1, "entry main;")).unwrap();
    assert_eq!(artifact.result_register(), None);
    assert_eq!(artifact.result_i64(), None);
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
}

#[test]
fn empty_body_remains_halt_only() {
    let artifact = compile_pure_result_nair_boundary(&source(1, "module demo; // tail\n")).unwrap();
    assert_eq!(artifact.result_register(), None);
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
}

#[test]
fn lowered_program_is_valid_existing_nair_0_6() {
    let artifact = compile_pure_result_nair_boundary(&source(1, "entry main returns 42;")).unwrap();
    artifact.program().validate().unwrap();
    let decoded = NairProgram::from_canonical_bytes(artifact.canonical_nair_bytes()).unwrap();
    assert_eq!(&decoded, artifact.program());
}

#[test]
fn c06_witness_has_explicit_domain() {
    let artifact = compile_pure_result_nair_boundary(&source(1, "entry main returns 42;")).unwrap();
    assert!(artifact
        .canonical_c06_bytes()
        .starts_with(b"NORDOI-C0.6-PURE-RESULT-NAIR\0"));
}

#[test]
fn c06_preserves_exact_c05_witness() {
    let src = source(1, "module demo; entry main returns 42;");
    let c05 = compile_pure_result_execution_plan_boundary(&src).unwrap();
    let c06 = compile_pure_result_nair_boundary(&src).unwrap();
    assert_eq!(c05.canonical_c05_bytes(), c06.plan().canonical_c05_bytes());
}

#[test]
fn c06_witness_contains_exact_nair_bytes() {
    let artifact = compile_pure_result_nair_boundary(&source(1, "entry main returns 42;")).unwrap();
    let witness = artifact.canonical_c06_bytes();
    assert!(witness
        .windows(artifact.canonical_nair_bytes().len())
        .any(|window| window == artifact.canonical_nair_bytes()));
}

#[test]
fn changing_result_changes_nair_bytes_and_c06_witness() {
    let a = compile_pure_result_nair_boundary(&source(1, "entry main returns 1;")).unwrap();
    let b = compile_pure_result_nair_boundary(&source(2, "entry main returns 2;")).unwrap();
    assert_ne!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_ne!(a.canonical_c06_bytes(), b.canonical_c06_bytes());
}

#[test]
fn changing_entry_name_keeps_operational_nair_but_changes_c06_witness() {
    let a = compile_pure_result_nair_boundary(&source(1, "entry alpha returns 42;")).unwrap();
    let b = compile_pure_result_nair_boundary(&source(2, "entry beta returns 42;")).unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_ne!(a.canonical_c06_bytes(), b.canonical_c06_bytes());
}

#[test]
fn module_identity_keeps_operational_nair_but_changes_c06_witness() {
    let a =
        compile_pure_result_nair_boundary(&source(1, "module a; entry main returns 42;")).unwrap();
    let b =
        compile_pure_result_nair_boundary(&source(2, "module b; entry main returns 42;")).unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_ne!(a.canonical_c06_bytes(), b.canonical_c06_bytes());
}

#[test]
fn comments_spacing_and_source_id_do_not_change_c06_identity() {
    let a = compile_pure_result_nair_boundary(&source(1, "module demo; entry main returns 42;"))
        .unwrap();
    let b = compile_pure_result_nair_boundary(&source(
        99,
        "module /*x*/ demo ;\n entry /*a*/ main /*b*/ returns /*c*/ 42 /*d*/ ;",
    ))
    .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_eq!(a.canonical_c06_bytes(), b.canonical_c06_bytes());
}

#[test]
fn declared_effect_does_not_become_nair_work_or_authority() {
    let artifact =
        compile_pure_result_nair_boundary(&source(1, "effect Network; entry main returns 7;"))
            .unwrap();
    assert_eq!(artifact.semantic_work_item_count(), 0);
    assert!(artifact.plan().required_effects().is_empty());
    assert!(!artifact.requires_host_authority());
    assert_eq!(artifact.nair_instruction_count(), 2);
}

#[test]
fn zero_result_is_encoded_as_const_not_no_result() {
    let zero = compile_pure_result_nair_boundary(&source(1, "entry main returns 0;")).unwrap();
    let none = compile_pure_result_nair_boundary(&source(2, "entry main;")).unwrap();
    assert_eq!(zero.result_register(), Some(RegisterId(0)));
    assert_eq!(zero.result_i64(), Some(0));
    assert_ne!(zero.canonical_nair_bytes(), none.canonical_nair_bytes());
}

#[test]
fn int_max_is_preserved_exactly_in_nair() {
    let artifact =
        compile_pure_result_nair_boundary(&source(1, "entry main returns 9223372036854775807;"))
            .unwrap();
    assert_eq!(artifact.result_i64(), Some(i64::MAX));
    match artifact.program().instructions().first().unwrap() {
        Instruction::Const { dst, value } => {
            assert_eq!(*dst, RegisterId(0));
            assert_eq!(value, &Value::Int(i64::MAX));
        }
        other => panic!("expected leading Const, got {other:?}"),
    }
}

#[test]
fn canonical_result_register_is_always_r0() {
    for value in [0_i64, 1, 42, i64::MAX] {
        let src = source(1, &format!("entry main returns {value};"));
        let artifact = compile_pure_result_nair_boundary(&src).unwrap();
        assert_eq!(artifact.result_register(), Some(RegisterId(0)));
    }
}

#[test]
fn c04_lowering_remains_frozen_and_rejects_result_source() {
    assert!(compile_nair_lowering_boundary(&source(1, "entry main returns 42;")).is_err());
}

#[test]
fn v01_execution_remains_frozen_and_rejects_result_source() {
    let error = execute_source_v01(&source(1, "entry main returns 42;")).unwrap_err();
    assert!(matches!(error, SourceExecutionError::Compiler(_)));
}

#[test]
fn c06_is_representation_only_and_does_not_claim_runtime_execution() {
    let artifact = compile_pure_result_nair_boundary(&source(1, "entry main returns 42;")).unwrap();
    assert!(matches!(
        artifact.plan().form(),
        PureResultPlanForm::Entry(_)
    ));
    assert_eq!(artifact.nair_instruction_count(), 2);
    assert_eq!(artifact.semantic_work_item_count(), 0);
}
