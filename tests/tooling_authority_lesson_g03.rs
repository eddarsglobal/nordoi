//! G0.3 R2 FIX1: candidate educational text only; not one of the frozen A/B arms.
//! These static checks verify fidelity, not novice comprehension or runtime speed.

const LESSON: &str = include_str!("../docs/NORDOI_G03_R2_AUTHORITY_LESSON_CANDIDATE.md");
const OFFICIAL_GUIDE: &str = include_str!("../docs/NORDOI_G02_FIRST_USE_QUICKSTART.md");

#[test]
fn prototype_keeps_explicit_source_effect_and_cli_grant_separate() {
    assert!(LESSON.contains("effect ConsoleWrite;"));
    assert!(LESSON.contains("Declaration is not permission."));
    assert!(LESSON.contains("A permission is not a declaration."));
    assert!(LESSON.contains(
        "cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console"
    ));
    assert!(LESSON.contains("V0.1 HALT-only"));
    assert!(LESSON.contains("P2.2–P2.7"));
    assert!(LESSON.contains("NOT_AUTHORIZED"));
}

#[test]
fn prototype_cannot_be_mistaken_for_study_evidence_or_new_syntax() {
    assert!(LESSON.contains("not part of the frozen A/B research materials"));
    assert!(LESSON.contains("not measured novice usability"));
    assert!(LESSON.contains("No new `.noi` grammar"));
    assert!(OFFICIAL_GUIDE.contains("declaration alone is not authorization"));
    assert!(OFFICIAL_GUIDE.contains("V0.1 HALT-only"));
}
