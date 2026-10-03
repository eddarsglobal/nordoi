use nordoi_kernel::{
    execute_nair, AtomSlot, AtomicKernel, DomainRef, DomainSlot, Instruction, NairError,
    NairProgram, RegisterId, TransactionSlot, Value,
};

fn sample_program() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(100),
        },
        Instruction::CreateDomain {
            dst: DomainSlot(0),
            name: "wallet".into(),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Slot(DomainSlot(0)),
            value: RegisterId(0),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(75),
        },
        Instruction::BeginTransaction {
            dst: TransactionSlot(0),
            domain: DomainRef::Slot(DomainSlot(0)),
        },
        Instruction::TxSet {
            tx: TransactionSlot(0),
            atom: AtomSlot(0),
            value: RegisterId(1),
        },
        Instruction::Commit {
            tx: TransactionSlot(0),
        },
        Instruction::Halt,
    ])
}

#[test]
fn canonical_round_trip_is_byte_stable() {
    let program = sample_program();
    let bytes = program.canonical_bytes().unwrap();
    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    let reencoded = decoded.canonical_bytes().unwrap();

    assert_eq!(program, decoded);
    assert_eq!(bytes, reencoded);
    assert_eq!(&bytes[0..4], b"NAIR");
}

#[test]
fn canonical_encoding_is_deterministic() {
    let a = sample_program().canonical_bytes().unwrap();
    let b = sample_program().canonical_bytes().unwrap();
    assert_eq!(a, b);
}

#[test]
fn invalid_magic_is_rejected() {
    let mut bytes = sample_program().canonical_bytes().unwrap();
    bytes[0] = b'X';
    assert_eq!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidMagic)
    );
}

#[test]
fn register_must_exist_before_use() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(99),
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownRegister(RegisterId(99)))
    );
}

#[test]
fn registers_are_single_assignment() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(1),
        },
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(2),
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::DuplicateRegister(RegisterId(0)))
    );
}

#[test]
fn non_finite_float_is_not_representable_in_valid_nair() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(7),
            value: Value::Float(f64::NAN),
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::NonFiniteFloat(RegisterId(7)))
    );
}

#[test]
fn self_dependency_is_rejected_before_nam_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::Connect {
            source: AtomSlot(0),
            dependent: AtomSlot(0),
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::SelfDependency(AtomSlot(0)))
    );
}

#[test]
fn transactions_must_close_before_halt() {
    let program = NairProgram::from_instructions(vec![
        Instruction::BeginTransaction {
            dst: TransactionSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnclosedTransactions(vec![TransactionSlot(0)]))
    );
}

#[test]
fn nair_executes_atomic_commit_on_nam() {
    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &sample_program()).unwrap();
    let atom = report.atom_bindings[&AtomSlot(0)];

    assert_eq!(kernel.get(atom).unwrap(), &Value::Int(75));
    assert_eq!(report.committed_transactions, 1);
    assert_eq!(report.rolled_back_transactions, 0);
    assert_eq!(report.scheduled_work, 1);
}

#[test]
fn nair_preserves_no_work_without_effect() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(5),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::BeginTransaction {
            dst: TransactionSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::TxSet {
            tx: TransactionSlot(0),
            atom: AtomSlot(0),
            value: RegisterId(0),
        },
        Instruction::Commit {
            tx: TransactionSlot(0),
        },
        Instruction::Halt,
    ]);

    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program).unwrap();

    assert_eq!(report.transaction_reports[0].changed_atoms, 0);
    assert_eq!(report.scheduled_work, 0);
}

#[test]
fn nair_rollback_never_mutates_atom() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Text("before".into()),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Text("after".into()),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::BeginTransaction {
            dst: TransactionSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::TxSet {
            tx: TransactionSlot(0),
            atom: AtomSlot(0),
            value: RegisterId(1),
        },
        Instruction::Rollback {
            tx: TransactionSlot(0),
        },
        Instruction::Halt,
    ]);

    let mut kernel = AtomicKernel::new();
    let report = execute_nair(&mut kernel, &program).unwrap();
    let atom = report.atom_bindings[&AtomSlot(0)];

    assert_eq!(kernel.get(atom).unwrap(), &Value::Text("before".into()));
    assert_eq!(report.rolled_back_transactions, 1);
    assert_eq!(report.scheduled_work, 0);
}
