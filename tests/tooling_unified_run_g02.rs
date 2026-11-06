//! G0.2 R2: opt-in unified `run` must preserve explicit P2.1 console authority.
//! Existing `run` without grant remains the original HALT-only V0.1 path.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const SOURCE: &[u8] = include_bytes!("../examples/p2_1_console/hello.noi");
const EXPECTED: &[u8] = b"Hello from NORDOI P2.1!\n";

fn example_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/p2_1_console/hello.noi")
}

#[test]
fn unified_run_with_explicit_grant_matches_existing_console_handler() {
    let path = example_path();
    let run = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("run")
        .arg(&path)
        .arg("--grant-console")
        .output()
        .expect("launch unified run");
    let legacy = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("console-run")
        .arg(&path)
        .arg("--grant-console")
        .output()
        .expect("launch P2.1 console-run");

    assert!(run.status.success(), "{run:?}");
    assert_eq!(run.stdout, EXPECTED);
    assert_eq!(run.status.code(), legacy.status.code());
    assert_eq!(run.stdout, legacy.stdout);
    assert_eq!(run.stderr, legacy.stderr);
    assert!(String::from_utf8_lossy(&run.stderr).contains("authority=EXPLICIT"));
}

#[test]
fn unified_run_without_grant_retains_original_halt_only_behavior() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["run", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch legacy run");
    child
        .stdin
        .take()
        .expect("stdin available")
        .write_all(b"entry main;\n")
        .expect("write HALT-only source");
    let result = child.wait_with_output().expect("wait for legacy run");
    assert!(result.status.success(), "{result:?}");

    let ungranted = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("run")
        .arg(example_path())
        .output()
        .expect("launch run without grant");
    assert!(!ungranted.status.success());
    assert!(!ungranted.stdout.ends_with(EXPECTED));
}

#[test]
fn unified_run_stdin_with_grant_emits_exact_console_bytes() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["run", "-", "--grant-console"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch run with stdin");
    child
        .stdin
        .take()
        .expect("stdin available")
        .write_all(SOURCE)
        .expect("write exact source");
    let result = child.wait_with_output().expect("wait for run");
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, EXPECTED);
    assert!(String::from_utf8_lossy(&result.stderr).contains("authority=EXPLICIT"));
}

#[test]
fn unified_run_rejects_undeclared_effect_before_emission() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["run", "-", "--grant-console"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch negative run");
    child
        .stdin
        .take()
        .expect("stdin available")
        .write_all(b"module app.main; entry main emits \"UNAUTHORIZED\";")
        .expect("write undeclared-effect source");
    let result = child.wait_with_output().expect("wait for negative run");
    assert_eq!(result.status.code(), Some(4));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("requires explicit"));
}

#[test]
fn unified_run_rejects_unknown_options_and_documents_explicit_grant() {
    let invalid = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("run")
        .arg(example_path())
        .arg("--unknown-option")
        .output()
        .expect("launch invalid invocation");
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());

    let help = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .expect("launch help");
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("nordoi run <path|-> [--grant-console]"));
}
