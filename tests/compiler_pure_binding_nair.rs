use nordoi_kernel::{
    compile_pure_binding_nair_boundary, compile_pure_expression_nair_boundary, Instruction,
    NairProgram, RegisterId, SourceId, SourceText, Value,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("c010-{id}.noi"), text).unwrap()
}

#[test]
fn binding_addition_erases_names_to_existing_nair_without_runtime_lookup() {
    let artifact = compile_pure_binding_nair_boundary(&source(
        1,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
    assert_eq!(artifact.result_i64(), Some(42));
    assert_eq!(artifact.result_register(), Some(RegisterId(2)));
    assert_eq!(artifact.nair_format_minor(), 7);
    assert_eq!(artifact.nair_instruction_count(), 4);
    assert_eq!(artifact.runtime_storage_item_count(), 0);
    assert_eq!(
        artifact.program().instructions(),
        &[
            Instruction::Const {
                dst: RegisterId(0),
                value: Value::Int(20),
            },
            Instruction::Const {
                dst: RegisterId(1),
                value: Value::Int(22),
            },
            Instruction::IntAddChecked {
                dst: RegisterId(2),
                lhs: RegisterId(0),
                rhs: RegisterId(1),
            },
            Instruction::Halt,
        ]
    );
}

#[test]
fn binding_names_have_zero_operational_cost_vs_equivalent_literals() {
    let bindings = compile_pure_binding_nair_boundary(&source(
        2,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
    let literals =
        compile_pure_expression_nair_boundary(&source(3, "entry main returns 20 + 22;")).unwrap();
    assert_eq!(
        bindings.canonical_nair_bytes(),
        literals.canonical_nair_bytes()
    );
    assert_eq!(
        bindings.nair_instruction_count(),
        literals.nair_instruction_count()
    );
    assert_eq!(bindings.result_register(), literals.result_register());
}

#[test]
fn unused_binding_costs_zero_nair_instructions() {
    let with_unused = compile_pure_binding_nair_boundary(&source(
        4,
        "const unused = 999; entry main returns 42;",
    ))
    .unwrap();
    let without = compile_pure_binding_nair_boundary(&source(5, "entry main returns 42;")).unwrap();
    assert_eq!(
        with_unused.canonical_nair_bytes(),
        without.canonical_nair_bytes()
    );
    assert_eq!(with_unused.nair_instruction_count(), 2);
    assert_eq!(with_unused.runtime_storage_item_count(), 0);
}

#[test]
fn multiple_unused_bindings_still_cost_zero_operational_work() {
    let many = compile_pure_binding_nair_boundary(&source(
        6,
        "const a = 1; const b = 2; const c = 3; entry main returns 42;",
    ))
    .unwrap();
    assert_eq!(
        many.program().instructions(),
        &[
            Instruction::Const {
                dst: RegisterId(0),
                value: Value::Int(42),
            },
            Instruction::Halt,
        ]
    );
    assert_eq!(many.runtime_storage_item_count(), 0);
}

#[test]
fn binding_reference_and_equal_literal_share_nair_but_not_c010_identity() {
    let by_binding =
        compile_pure_binding_nair_boundary(&source(7, "const x = 42; entry main returns x;"))
            .unwrap();
    let by_literal =
        compile_pure_binding_nair_boundary(&source(8, "const x = 42; entry main returns 42;"))
            .unwrap();
    assert_eq!(
        by_binding.canonical_nair_bytes(),
        by_literal.canonical_nair_bytes()
    );
    assert_ne!(
        by_binding.plan().canonical_c09_bytes(),
        by_literal.plan().canonical_c09_bytes()
    );
    assert_ne!(
        by_binding.canonical_c010_bytes(),
        by_literal.canonical_c010_bytes()
    );
}

#[test]
fn single_binding_literal_stays_on_certified_nair_06() {
    let artifact = compile_pure_binding_nair_boundary(&source(
        9,
        "const answer = 42; entry main returns answer;",
    ))
    .unwrap();
    assert_eq!(artifact.nair_format_minor(), 6);
    assert_eq!(artifact.result_register(), Some(RegisterId(0)));
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
fn grouped_binding_expression_preserves_postfix_calculation_order() {
    let artifact = compile_pure_binding_nair_boundary(&source(
        10,
        "const x = 1; const y = 2; const z = 3; entry main returns x + (y + z);",
    ))
    .unwrap();
    assert_eq!(artifact.result_register(), Some(RegisterId(4)));
    assert_eq!(
        artifact.program().instructions(),
        &[
            Instruction::Const {
                dst: RegisterId(0),
                value: Value::Int(1),
            },
            Instruction::Const {
                dst: RegisterId(1),
                value: Value::Int(2),
            },
            Instruction::Const {
                dst: RegisterId(2),
                value: Value::Int(3),
            },
            Instruction::IntAddChecked {
                dst: RegisterId(3),
                lhs: RegisterId(1),
                rhs: RegisterId(2),
            },
            Instruction::IntAddChecked {
                dst: RegisterId(4),
                lhs: RegisterId(0),
                rhs: RegisterId(3),
            },
            Instruction::Halt,
        ]
    );
}

#[test]
fn equal_value_different_grouping_remains_distinct_operational_nair() {
    let left = compile_pure_binding_nair_boundary(&source(
        11,
        "const x = 1; const y = 2; const z = 3; entry main returns (x + y) + z;",
    ))
    .unwrap();
    let right = compile_pure_binding_nair_boundary(&source(
        12,
        "const x = 1; const y = 2; const z = 3; entry main returns x + (y + z);",
    ))
    .unwrap();
    assert_eq!(left.result_i64(), Some(6));
    assert_eq!(right.result_i64(), Some(6));
    assert_ne!(left.canonical_nair_bytes(), right.canonical_nair_bytes());
    assert_ne!(left.canonical_c010_bytes(), right.canonical_c010_bytes());
}

#[test]
fn declaration_order_changes_neither_operational_nair_nor_c010_witness() {
    let a = compile_pure_binding_nair_boundary(&source(
        13,
        "const x = 20; const y = 22; entry main returns x + y;",
    ))
    .unwrap();
    let b = compile_pure_binding_nair_boundary(&source(
        14,
        "const y = 22; const x = 20; entry main returns x + y;",
    ))
    .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_eq!(a.canonical_c010_bytes(), b.canonical_c010_bytes());
}

#[test]
fn comments_spacing_and_source_id_do_not_change_c010_identity() {
    let a = compile_pure_binding_nair_boundary(&source(
        15,
        "module demo; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let b = compile_pure_binding_nair_boundary(&source(
        99,
        "module /*m*/ demo; const /*c*/ x = 20; entry main returns (x) + /*v*/ 22;",
    ))
    .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_eq!(a.canonical_c010_bytes(), b.canonical_c010_bytes());
}

#[test]
fn module_identity_changes_c010_witness_but_not_operational_nair() {
    let a = compile_pure_binding_nair_boundary(&source(
        16,
        "module a; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let b = compile_pure_binding_nair_boundary(&source(
        17,
        "module b; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_ne!(a.canonical_c010_bytes(), b.canonical_c010_bytes());
}

#[test]
fn entry_name_changes_c010_witness_but_not_operational_nair() {
    let a = compile_pure_binding_nair_boundary(&source(
        18,
        "const x = 20; entry alpha returns x + 22;",
    ))
    .unwrap();
    let b =
        compile_pure_binding_nair_boundary(&source(19, "const x = 20; entry beta returns x + 22;"))
            .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_ne!(a.canonical_c010_bytes(), b.canonical_c010_bytes());
}

#[test]
fn c010_witness_contains_exact_c09_plan_and_nair_bytes() {
    let artifact =
        compile_pure_binding_nair_boundary(&source(20, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    let witness = artifact.canonical_c010_bytes();
    let c09 = artifact.plan().canonical_c09_bytes();
    let nair = artifact.canonical_nair_bytes();
    assert!(witness
        .windows(c09.len())
        .any(|window| window == c09.as_slice()));
    assert!(witness.windows(nair.len()).any(|window| window == nair));
    assert!(witness.starts_with(b"NORDOI-C0.10-PURE-BINDING-NAIR\0"));
}

#[test]
fn nair_07_binding_program_validates_and_round_trips() {
    let artifact =
        compile_pure_binding_nair_boundary(&source(21, "const x = 20; entry main returns x + 22;"))
            .unwrap();
    artifact.program().validate().unwrap();
    let decoded = NairProgram::from_canonical_bytes(artifact.canonical_nair_bytes()).unwrap();
    assert_eq!(&decoded, artifact.program());
    assert_eq!(decoded.required_format_minor(), 7);
}

#[test]
fn empty_body_lowers_to_halt_only_without_binding_state() {
    let artifact =
        compile_pure_binding_nair_boundary(&source(22, "module demo; // tail\n")).unwrap();
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
    assert_eq!(artifact.nair_format_minor(), 6);
    assert_eq!(artifact.result_register(), None);
    assert_eq!(artifact.runtime_storage_item_count(), 0);
}

#[test]
fn entry_without_expression_lowers_to_halt_only_even_with_bindings() {
    let artifact =
        compile_pure_binding_nair_boundary(&source(23, "const x = 20; const y = 22; entry main;"))
            .unwrap();
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
    assert_eq!(artifact.nair_instruction_count(), 1);
    assert_eq!(artifact.nair_format_minor(), 6);
    assert_eq!(artifact.runtime_storage_item_count(), 0);
}

#[test]
fn zero_result_is_distinct_from_no_expression() {
    let zero =
        compile_pure_binding_nair_boundary(&source(24, "const z = 0; entry main returns z;"))
            .unwrap();
    let none = compile_pure_binding_nair_boundary(&source(25, "const z = 0; entry main;")).unwrap();
    assert_eq!(zero.result_i64(), Some(0));
    assert_eq!(zero.result_register(), Some(RegisterId(0)));
    assert_eq!(none.result_i64(), None);
    assert_eq!(none.result_register(), None);
    assert_ne!(zero.canonical_nair_bytes(), none.canonical_nair_bytes());
}

#[test]
fn declared_effect_name_grants_no_effect_or_authority_to_binding_lowering() {
    let artifact = compile_pure_binding_nair_boundary(&source(
        26,
        "effect Network; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    assert_eq!(artifact.semantic_work_item_count(), 0);
    assert_eq!(artifact.runtime_storage_item_count(), 0);
    assert!(artifact.plan().required_effects().is_empty());
    assert!(!artifact.requires_host_authority());
}

#[test]
fn repeated_equal_source_is_byte_deterministic() {
    let a = compile_pure_binding_nair_boundary(&source(
        27,
        "module demo; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    let b = compile_pure_binding_nair_boundary(&source(
        27,
        "module demo; const x = 20; entry main returns x + 22;",
    ))
    .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_eq!(a.canonical_c010_bytes(), b.canonical_c010_bytes());
}

#[test]
fn overflow_fails_before_c010_publication() {
    assert!(compile_pure_binding_nair_boundary(&source(
        28,
        "const max = 9223372036854775807; entry main returns max + 1;",
    ))
    .is_err());
}
