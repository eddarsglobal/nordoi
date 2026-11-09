//! NORDOI G0.3 R4X: real cross-signed owner/candidate enrollment preflight.
//! A pair of valid SSHSIG signatures does NOT establish separate administration.
//! The only final admission decision in this test-only lab is FAIL CLOSED.
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
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(1);
    const OWNER_NS: &str = "nordoi-r4x-owner-enrollment@eddarsglobal";
    const GUARDIAN_NS: &str = "nordoi-r4x-guardian-proof@eddarsglobal";
    const OWNER_ID: &str = "nordoi-r4x-test-owner";
    const GUARDIAN_ID: &str = "nordoi-r4x-test-candidate";
    const MAX_VALIDITY: u64 = 300;
    const FLOOR_EPOCH: u64 = 4;
    const REMOTE: &str = "https://github.com/eddarsglobal/nordoi.git";
    const OBSERVER_REF: &str = "refs/heads/evidence/g03-r4u-signed-observer-ledger-20261109";
    const OBSERVER_HEAD: &str = "15adf075050a233705b3bad6eae62b6f9e6f946d";

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("nordoi-r4x-{}-{id}", std::process::id()));
            fs::create_dir(&dir).expect("create fresh disposable research directory");
            Self(fs::canonicalize(&dir).expect("canonical disposable directory"))
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove only disposable research directory");
        }
    }

    fn invoke(mut c: Command, input: Option<&[u8]>) -> Result<Output, &'static str> {
        if input.is_some() {
            c.stdin(Stdio::piped());
        }
        let mut child = c
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| "NATIVE_TOOL_UNAVAILABLE")?;
        if let Some(payload) = input {
            child
                .stdin
                .take()
                .ok_or("MISSING_STDIN")?
                .write_all(payload)
                .map_err(|_| "NATIVE_STDIN_FAILED")?;
        }
        child
            .wait_with_output()
            .map_err(|_| "NATIVE_PROCESS_FAILED")
    }

    fn new_key(dir: &Path, name: &str) -> PathBuf {
        let key = dir.join(name);
        let mut c = Command::new("/usr/bin/ssh-keygen");
        c.args(["-q", "-t", "ed25519", "-N", "", "-f"]).arg(&key);
        assert!(
            invoke(c, None).unwrap().status.success(),
            "real Ed25519 keygen must succeed"
        );
        key
    }
    fn public(key: &Path) -> String {
        let s = fs::read_to_string(format!("{}.pub", key.display())).unwrap();
        let x: Vec<_> = s.split_whitespace().collect();
        assert!(x.len() >= 2);
        assert_eq!(x[0], "ssh-ed25519");
        format!("{} {}", x[0], x[1])
    }
    fn digest(bytes: &[u8]) -> String {
        let mut c = Command::new("/usr/bin/shasum");
        c.args(["-a", "256"]);
        let result = invoke(c, Some(bytes)).expect("native sha256");
        assert!(result.status.success());
        String::from_utf8(result.stdout)
            .unwrap()
            .split_ascii_whitespace()
            .next()
            .unwrap()
            .to_owned()
    }
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    fn random_challenge() -> String {
        let mut x = [0u8; 32];
        File::open("/dev/urandom")
            .unwrap()
            .read_exact(&mut x)
            .unwrap();
        let mut s = String::with_capacity(64);
        for b in x {
            use std::fmt::Write as _;
            write!(&mut s, "{b:02x}").unwrap();
        }
        s
    }
    fn request(
        owner: &str,
        guardian: &str,
        nonce: &str,
        issue: u64,
        expiry: u64,
        epoch: u64,
    ) -> String {
        format!(
            "NORDOI-R4X-DUAL-ENROLLMENT-V1\n\
             project=eddarsglobal/nordoi\n\
             owner_root_sha256={}\n\
             candidate_guardian_sha256={}\n\
             nonce={nonce}\n\
             issued_unix={issue}\n\
             expires_unix={expiry}\n\
             floor_epoch={epoch}\n",
            digest(owner.as_bytes()),
            digest(guardian.as_bytes())
        )
    }

    fn sign(path: &Path, key: &Path, namespace: &str) -> PathBuf {
        let mut c = Command::new("/usr/bin/ssh-keygen");
        c.args(["-Y", "sign", "-f"])
            .arg(key)
            .args(["-n", namespace])
            .arg(path);
        assert!(invoke(c, None).unwrap().status.success());
        PathBuf::from(format!("{}.sig", path.display()))
    }
    fn sshsig(
        lab: &Scratch,
        payload: &str,
        signature: &Path,
        pubkey: &str,
        principal: &str,
        namespace: &str,
    ) -> bool {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let allowed = lab.0.join(format!("allowed-{id}"));
        if fs::write(&allowed, format!("{principal} {pubkey}\n")).is_err() {
            return false;
        }
        let mut c = Command::new("/usr/bin/ssh-keygen");
        c.args(["-Y", "verify", "-f"])
            .arg(&allowed)
            .args(["-I", principal, "-n", namespace, "-s"])
            .arg(signature);
        invoke(c, Some(payload.as_bytes())).is_ok_and(|o| o.status.success())
    }

    struct Packet {
        bytes: String,
        owner_pub: String,
        guardian_pub: String,
        nonce: String,
        issue: u64,
        expiry: u64,
        epoch: u64,
        owner_sig: PathBuf,
        guardian_sig: PathBuf,
    }
    fn packet(
        lab: &Scratch,
        owner: &Path,
        guardian: &Path,
        nonce: &str,
        issue: u64,
        expiry: u64,
        epoch: u64,
    ) -> Packet {
        let owner_pub = public(owner);
        let guardian_pub = public(guardian);
        let bytes = request(&owner_pub, &guardian_pub, nonce, issue, expiry, epoch);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let a = lab.0.join(format!("owner-{id}.txt"));
        let b = lab.0.join(format!("candidate-{id}.txt"));
        fs::write(&a, &bytes).unwrap();
        fs::write(&b, &bytes).unwrap();
        let owner_sig = sign(&a, owner, OWNER_NS);
        let guardian_sig = sign(&b, guardian, GUARDIAN_NS);
        Packet {
            bytes,
            owner_pub,
            guardian_pub,
            nonce: nonce.to_owned(),
            issue,
            expiry,
            epoch,
            owner_sig,
            guardian_sig,
        }
    }
    fn fixture() -> (Scratch, PathBuf, PathBuf) {
        let lab = Scratch::new();
        let owner = new_key(&lab.0, "owner");
        let guardian = new_key(&lab.0, "guardian");
        (lab, owner, guardian)
    }
    fn check_crypto(
        lab: &Scratch,
        p: &Packet,
        expected_owner_pin: &str,
        expected_guardian_pin: &str,
        expected_nonce: &str,
        clock: u64,
        expected_epoch: u64,
    ) -> Result<(), &'static str> {
        if p.owner_pub == p.guardian_pub {
            return Err("SAME_CRYPTOGRAPHIC_KEY_NOT_SEPARATE_ROLES");
        }
        if digest(p.owner_pub.as_bytes()) != expected_owner_pin {
            return Err("OWNER_ROOT_PIN_MISMATCH");
        }
        if digest(p.guardian_pub.as_bytes()) != expected_guardian_pin {
            return Err("GUARDIAN_ROOT_PIN_MISMATCH");
        }
        if p.nonce != expected_nonce
            || p.nonce.len() != 64
            || !p
                .nonce
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err("FRESH_CHALLENGE_MISMATCH");
        }
        if p.epoch != expected_epoch || p.epoch < FLOOR_EPOCH {
            return Err("EPOCH_DOWNGRADE_OR_WRONG_EPOCH");
        }
        if p.issue > clock || p.expiry < clock {
            return Err("ENROLLMENT_EXPIRED_OR_NOT_YET_VALID");
        }
        if p.expiry <= p.issue || p.expiry - p.issue > MAX_VALIDITY {
            return Err("ENROLLMENT_TTL_OUT_OF_BOUNDS");
        }
        if p.bytes
            != request(
                &p.owner_pub,
                &p.guardian_pub,
                &p.nonce,
                p.issue,
                p.expiry,
                p.epoch,
            )
        {
            return Err("NONCANONICAL_ENROLLMENT_PAYLOAD");
        }
        if !sshsig(
            lab,
            &p.bytes,
            &p.owner_sig,
            &p.owner_pub,
            OWNER_ID,
            OWNER_NS,
        ) {
            return Err("OWNER_SIGNATURE_NOT_VERIFIED");
        }
        if !sshsig(
            lab,
            &p.bytes,
            &p.guardian_sig,
            &p.guardian_pub,
            GUARDIAN_ID,
            GUARDIAN_NS,
        ) {
            return Err("GUARDIAN_SIGNATURE_NOT_VERIFIED");
        }
        Ok(())
    }
    fn blocked_admission(
        lab: &Scratch,
        p: &Packet,
        owner_pin: &str,
        guardian_pin: &str,
        nonce: &str,
        clock: u64,
        epoch: u64,
    ) -> Result<(), &'static str> {
        check_crypto(lab, p, owner_pin, guardian_pin, nonce, clock, epoch)?;
        // The test operator generated BOTH keys and pins. An actual separate
        // authenticated administrator and protected remote log are absent.
        Err("INDEPENDENT_ADMINISTRATIVE_PROVENANCE_NOT_ENROLLED")
    }
    fn good_packet() -> (Scratch, Packet) {
        let (lab, owner, guardian) = fixture();
        let n = random_challenge();
        let t = now();
        let p = packet(&lab, &owner, &guardian, &n, t, t + 120, FLOOR_EPOCH);
        (lab, p)
    }
    fn approved(p: &Packet) -> (String, String) {
        (
            digest(p.owner_pub.as_bytes()),
            digest(p.guardian_pub.as_bytes()),
        )
    }

    #[test]
    fn native_real_cross_signed_owner_and_candidate_packet_still_denied_admin() {
        let (lab, p) = good_packet();
        let (owner, guardian) = approved(&p);
        assert_ne!(p.owner_pub, p.guardian_pub);
        assert_eq!(
            check_crypto(&lab, &p, &owner, &guardian, &p.nonce, now(), FLOOR_EPOCH),
            Ok(())
        );
        assert_eq!(
            blocked_admission(&lab, &p, &owner, &guardian, &p.nonce, now(), FLOOR_EPOCH),
            Err("INDEPENDENT_ADMINISTRATIVE_PROVENANCE_NOT_ENROLLED")
        );
    }
    #[test]
    fn native_fresh_challenge_required_against_valid_old_dual_signatures() {
        let (lab, p) = good_packet();
        let (owner, guardian) = approved(&p);
        let next = random_challenge();
        assert_ne!(next, p.nonce);
        assert_eq!(
            check_crypto(&lab, &p, &owner, &guardian, &next, now(), FLOOR_EPOCH),
            Err("FRESH_CHALLENGE_MISMATCH")
        );
    }
    #[test]
    fn native_substituted_owner_pin_is_denied_before_signature_acceptance() {
        let (lab, p) = good_packet();
        let (_, guardian) = approved(&p);
        assert_eq!(
            check_crypto(
                &lab,
                &p,
                &"0".repeat(64),
                &guardian,
                &p.nonce,
                now(),
                FLOOR_EPOCH
            ),
            Err("OWNER_ROOT_PIN_MISMATCH")
        );
    }
    #[test]
    fn native_substituted_guardian_pin_is_denied_before_signature_acceptance() {
        let (lab, p) = good_packet();
        let (owner, _) = approved(&p);
        assert_eq!(
            check_crypto(
                &lab,
                &p,
                &owner,
                &"0".repeat(64),
                &p.nonce,
                now(),
                FLOOR_EPOCH
            ),
            Err("GUARDIAN_ROOT_PIN_MISMATCH")
        );
    }
    #[test]
    fn native_wrong_owner_signature_namespace_fails_native_ssh_verify() {
        let (lab, p) = good_packet();
        assert!(!sshsig(
            &lab,
            &p.bytes,
            &p.owner_sig,
            &p.owner_pub,
            OWNER_ID,
            GUARDIAN_NS,
        ));
    }
    #[test]
    fn native_wrong_candidate_signature_namespace_fails_native_ssh_verify() {
        let (lab, p) = good_packet();
        assert!(!sshsig(
            &lab,
            &p.bytes,
            &p.guardian_sig,
            &p.guardian_pub,
            GUARDIAN_ID,
            OWNER_NS,
        ));
    }
    #[test]
    fn native_mixed_signature_from_previous_valid_packet_is_not_current() {
        let (lab, owner, guardian) = fixture();
        let t = now();
        let old = packet(
            &lab,
            &owner,
            &guardian,
            &random_challenge(),
            t,
            t + 120,
            FLOOR_EPOCH,
        );
        let mut fresh = packet(
            &lab,
            &owner,
            &guardian,
            &random_challenge(),
            t,
            t + 120,
            FLOOR_EPOCH,
        );
        let (op, gp) = approved(&fresh);
        fresh.guardian_sig = old.guardian_sig;
        assert_eq!(
            check_crypto(&lab, &fresh, &op, &gp, &fresh.nonce, t, FLOOR_EPOCH),
            Err("GUARDIAN_SIGNATURE_NOT_VERIFIED")
        );
    }
    #[test]
    fn native_tampered_signed_epoch_payload_is_not_canonical() {
        let (lab, mut p) = good_packet();
        let (op, gp) = approved(&p);
        p.bytes = p.bytes.replace("floor_epoch=4", "floor_epoch=5");
        assert_eq!(
            check_crypto(&lab, &p, &op, &gp, &p.nonce, now(), FLOOR_EPOCH),
            Err("NONCANONICAL_ENROLLMENT_PAYLOAD")
        );
    }
    #[test]
    fn native_expired_and_future_dated_signatures_fail_closed() {
        let (lab, p) = good_packet();
        let (op, gp) = approved(&p);
        assert_eq!(
            check_crypto(&lab, &p, &op, &gp, &p.nonce, p.expiry + 1, FLOOR_EPOCH),
            Err("ENROLLMENT_EXPIRED_OR_NOT_YET_VALID")
        );
        assert_eq!(
            check_crypto(&lab, &p, &op, &gp, &p.nonce, p.issue - 1, FLOOR_EPOCH),
            Err("ENROLLMENT_EXPIRED_OR_NOT_YET_VALID")
        );
    }
    #[test]
    fn native_oversized_validity_window_is_not_accepted() {
        let (lab, owner, guardian) = fixture();
        let t = now();
        let p = packet(
            &lab,
            &owner,
            &guardian,
            &random_challenge(),
            t,
            t + MAX_VALIDITY + 1,
            FLOOR_EPOCH,
        );
        let (op, gp) = approved(&p);
        assert_eq!(
            check_crypto(&lab, &p, &op, &gp, &p.nonce, t, FLOOR_EPOCH),
            Err("ENROLLMENT_TTL_OUT_OF_BOUNDS")
        );
    }
    #[test]
    fn native_epoch_downgrade_denied_even_with_valid_signatures() {
        let (lab, owner, guardian) = fixture();
        let t = now();
        let p = packet(&lab, &owner, &guardian, &random_challenge(), t, t + 120, 3);
        let (op, gp) = approved(&p);
        assert_eq!(
            check_crypto(&lab, &p, &op, &gp, &p.nonce, t, FLOOR_EPOCH),
            Err("EPOCH_DOWNGRADE_OR_WRONG_EPOCH")
        );
    }
    #[test]
    fn native_identical_owner_and_candidate_key_cannot_claim_two_roles() {
        let lab = Scratch::new();
        let owner = new_key(&lab.0, "owner");
        let t = now();
        let p = packet(
            &lab,
            &owner,
            &owner,
            &random_challenge(),
            t,
            t + 120,
            FLOOR_EPOCH,
        );
        let (op, gp) = approved(&p);
        assert_eq!(
            check_crypto(&lab, &p, &op, &gp, &p.nonce, t, FLOOR_EPOCH),
            Err("SAME_CRYPTOGRAPHIC_KEY_NOT_SEPARATE_ROLES")
        );
    }
    #[test]
    fn native_unsigned_owner_or_candidate_request_is_rejected() {
        let (lab, p) = good_packet();
        let (op, gp) = approved(&p);
        fs::remove_file(&p.owner_sig).unwrap();
        assert_eq!(
            check_crypto(&lab, &p, &op, &gp, &p.nonce, now(), FLOOR_EPOCH),
            Err("OWNER_SIGNATURE_NOT_VERIFIED")
        );
        let (lab2, p2) = good_packet();
        let (op2, gp2) = approved(&p2);
        fs::remove_file(&p2.guardian_sig).unwrap();
        assert_eq!(
            check_crypto(&lab2, &p2, &op2, &gp2, &p2.nonce, now(), FLOOR_EPOCH),
            Err("GUARDIAN_SIGNATURE_NOT_VERIFIED")
        );
    }
    #[test]
    fn native_live_same_owner_github_observer_is_not_independent_operator() {
        let mut c = Command::new("/usr/bin/git");
        c.args(["ls-remote", REMOTE, OBSERVER_REF])
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null");
        let output = invoke(c, None).unwrap();
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            format!("{OBSERVER_HEAD}\t{OBSERVER_REF}\n").as_bytes()
        );
        let (lab, p) = good_packet();
        let (op, gp) = approved(&p);
        assert_eq!(
            blocked_admission(&lab, &p, &op, &gp, &p.nonce, now(), FLOOR_EPOCH),
            Err("INDEPENDENT_ADMINISTRATIVE_PROVENANCE_NOT_ENROLLED")
        );
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn portable_targets_have_no_admitted_independent_external_guardian() {
    assert_ne!(std::env::consts::OS, "macos");
}
#[test]
fn reference_kernel_still_denies_external_host_io() {
    let (mut model, permit) = Model::bootstrap(
        0x58,
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
