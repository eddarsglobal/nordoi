//! G0.3 R3-R17 — exact symbol witness for denial/grant codegen inspection.
//! Rust API contract probe; DOES NOT benchmark speed or authorize production.
use std::hint::black_box;
use nordoi_kernel::{AtomicKernel, Capability};

#[no_mangle]
#[inline(never)]
pub fn nordoi_r17_require_witness(
    kernel: &AtomicKernel,
    capability: &Capability,
) -> u8 {
    if black_box(kernel).require(black_box(capability)).is_err() {
        1
    } else {
        0
    }
}

fn main() {
    let capability = Capability::ConsoleWrite;
    let mut baseline = AtomicKernel::new();

    for _ in 0..32 {
        assert_eq!(
            nordoi_r17_require_witness(black_box(&baseline), black_box(&capability)),
            1,
            "denial must be fail closed"
        );
    }
    assert_eq!(baseline.pending_work(), 0, "require must not schedule work");

    baseline.allow(capability.clone());
    assert_eq!(
        nordoi_r17_require_witness(black_box(&baseline), black_box(&capability)),
        0,
        "explicit grant must be accepted"
    );
    assert_eq!(
        nordoi_r17_require_witness(&AtomicKernel::new(), &capability),
        1,
        "authorization cannot leak between kernels"
    );
    println!("G03_R3_R17_REQUIRE_WITNESS=PASS");
    println!("PRODUCTION=NOT_AUTHORIZED");
}
