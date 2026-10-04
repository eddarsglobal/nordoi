use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_l04_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn semantic_command_reports_type_effect_counts_and_l04_witness() {
    let path = source_file(
        "semantic",
        "module demo;\ntype UserId;\neffect Network;\nbody\n",
    );
    let output = Command::new(nordoi())
        .args(["semantic", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("semantic command must run");
    cleanup(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("semantic output must be UTF-8");
    assert!(stdout.contains("nsir module=\"demo\" body=UNLOWERED identity="));
    assert!(stdout.contains("types=1 effects=1 semantic="));
}

#[test]
fn malformed_leading_type_declaration_fails_closed_in_semantic_command() {
    let path = source_file("bad_type", "type");
    let output = Command::new(nordoi())
        .args(["semantic", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("semantic command must run");
    cleanup(&path);

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).expect("diagnostic must be UTF-8");
    assert!(stderr.contains("error[semantic]:"));
    assert!(stderr.contains("expected a type name"));
}

#[test]
fn effect_declaration_does_not_claim_execution_or_authority() {
    let path = source_file("effect", "effect Network;\n");
    let output = Command::new(nordoi())
        .args(["semantic", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("semantic command must run");
    cleanup(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("semantic output must be UTF-8");
    assert!(stdout.contains("types=0 effects=1"));
    assert!(stdout.contains("body=UNLOWERED"));
}
