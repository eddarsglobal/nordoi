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
        "nordoi-v08-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v08_branch_run() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi branch-run <path|-> <key-code>"));
    assert!(stdout.contains("NAIR 0.10"));
}

#[test]
fn branch_run_executes_true_path() {
    let path = source_file(
        "true",
        "module demo.branch; input key_code; entry main returns if key_code > 40 { 100 } else { 200 };",
    );
    let output = Command::new(nordoi())
        .args(["branch-run", path.to_str().unwrap(), "41"])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(100)"));
    assert!(stdout.contains("runtime-calls=0 runtime-branches=1"));
    assert!(stdout.contains("nair-minor=0.10"));
    assert!(stdout.contains("BRANCH_VALUE"));
    assert!(stdout.contains("authority=NONE input-boundary=CANONICAL"));
}

#[test]
fn branch_run_executes_false_path() {
    let path = source_file(
        "false",
        "input key_code; entry main returns if key_code > 40 { 100 } else { 200 };",
    );
    let output = Command::new(nordoi())
        .args(["branch-run", path.to_str().unwrap(), "39"])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(200)"));
    assert!(stdout.contains("runtime-branches=1"));
}

#[test]
fn branch_run_rejects_non_numeric_key_code_as_usage_error() {
    let path = source_file(
        "bad-key",
        "input key_code; entry main returns if key_code > 40 { 100 } else { 200 };",
    );
    let output = Command::new(nordoi())
        .args(["branch-run", path.to_str().unwrap(), "abc"])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(2));
}
