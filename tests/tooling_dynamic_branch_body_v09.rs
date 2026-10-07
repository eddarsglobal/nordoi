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
        "nordoi-v09-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v09_branch_body_run() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi branch-body-run <path|-> <key-code>"));
    assert!(stdout.contains("NAIR 0.11"));
}

#[test]
fn branch_body_run_executes_true_path_selectively() {
    let path = source_file(
        "true",
        "module demo.selective; input key_code; entry main returns if key_code > 40 { key_code + 100 } else { key_code + 200 };",
    );
    let output = Command::new(nordoi())
        .args(["branch-body-run", path.to_str().unwrap(), "41"])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(141)"));
    assert!(stdout.contains("runtime-calls=0 runtime-branches=1"));
    assert!(stdout.contains("selected-branch-instructions=3 discarded-branch-instructions=0"));
    assert!(stdout.contains("nair-minor=0.11"));
    assert!(stdout.contains("BRANCH_EVAL"));
    assert!(stdout.contains("authority=NONE input-boundary=CANONICAL"));
}

#[test]
fn branch_body_run_executes_false_path_selectively() {
    let path = source_file(
        "false",
        "input key_code; entry main returns if key_code > 40 { key_code + 100 } else { key_code + 200 };",
    );
    let output = Command::new(nordoi())
        .args(["branch-body-run", path.to_str().unwrap(), "39"])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(239)"));
    assert!(stdout.contains("selected-branch-instructions=3 discarded-branch-instructions=0"));
}

#[test]
fn branch_body_run_rejects_non_numeric_key_code_as_usage_error() {
    let path = source_file(
        "bad-key",
        "input key_code; entry main returns if key_code > 40 { key_code + 100 } else { key_code + 200 };",
    );
    let output = Command::new(nordoi())
        .args(["branch-body-run", path.to_str().unwrap(), "abc"])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(2));
}
