//! G0.3 R3-R1: documentation-only build-once onboarding hypothesis.
//! This checks text contract fidelity, not novice learning or performance.

const TUTORIAL: &str =
    include_str!("../docs/NORDOI_G03_R3_R1_BUILD_ONCE_TUTORIAL_CANDIDATE.md");
const OFFICIAL: &str = include_str!("../docs/NORDOI_G02_FIRST_USE_QUICKSTART.md");
const HELLO: &str = include_str!("../examples/p2_1_console/hello.noi");

#[test]
fn build_once_tutorial_uses_only_existing_commands() {
    assert!(TUTORIAL.contains("cargo build --release --bin nordoi"));
    assert!(TUTORIAL
        .contains("./target/release/nordoi run examples/p2_1_console/hello.noi --grant-console"));
    assert!(TUTORIAL.contains(
        r".\target\release\nordoi.exe run examples/p2_1_console/hello.noi --grant-console"
    ));
    assert!(TUTORIAL.contains("CARGO_TARGET_DIR"));
    assert!(TUTORIAL.contains("cargo run --bin nordoi -- run"));
}

#[test]
fn tutorial_preserves_authority_and_research_boundaries() {
    assert!(HELLO.contains("effect ConsoleWrite;"));
    assert!(TUTORIAL.contains("does not grant permission"));
    assert!(TUTORIAL.contains("explicit and separate"));
    assert!(TUTORIAL.contains("exit code 5, no program stdout"));
    assert!(TUTORIAL.contains("V0.1 HALT-only"));
    assert!(TUTORIAL.contains("P2.2–P2.7"));
    assert!(TUTORIAL.contains("original R2 study cards A/B remain frozen"));
    assert!(TUTORIAL.contains("NON-NORMATIVE DOCUMENTATION EXPERIMENT"));
    assert!(OFFICIAL.contains(
        "cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console"
    ));
}
