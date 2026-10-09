use std::process::Command;

#[test]
fn help_lists_g01_conformance_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("conformance [--json]"));
    assert!(stdout.contains("G0.1"));
    assert!(stdout.contains("Future-Native Gate"));
}

#[test]
fn conformance_text_separates_evidence_from_delivery() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("conformance")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("principles=322"));
    assert!(stdout.contains("certified=310"));
    assert!(stdout.contains("partial=12"));
    assert!(stdout.contains("evidence-coverage=96.27%"));
    assert!(stdout.contains("full-v1-delivery-estimate=58.35%"));
    assert!(stdout.contains("PLANNING_NOT_CERTIFICATION"));
}

#[test]
fn conformance_json_is_schema_versioned_and_deterministic() {
    let first = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["conformance", "--json"])
        .output()
        .unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["conformance", "--json"])
        .output()
        .unwrap();
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let stdout = String::from_utf8(first.stdout).unwrap();
    assert!(stdout.contains("\"schema\":\"nordoi.constitutional-conformance.g0.1\""));
    assert!(stdout.contains("\"fullV1DeliveryEstimateBasisPoints\":5835"));
    assert!(stdout.contains("\"futureNativeGate\":\"mandatory_governance\""));
}

#[test]
fn conformance_does_not_change_public_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}
