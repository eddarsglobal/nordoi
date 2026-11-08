//! G0.3 R3-R13: semantics cannot weaken when a duplicate lookup is removed.
use nordoi_kernel::{AtomId, AtomicError, AtomicKernel, Capability, DomainId, Value};

#[test]
fn no_op_set_is_zero_work_and_missing_atom_still_fails() {
    let mut k = AtomicKernel::new();
    let id = k.create_atom(7_i64);
    for _ in 0..100 { assert!(!k.set(id, 7_i64).unwrap()); }
    assert_eq!(k.version(id).unwrap(), 0);
    assert_eq!(k.pending_work(), 0);
    assert_eq!(k.set(AtomId(9999), 1_i64), Err(AtomicError::UnknownAtom(AtomId(9999))));
    assert_eq!(k.require(&Capability::ConsoleWrite),
        Err(AtomicError::CapabilityDenied(Capability::ConsoleWrite)));
}

#[test]
fn changed_write_and_dependency_stay_deduplicated() {
    let mut k = AtomicKernel::new();
    let a = k.create_atom(0_i64);
    let dependent = k.create_atom(0_i64);
    k.connect(a, dependent).unwrap();
    assert!(k.set(a, 1_i64).unwrap());
    assert!(k.set(a, 2_i64).unwrap());
    assert_eq!(k.pending_work(), 2);
    assert_eq!(k.version(a).unwrap(), 2);
    assert_eq!(k.get(a).unwrap(), &Value::Int(2));
    assert_eq!(k.flush().len(), 2);
    assert_eq!(k.pending_work(), 0);
}

#[test]
fn owner_guard_and_noop_commit_remain_valid() {
    let mut k = AtomicKernel::new();
    let owner = k.create_domain("owner").unwrap();
    let intruder = k.create_domain("intruder").unwrap();
    let id = k.create_atom_owned(owner, 9_i64).unwrap();
    assert_eq!(k.set_owned(intruder, id, 10_i64),
        Err(AtomicError::OwnershipViolation { atom: id, owner, attempted: intruder }));
    assert!(matches!(k.begin_transaction(DomainId(9999)), Err(AtomicError::UnknownDomain(_))));
    let mut tx = k.begin_transaction(owner).unwrap();
    tx.set(&k, id, 9_i64).unwrap();
    let report = k.commit(tx).unwrap();
    assert_eq!((report.changed_atoms, report.scheduled_atoms), (0, 0));
    assert_eq!(k.version(id).unwrap(), 0);
    assert_eq!(k.pending_work(), 0);
}
