use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn nordoi() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nordoi"))
}

fn source_file(label: &str, text: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "nordoi-c06-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_c06_result_lower_without_rewriting_old_commands() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi result-lower <path|->"));
    assert!(stdout.contains("C0.6 result-lower is additive"));
    assert!(stdout.contains("C0.4 lower performs the explicit NAIR 0.6 lowering"));
    assert!(stdout.contains("V0.1 run is the explicit source-to-closed-runtime execution boundary"));
}

#[test]
fn result_lower_reports_const_r0_and_halt_for_integer_result() {
    let path = source_file(
        "int",
        "module demo; type User; effect Network; entry main returns 42;",
    );
    let output = Command::new(nordoi())
        .args(["result-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("form=ENTRY entry=\"main\" result=INT(42)"));
    assert!(stdout.contains("work=0 effects=0 authority=NONE"));
    assert!(stdout.contains("nair-instructions=2 result-register=r0"));
    assert!(stdout.contains("instructions=[CONST r0 INT(42),HALT]"));
    assert!(stdout.contains("runtime=NOT_INVOKED"));
    assert!(stdout.contains("c05="));
    assert!(stdout.contains("c06="));
}

#[test]
fn result_lower_reports_halt_only_for_entry_without_result() {
    let path = source_file("none", "entry main;");
    let output = Command::new(nordoi())
        .args(["result-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=NONE"));
    assert!(stdout.contains("nair-instructions=1 result-register=NONE"));
    assert!(stdout.contains("instructions=[HALT]"));
}

#[test]
fn result_lower_reports_halt_only_for_empty_body() {
    let path = source_file("empty", "module demo; // tail\n");
    let output = Command::new(nordoi())
        .args(["result-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("form=EMPTY result=NONE"));
    assert!(stdout.contains("instructions=[HALT]"));
}

#[test]
fn noncanonical_literal_fails_closed_at_result_lower_boundary() {
    let path = source_file("bad", "entry main returns 01;");
    let output = Command::new(nordoi())
        .args(["result-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[result-lower]:"));
    assert!(stderr.contains("canonical decimal"));
}

#[test]
fn certified_c04_lower_and_v01_run_remain_frozen_for_result_source() {
    let path = source_file("frozen", "entry main returns 42;");
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
fn repeated_result_lower_output_is_deterministic() {
    let path = source_file("deterministic", "module demo; entry main returns 42;");
    let first = Command::new(nordoi())
        .args(["result-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    let second = Command::new(nordoi())
        .args(["result-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn int_max_is_reported_exactly() {
    let path = source_file("max", "entry main returns 9223372036854775807;");
    let output = Command::new(nordoi())
        .args(["result-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(9223372036854775807)"));
    assert!(stdout.contains("CONST r0 INT(9223372036854775807)"));
}

#[test]
fn stdin_result_lower_is_supported() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(nordoi())
        .args(["result-lower", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"entry main returns 42;")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("instructions=[CONST r0 INT(42),HALT]"));
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
