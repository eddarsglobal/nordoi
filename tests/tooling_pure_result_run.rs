use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn nordoi() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nordoi"))
}

fn source_file(label: &str, text: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "nordoi-v02-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v02_result_run_without_rewriting_v01_run() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi result-run <path|->"));
    assert!(stdout.contains("V0.2 result-run executes C0.6"));
    assert!(stdout.contains("V0.1 run is the explicit source-to-closed-runtime execution boundary"));
}

#[test]
fn result_run_executes_and_reports_integer_result() {
    let path = source_file(
        "int",
        "module demo; type User; effect Network; entry main returns 42;",
    );
    let output = Command::new(nordoi())
        .args(["result-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("form=ENTRY entry=\"main\" result=INT(42) result-register=r0"));
    assert!(stdout.contains("work=0 effects=0 authority=NONE nair-instructions=2"));
    assert!(stdout.contains("executed=2 input=0 registers=1"));
    assert!(stdout.contains("domains=0 atoms=0 transactions=0 frames=0 bridges=0 scheduled=0"));
    assert!(stdout.contains("quiescent=true result=INT(42)"));
    assert!(stdout.contains("c06="));
    assert!(stdout.contains("receipt="));
}

#[test]
fn result_run_preserves_zero_as_a_real_result() {
    let path = source_file("zero", "entry main returns 0;");
    let output = Command::new(nordoi())
        .args(["result-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(0) result-register=r0"));
    assert!(stdout.contains("registers=1"));
    assert!(stdout.contains("result=INT(0)"));
}

#[test]
fn result_run_reports_no_result_without_register() {
    let path = source_file("none", "entry main;");
    let output = Command::new(nordoi())
        .args(["result-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=NONE result-register=NONE"));
    assert!(stdout.contains("nair-instructions=1"));
    assert!(stdout.contains("executed=1 input=0 registers=0"));
    assert!(stdout.contains("quiescent=true result=NONE"));
}

#[test]
fn noncanonical_result_fails_with_frontend_exit_code() {
    let path = source_file("bad", "entry main returns 01;");
    let output = Command::new(nordoi())
        .args(["result-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[result-run]:"));
}

#[test]
fn repeated_result_run_output_is_deterministic() {
    let path = source_file("deterministic", "module demo; entry main returns 42;");
    let first = Command::new(nordoi())
        .args(["result-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    let second = Command::new(nordoi())
        .args(["result-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn stdin_result_run_is_supported() {
    let mut child = Command::new(nordoi())
        .args(["result-run", "-"])
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
        .contains("quiescent=true result=INT(42)"));
}

#[test]
fn certified_v01_run_still_rejects_result_source() {
    let path = source_file("v01-frozen", "entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
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
