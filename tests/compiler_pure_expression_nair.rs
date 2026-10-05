use nordoi_kernel::{
    compile_pure_expression_nair_boundary, compile_pure_result_nair_boundary, Instruction,
    NairProgram, RegisterId, SourceId, SourceText, Value,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), format!("c08-{id}.noi"), text).unwrap()
}

#[test]
fn addition_lowers_faithfully_without_constant_folding() {
    let artifact =
        compile_pure_expression_nair_boundary(&source(1, "entry main returns 20 + 22;")).unwrap();
    assert_eq!(artifact.result_i64(), Some(42));
    assert_eq!(artifact.result_register(), Some(RegisterId(2)));
    assert_eq!(artifact.nair_format_minor(), 7);
    assert_eq!(artifact.nair_instruction_count(), 4);
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
fn every_postfix_node_becomes_exactly_one_nair_instruction_plus_halt() {
    let artifact =
        compile_pure_expression_nair_boundary(&source(1, "entry main returns 1 + (2 + 3);"))
            .unwrap();
    let nodes = artifact.plan().expression().unwrap().node_count();
    assert_eq!(nodes, 5);
    assert_eq!(artifact.nair_instruction_count(), nodes + 1);
}

#[test]
fn grouped_expression_preserves_postfix_order_in_ssa_registers() {
    let artifact =
        compile_pure_expression_nair_boundary(&source(1, "entry main returns 1 + (2 + 3);"))
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
fn left_grouped_expression_preserves_distinct_postfix_order() {
    let artifact =
        compile_pure_expression_nair_boundary(&source(1, "entry main returns (1 + 2) + 3;"))
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
            Instruction::IntAddChecked {
                dst: RegisterId(2),
                lhs: RegisterId(0),
                rhs: RegisterId(1),
            },
            Instruction::Const {
                dst: RegisterId(3),
                value: Value::Int(3),
            },
            Instruction::IntAddChecked {
                dst: RegisterId(4),
                lhs: RegisterId(2),
                rhs: RegisterId(3),
            },
            Instruction::Halt,
        ]
    );
}

#[test]
fn equal_value_different_grouping_produces_distinct_nair_and_c08_witness() {
    let left = compile_pure_expression_nair_boundary(&source(1, "entry main returns (1 + 2) + 3;"))
        .unwrap();
    let right =
        compile_pure_expression_nair_boundary(&source(2, "entry main returns 1 + (2 + 3);"))
            .unwrap();
    assert_eq!(left.result_i64(), right.result_i64());
    assert_ne!(left.canonical_nair_bytes(), right.canonical_nair_bytes());
    assert_ne!(left.canonical_c08_bytes(), right.canonical_c08_bytes());
}

#[test]
fn literal_only_expression_reuses_certified_06_representation() {
    let expression =
        compile_pure_expression_nair_boundary(&source(1, "entry main returns 42;")).unwrap();
    let result = compile_pure_result_nair_boundary(&source(2, "entry main returns 42;")).unwrap();
    assert_eq!(expression.nair_format_minor(), 6);
    assert_eq!(expression.result_register(), Some(RegisterId(0)));
    assert_eq!(
        expression.canonical_nair_bytes(),
        result.canonical_nair_bytes()
    );
}

#[test]
fn zero_literal_is_not_confused_with_absent_expression() {
    let zero = compile_pure_expression_nair_boundary(&source(1, "entry main returns 0;")).unwrap();
    let none = compile_pure_expression_nair_boundary(&source(2, "entry main;")).unwrap();
    assert_eq!(zero.result_i64(), Some(0));
    assert_eq!(zero.result_register(), Some(RegisterId(0)));
    assert_eq!(zero.nair_format_minor(), 6);
    assert_eq!(none.result_i64(), None);
    assert_eq!(none.result_register(), None);
    assert_ne!(zero.canonical_nair_bytes(), none.canonical_nair_bytes());
}

#[test]
fn entry_without_expression_lowers_to_halt_only() {
    let artifact = compile_pure_expression_nair_boundary(&source(1, "entry main;")).unwrap();
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
    assert_eq!(artifact.nair_instruction_count(), 1);
    assert_eq!(artifact.nair_format_minor(), 6);
    assert_eq!(artifact.result_register(), None);
}

#[test]
fn empty_body_lowers_to_halt_only() {
    let artifact =
        compile_pure_expression_nair_boundary(&source(1, "module demo; // tail\n")).unwrap();
    assert_eq!(artifact.program().instructions(), &[Instruction::Halt]);
    assert_eq!(artifact.nair_format_minor(), 6);
}

#[test]
fn addition_program_validates_and_round_trips_as_nair_07() {
    let artifact =
        compile_pure_expression_nair_boundary(&source(1, "entry main returns 20 + 22;")).unwrap();
    artifact.program().validate().unwrap();
    let decoded = NairProgram::from_canonical_bytes(artifact.canonical_nair_bytes()).unwrap();
    assert_eq!(&decoded, artifact.program());
    assert_eq!(decoded.required_format_minor(), 7);
}

#[test]
fn c08_witness_contains_exact_c07_plan_and_nair_bytes() {
    let artifact =
        compile_pure_expression_nair_boundary(&source(1, "entry main returns 20 + 22;")).unwrap();
    let witness = artifact.canonical_c08_bytes();
    let c07 = artifact.plan().canonical_c07_bytes();
    let nair = artifact.canonical_nair_bytes();
    assert!(witness
        .windows(c07.len())
        .any(|window| window == c07.as_slice()));
    assert!(witness.windows(nair.len()).any(|window| window == nair));
}

#[test]
fn repeated_equal_source_is_byte_deterministic() {
    let a = compile_pure_expression_nair_boundary(&source(
        1,
        "module demo; entry main returns 20 + 22;",
    ))
    .unwrap();
    let b = compile_pure_expression_nair_boundary(&source(
        1,
        "module demo; entry main returns 20 + 22;",
    ))
    .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_eq!(a.canonical_c08_bytes(), b.canonical_c08_bytes());
}

#[test]
fn comments_spacing_and_source_id_do_not_change_lowering_identity() {
    let a = compile_pure_expression_nair_boundary(&source(
        1,
        "module demo; entry main returns 20 + 22;",
    ))
    .unwrap();
    let b = compile_pure_expression_nair_boundary(&source(
        99,
        "module /*m*/ demo ; entry /*e*/ main returns ( 20 /*a*/ + /*b*/ 22 ) ;",
    ))
    .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_eq!(a.canonical_c08_bytes(), b.canonical_c08_bytes());
}

#[test]
fn module_identity_changes_c08_witness_but_not_operational_nair() {
    let a =
        compile_pure_expression_nair_boundary(&source(1, "module a; entry main returns 20 + 22;"))
            .unwrap();
    let b =
        compile_pure_expression_nair_boundary(&source(2, "module b; entry main returns 20 + 22;"))
            .unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_ne!(a.canonical_c08_bytes(), b.canonical_c08_bytes());
}

#[test]
fn entry_name_changes_c08_witness_but_not_operational_nair() {
    let a =
        compile_pure_expression_nair_boundary(&source(1, "entry alpha returns 20 + 22;")).unwrap();
    let b =
        compile_pure_expression_nair_boundary(&source(2, "entry beta returns 20 + 22;")).unwrap();
    assert_eq!(a.canonical_nair_bytes(), b.canonical_nair_bytes());
    assert_ne!(a.canonical_c08_bytes(), b.canonical_c08_bytes());
}

#[test]
fn declared_effect_does_not_become_work_effect_or_authority() {
    let artifact = compile_pure_expression_nair_boundary(&source(
        1,
        "effect Network; entry main returns 20 + 22;",
    ))
    .unwrap();
    assert_eq!(artifact.semantic_work_item_count(), 0);
    assert!(artifact.plan().required_effects().is_empty());
    assert!(!artifact.requires_host_authority());
}

#[test]
fn final_result_register_is_last_ssa_definition() {
    let artifact =
        compile_pure_expression_nair_boundary(&source(1, "entry main returns 1 + 2 + 3 + 4;"))
            .unwrap();
    let nodes = artifact.plan().expression().unwrap().node_count();
    assert_eq!(
        artifact.result_register(),
        Some(RegisterId((nodes - 1) as u32))
    );
}

#[test]
fn l07_overflow_fails_before_c08_publication() {
    assert!(compile_pure_expression_nair_boundary(&source(
        1,
        "entry main returns 9223372036854775807 + 1;",
    ))
    .is_err());
}

#[test]
fn unsupported_operator_fails_before_c08_publication() {
    assert!(
        compile_pure_expression_nair_boundary(&source(1, "entry main returns 6 * 7;",)).is_err()
    );
}
