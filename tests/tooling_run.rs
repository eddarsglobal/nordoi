use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_v01_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v01_run_command_and_preserves_c04_boundary_text() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi run <path|->"));
    assert!(
        stdout.contains("Execute the certified C0.4 HALT-only NAIR through the closed runtime.")
    );
    assert!(stdout.contains("C0.4 lowering does not execute the runtime."));
    assert!(
        stdout.contains("V0.1 run is the explicit source-to-closed-runtime execution boundary.")
    );
}

#[test]
fn run_command_executes_named_entry_and_reports_quiescence() {
    let path = source_file("entry", "module demo; type A; effect Net; entry main;");
    let output = Command::new(nordoi())
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "run module=\"demo\" form=ENTRY entry=\"main\" work=0 effects=0 authority=NONE nair-instructions=1"
    ));
    assert!(stdout.contains("c04="));
    assert!(stdout.contains("receipt="));
    assert!(stdout.contains("executed=1 input=0 domains=0 atoms=0 transactions=0 frames=0 bridges=0 scheduled=0 quiescent=true result=HALTED"));
}

#[test]
fn run_command_executes_empty_body_as_halt() {
    let path = source_file("empty", "module demo; // tail\n");
    let output = Command::new(nordoi())
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("form=EMPTY work=0 effects=0 authority=NONE nair-instructions=1"));
}

#[test]
fn run_command_rejects_unknown_body_with_frontend_exit_code() {
    let path = source_file("bad", "future_body");
    let output = Command::new(nordoi())
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[run]:"));
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

#[test]
fn repeated_run_output_is_deterministic_for_equal_source() {
    let path = source_file("deterministic", "module demo; entry main;");
    let first = Command::new(nordoi())
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    let second = Command::new(nordoi())
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
}
