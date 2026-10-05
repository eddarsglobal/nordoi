use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_c05_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_c05_result_plan_without_claiming_lowering_or_execution() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi result-plan <path|->"));
    assert!(stdout.contains("Print the C0.5 pure-result execution plan."));
    assert!(stdout.contains(
        "C0.5 result-plan is additive: it plans L0.6 values but does not lower NAIR or execute runtime work."
    ));
    assert!(stdout.contains("C0.3 plan does not lower or execute NAIR."));
}

#[test]
fn result_plan_reports_integer_zero_work_and_witnesses() {
    let path = source_file(
        "int",
        "module demo; type A; effect Net; entry main returns 42;",
    );
    let output = Command::new(nordoi())
        .args(["result-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "result-plan module=\"demo\" form=ENTRY entry=\"main\" result=INT(42) work=0 effects=0 authority=NONE"
    ));
    assert!(stdout.contains("l06="));
    assert!(stdout.contains("c05="));
    assert!(stdout.contains("lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"));
}

#[test]
fn result_plan_reports_entry_without_result_as_none() {
    let path = source_file("none", "entry main;");
    let output = Command::new(nordoi())
        .args(["result-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("form=ENTRY entry=\"main\" result=NONE work=0 effects=0 authority=NONE"));
}

#[test]
fn result_plan_reports_empty_body() {
    let path = source_file("empty", "module demo; // tail\n");
    let output = Command::new(nordoi())
        .args(["result-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("form=EMPTY result=NONE work=0 effects=0 authority=NONE"));
}

#[test]
fn noncanonical_literal_fails_closed_at_result_plan_boundary() {
    let path = source_file("bad", "entry main returns 01;");
    let output = Command::new(nordoi())
        .args(["result-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[result-plan]:"));
    assert!(stderr.contains("canonical decimal"));
}

#[test]
fn c03_plan_still_rejects_l06_result_source() {
    let path = source_file("c03_frozen", "entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[plan]:"));
}

#[test]
fn lower_and_run_remain_frozen_for_result_source() {
    let path = source_file("future", "entry main returns 42;");
    let lower = Command::new(nordoi())
        .args(["lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    let run = Command::new(nordoi())
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(lower.status.code(), Some(4));
    assert_eq!(run.status.code(), Some(4));
}

#[test]
fn repeated_result_plan_output_is_deterministic() {
    let path = source_file("deterministic", "module demo; entry main returns 42;");
    let first = Command::new(nordoi())
        .args(["result-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    let second = Command::new(nordoi())
        .args(["result-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn certified_c02_version_output_remains_unchanged() {
    let output = Command::new(nordoi()).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}
