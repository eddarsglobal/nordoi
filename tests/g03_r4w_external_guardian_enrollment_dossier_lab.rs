//! NORDOI G0.3 R4W — external custodian enrollment dossier, real SSHSIG
//! key-possession checks, and fail-closed administrative provenance gate.
//! Research only: signing a request does not authenticate a separate operator.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;
use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod native {
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(1);
    const SIGN_NS: &str = "nordoi-r4w-custodian-request@eddarsglobal";
    const SIGN_ID: &str = "nordoi-r4w-enrollment-candidate";
    const OTHER_NS: &str = "nordoi-r4w-observer-highwater@eddarsglobal";
    const REMOTE_URL: &str = "https://github.com/eddarsglobal/nordoi.git";
    const REMOTE_REF: &str = "refs/heads/evidence/g03-r4u-signed-observer-ledger-20261109";
    const REMOTE_COMMIT: &str = "15adf075050a233705b3bad6eae62b6f9e6f946d";

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let n = SEQ.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "nordoi-r4w-enrollment-{}-{n}", std::process::id()
            ));
            fs::create_dir(&dir).expect("new test-only directory");
            Self(fs::canonicalize(dir).expect("canonical test-only directory"))
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("cleanup test-only directory");
        }
    }
    fn command(mut c: Command, input: Option<&[u8]>) -> Result<Output, &'static str> {
        if input.is_some() {
            c.stdin(Stdio::piped());
        }
        let mut child = c.stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|_| "NATIVE_COMMAND_NOT_AVAILABLE")?;
        if let Some(input) = input {
            child.stdin.take().ok_or("NO_NATIVE_STDIN")?
                .write_all(input).map_err(|_| "NATIVE_INPUT_FAILED")?;
        }
        child.wait_with_output().map_err(|_| "NATIVE_COMMAND_FAILED_TO_EXIT")
    }
    fn ssh_key(directory: &Path, name: &str) -> PathBuf {
        let private = directory.join(name);
        let mut c = Command::new("/usr/bin/ssh-keygen");
        c.args(["-q", "-t", "ed25519", "-N", "", "-f"])
            .arg(&private);
        let out = command(c, None).expect("create real disposable Ed25519 key");
        assert!(out.status.success(), "real SSH keygen failed");
        private
    }
    fn public_key(private: &Path) -> String {
        let content = fs::read_to_string(format!("{}.pub", private.display()))
            .expect("disposable key public half");
        let p: Vec<_> = content.split_whitespace().collect();
        assert!(p.len() >= 2 && p[0] == "ssh-ed25519");
        format!("{} {}", p[0], p[1])
    }
    fn fresh_nonce() -> String {
        let mut bytes = [0_u8; 32];
        File::open("/dev/urandom").expect("random source")
            .read_exact(&mut bytes).expect("256-bit challenge");
        let mut output = String::with_capacity(64);
        for b in bytes {
            use std::fmt::Write as _;
            write!(&mut output, "{b:02x}").unwrap();
        }
        output
    }
    fn record(guardian: &str, nonce: &str, epoch: u64) -> String {
        format!(
            "NORDOI-R4W-CUSTODIAN-ENROLLMENT-V1\nproject=eddarsglobal/nordoi\n\
             guardian_key={guardian}\nnonce={nonce}\nrequested_epoch={epoch}\n"
        )
    }
    fn canonical_record(actual: &str, guardian: &str, nonce: &str, epoch: u64) -> bool {
        nonce.len() == 64
            && nonce.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            && guardian.starts_with("ssh-ed25519 ")
            && epoch > 0
            && actual == record(guardian, nonce, epoch)
    }
    fn sign(message: &Path, key: &Path, namespace: &str) -> PathBuf {
        let mut c = Command::new("/usr/bin/ssh-keygen");
        c.args(["-Y", "sign", "-f"]).arg(key).args(["-n", namespace]).arg(message);
        let out = command(c, None).expect("actual OpenSSH SSHSIG");
        assert!(out.status.success(), "SSH signature generation failed");
        PathBuf::from(format!("{}.sig", message.display()))
    }
    fn verify(msg: &str, sig: &Path, pubkey: &str, namespace: &str, directory: &Path) -> bool {
        let allow = directory.join(format!(
            "allowed-{}.txt", SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        if fs::write(&allow, format!("{SIGN_ID} {pubkey}\n")).is_err() {
            return false;
        }
        let mut c = Command::new("/usr/bin/ssh-keygen");
        c.args(["-Y", "verify", "-f"]).arg(allow)
            .args(["-I", SIGN_ID, "-n", namespace, "-s"]).arg(sig);
        command(c, Some(msg.as_bytes())).is_ok_and(|out| out.status.success())
    }
    struct Proof {
        message: String,
        signature: PathBuf,
        candidate: String,
        nonce: String,
    }
    fn generate_proof(temp: &Temp, key: &Path, nonce: &str, epoch: u64) -> Proof {
        let candidate = public_key(key);
        let message = record(&candidate, nonce, epoch);
        let path = temp.0.join(format!(
            "candidate-{}.txt", SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, &message).unwrap();
        let signature = sign(&path, key, SIGN_NS);
        Proof { message, signature, candidate, nonce: nonce.to_owned() }
    }
    #[derive(Clone)]
    struct IndependentCustody {
        publisher_controller: &'static str,
        candidate_controller: &'static str,
        verified_independent_operator: bool,
        owner_approved_pin_oob: bool,
        protected_remote_highwater: bool,
        deployment_pin_protected: bool,
        recovery_policy_authorized: bool,
        external_signed_provenance: bool,
    }
    impl IndependentCustody {
        fn actual_r4w() -> Self {
            Self {
                publisher_controller: "eddarsglobal",
                candidate_controller: "eddarsglobal",
                verified_independent_operator: false,
                owner_approved_pin_oob: false,
                protected_remote_highwater: false,
                deployment_pin_protected: false,
                recovery_policy_authorized: false,
                external_signed_provenance: false,
            }
        }
    }
    fn audit(claims: &IndependentCustody) -> Result<(), &'static str> {
        if claims.publisher_controller == claims.candidate_controller {
            return Err("SAME_ADMINISTRATOR");
        }
        if !claims.verified_independent_operator {
            return Err("EXTERNAL_OPERATOR_IDENTITY_NOT_VERIFIED");
        }
        if !claims.owner_approved_pin_oob {
            return Err("OWNER_APPROVED_ROOT_NOT_OBTAINED_OUT_OF_BAND");
        }
        if !claims.protected_remote_highwater {
            return Err("IMMUTABLE_EXTERNAL_HIGHWATER_NOT_ATTESTED");
        }
        if !claims.deployment_pin_protected {
            return Err("VERIFIER_PIN_REPLACEABLE_BY_PRIMARY_ADMIN");
        }
        if !claims.recovery_policy_authorized {
            return Err("RECOVERY_NOT_AUTHORIZED");
        }
        if !claims.external_signed_provenance {
            return Err("EXTERNAL_PROVENANCE_NOT_PRESENT");
        }
        // Claims are untrusted strings/flags. No separate trust oracle exists.
        Err("RESEARCH_CANNOT_CERTIFY_REAL_OPERATOR_CUSTODY")
    }
    fn admit(proof: Option<&Proof>, expected_key: &str, expected_nonce: &str,
             epoch: u64, temp: &Temp, claims: &IndependentCustody)
        -> Result<(), &'static str> {
        let p = proof.ok_or("SIGNED_ENROLLMENT_REQUIRED")?;
        if !canonical_record(&p.message, expected_key, expected_nonce, epoch)
            || p.candidate != expected_key || p.nonce != expected_nonce {
            return Err("ENROLLMENT_NONCANONICAL_OR_WRONG_CHALLENGE");
        }
        if !verify(&p.message, &p.signature, expected_key, SIGN_NS, &temp.0) {
            return Err("ENROLLMENT_SIGNATURE_INVALID");
        }
        audit(claims)
    }

    #[test]
    fn native_real_ssh_ed25519_key_possession_proves_only_possession() {
        let temp = Temp::new();
        let key = ssh_key(&temp.0, "candidate-a");
        let nonce = fresh_nonce();
        let proof = generate_proof(&temp, &key, &nonce, 4);
        assert!(verify(&proof.message, &proof.signature,
                       &proof.candidate, SIGN_NS, &temp.0));
        assert_eq!(admit(Some(&proof), &proof.candidate, &nonce, 4, &temp,
                         &IndependentCustody::actual_r4w()), Err("SAME_ADMINISTRATOR"));
    }
    #[test]
    fn native_new_256_bit_challenge_rejects_prior_authentic_signature() {
        let temp = Temp::new();
        let key = ssh_key(&temp.0, "candidate-a");
        let first = fresh_nonce();
        let second = fresh_nonce();
        assert_ne!(first, second);
        let proof = generate_proof(&temp, &key, &first, 4);
        assert_eq!(admit(Some(&proof), &proof.candidate, &second, 4, &temp,
                         &IndependentCustody::actual_r4w()),
                   Err("ENROLLMENT_NONCANONICAL_OR_WRONG_CHALLENGE"));
    }
    #[test]
    fn native_wrong_public_key_cannot_replace_owner_enrollment() {
        let temp = Temp::new();
        let key = ssh_key(&temp.0, "candidate-a");
        let other = ssh_key(&temp.0, "candidate-b");
        let nonce = fresh_nonce();
        let proof = generate_proof(&temp, &key, &nonce, 4);
        assert_eq!(admit(Some(&proof), &public_key(&other), &nonce, 4, &temp,
                         &IndependentCustody::actual_r4w()),
                   Err("ENROLLMENT_NONCANONICAL_OR_WRONG_CHALLENGE"));
        assert!(!verify(&proof.message, &proof.signature,
                        &public_key(&other), SIGN_NS, &temp.0));
    }
    #[test]
    fn native_wrong_sshsig_namespace_denied_even_for_same_key() {
        let temp = Temp::new();
        let key = ssh_key(&temp.0, "candidate-a");
        let nonce = fresh_nonce();
        let proof = generate_proof(&temp, &key, &nonce, 4);
        assert!(!verify(&proof.message, &proof.signature,
                        &proof.candidate, OTHER_NS, &temp.0));
    }
    #[test]
    fn native_modified_enrollment_bytes_denied() {
        let temp = Temp::new();
        let key = ssh_key(&temp.0, "candidate-a");
        let nonce = fresh_nonce();
        let proof = generate_proof(&temp, &key, &nonce, 4);
        let tampered = proof.message.replace("requested_epoch=4", "requested_epoch=5");
        assert!(!verify(&tampered, &proof.signature, &proof.candidate,
                        SIGN_NS, &temp.0));
        assert_eq!(admit(Some(&proof), &proof.candidate, &nonce, 5, &temp,
                         &IndependentCustody::actual_r4w()),
                   Err("ENROLLMENT_NONCANONICAL_OR_WRONG_CHALLENGE"));
    }
    #[test]
    fn native_missing_external_enrollment_does_not_fall_back_to_two_keys() {
        let temp = Temp::new();
        let key = ssh_key(&temp.0, "candidate-a");
        let nonce = fresh_nonce();
        assert_eq!(admit(None, &public_key(&key), &nonce, 4, &temp,
                         &IndependentCustody::actual_r4w()),
                   Err("SIGNED_ENROLLMENT_REQUIRED"));
    }
    #[test]
    fn native_signed_request_with_same_repo_owner_is_not_an_external_guardian() {
        let temp = Temp::new();
        let key = ssh_key(&temp.0, "candidate-a");
        let nonce = fresh_nonce();
        let proof = generate_proof(&temp, &key, &nonce, 4);
        assert_eq!(admit(Some(&proof), &proof.candidate, &nonce, 4, &temp,
                         &IndependentCustody::actual_r4w()), Err("SAME_ADMINISTRATOR"));
    }
    #[test]
    fn native_distinct_names_do_not_replace_external_operator_attestation() {
        let mut c = IndependentCustody::actual_r4w();
        c.candidate_controller = "unverified-other-operator";
        assert_eq!(audit(&c), Err("EXTERNAL_OPERATOR_IDENTITY_NOT_VERIFIED"));
    }
    #[test]
    fn native_self_asserted_operator_without_owner_out_of_band_pin_is_denied() {
        let mut c = IndependentCustody::actual_r4w();
        c.candidate_controller = "unverified-other-operator";
        c.verified_independent_operator = true;
        assert_eq!(audit(&c), Err("OWNER_APPROVED_ROOT_NOT_OBTAINED_OUT_OF_BAND"));
    }
    #[test]
    fn native_observer_without_protected_monotonic_head_is_denied() {
        let mut c = IndependentCustody::actual_r4w();
        c.candidate_controller = "unverified-other-operator";
        c.verified_independent_operator = true;
        c.owner_approved_pin_oob = true;
        assert_eq!(audit(&c), Err("IMMUTABLE_EXTERNAL_HIGHWATER_NOT_ATTESTED"));
    }
    #[test]
    fn native_replaceable_verifier_and_recovery_ambiguity_are_denied() {
        let mut c = IndependentCustody::actual_r4w();
        c.candidate_controller = "unverified-other-operator";
        c.verified_independent_operator = true;
        c.owner_approved_pin_oob = true;
        c.protected_remote_highwater = true;
        assert_eq!(audit(&c), Err("VERIFIER_PIN_REPLACEABLE_BY_PRIMARY_ADMIN"));
        c.deployment_pin_protected = true;
        assert_eq!(audit(&c), Err("RECOVERY_NOT_AUTHORIZED"));
    }
    #[test]
    fn native_all_green_self_declaration_still_cannot_authorize_production() {
        let mut c = IndependentCustody::actual_r4w();
        c.candidate_controller = "unverified-other-operator";
        c.verified_independent_operator = true;
        c.owner_approved_pin_oob = true;
        c.protected_remote_highwater = true;
        c.deployment_pin_protected = true;
        c.recovery_policy_authorized = true;
        c.external_signed_provenance = true;
        assert_eq!(audit(&c), Err("RESEARCH_CANNOT_CERTIFY_REAL_OPERATOR_CUSTODY"));
    }
    #[test]
    fn native_actual_github_observer_commit_is_not_independently_administered() {
        let mut c = Command::new("/usr/bin/git");
        c.args(["ls-remote", REMOTE_URL, REMOTE_REF])
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ASKPASS", "/usr/bin/false");
        let out = command(c, None).expect("actual network observation");
        assert!(out.status.success(), "GitHub evidence ref must be accessible");
        assert_eq!(out.stdout, format!("{REMOTE_COMMIT}\t{REMOTE_REF}\n").as_bytes());
        assert_eq!(audit(&IndependentCustody::actual_r4w()), Err("SAME_ADMINISTRATOR"));
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn portable_targets_cannot_claim_native_independent_guardian_custody() {
    assert_ne!(std::env::consts::OS, "macos");
}
#[test]
fn reference_model_still_denies_external_io_without_production_authorization() {
    let (mut model, permit) = Model::bootstrap(
        0x56, ScopeBudget { tasks: 0, resources: 0, children: 0 }, 16
    );
    let root = model.root();
    assert_eq!(
        model.host_grant(&permit, root, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(model.event_count(), 0);
}
