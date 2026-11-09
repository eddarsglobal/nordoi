//! NORDOI G0.3 R4V — real macOS signature verification plus a fail-closed
//! independent-administrative-custody qualification gate. Research-only.
//! Different public keys and a GitHub ref do NOT prove different administrators.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;
use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod native {
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(1);
    const URL: &str = "https://github.com/eddarsglobal/nordoi.git";
    const OBSERVER_REF: &str = "refs/heads/evidence/g03-r4u-signed-observer-ledger-20261109";
    const OBSERVER_COMMIT: &str = "15adf075050a233705b3bad6eae62b6f9e6f946d";
    const PACK_PATH: &str = "research/r4u_signed_observer_ledger/evidence_pack.txt";
    const PACK_SHA: &str = "692809e55aed8ad188208e04653d3bf4fdb9945aa7fa661db6ee22fe8100400d";
    const GUARDIAN: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHfClkGUTZWf18i0YPkUCj5R+jSwE2eF3sGASALCLvL6";
    const OBSERVER: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGRMxyrMD2G9xpR3YPmCgtp9cinas6Rxz7oiuYMoS91f";
    const GUARDIAN_ID: &str = "nordoi-r4u-guardian";
    const OBSERVER_ID: &str = "nordoi-r4u-observer";
    const GUARDIAN_NS: &str = "nordoi-r4u-guardian-log@eddarsglobal";
    const OBSERVER_NS: &str = "nordoi-r4u-observer-highwater@eddarsglobal";
    const EPOCH4_SHA: &str = "65b71123804659cea8b334c783f51d7e63ad3c8283057c22e432a8a54f6c9d1c";

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let folder =
                std::env::temp_dir().join(format!("nordoi-r4v-{}-{n}", std::process::id()));
            fs::create_dir(&folder).expect("new temporary R4V fixture only");
            Self(fs::canonicalize(folder).expect("R4V fixture canonical path"))
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("delete only temporary research fixture");
        }
    }

    fn run(mut command: Command, input: Option<&[u8]>) -> Result<Output, &'static str> {
        if input.is_some() {
            command.stdin(Stdio::piped());
        }
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| "NATIVE_TOOL_UNAVAILABLE")?;
        if let Some(data) = input {
            child
                .stdin
                .take()
                .ok_or("NATIVE_STDIN_UNAVAILABLE")?
                .write_all(data)
                .map_err(|_| "NATIVE_INPUT_FAILED")?;
        }
        child
            .wait_with_output()
            .map_err(|_| "NATIVE_TOOL_DID_NOT_EXIT")
    }

    fn git(command: &mut Command) -> Result<Output, &'static str> {
        command
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ASKPASS", "/usr/bin/false");
        let result = command.output().map_err(|_| "GIT_HOST_UNAVAILABLE")?;
        if !result.status.success() {
            return Err("GIT_REMOTE_NOT_AVAILABLE");
        }
        Ok(result)
    }

    fn hash(bytes: &[u8]) -> String {
        let mut command = Command::new("/usr/bin/shasum");
        command.args(["-a", "256"]);
        let output = run(command, Some(bytes)).expect("native SHA256");
        assert!(output.status.success());
        let str_out = String::from_utf8(output.stdout).expect("SHA ASCII");
        str_out
            .split_ascii_whitespace()
            .next()
            .expect("SHA digest")
            .to_owned()
    }

    fn part<'a>(pack: &'a str, name: &str) -> Result<&'a str, &'static str> {
        let start = format!("[{name}]\n");
        let end = format!("[/{name}]\n");
        if pack.matches(start.as_str()).count() != 1 || pack.matches(end.as_str()).count() != 1 {
            return Err("EVIDENCE_SECTION_AMBIGUOUS");
        }
        let (_, tail) = pack
            .split_once(start.as_str())
            .ok_or("EVIDENCE_SECTION_MISSING")?;
        let (contents, _) = tail
            .split_once(end.as_str())
            .ok_or("EVIDENCE_SECTION_UNTERMINATED")?;
        if contents.is_empty() || !contents.ends_with('\n') {
            return Err("EVIDENCE_SECTION_NONCANONICAL");
        }
        Ok(contents)
    }

    fn ssh_verify(
        data: &str,
        signature: &str,
        key: &str,
        principal: &str,
        namespace: &str,
    ) -> bool {
        let scratch = Scratch::new();
        let signature_path = scratch.0.join("receipt.sig");
        let signers_path = scratch.0.join("allowed_signers");
        if fs::write(&signature_path, signature).is_err()
            || fs::write(&signers_path, format!("{principal} {key}\n")).is_err()
        {
            return false;
        }
        let mut verify = Command::new("/usr/bin/ssh-keygen");
        verify
            .args(["-Y", "verify", "-f"])
            .arg(signers_path)
            .args(["-I", principal, "-n", namespace, "-s"])
            .arg(signature_path);
        run(verify, Some(data.as_bytes())).is_ok_and(|result| result.status.success())
    }

    fn live_public_pack() -> Result<String, &'static str> {
        let mut ls = Command::new("/usr/bin/git");
        ls.args(["ls-remote", URL, OBSERVER_REF]);
        let result = git(&mut ls)?;
        if result.stdout != format!("{OBSERVER_COMMIT}\t{OBSERVER_REF}\n").as_bytes() {
            return Err("REMOTE_OBSERVER_REF_UNTRUSTED");
        }
        let scratch = Scratch::new();
        let repo = scratch.0.join("git");
        fs::create_dir(&repo).map_err(|_| "GIT_TEMP_FAILURE")?;
        let mut init = Command::new("/usr/bin/git");
        init.args(["init", "-q"]).arg(&repo);
        git(&mut init)?;
        let mut fetch = Command::new("/usr/bin/git");
        fetch
            .arg("-C")
            .arg(&repo)
            .args(["fetch", "--depth=1", "--no-tags", URL, OBSERVER_REF]);
        git(&mut fetch)?;
        let mut rev = Command::new("/usr/bin/git");
        rev.arg("-C").arg(&repo).args(["rev-parse", "FETCH_HEAD"]);
        if git(&mut rev)?.stdout != format!("{OBSERVER_COMMIT}\n").as_bytes() {
            return Err("FETCHED_OBSERVER_COMMIT_CHANGED");
        }
        let mut read = Command::new("/usr/bin/git");
        read.arg("-C")
            .arg(&repo)
            .arg("show")
            .arg(format!("FETCH_HEAD:{PACK_PATH}"));
        let bytes = git(&mut read)?.stdout;
        if bytes.is_empty() || bytes.len() > 8192 || hash(&bytes) != PACK_SHA {
            return Err("OBSERVER_EVIDENCE_NOT_PINNED");
        }
        String::from_utf8(bytes).map_err(|_| "OBSERVER_EVIDENCE_NOT_UTF8")
    }

    // These claims arrive from the same owner-controlled research environment.
    // They are NOT authenticated evidence of administrative independence.
    #[derive(Clone)]
    struct OperatorClaims<'a> {
        publisher_admin: &'a str,
        guardian_admin: &'a str,
        observer_admin: &'a str,
        external_operator_attested: bool,
        owner_pin_out_of_band_verified: bool,
        append_only_remote_head_attested: bool,
        verifier_install_protected: bool,
        recovery_reviewed: bool,
    }

    impl<'a> OperatorClaims<'a> {
        fn same_repo() -> Self {
            Self {
                publisher_admin: "eddarsglobal",
                guardian_admin: "eddarsglobal",
                observer_admin: "eddarsglobal",
                external_operator_attested: false,
                owner_pin_out_of_band_verified: false,
                append_only_remote_head_attested: false,
                verifier_install_protected: false,
                recovery_reviewed: false,
            }
        }
    }

    fn lab_custody_gate(c: &OperatorClaims<'_>) -> Result<(), &'static str> {
        if c.publisher_admin == c.guardian_admin
            || c.publisher_admin == c.observer_admin
            || c.guardian_admin == c.observer_admin
        {
            return Err("ADMINISTRATIVE_CONTROLLER_NOT_DISTINCT");
        }
        if !c.external_operator_attested {
            return Err("INDEPENDENT_OPERATOR_NOT_ATTESTED");
        }
        if !c.owner_pin_out_of_band_verified {
            return Err("OWNER_TRUST_ROOT_NOT_OUT_OF_BAND");
        }
        if !c.append_only_remote_head_attested {
            return Err("REMOTE_MONOTONIC_HEAD_NOT_ATTESTED");
        }
        if !c.verifier_install_protected {
            return Err("VERIFIER_INSTALL_NOT_PROTECTED");
        }
        if !c.recovery_reviewed {
            return Err("RECOVERY_GOVERNANCE_UNVERIFIED");
        }
        // This research process cannot convert asserted booleans/labels
        // into a real external operator's verified provenance. No allow path.
        Err("EXTERNAL_CUSTODY_PROVENANCE_UNAVAILABLE_IN_RESEARCH")
    }

    #[test]
    fn native_actual_remote_ed25519_guardian_observer_both_verify_but_admin_gate_denies() {
        let pack = live_public_pack().expect("real pinned same-admin GitHub evidence");
        assert_eq!(part(&pack, "guardian_public").unwrap().trim_end(), GUARDIAN);
        assert_eq!(part(&pack, "observer_public").unwrap().trim_end(), OBSERVER);
        let e4 = part(&pack, "epoch4").unwrap();
        assert_eq!(hash(e4.as_bytes()), EPOCH4_SHA);
        assert!(ssh_verify(
            e4,
            part(&pack, "epoch4_guardian_sig").unwrap(),
            GUARDIAN,
            GUARDIAN_ID,
            GUARDIAN_NS
        ));
        assert!(ssh_verify(
            e4,
            part(&pack, "epoch4_observer_sig").unwrap(),
            OBSERVER,
            OBSERVER_ID,
            OBSERVER_NS
        ));
        assert_ne!(GUARDIAN, OBSERVER);
        assert_eq!(
            lab_custody_gate(&OperatorClaims::same_repo()),
            Err("ADMINISTRATIVE_CONTROLLER_NOT_DISTINCT")
        );
    }

    #[test]
    fn native_second_public_key_under_same_account_is_not_separate_authority() {
        let c = OperatorClaims::same_repo();
        assert_ne!(GUARDIAN, OBSERVER);
        assert_eq!(
            lab_custody_gate(&c),
            Err("ADMINISTRATIVE_CONTROLLER_NOT_DISTINCT")
        );
    }

    #[test]
    fn native_different_names_without_external_operator_attestation_denied() {
        let mut c = OperatorClaims::same_repo();
        c.guardian_admin = "hypothetical-guardian";
        c.observer_admin = "hypothetical-observer";
        assert_eq!(
            lab_custody_gate(&c),
            Err("INDEPENDENT_OPERATOR_NOT_ATTESTED")
        );
    }

    #[test]
    fn native_self_approved_owner_root_claim_is_not_enrollment_evidence() {
        let mut c = OperatorClaims::same_repo();
        c.guardian_admin = "hypothetical-guardian";
        c.observer_admin = "hypothetical-observer";
        c.external_operator_attested = true;
        assert_eq!(
            lab_custody_gate(&c),
            Err("OWNER_TRUST_ROOT_NOT_OUT_OF_BAND")
        );
    }

    #[test]
    fn native_signed_current_head_without_append_only_remote_commit_denied() {
        let mut c = OperatorClaims::same_repo();
        c.guardian_admin = "hypothetical-guardian";
        c.observer_admin = "hypothetical-observer";
        c.external_operator_attested = true;
        c.owner_pin_out_of_band_verified = true;
        assert_eq!(
            lab_custody_gate(&c),
            Err("REMOTE_MONOTONIC_HEAD_NOT_ATTESTED")
        );
    }

    #[test]
    fn native_repo_admin_that_can_replace_verifier_still_blocks_admission() {
        let mut c = OperatorClaims::same_repo();
        c.guardian_admin = "hypothetical-guardian";
        c.observer_admin = "hypothetical-observer";
        c.external_operator_attested = true;
        c.owner_pin_out_of_band_verified = true;
        c.append_only_remote_head_attested = true;
        assert_eq!(lab_custody_gate(&c), Err("VERIFIER_INSTALL_NOT_PROTECTED"));
    }

    #[test]
    fn native_unreviewed_recovery_is_not_safe_monotonic_restore() {
        let mut c = OperatorClaims::same_repo();
        c.guardian_admin = "hypothetical-guardian";
        c.observer_admin = "hypothetical-observer";
        c.external_operator_attested = true;
        c.owner_pin_out_of_band_verified = true;
        c.append_only_remote_head_attested = true;
        c.verifier_install_protected = true;
        assert_eq!(lab_custody_gate(&c), Err("RECOVERY_GOVERNANCE_UNVERIFIED"));
    }

    #[test]
    fn native_even_self_declared_all_green_cannot_certify_real_external_custody() {
        let mut c = OperatorClaims::same_repo();
        c.guardian_admin = "hypothetical-guardian";
        c.observer_admin = "hypothetical-observer";
        c.external_operator_attested = true;
        c.owner_pin_out_of_band_verified = true;
        c.append_only_remote_head_attested = true;
        c.verifier_install_protected = true;
        c.recovery_reviewed = true;
        assert_eq!(
            lab_custody_gate(&c),
            Err("EXTERNAL_CUSTODY_PROVENANCE_UNAVAILABLE_IN_RESEARCH")
        );
    }

    #[test]
    fn native_github_branch_off_device_does_not_create_other_administrator() {
        let c = OperatorClaims::same_repo();
        assert!(URL.contains("eddarsglobal/nordoi.git"));
        assert!(OBSERVER_REF.contains("evidence/g03-r4u"));
        assert_eq!(
            lab_custody_gate(&c),
            Err("ADMINISTRATIVE_CONTROLLER_NOT_DISTINCT")
        );
    }

    #[test]
    fn native_authentic_old_observer_receipt_remains_signed_but_not_current() {
        let pack = live_public_pack().expect("fresh GitHub ref before custody review");
        let e3 = part(&pack, "epoch3").unwrap();
        assert!(ssh_verify(
            e3,
            part(&pack, "epoch3_observer_sig").unwrap(),
            OBSERVER,
            OBSERVER_ID,
            OBSERVER_NS
        ));
        assert_ne!(hash(e3.as_bytes()), EPOCH4_SHA);
        assert_eq!(
            lab_custody_gate(&OperatorClaims::same_repo()),
            Err("ADMINISTRATIVE_CONTROLLER_NOT_DISTINCT")
        );
    }

    #[test]
    fn native_guardian_signed_fork_lacks_observer_signature_and_external_operator_proof() {
        let pack = live_public_pack().expect("real GitHub fork evidence");
        let fork = part(&pack, "epoch4_fork").unwrap();
        assert!(ssh_verify(
            fork,
            part(&pack, "epoch4_fork_guardian_sig").unwrap(),
            GUARDIAN,
            GUARDIAN_ID,
            GUARDIAN_NS
        ));
        assert!(!ssh_verify(
            fork,
            part(&pack, "epoch4_observer_sig").unwrap(),
            OBSERVER,
            OBSERVER_ID,
            OBSERVER_NS
        ));
        assert_eq!(
            lab_custody_gate(&OperatorClaims::same_repo()),
            Err("ADMINISTRATIVE_CONTROLLER_NOT_DISTINCT")
        );
    }

    #[test]
    fn native_absent_remote_or_cross_domain_signature_cannot_replace_admin_proof() {
        let pack = live_public_pack().expect("remote reachable on actual Mac test");
        let e4 = part(&pack, "epoch4").unwrap();
        let observer_sig = part(&pack, "epoch4_observer_sig").unwrap();
        assert!(!ssh_verify(
            e4,
            observer_sig,
            OBSERVER,
            OBSERVER_ID,
            GUARDIAN_NS
        ));
        assert_eq!(
            lab_custody_gate(&OperatorClaims::same_repo()),
            Err("ADMINISTRATIVE_CONTROLLER_NOT_DISTINCT")
        );
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn portable_cannot_assert_real_independent_external_operators() {
    assert_ne!(std::env::consts::OS, "macos");
}

#[test]
fn reference_model_external_io_and_automatic_custody_are_not_authorized() {
    let (mut model, permit) = Model::bootstrap(
        0x55,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        16,
    );
    let root = model.root();
    assert_eq!(
        model.host_grant(&permit, root, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(model.event_count(), 0);
}
