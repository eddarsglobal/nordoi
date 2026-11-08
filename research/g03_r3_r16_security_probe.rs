//! NORDOI G0.3 R3-R16 — security correctness must outrank microbenchmark speed.
//! External read-only probe for baseline and candidate, without modifying either crate.
use nordoi_kernel::{AtomId, AtomicError, AtomicKernel, Capability, Value};

fn main() {
    let mut kernel = AtomicKernel::new();
    let console = Capability::ConsoleWrite;
    assert_eq!(
        kernel.require(&console),
        Err(AtomicError::CapabilityDenied(console.clone())),
        "console must be denied by default"
    );

    let atom = kernel.create_atom(5_i64);
    assert!(!kernel.set(atom, 5_i64).expect("same value"));
    assert_eq!(kernel.version(atom).expect("version"), 0);
    assert_eq!(kernel.pending_work(), 0);

    assert_eq!(
        kernel.set(AtomId(u64::MAX), 1_i64),
        Err(AtomicError::UnknownAtom(AtomId(u64::MAX))),
        "unknown atom rejected on write"
    );
    let owner = kernel.create_domain("owner").expect("owner");
    let intruder = kernel.create_domain("intruder").expect("intruder");
    let protected = kernel.create_atom_owned(owner, 10_i64).expect("protected");

    assert!(matches!(
        kernel.set_owned(intruder, protected, 11_i64),
        Err(AtomicError::OwnershipViolation { .. })
    ), "intruder write rejected");

    let mut tx = kernel.begin_transaction(owner).expect("begin tx");
    tx.set(&kernel, protected, 20_i64).expect("stage");
    kernel.set_owned(owner, protected, 15_i64).expect("change version");
    kernel.flush();
    assert!(matches!(
        kernel.commit(tx), Err(AtomicError::TransactionConflict { .. })
    ), "stale transaction must not commit");
    assert_eq!(kernel.get(protected).unwrap(), &Value::Int(15));
    assert_eq!(kernel.get(atom).unwrap(), &Value::Int(5));

    assert_eq!(
        kernel.require(&console),
        Err(AtomicError::CapabilityDenied(console.clone())),
        "unrelated writes cannot introduce authority"
    );
    kernel.allow(console.clone());
    assert!(kernel.require(&console).is_ok(), "explicit grant must work");
    assert_eq!(
        AtomicKernel::new().require(&console),
        Err(AtomicError::CapabilityDenied(console)),
        "grant never leaks into another kernel"
    );

    println!("G03_R3_R16_SECURITY_PROBE=PASS");
    println!("PRODUCTION=NOT_AUTHORIZED");
}
