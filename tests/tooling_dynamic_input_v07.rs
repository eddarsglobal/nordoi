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
        "nordoi-v07-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v07_dynamic_run() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi dynamic-run <path|-> <key-code>"));
    assert!(stdout.contains("NAIR 0.9"));
}

#[test]
fn dynamic_run_executes_runtime_input_to_42() {
    let path = source_file(
        "add",
        "module demo.dynamic; input key_code; const bias = 2; entry main returns key_code + bias;",
    );
    let output = Command::new(nordoi())
        .args(["dynamic-run", path.to_str().unwrap(), "40"])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(42)"));
    assert!(stdout.contains("dynamic=true"));
    assert!(stdout.contains("runtime-computed=true"));
    assert!(stdout.contains("runtime-calls=0 runtime-branches=0"));
    assert!(stdout.contains("nair-instructions=4 nair-minor=0.9"));
    assert!(stdout.contains("READ_INPUT_KEY_CODE"));
}

#[test]
fn dynamic_run_rejects_non_numeric_key_code_as_usage_error() {
    let path = source_file(
        "bad-key",
        "input key_code; entry main returns key_code + 2;",
    );
    let output = Command::new(nordoi())
        .args(["dynamic-run", path.to_str().unwrap(), "abc"])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn certified_version_output_remains_unchanged() {
    let output = Command::new(nordoi()).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}
