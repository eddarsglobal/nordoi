use nordoi_kernel::{
    execute_nair_with_render_and_input_observed, AtomicKernel, AtomicRenderCore, InputBatch,
    Instruction, NairError, NairProgram, RegisterId, Value, NAIR_FORMAT_MINOR,
    NAIR_INTEGER_ARITHMETIC_MINOR, NAIR_INTEGER_COMPARISON_MINOR, NAIR_LATEST_FORMAT_MINOR,
};

#[derive(Clone, Copy)]
enum Cmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

fn compare_instruction(
    kind: Cmp,
    dst: RegisterId,
    lhs: RegisterId,
    rhs: RegisterId,
) -> Instruction {
    match kind {
        Cmp::Eq => Instruction::IntEq { dst, lhs, rhs },
        Cmp::Ne => Instruction::IntNe { dst, lhs, rhs },
        Cmp::Lt => Instruction::IntLt { dst, lhs, rhs },
        Cmp::Le => Instruction::IntLe { dst, lhs, rhs },
        Cmp::Gt => Instruction::IntGt { dst, lhs, rhs },
        Cmp::Ge => Instruction::IntGe { dst, lhs, rhs },
    }
}

fn compare_program(kind: Cmp, lhs: i64, rhs: i64) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(lhs),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(rhs),
        },
        compare_instruction(kind, RegisterId(2), RegisterId(0), RegisterId(1)),
        Instruction::Halt,
    ])
}

fn execute_result(program: &NairProgram) -> Value {
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let input = InputBatch::default();
    let observed =
        execute_nair_with_render_and_input_observed(&mut kernel, &mut render, &input, program)
            .unwrap();
    assert_eq!(observed.execution.execution.executed_instructions, 4);
    assert_eq!(observed.execution.execution.created_domains, 0);
    assert_eq!(observed.execution.execution.created_atoms, 0);
    assert_eq!(observed.execution.execution.committed_transactions, 0);
    assert_eq!(observed.execution.execution.rolled_back_transactions, 0);
    assert_eq!(observed.execution.execution.scheduled_work, 0);
    assert_eq!(observed.final_registers.len(), 3);
    observed.register(RegisterId(2)).unwrap().clone()
}

#[test]
fn nair_08_extends_latest_minor_without_rewriting_certified_base() {
    assert_eq!(NAIR_FORMAT_MINOR, 6);
    assert_eq!(NAIR_INTEGER_ARITHMETIC_MINOR, 7);
    assert_eq!(NAIR_INTEGER_COMPARISON_MINOR, 8);
    assert_eq!(NAIR_LATEST_FORMAT_MINOR, 14);
}

#[test]
fn old_06_const_program_keeps_exact_bytes() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(42),
        },
        Instruction::Halt,
    ]);
    assert_eq!(program.required_format_minor(), 6);
    assert_eq!(
        program.canonical_bytes().unwrap(),
        hex_bytes("4e41495200000600020000000100000000032a00000000000000ff")
    );
}

#[test]
fn old_07_add_program_stays_07() {
    let program = NairProgram::from_instructions(vec![
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
    ]);
    assert_eq!(program.required_format_minor(), 7);
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 7);
}

#[test]
fn every_integer_comparison_requires_08() {
    for kind in [Cmp::Eq, Cmp::Ne, Cmp::Lt, Cmp::Le, Cmp::Gt, Cmp::Ge] {
        let program = compare_program(kind, 20, 22);
        let bytes = program.canonical_bytes().unwrap();
        assert_eq!(program.required_format_minor(), 8);
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 8);
        assert_eq!(NairProgram::from_canonical_bytes(&bytes).unwrap(), program);
    }
}

#[test]
fn equality_and_inequality_execute_to_bool() {
    assert_eq!(
        execute_result(&compare_program(Cmp::Eq, 22, 22)),
        Value::Bool(true)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Eq, 20, 22)),
        Value::Bool(false)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Ne, 20, 22)),
        Value::Bool(true)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Ne, 22, 22)),
        Value::Bool(false)
    );
}

#[test]
fn ordered_comparisons_execute_to_bool() {
    assert_eq!(
        execute_result(&compare_program(Cmp::Lt, 20, 22)),
        Value::Bool(true)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Lt, 22, 20)),
        Value::Bool(false)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Le, 22, 22)),
        Value::Bool(true)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Gt, 22, 20)),
        Value::Bool(true)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Gt, 20, 22)),
        Value::Bool(false)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Ge, 22, 22)),
        Value::Bool(true)
    );
}

#[test]
fn signed_integer_order_is_exact() {
    assert_eq!(
        execute_result(&compare_program(Cmp::Lt, -1, 0)),
        Value::Bool(true)
    );
    assert_eq!(
        execute_result(&compare_program(Cmp::Gt, i64::MAX, i64::MIN)),
        Value::Bool(true)
    );
}

#[test]
fn comparison_rejects_unknown_operand_before_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::IntEq {
            dst: RegisterId(2),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        program.validate(),
        Err(NairError::UnknownRegister(RegisterId(1)))
    );
}

#[test]
fn comparison_rejects_non_integer_operand_before_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(true),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(1),
        },
        Instruction::IntEq {
            dst: RegisterId(2),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        program.validate(),
        Err(NairError::IntegerCompareOperandNotInt(RegisterId(0)))
    );
}

#[test]
fn comparison_preserves_ssa_destination_rule() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(1),
        },
        Instruction::IntEq {
            dst: RegisterId(1),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        program.validate(),
        Err(NairError::DuplicateRegister(RegisterId(1)))
    );
}

#[test]
fn comparison_opcode_is_rejected_under_07_header() {
    let mut bytes = compare_program(Cmp::Eq, 20, 22).canonical_bytes().unwrap();
    bytes[6..8].copy_from_slice(&7u16.to_le_bytes());
    assert_eq!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidOpcode(0x03))
    );
}

#[test]
fn int_le_has_exact_canonical_08_bytes() {
    let bytes = compare_program(Cmp::Le, 20, 22).canonical_bytes().unwrap();
    assert_eq!(
        bytes,
        hex_bytes("4e41495200000800040000000100000000031400000000000000010100000003160000000000000006020000000000000001000000ff")
    );
}

#[test]
fn distinct_comparison_operators_have_distinct_canonical_bytes() {
    let eq = compare_program(Cmp::Eq, 20, 22).canonical_bytes().unwrap();
    let ne = compare_program(Cmp::Ne, 20, 22).canonical_bytes().unwrap();
    let lt = compare_program(Cmp::Lt, 20, 22).canonical_bytes().unwrap();
    assert_ne!(eq, ne);
    assert_ne!(eq, lt);
    assert_ne!(ne, lt);
}

#[test]
fn comparison_encoding_is_deterministic() {
    let a = compare_program(Cmp::Le, 20, 22).canonical_bytes().unwrap();
    let b = compare_program(Cmp::Le, 20, 22).canonical_bytes().unwrap();
    assert_eq!(a, b);
}

#[test]
fn bool_const_remains_base_06_and_needs_no_comparison_opcode() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(true),
        },
        Instruction::Halt,
    ]);
    assert_eq!(program.required_format_minor(), 6);
    assert_eq!(execute_result_two_instruction(&program), Value::Bool(true));
}

fn execute_result_two_instruction(program: &NairProgram) -> Value {
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let input = InputBatch::default();
    let observed =
        execute_nair_with_render_and_input_observed(&mut kernel, &mut render, &input, program)
            .unwrap();
    assert_eq!(observed.execution.execution.executed_instructions, 2);
    observed.register(RegisterId(0)).unwrap().clone()
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}
