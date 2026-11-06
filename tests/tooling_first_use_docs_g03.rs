//! G0.3 R1: first-use documentation must reflect the already certified R2 CLI.
//! This file does not change or extend the public .noi language.

const GUIDE: &str = include_str!("../docs/NORDOI_G02_FIRST_USE_QUICKSTART.md");

#[test]
fn guide_leads_with_existing_unified_run_and_keeps_legacy_denial_example() {
    const PRIMARY: &str =
        "cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console";
    const LEGACY_DENIAL: &str =
        "cargo run --bin nordoi -- console-run examples/p2_1_console/hello.noi";
    let primary = GUIDE.find(PRIMARY).expect("unified R2 first-use command");
    let legacy_denial = GUIDE
        .find(LEGACY_DENIAL)
        .expect("legacy P2.1 denial example");
    assert!(primary < legacy_denial, "primary run should appear first");
    assert!(GUIDE.contains("exit code `5`"));
    assert!(GUIDE.contains("no program stdout"));
}

#[test]
fn guide_preserves_explicit_effect_authority_and_scope() {
    assert!(GUIDE.contains("effect ConsoleWrite;"));
    assert!(GUIDE.contains("declaration alone is not authorization"));
    assert!(GUIDE.contains("V0.1 HALT-only"));
    assert!(GUIDE.contains("P2.2–P2.7"));
    assert!(GUIDE.contains("NOT AUTHORIZED"));
    assert!(!GUIDE.contains("First-use quickstart (candidate)"));
}
