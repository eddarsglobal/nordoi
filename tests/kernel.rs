use nordoi_kernel::{AtomicError, AtomicKernel, Capability};

#[test]
fn identical_write_creates_zero_work() {
    let mut kernel = AtomicKernel::new();
    let counter = kernel.create_atom(10_i64);

    assert!(!kernel.set(counter, 10_i64).unwrap());
    assert_eq!(kernel.pending_work(), 0);
    assert_eq!(kernel.version(counter).unwrap(), 0);
}

#[test]
fn only_affected_atoms_are_scheduled() {
    let mut kernel = AtomicKernel::new();

    let count = kernel.create_atom(0_i64);
    let label = kernel.create_atom("0");
    let unrelated = kernel.create_atom("untouched");

    kernel.connect(count, label).unwrap();

    assert!(kernel.set(count, 1_i64).unwrap());

    let work = kernel.flush();

    assert!(work.contains(&count));
    assert!(work.contains(&label));
    assert!(!work.contains(&unrelated));
}

#[test]
fn capabilities_are_denied_by_default() {
    let mut kernel = AtomicKernel::new();
    let network = Capability::Network("api.example.com".into());

    assert_eq!(
        kernel.require(&network),
        Err(AtomicError::CapabilityDenied(network.clone()))
    );

    kernel.allow(network.clone());
    assert!(kernel.require(&network).is_ok());
}

#[test]
fn duplicate_invalidations_collapse() {
    let mut kernel = AtomicKernel::new();

    let source = kernel.create_atom(0_i64);
    let dependent = kernel.create_atom(0_i64);

    kernel.connect(source, dependent).unwrap();

    kernel.set(source, 1_i64).unwrap();
    kernel.set(source, 2_i64).unwrap();

    // source + dependent, not four separate jobs.
    assert_eq!(kernel.pending_work(), 2);
}
