//! G0.3 R3-R6: first-program failure surfaces are observed, not rewritten.
//! No new .noi grammar, permissions or diagnostic contracts are introduced.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const HELLO: &[u8] = include_bytes!("../examples/p2_1_console/hello.noi");
const EXPECTED: &[u8] = b"Hello from NORDOI P2.1!\n";

fn invoke(args: &[&str], input: Option<&[u8]>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nordoi"));
    command.args(args);
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn NORDOI first-error audit fixture");
    if let Some(bytes) = input {
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(bytes)
            .expect("write test fixture bytes");
    }
    child.wait_with_output().expect("collect CLI outputs")
}

#[test]
fn first_program_success_is_separate_from_receipt() {
    let output = invoke(&["run", "-", "--grant-console"], Some(HELLO));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, EXPECTED);
    assert!(String::from_utf8_lossy(&output.stderr).contains("authority=EXPLICIT"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("ambient-authority=NONE"));
}

#[test]
fn denied_console_is_runtime_failure_without_output() {
    let denied = invoke(&["console-run", "-"], Some(HELLO));
    assert_eq!(denied.status.code(), Some(5));
    assert!(denied.stdout.is_empty());
    let error = String::from_utf8_lossy(&denied.stderr);
    assert!(error.contains("error[console-run]"));
    assert!(error.contains("capability denied"));
    assert!(error.contains("ConsoleWrite"));
}

#[test]
fn granted_but_undeclared_effect_is_frontend_failure() {
    let denied = invoke(
        &["run", "-", "--grant-console"],
        Some(b"module app.main; entry main emits \"NO\";"),
    );
    assert_eq!(denied.status.code(), Some(4));
    assert!(denied.stdout.is_empty());
    let error = String::from_utf8_lossy(&denied.stderr);
    assert!(error.contains("error[console-run]"));
    assert!(error.contains("requires explicit"));
    assert!(error.contains("effect ConsoleWrite;"));
}

#[test]
fn misspelled_emits_is_syntax_failure_not_implicit_permission() {
    let denied = invoke(
        &["run", "-", "--grant-console"],
        Some(b"module app.main; effect ConsoleWrite; entry main emtis \"NO\";"),
    );
    assert_eq!(denied.status.code(), Some(4));
    assert!(denied.stdout.is_empty());
    let error = String::from_utf8_lossy(&denied.stderr);
    assert!(error.contains("error[console-run]"));
    assert!(error.contains("expected contextual 'emits'"));
}

#[test]
fn usage_error_returns_help_on_stderr_and_never_program_stdout() {
    for args in [
        vec!["run", "-", "--unknown-option"],
        vec!["run"],
        vec!["rnu", "-"],
    ] {
        let failed = invoke(&args, None);
        assert_eq!(failed.status.code(), Some(2));
        assert!(failed.stdout.is_empty());
        let error = String::from_utf8_lossy(&failed.stderr);
        assert!(error.contains("error[usage]"));
        assert!(error.contains("Usage:"));
    }
}

#[test]
fn missing_file_and_invalid_utf8_are_different_input_failure_stages() {
    let missing = invoke(
        &["run", "NORDOI_R3_R6_MISSING_DIRECTORY/hello.noi", "--grant-console"],
        None,
    );
    assert_eq!(missing.status.code(), Some(3));
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("error[io]"));

    let invalid = invoke(&["run", "-", "--grant-console"], Some(&[0xff, 0xfe]));
    assert_eq!(invalid.status.code(), Some(3));
    assert!(invalid.stdout.is_empty());
    let error = String::from_utf8_lossy(&invalid.stderr);
    assert!(error.contains("error[input]"));
    assert!(error.contains("not valid UTF-8"));
}

#[test]
fn plain_ungranted_run_remains_a_distinct_legacy_route() {
    let result = invoke(&["run", "-"], Some(HELLO));
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("capability denied"));
}
