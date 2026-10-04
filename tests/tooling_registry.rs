use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_c02_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn version_reports_c02_compiler_boundary() {
    let output = Command::new(nordoi()).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}

#[test]
fn semantic_command_reports_registry_witness_and_typed_ids() {
    let path = source_file(
        "registry",
        "module demo; type Z; type A; effect Net; effect Clock;",
    );
    let output = Command::new(nordoi())
        .args(["semantic", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("registry="));
    assert!(stdout.contains("symbols types=[1:A,2:Z] effects=[1:Clock,2:Net]"));
    assert!(stdout.contains("body=UNLOWERED"));
}

#[test]
fn semantic_symbol_output_is_independent_of_declaration_order() {
    let first = source_file("first", "type Z; effect Net; type A; effect Clock;");
    let second = source_file("second", "effect Clock; type A; effect Net; type Z;");
    let first_output = Command::new(nordoi())
        .args(["semantic", first.to_str().unwrap()])
        .output()
        .unwrap();
    let second_output = Command::new(nordoi())
        .args(["semantic", second.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&first);
    cleanup(&second);
    assert!(first_output.status.success());
    assert!(second_output.status.success());
    let first_stdout = String::from_utf8(first_output.stdout).unwrap();
    let second_stdout = String::from_utf8(second_output.stdout).unwrap();
    let first_symbols = first_stdout
        .lines()
        .find(|line| line.starts_with("symbols "))
        .unwrap();
    let second_symbols = second_stdout
        .lines()
        .find(|line| line.starts_with("symbols "))
        .unwrap();
    assert_eq!(first_symbols, second_symbols);
}
