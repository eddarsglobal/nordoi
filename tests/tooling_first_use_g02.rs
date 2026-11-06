//! G0.2: first-use regression for the existing, capability-gated console path.
//! No new .noi syntax, runtime permission, opcode, or execution dependency.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const EXPECTED: &[u8] = b"Hello from NORDOI P2.1!\n";
const SOURCE: &[u8] = include_bytes!("../examples/p2_1_console/hello.noi");

fn example_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/p2_1_console/hello.noi")
}

#[test]
fn g02_first_use_checked_in_example_runs_with_explicit_grant() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("console-run")
        .arg(example_path())
        .arg("--grant-console")
        .output()
        .expect("launch NORDOI CLI");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, EXPECTED);
    let receipt = String::from_utf8(output.stderr).expect("UTF-8 receipt");
    assert!(receipt.contains("authority=EXPLICIT"));
    assert!(receipt.contains("ambient-authority=NONE"));
}

#[test]
fn g02_first_use_checked_in_example_without_grant_fails_closed() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("console-run")
        .arg(example_path())
        .output()
        .expect("launch NORDOI CLI");

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty(), "ungranted output must be empty");
    assert!(String::from_utf8_lossy(&output.stderr).contains("capability denied"));
}

#[test]
fn g02_first_use_stdin_matches_checked_in_example() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["console-run", "-", "--grant-console"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch NORDOI CLI with stdin");

    child
        .stdin
        .take()
        .expect("stdin present")
        .write_all(SOURCE)
        .expect("write source bytes");
    let output = child.wait_with_output().expect("wait for NORDOI CLI");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, EXPECTED);
    assert!(String::from_utf8_lossy(&output.stderr).contains("authority=EXPLICIT"));
}
