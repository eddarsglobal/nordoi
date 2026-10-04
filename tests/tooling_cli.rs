use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_t01_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_all_t01_commands() {
    let output = Command::new(nordoi())
        .arg("--help")
        .output()
        .expect("nordoi --help must run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help must be UTF-8");
    assert!(stdout.contains("nordoi lex <path|->"));
    assert!(stdout.contains("nordoi parse <path|->"));
    assert!(stdout.contains("nordoi module <path|->"));
    assert!(stdout.contains("nordoi semantic <path|->"));
    assert!(stdout.contains("does not lower or execute NAIR"));
}

#[test]
fn version_reports_tool_kernel_and_nair_versions() {
    let output = Command::new(nordoi())
        .arg("--version")
        .output()
        .expect("nordoi --version must run");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("version must be UTF-8"),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}

#[test]
fn lex_command_prints_lossless_token_kinds_and_eof() {
    let path = source_file("lex", "module alpha;\n");
    let output = Command::new(nordoi())
        .args(["lex", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("lex command must run");
    cleanup(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("lex output must be UTF-8");
    assert!(stdout.contains("token IDENTIFIER 0..6"));
    assert!(stdout.contains("\"module\""));
    assert!(stdout.contains("token PUNCTUATION(;)"));
    assert!(stdout.contains("token WHITESPACE"));
    assert!(stdout.contains("token EOF"));
}

#[test]
fn parse_command_prints_bounded_structural_groups() {
    let path = source_file("parse", "alpha({beta})");
    let output = Command::new(nordoi())
        .args(["parse", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("parse command must run");
    cleanup(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("parse output must be UTF-8");
    assert!(stdout.contains("group PARENTHESIS"));
    assert!(stdout.contains("group BRACE"));
    assert!(stdout.contains("token EOF"));
}

#[test]
fn module_command_prints_canonical_qualified_identity() {
    let path = source_file("module", "module alpha /* x */ . beta;\n");
    let output = Command::new(nordoi())
        .args(["module", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("module command must run");
    cleanup(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("module output must be UTF-8");
    assert!(stdout.contains("module \"alpha.beta\""));
    assert!(stdout.contains("segments=2"));
}

#[test]
fn module_command_reports_anonymous_unit_without_inference() {
    let path = source_file("anonymous", "alpha\n");
    let output = Command::new(nordoi())
        .args(["module", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("module command must run");
    cleanup(&path);

    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .expect("module output must be UTF-8")
        .contains("module <anonymous>"));
}

#[test]
fn stdin_is_supported_without_creating_filesystem_semantics() {
    let mut child = Command::new(nordoi())
        .args(["module", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("stdin module command must spawn");
    child
        .stdin
        .take()
        .expect("stdin pipe must exist")
        .write_all(b"module stdin.example;\n")
        .expect("stdin source must be writable");
    let output = child.wait_with_output().expect("stdin command must finish");

    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .expect("module output must be UTF-8")
        .contains("module \"stdin.example\""));
}

#[test]
fn parse_failure_returns_frontend_exit_code_and_positioned_diagnostic() {
    let path = source_file("parse_error", "alpha(]");
    let output = Command::new(nordoi())
        .args(["parse", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("parse command must run");
    cleanup(&path);

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).expect("diagnostic must be UTF-8");
    assert!(stderr.contains("error[parse]:"));
    assert!(stderr.contains(":1:7:"));
    assert!(stderr.contains("mismatched closing delimiter"));
}

#[test]
fn module_failure_returns_frontend_exit_code() {
    let path = source_file("module_error", "module alpha beta;");
    let output = Command::new(nordoi())
        .args(["module", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("module command must run");
    cleanup(&path);

    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .expect("diagnostic must be UTF-8")
        .contains("error[module]:"));
}

#[test]
fn invalid_utf8_is_rejected_before_frontend_publication() {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "nordoi_t01_{}_invalid_utf8.noi",
        std::process::id()
    ));
    fs::write(&path, [0xff, 0xfe]).expect("invalid UTF-8 fixture must be writable");
    let output = Command::new(nordoi())
        .args(["lex", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("lex command must run");
    cleanup(&path);

    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8(output.stderr)
        .expect("diagnostic must be UTF-8")
        .contains("source is not valid UTF-8"));
}

#[test]
fn missing_source_path_is_a_usage_error() {
    let output = Command::new(nordoi())
        .arg("lex")
        .output()
        .expect("usage error command must run");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .expect("usage output must be UTF-8")
        .contains("error[usage]"));
}

#[test]
fn missing_file_is_an_io_error() {
    let output = Command::new(nordoi())
        .args(["lex", "this-file-must-not-exist-t01.noi"])
        .output()
        .expect("missing-file command must run");
    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8(output.stderr)
        .expect("I/O diagnostic must be UTF-8")
        .contains("error[io]"));
}

#[test]
fn tool_output_is_deterministic_for_equal_source() {
    let path = source_file("deterministic", "module stable.tool;\n(alpha)\n");
    let path = path.to_str().expect("temp path must be UTF-8").to_owned();
    let first = Command::new(nordoi())
        .args(["parse", path.as_str()])
        .output()
        .expect("first parse must run");
    let second = Command::new(nordoi())
        .args(["parse", path.as_str()])
        .output()
        .expect("second parse must run");
    let _ = fs::remove_file(&path);

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn semantic_command_prints_validated_nsir_without_claiming_body_semantics() {
    let path = source_file("semantic", "module alpha.beta;\nanything + here\n");
    let output = Command::new(nordoi())
        .args(["semantic", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("semantic command must run");
    cleanup(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("semantic output must be UTF-8");
    assert!(stdout.contains("nsir module=\"alpha.beta\" body=UNLOWERED identity="));
    assert!(stdout.contains("origin file=0.."));
}

#[test]
fn semantic_command_reports_anonymous_identity_explicitly() {
    let path = source_file("semantic_anonymous", "body\n");
    let output = Command::new(nordoi())
        .args(["semantic", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("semantic command must run");
    cleanup(&path);

    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .expect("semantic output must be UTF-8")
        .contains("nsir module=<anonymous> body=UNLOWERED"));
}

#[test]
fn semantic_command_inherits_frontend_failure_code() {
    let path = source_file("semantic_error", "module alpha beta;");
    let output = Command::new(nordoi())
        .args(["semantic", path.to_str().expect("temp path must be UTF-8")])
        .output()
        .expect("semantic command must run");
    cleanup(&path);

    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .expect("semantic diagnostic must be UTF-8")
        .contains("error[semantic]:"));
}
