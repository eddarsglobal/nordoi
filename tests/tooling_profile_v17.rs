use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_reference(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("nordoi-v17-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(root.join("src/app")).unwrap();
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/profile1_reference");
    for relative in ["NORDOI.toml", "src/app/rules.noi", "src/app/main.noi"] {
        fs::write(
            root.join(relative),
            fs::read(example.join(relative)).unwrap(),
        )
        .unwrap();
    }
    root
}

fn build_and_release(root: &Path) {
    let root_text = root.to_str().unwrap();
    let commands: [&[&str]; 4] = [
        &["build", root_text],
        &["build", root_text, "--locked"],
        &["release-check", root_text, "--json"],
        &["release", root_text, "--json"],
    ];
    for args in commands {
        let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "command {:?} failed: stdout={} stderr={}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn help_lists_v17_final_profile_certification_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("profile1-certify <project-root> [--json]"));
    assert!(stdout.contains("V1.7"));
    assert!(stdout.contains("Production Profile 1"));
}

#[test]
fn reference_application_completes_full_cross_platform_release_rehearsal() {
    let root = temp_reference("rehearsal");
    let root_text = root.to_str().unwrap();

    let check = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["check", root_text, "--json"])
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(String::from_utf8_lossy(&check.stdout).contains("\"status\":\"pass\""));

    build_and_release(&root);
    let dist_before = [
        fs::read(root.join("dist/profile1-reference-1.0.0.npkg")).unwrap(),
        fs::read(root.join("dist/profile1-reference-1.0.0.npkg.sha256")).unwrap(),
        fs::read(root.join("dist/profile1-reference-1.0.0.provenance")).unwrap(),
    ];

    let certificate = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["profile1-certify", root_text, "--json"])
        .output()
        .unwrap();
    assert!(
        certificate.status.success(),
        "{}",
        String::from_utf8_lossy(&certificate.stderr)
    );
    let json = String::from_utf8(certificate.stdout).unwrap();
    assert!(json.contains("\"schema\":\"nordoi.production-profile-1.v1\""));
    assert!(json.contains("\"status\":\"certified\""));
    assert!(json.contains("\"project\":\"profile1-reference\""));
    assert!(json.contains("\"nairMinor\":14"));
    assert!(json.contains("\"authority\":\"NONE\""));

    assert_eq!(
        fs::read(root.join("dist/profile1-reference-1.0.0.npkg")).unwrap(),
        dist_before[0]
    );
    assert_eq!(
        fs::read(root.join("dist/profile1-reference-1.0.0.npkg.sha256")).unwrap(),
        dist_before[1]
    );
    assert_eq!(
        fs::read(root.join("dist/profile1-reference-1.0.0.provenance")).unwrap(),
        dist_before[2]
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn substituted_distribution_package_is_rejected_by_profile1_cli() {
    let root = temp_reference("dist-substitution");
    build_and_release(&root);
    let package = root.join("dist/profile1-reference-1.0.0.npkg");
    let mut bytes = fs::read(&package).unwrap();
    bytes[0] ^= 1;
    fs::write(&package, bytes).unwrap();

    let root_text = root.to_str().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["profile1-certify", root_text, "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("\"schema\":\"nordoi.production-profile-1.v1\""));
    assert!(json.contains("\"status\":\"error\""));
    assert!(json.contains("\"stage\":\"dist-package\""));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checksum_provenance_and_source_drift_fail_closed_and_version_stays_frozen() {
    let root = temp_reference("adversarial");
    build_and_release(&root);
    let root_text = root.to_str().unwrap();
    let checksum_path = root.join("dist/profile1-reference-1.0.0.npkg.sha256");
    let provenance_path = root.join("dist/profile1-reference-1.0.0.provenance");
    let checksum = fs::read(&checksum_path).unwrap();
    let provenance = fs::read(&provenance_path).unwrap();

    fs::write(&checksum_path, b"00  profile1-reference-1.0.0.npkg\n").unwrap();
    let checksum_failure = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["profile1-certify", root_text, "--json"])
        .output()
        .unwrap();
    assert_eq!(checksum_failure.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&checksum_failure.stdout).contains("\"stage\":\"checksum\""));
    fs::write(&checksum_path, &checksum).unwrap();

    let mut forged_provenance = provenance.clone();
    forged_provenance.extend_from_slice(b"host = \"forbidden\"\n");
    fs::write(&provenance_path, forged_provenance).unwrap();
    let provenance_failure = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["profile1-certify", root_text, "--json"])
        .output()
        .unwrap();
    assert_eq!(provenance_failure.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&provenance_failure.stdout).contains("\"stage\":\"provenance\"")
    );
    fs::write(&provenance_path, &provenance).unwrap();

    fs::write(
        root.join("src/app/rules.noi"),
        "module app.rules; fn classify(x) returns if x > 40 { x + 101 } else { x + 200 };",
    )
    .unwrap();
    let drift = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["profile1-certify", root_text, "--json"])
        .output()
        .unwrap();
    assert_eq!(drift.status.code(), Some(4));
    let drift_json = String::from_utf8(drift.stdout).unwrap();
    assert!(drift_json.contains("\"schema\":\"nordoi.production-profile-1.v1\""));
    assert!(drift_json.contains("\"stage\":\"lock\""));

    let version = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
    let _ = fs::remove_dir_all(root);
}
