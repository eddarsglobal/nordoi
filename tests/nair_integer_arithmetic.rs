use nordoi_kernel::{
    execute_nair_with_render_and_input_observed, AtomicKernel, AtomicRenderCore, InputBatch,
    Instruction, NairError, NairProgram, RegisterId, Value, NAIR_FORMAT_MINOR,
    NAIR_INTEGER_ARITHMETIC_MINOR, NAIR_LATEST_FORMAT_MINOR,
};

fn add_program(lhs: i64, rhs: i64) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(lhs),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(rhs),
        },
        Instruction::IntAddChecked {
            dst: RegisterId(2),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::Halt,
    ])
}

#[test]
fn certified_nair_06_base_minor_remains_frozen() {
    assert_eq!(NAIR_FORMAT_MINOR, 6);
    assert_eq!(NAIR_LATEST_FORMAT_MINOR, 10);
    assert_eq!(NAIR_INTEGER_ARITHMETIC_MINOR, 7);
}

#[test]
fn old_programs_keep_exact_06_header() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(42),
        },
        Instruction::Halt,
    ]);
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(program.required_format_minor(), 6);
    assert_eq!(&bytes[0..4], b"NAIR");
    assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 0);
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 6);
    assert_eq!(
        bytes,
        hex_bytes("4e41495200000600020000000100000000032a00000000000000ff")
    );
}

#[test]
fn checked_integer_add_requires_07_header() {
    let program = add_program(20, 22);
    let bytes = program.canonical_bytes().unwrap();
    assert_eq!(program.required_format_minor(), 7);
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 7);
    assert_eq!(NairProgram::from_canonical_bytes(&bytes).unwrap(), program);
}

#[test]
fn checked_integer_add_executes_to_42() {
    let program = add_program(20, 22);
    let mut kernel = AtomicKernel::new();
    let mut render = AtomicRenderCore::new();
    let input = InputBatch::default();
    let observed =
        execute_nair_with_render_and_input_observed(&mut kernel, &mut render, &input, &program)
            .unwrap();
    assert_eq!(observed.execution.execution.executed_instructions, 4);
    assert_eq!(observed.final_registers.len(), 3);
    assert_eq!(observed.register(RegisterId(2)), Some(&Value::Int(42)));
}

#[test]
fn checked_integer_add_is_deterministic() {
    let a = add_program(20, 22).canonical_bytes().unwrap();
    let b = add_program(20, 22).canonical_bytes().unwrap();
    assert_eq!(a, b);
}

#[test]
fn checked_integer_add_rejects_unknown_operand_before_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::IntAddChecked {
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
fn checked_integer_add_rejects_non_integer_operand_before_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(true),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(1),
        },
        Instruction::IntAddChecked {
            dst: RegisterId(2),
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        },
        Instruction::Halt,
    ]);
    assert_eq!(
        program.validate(),
        Err(NairError::IntegerAddOperandNotInt(RegisterId(0)))
    );
}

#[test]
fn checked_integer_add_rejects_overflow_before_execution() {
    let program = add_program(i64::MAX, 1);
    assert_eq!(
        program.validate(),
        Err(NairError::IntegerAddOverflow {
            lhs: RegisterId(0),
            rhs: RegisterId(1),
        })
    );
}

#[test]
fn checked_integer_add_preserves_ssa_destination_rule() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(2),
        },
        Instruction::IntAddChecked {
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
fn arithmetic_opcode_is_rejected_under_06_header() {
    let mut bytes = add_program(20, 22).canonical_bytes().unwrap();
    bytes[6..8].copy_from_slice(&6u16.to_le_bytes());
    assert_eq!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidOpcode(0x02))
    );
}

#[test]
fn version_07_decoder_still_accepts_06_programs() {
    let old = hex_bytes("4e41495200000600020000000100000000032a00000000000000ff");
    let decoded = NairProgram::from_canonical_bytes(&old).unwrap();
    assert_eq!(decoded.required_format_minor(), 6);
    assert_eq!(decoded.canonical_bytes().unwrap(), old);
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}
