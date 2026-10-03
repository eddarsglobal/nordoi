use nordoi_kernel::{AtomicError, AtomicKernel, DomainId, Value};

#[test]
fn transaction_is_invisible_until_commit() {
    let mut kernel = AtomicKernel::new();
    let domain = kernel.create_domain("wallet").unwrap();
    let balance = kernel.create_atom_owned(domain, 100_i64).unwrap();

    let mut tx = kernel.begin_transaction(domain).unwrap();
    tx.set(&kernel, balance, 75_i64).unwrap();

    assert_eq!(kernel.get(balance).unwrap(), &Value::Int(100));
    assert_eq!(kernel.pending_work(), 0);

    let report = kernel.commit(tx).unwrap();
    assert_eq!(report.changed_atoms, 1);
    assert_eq!(kernel.get(balance).unwrap(), &Value::Int(75));
}

#[test]
fn rollback_is_zero_work() {
    let mut kernel = AtomicKernel::new();
    let domain = kernel.create_domain("editor").unwrap();
    let atom = kernel.create_atom_owned(domain, "before").unwrap();

    let mut tx = kernel.begin_transaction(domain).unwrap();
    tx.set(&kernel, atom, "after").unwrap();
    let report = tx.rollback();

    assert_eq!(report.changed_atoms, 0);
    assert_eq!(report.scheduled_atoms, 0);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Text("before".into()));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn non_owner_cannot_stage_write() {
    let mut kernel = AtomicKernel::new();
    let owner = kernel.create_domain("owner").unwrap();
    let intruder = kernel.create_domain("intruder").unwrap();
    let atom = kernel.create_atom_owned(owner, 1_i64).unwrap();

    let mut tx = kernel.begin_transaction(intruder).unwrap();
    let result = tx.set(&kernel, atom, 2_i64);

    assert_eq!(
        result,
        Err(AtomicError::OwnershipViolation {
            atom,
            owner,
            attempted: intruder,
        })
    );
}

#[test]
fn failed_commit_changes_nothing() {
    let mut kernel = AtomicKernel::new();
    let domain = kernel.create_domain("account").unwrap();
    let a = kernel.create_atom_owned(domain, 10_i64).unwrap();
    let b = kernel.create_atom_owned(domain, 20_i64).unwrap();

    let mut tx = kernel.begin_transaction(domain).unwrap();
    tx.set(&kernel, a, 11_i64).unwrap();
    tx.set(&kernel, b, 21_i64).unwrap();

    // Concurrent/direct owner mutation makes the transaction stale.
    kernel.set_owned(domain, a, 99_i64).unwrap();
    kernel.flush();

    let result = kernel.commit(tx);
    assert!(matches!(result, Err(AtomicError::TransactionConflict { atom, .. }) if atom == a));

    // `b` must remain untouched: commit was all-or-nothing.
    assert_eq!(kernel.get(a).unwrap(), &Value::Int(99));
    assert_eq!(kernel.get(b).unwrap(), &Value::Int(20));
}

#[test]
fn multiple_writes_to_same_atom_collapse_inside_transaction() {
    let mut kernel = AtomicKernel::new();
    let domain = kernel.create_domain("counter").unwrap();
    let atom = kernel.create_atom_owned(domain, 0_i64).unwrap();

    let mut tx = kernel.begin_transaction(domain).unwrap();
    tx.set(&kernel, atom, 1_i64).unwrap();
    tx.set(&kernel, atom, 2_i64).unwrap();
    tx.set(&kernel, atom, 3_i64).unwrap();

    assert_eq!(tx.staged_writes(), 1);
    let report = kernel.commit(tx).unwrap();
    assert_eq!(report.staged_writes, 1);
    assert_eq!(report.changed_atoms, 1);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Int(3));
}

#[test]
fn transaction_schedules_dependency_union_once() {
    let mut kernel = AtomicKernel::new();
    let domain = kernel.create_domain("ui").unwrap();
    let a = kernel.create_atom_owned(domain, 0_i64).unwrap();
    let b = kernel.create_atom_owned(domain, 0_i64).unwrap();
    let shared = kernel.create_atom_owned(domain, 0_i64).unwrap();

    kernel.connect(a, shared).unwrap();
    kernel.connect(b, shared).unwrap();

    let mut tx = kernel.begin_transaction(domain).unwrap();
    tx.set(&kernel, a, 1_i64).unwrap();
    tx.set(&kernel, b, 1_i64).unwrap();

    let report = kernel.commit(tx).unwrap();
    assert_eq!(report.changed_atoms, 2);
    assert_eq!(report.scheduled_atoms, 3);
    assert_eq!(kernel.pending_work(), 3);
}

#[test]
fn ownership_transfer_is_explicit() {
    let mut kernel = AtomicKernel::new();
    let first = kernel.create_domain("first").unwrap();
    let second = kernel.create_domain("second").unwrap();
    let atom = kernel.create_atom_owned(first, 1_i64).unwrap();

    kernel.transfer_atom(atom, first, second).unwrap();
    assert_eq!(kernel.owner_of(atom).unwrap(), second);

    assert_eq!(
        kernel.set_owned(first, atom, 2_i64),
        Err(AtomicError::OwnershipViolation {
            atom,
            owner: second,
            attempted: first,
        })
    );

    assert!(kernel.set_owned(second, atom, 2_i64).unwrap());
}

#[test]
fn unknown_domain_is_rejected() {
    let mut kernel = AtomicKernel::new();
    assert!(matches!(
        kernel.begin_transaction(DomainId(9_999)),
        Err(AtomicError::UnknownDomain(DomainId(9_999)))
    ));
}
