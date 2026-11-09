//! G0.3 R4T: R4S guardian chain + owner-pinned enrollment and highwater receipt lab.
//! tamper/replay tests, and whole-history rollback counterexample.
//! NO independent administrative custodian, no production authorization.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;
use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod native {
    use std::fs::{self};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};

    const CHECKPOINT_NS: &str = "nordoi-r4s-checkpoint@eddarsglobal";
    const ROTATION_NS: &str = "nordoi-r4s-guardian-rotation@eddarsglobal";
    const PRINCIPAL: &str = "nordoi-r4s-guardian";
    const URL: &str = "https://github.com/eddarsglobal/nordoi.git";
    const REMOTE_REF: &str = "refs/heads/evidence/g03-r4p-external-signed-checkpoint-20261109";
    const REMOTE_HEAD: &str = "ff8abe01cc1a527dd84b463dec1736d20a5391a4";
    const REMOTE_ROOT: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIE1i2sBt6ulBFM32PSg6dUeMGG1EBAj3URQ0XUaskrAx";
    const REMOTE_ID: &str = "nordoi-r4p-remote";
    const REMOTE_NS: &str = "nordoi-r4p-external-witness@eddarsglobal";
    const REMOTE_DIR: &str = "research/r4p_external_witness/";
    const FLOOR: &str =
        "NORDOI-R4P-EXTERNAL-WITNESS-V1\nproject=eddarsglobal/nordoi\nepoch=2\nsequence=2\n";
    static NEXT: AtomicU64 = AtomicU64::new(1);

    fn invoke(mut command: Command, input: Option<&[u8]>) -> Result<Output, &'static str> {
        if input.is_some() {
            command.stdin(Stdio::piped());
        }
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| "COMMAND_NOT_AVAILABLE")?;
        if let Some(bytes) = input {
            child
                .stdin
                .take()
                .ok_or("NO_STDIN")?
                .write_all(bytes)
                .map_err(|_| "COMMAND_STDIN_ERROR")?;
        }
        child.wait_with_output().map_err(|_| "COMMAND_NO_RESULT")
    }
    fn digest(bytes: &[u8]) -> String {
        let mut command = Command::new("/usr/bin/shasum");
        command.args(["-a", "256"]);
        let result = invoke(command, Some(bytes)).expect("native SHA256 command");
        assert!(result.status.success(), "native SHA256 is required");
        let out = String::from_utf8(result.stdout).unwrap();
        let hash = out.split_ascii_whitespace().next().unwrap().to_owned();
        assert_eq!(hash.len(), 64);
        hash
    }
    fn valid_key(s: &str) -> bool {
        let parts: Vec<_> = s.split(' ').collect();
        parts.len() == 2
            && parts[0] == "ssh-ed25519"
            && parts[1].len() >= 40
            && parts[1]
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=')
    }
    fn key(private: &Path) -> String {
        let name = format!("{}.pub", private.display());
        let text = fs::read_to_string(name).expect("read ephemeral public Ed25519 key");
        let parts: Vec<_> = text.split_whitespace().collect();
        let result = format!("{} {}", parts[0], parts[1]);
        assert!(valid_key(&result));
        result
    }
    fn hex64(s: &str) -> bool {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Record {
        epoch: u64,
        seq: u64,
        predecessor: String,
        publisher: String,
        successor: String,
    }
    impl Record {
        fn encode(&self) -> String {
            format!(
                "NORDOI-R4S-RECORD-V1\nepoch={}\nsequence={}\nprevious={}\npublisher={}\nnext_guardian={}\n",
                self.epoch, self.seq, self.predecessor, self.publisher, self.successor
            )
        }
        fn parse(input: &str) -> Option<Self> {
            let lines: Vec<_> = input.lines().collect();
            if lines.len() != 6 || lines[0] != "NORDOI-R4S-RECORD-V1" {
                return None;
            }
            let r = Self {
                epoch: lines[1].strip_prefix("epoch=")?.parse().ok()?,
                seq: lines[2].strip_prefix("sequence=")?.parse().ok()?,
                predecessor: lines[3].strip_prefix("previous=")?.to_owned(),
                publisher: lines[4].strip_prefix("publisher=")?.to_owned(),
                successor: lines[5].strip_prefix("next_guardian=")?.to_owned(),
            };
            (r.epoch > 0
                && r.seq > 0
                && hex64(&r.predecessor)
                && valid_key(&r.publisher)
                && (r.successor == "none" || valid_key(&r.successor))
                && r.encode() == input)
                .then_some(r)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Custody {
        epoch: u64,
        seq: u64,
        digest: String,
        guardian: String,
        publisher: String,
    }
    struct Signed {
        message: PathBuf,
        signature: PathBuf,
        successor_signature: Option<PathBuf>,
    }
    fn sign(path: &Path, private: &Path, namespace: &str) -> PathBuf {
        let mut cmd = Command::new("/usr/bin/ssh-keygen");
        cmd.args(["-Y", "sign", "-f"])
            .arg(private)
            .args(["-n", namespace])
            .arg(path);
        let out = invoke(cmd, None).expect("OpenSSH sign");
        assert!(out.status.success(), "Ed25519 signer failed");
        let sig = PathBuf::from(format!("{}.sig", path.display()));
        assert!(sig.is_file());
        sig
    }
    fn verify(
        path: &Path,
        sig: &Path,
        pubkey: &str,
        principal: &str,
        namespace: &str,
        temp: &Path,
    ) -> bool {
        let data = match fs::read(path) {
            Ok(v) if v.len() < 2048 => v,
            _ => return false,
        };
        if !valid_key(pubkey) {
            return false;
        }
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let allowed = temp.join(format!("allowed_{n}.txt"));
        if fs::write(&allowed, format!("{principal} {pubkey}\n")).is_err() {
            return false;
        }
        let mut cmd = Command::new("/usr/bin/ssh-keygen");
        cmd.args(["-Y", "verify", "-f"])
            .arg(&allowed)
            .args(["-I", principal, "-n", namespace, "-s"])
            .arg(sig);
        invoke(cmd, Some(&data)).is_ok_and(|v| v.status.success())
    }
    struct Lab {
        dir: PathBuf,
        signer: PathBuf,
        verifier: PathBuf,
        guardian1: PathBuf,
        guardian2: PathBuf,
        rogue: PathBuf,
        publisher1: PathBuf,
        publisher2: PathBuf,
        initial: Custody,
    }
    impl Lab {
        fn new() -> Self {
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("nordoi-r4s-{}-{n}", std::process::id()));
            fs::create_dir(&dir).unwrap();
            let dir = fs::canonicalize(dir).unwrap();
            let signer = dir.join("private_signers");
            let verifier = dir.join("public_verifier");
            fs::create_dir(&signer).unwrap();
            fs::create_dir(&verifier).unwrap();
            let guardian1 = signer.join("guardian1");
            let guardian2 = signer.join("guardian2");
            let rogue = signer.join("rogue");
            let publisher1 = signer.join("publisher1");
            let publisher2 = signer.join("publisher2");
            for (p, label) in [
                (&guardian1, "r4s-guardian1"),
                (&guardian2, "r4s-guardian2"),
                (&rogue, "r4s-rogue"),
                (&publisher1, "r4s-publisher1"),
                (&publisher2, "r4s-publisher2"),
            ] {
                let mut cmd = Command::new("/usr/bin/ssh-keygen");
                cmd.args(["-q", "-t", "ed25519", "-N", "", "-C", label, "-f"])
                    .arg(p);
                assert!(invoke(cmd, None).unwrap().status.success());
            }
            let initial = Custody {
                epoch: 2,
                seq: 2,
                digest: digest(FLOOR.as_bytes()),
                guardian: key(&guardian1),
                publisher: key(&publisher1),
            };
            Self {
                dir,
                signer,
                verifier,
                guardian1,
                guardian2,
                rogue,
                publisher1,
                publisher2,
                initial,
            }
        }
        fn guardian(&self, which: u8) -> &Path {
            match which {
                1 => &self.guardian1,
                2 => &self.guardian2,
                3 => &self.rogue,
                _ => panic!("fixture signer id"),
            }
        }
        fn next(&self, prior: &Custody, rotation: bool, new_publisher: bool) -> Record {
            Record {
                epoch: prior.epoch + 1,
                seq: prior.seq + 1,
                predecessor: prior.digest.clone(),
                publisher: key(if new_publisher {
                    &self.publisher2
                } else {
                    &self.publisher1
                }),
                successor: if rotation {
                    key(&self.guardian2)
                } else {
                    "none".to_owned()
                },
            }
        }
        fn issue(
            &self,
            name: &str,
            record: &Record,
            signer: u8,
            proof: bool,
            namespace: &str,
        ) -> Signed {
            assert!(name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
            let message = self.signer.join(format!("{name}.record"));
            fs::write(&message, record.encode()).unwrap();
            let signature = sign(&message, self.guardian(signer), namespace);
            let successor_signature = if proof {
                // Detached SSHSIG verifies the message bytes, not the path.
                // Separate source filename prevents OpenSSH overwrite prompts.
                let proof_message = self.signer.join(format!("{name}.next-proof"));
                fs::copy(&message, &proof_message).unwrap();
                Some(sign(&proof_message, &self.guardian2, ROTATION_NS))
            } else {
                None
            };
            Signed {
                message,
                signature,
                successor_signature,
            }
        }
        fn admit(&self, trusted: &Custody, signed: &Signed) -> Result<Custody, &'static str> {
            if !verify(
                &signed.message,
                &signed.signature,
                &trusted.guardian,
                PRINCIPAL,
                CHECKPOINT_NS,
                &self.verifier,
            ) {
                return Err("GUARDIAN_SIGNATURE_INVALID");
            }
            let raw = fs::read_to_string(&signed.message).map_err(|_| "RECORD_UNAVAILABLE")?;
            let checkpoint = Record::parse(&raw).ok_or("RECORD_NOT_CANONICAL")?;
            if checkpoint.seq != trusted.seq.checked_add(1).ok_or("SEQUENCE_OVERFLOW")?
                || checkpoint.epoch != trusted.epoch.checked_add(1).ok_or("EPOCH_OVERFLOW")?
            {
                return Err("NONMONOTONIC_CHECKPOINT");
            }
            if checkpoint.predecessor != trusted.digest {
                return Err("PREDECESSOR_MISMATCH");
            }
            if checkpoint.successor != "none" {
                if checkpoint.successor == trusted.guardian {
                    return Err("SELF_ROTATION_REJECTED");
                }
                let proof = signed
                    .successor_signature
                    .as_ref()
                    .ok_or("NEW_GUARDIAN_PROOF_REQUIRED")?;
                if !verify(
                    &signed.message,
                    proof,
                    &checkpoint.successor,
                    PRINCIPAL,
                    ROTATION_NS,
                    &self.verifier,
                ) {
                    return Err("NEW_GUARDIAN_PROOF_INVALID");
                }
            }
            Ok(Custody {
                epoch: checkpoint.epoch,
                seq: checkpoint.seq,
                digest: digest(raw.as_bytes()),
                guardian: if checkpoint.successor == "none" {
                    trusted.guardian.clone()
                } else {
                    checkpoint.successor
                },
                publisher: checkpoint.publisher,
            })
        }
    }
    impl Drop for Lab {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("delete test fixture only");
        }
    }
    fn git(cmd: &mut Command) -> Result<Output, &'static str> {
        cmd.env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ASKPASS", "/usr/bin/false");
        let out = cmd.output().map_err(|_| "GIT_UNAVAILABLE")?;
        if !out.status.success() {
            return Err("GIT_FAILED");
        }
        Ok(out)
    }
    fn fetch(repo: &Path, name: &str) -> Result<Vec<u8>, &'static str> {
        let mut cmd = Command::new("/usr/bin/git");
        cmd.arg("-C")
            .arg(repo)
            .arg("show")
            .arg(format!("FETCH_HEAD:{REMOTE_DIR}{name}"));
        let out = git(&mut cmd)?;
        if out.stdout.is_empty() || out.stdout.len() > 4096 {
            return Err("REMOTE_OBJECT_MISSING");
        }
        Ok(out.stdout)
    }
    fn check_live_anchor(lab: &Lab) -> Result<(), &'static str> {
        let mut ls = Command::new("/usr/bin/git");
        ls.args(["ls-remote", URL, REMOTE_REF]);
        if git(&mut ls)?.stdout != format!("{REMOTE_HEAD}\t{REMOTE_REF}\n").as_bytes() {
            return Err("REMOTE_HEAD_MISMATCH");
        }
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let repo = lab.dir.join(format!("remote_{n}"));
        fs::create_dir(&repo).map_err(|_| "TEMP_REPO")?;
        let mut init = Command::new("/usr/bin/git");
        init.args(["init", "-q"]).arg(&repo);
        git(&mut init)?;
        let mut pull = Command::new("/usr/bin/git");
        pull.arg("-C")
            .arg(&repo)
            .args(["fetch", "--depth=1", "--no-tags", URL, REMOTE_REF]);
        git(&mut pull)?;
        let mut rev = Command::new("/usr/bin/git");
        rev.arg("-C").arg(&repo).args(["rev-parse", "FETCH_HEAD"]);
        if git(&mut rev)?.stdout != format!("{REMOTE_HEAD}\n").as_bytes() {
            return Err("REMOTE_COMMIT_CHANGED");
        }
        if fetch(&repo, "root_public.txt")? != format!("{REMOTE_ROOT}\n").as_bytes() {
            return Err("REMOTE_SIGNER_SUBSTITUTED");
        }
        let message = fetch(&repo, "witness_v2.txt")?;
        if message != FLOOR.as_bytes() {
            return Err("REMOTE_FLOOR_CHANGED");
        }
        let sig = fetch(&repo, "witness_v2.txt.sig")?;
        let msg_path = repo.join("floor.txt");
        let sig_path = repo.join("floor.txt.sig");
        fs::write(&msg_path, &message).map_err(|_| "REMOTE_WRITE")?;
        fs::write(&sig_path, sig).map_err(|_| "REMOTE_WRITE")?;
        if !verify(
            &msg_path,
            &sig_path,
            REMOTE_ROOT,
            REMOTE_ID,
            REMOTE_NS,
            &lab.verifier,
        ) {
            return Err("REMOTE_SSHSIG_INVALID");
        }
        if lab.initial.digest != digest(&message) {
            return Err("REMOTE_GENESIS_DIGEST_MISMATCH");
        }
        Ok(())
    }

    #[test]
    fn native_real_github_v2_anchor_and_sshsig_are_verified() {
        let lab = Lab::new();
        check_live_anchor(&lab).unwrap();
        assert_eq!((lab.initial.epoch, lab.initial.seq), (2, 2));
    }
    #[test]
    fn native_monotonic_three_four_five_and_guardian_rotation_pass() {
        let lab = Lab::new();
        let r3 = lab.next(&lab.initial, false, false);
        let s3 = lab.issue("g3", &r3, 1, false, CHECKPOINT_NS);
        let t3 = lab.admit(&lab.initial, &s3).unwrap();
        let r4 = lab.next(&t3, true, true);
        let s4 = lab.issue("g4", &r4, 1, true, CHECKPOINT_NS);
        let t4 = lab.admit(&t3, &s4).unwrap();
        assert_eq!(t4.guardian, key(&lab.guardian2));
        assert_eq!(t4.publisher, key(&lab.publisher2));
        let r5 = lab.next(&t4, false, true);
        let s5 = lab.issue("g5", &r5, 2, false, CHECKPOINT_NS);
        let t5 = lab.admit(&t4, &s5).unwrap();
        assert_eq!((t5.epoch, t5.seq), (5, 5));
    }
    #[test]
    fn native_old_signed_checkpoint_fails_after_new_checkpoint() {
        let lab = Lab::new();
        let s3 = lab.issue(
            "g3",
            &lab.next(&lab.initial, false, false),
            1,
            false,
            CHECKPOINT_NS,
        );
        let t3 = lab.admit(&lab.initial, &s3).unwrap();
        let s4 = lab.issue("g4", &lab.next(&t3, false, false), 1, false, CHECKPOINT_NS);
        let t4 = lab.admit(&t3, &s4).unwrap();
        assert_eq!(lab.admit(&t4, &s3), Err("NONMONOTONIC_CHECKPOINT"));
    }
    #[test]
    fn native_same_sequence_equivocation_fails_from_accepted_head() {
        let lab = Lab::new();
        let s3 = lab.issue(
            "g3",
            &lab.next(&lab.initial, false, false),
            1,
            false,
            CHECKPOINT_NS,
        );
        let t3 = lab.admit(&lab.initial, &s3).unwrap();
        let s4 = lab.issue(
            "accepted",
            &lab.next(&t3, false, false),
            1,
            false,
            CHECKPOINT_NS,
        );
        let t4 = lab.admit(&t3, &s4).unwrap();
        let alternate = lab.issue("fork", &lab.next(&t3, false, true), 1, false, CHECKPOINT_NS);
        assert_eq!(lab.admit(&t4, &alternate), Err("NONMONOTONIC_CHECKPOINT"));
    }
    #[test]
    fn native_wrong_predecessor_and_gap_are_denied() {
        let lab = Lab::new();
        let mut cp = lab.next(&lab.initial, false, false);
        cp.predecessor = "0".repeat(64);
        let s = lab.issue("wrong-predecessor", &cp, 1, false, CHECKPOINT_NS);
        assert_eq!(lab.admit(&lab.initial, &s), Err("PREDECESSOR_MISMATCH"));
        cp.predecessor = lab.initial.digest.clone();
        cp.epoch += 4;
        let gap = lab.issue("future-gap", &cp, 1, false, CHECKPOINT_NS);
        assert_eq!(
            lab.admit(&lab.initial, &gap),
            Err("NONMONOTONIC_CHECKPOINT")
        );
    }
    #[test]
    fn native_forged_guardian_and_wrong_namespace_fail_closed() {
        let lab = Lab::new();
        let cp = lab.next(&lab.initial, false, false);
        let rogue = lab.issue("rogue", &cp, 3, false, CHECKPOINT_NS);
        assert_eq!(
            lab.admit(&lab.initial, &rogue),
            Err("GUARDIAN_SIGNATURE_INVALID")
        );
        let wrong = lab.issue("domain", &cp, 1, false, ROTATION_NS);
        assert_eq!(
            lab.admit(&lab.initial, &wrong),
            Err("GUARDIAN_SIGNATURE_INVALID")
        );
    }
    #[test]
    fn native_rotation_needs_new_guardian_proof_and_revokes_old_signer() {
        let lab = Lab::new();
        let cp = lab.next(&lab.initial, true, false);
        let old_only = lab.issue("only-old", &cp, 1, false, CHECKPOINT_NS);
        assert_eq!(
            lab.admit(&lab.initial, &old_only),
            Err("NEW_GUARDIAN_PROOF_REQUIRED")
        );
        let both = lab.issue("both-guardians", &cp, 1, true, CHECKPOINT_NS);
        let next = lab.admit(&lab.initial, &both).unwrap();
        assert_eq!(next.guardian, key(&lab.guardian2));
        let after = lab.next(&next, false, false);
        let revoked = lab.issue("former-guardian", &after, 1, false, CHECKPOINT_NS);
        assert_eq!(
            lab.admit(&next, &revoked),
            Err("GUARDIAN_SIGNATURE_INVALID")
        );
        let accepted = lab.issue("successor-guardian", &after, 2, false, CHECKPOINT_NS);
        assert_eq!(lab.admit(&next, &accepted).unwrap().seq, 4);
    }
    #[test]
    fn native_rotation_proof_tampering_and_signed_message_edit_denied() {
        let lab = Lab::new();
        let cp = lab.next(&lab.initial, true, false);
        let signed = lab.issue("proof", &cp, 1, true, CHECKPOINT_NS);
        fs::write(signed.successor_signature.as_ref().unwrap(), b"corrupt").unwrap();
        assert_eq!(
            lab.admit(&lab.initial, &signed),
            Err("NEW_GUARDIAN_PROOF_INVALID")
        );
        let altered = lab.issue("altered", &cp, 1, true, CHECKPOINT_NS);
        fs::write(&altered.message, b"forged").unwrap();
        assert_eq!(
            lab.admit(&lab.initial, &altered),
            Err("GUARDIAN_SIGNATURE_INVALID")
        );
    }
    #[test]
    fn native_absent_signed_guardian_response_never_falls_back_to_local() {
        let lab = Lab::new();
        let cp = lab.next(&lab.initial, false, false);
        let signed = lab.issue("no-signature", &cp, 1, false, CHECKPOINT_NS);
        fs::remove_file(&signed.signature).unwrap();
        assert_eq!(
            lab.admit(&lab.initial, &signed),
            Err("GUARDIAN_SIGNATURE_INVALID")
        );
    }
    #[test]
    fn native_full_verifier_and_guardian_history_restore_is_known_attack() {
        let lab = Lab::new();
        let signed3 = lab.issue(
            "g3",
            &lab.next(&lab.initial, false, false),
            1,
            false,
            CHECKPOINT_NS,
        );
        let t3 = lab.admit(&lab.initial, &signed3).unwrap();
        let signed4 = lab.issue("g4", &lab.next(&t3, false, false), 1, false, CHECKPOINT_NS);
        let t4 = lab.admit(&t3, &signed4).unwrap();
        assert_eq!(t4.seq, 4);
        // Negative finding: if the attacker controls BOTH the client's
        // last trusted head AND the writable witness historical records,
        // an old signed checkpoint may be replayed. Not production security.
        assert_eq!(lab.admit(&lab.initial, &signed3), Ok(t3));
    }

    // R4T independent-custodian *protocol* fixture. Both roles and their
    // private keys are controlled by the SAME laboratory process/account.
    // Signed witness receipts do not provide tamper-proof monotonic storage.
    const OWNER_NS: &str = "nordoi-r4t-owner-enrollment@eddarsglobal";
    const WITNESS_NS: &str = "nordoi-r4t-guardian-highwater@eddarsglobal";
    const CUSTODIAN_ID: &str = "nordoi-r4t-external-guardian-fixture";
    const OWNER_ID: &str = "nordoi-r4t-owner-fixture";

    fn generate_owner_fixture(lab: &Lab, name: &str) -> PathBuf {
        let path = lab.signer.join(name);
        let mut command = Command::new("/usr/bin/ssh-keygen");
        command
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "r4t-owner-fixture", "-f"])
            .arg(&path);
        assert!(
            invoke(command, None).unwrap().status.success(),
            "fresh disposable offline owner Ed25519 signer"
        );
        path
    }

    fn enrollment(lab: &Lab, owner_key: &Path, guardian_key: &str,
                  name: &str, namespace: &str) -> Signed {
        let message = lab.signer.join(format!("{name}.enrollment"));
        fs::write(
            &message,
            format!(
                "NORDOI-R4T-ENROLLMENT-V1\nproject=eddarsglobal/nordoi\nguardian={guardian_key}\n"
            ),
        )
        .unwrap();
        let signature = sign(&message, owner_key, namespace);
        Signed { message, signature, successor_signature: None }
    }

    fn check_owner_enrollment(lab: &Lab, owner_public: &str,
                              owner_approved_pin: Option<&str>,
                              requested_guardian: &str, signed: &Signed)
        -> Result<(), &'static str> {
        let pinned = owner_approved_pin.ok_or("OWNER_TRUST_NOT_ENROLLED")?;
        // The expected owner's digest MUST pre-exist the candidate.
        // The test generates both, so real out-of-band enrollment is absent.
        if !hex64(pinned) || digest(owner_public.as_bytes()) != pinned {
            return Err("OWNER_ROOT_NOT_PREAPPROVED");
        }
        if !verify(&signed.message, &signed.signature, owner_public,
                   OWNER_ID, OWNER_NS, &lab.verifier) {
            return Err("OWNER_ENROLLMENT_SIGNATURE_UNTRUSTED");
        }
        let actual = fs::read_to_string(&signed.message)
            .map_err(|_| "OWNER_ENROLLMENT_UNREADABLE")?;
        let required = format!(
            "NORDOI-R4T-ENROLLMENT-V1\nproject=eddarsglobal/nordoi\nguardian={requested_guardian}\n"
        );
        if actual != required {
            return Err("GUARDIAN_NOT_OWNER_APPROVED");
        }
        Ok(())
    }

    // Canonical signed external high-water receipt, independently signed
    // under the fixture-owner signer. It is NOT a real external log head.
    fn signed_highwater(lab: &Lab, owner_key: &Path, custody: &Custody,
                        revoked_pub: &str, name: &str, namespace: &str) -> Signed {
        let message = lab.signer.join(format!("{name}.receipt"));
        fs::write(
            &message,
            format!(
                "NORDOI-R4T-HIGHWATER-V1\nepoch={}\nsequence={}\nhead_sha256={}\npublisher={}\nrevoked={}\n",
                custody.epoch, custody.seq, custody.digest,
                custody.publisher, revoked_pub,
            ),
        )
        .unwrap();
        let signature = sign(&message, owner_key, namespace);
        Signed { message, signature, successor_signature: None }
    }

    fn admit_with_custodian_receipt(
        lab: &Lab,
        requested: &Custody,
        requested_publisher: &str,
        signed_receipt: Option<&Signed>,
        verified_owner_root: &str,
        independently_pinned_root_digest: Option<&str>,
    ) -> Result<(), &'static str> {
        let approved = independently_pinned_root_digest
            .ok_or("INDEPENDENT_OWNER_PIN_REQUIRED")?;
        if !hex64(approved) || digest(verified_owner_root.as_bytes()) != approved {
            return Err("INDEPENDENT_OWNER_PIN_MISMATCH");
        }
        let receipt = signed_receipt.ok_or("GUARDIAN_RECEIPT_UNAVAILABLE")?;
        if !verify(&receipt.message, &receipt.signature, verified_owner_root,
                   CUSTODIAN_ID, WITNESS_NS, &lab.verifier) {
            return Err("GUARDIAN_RECEIPT_SIGNATURE_INVALID");
        }
        let actual = fs::read_to_string(&receipt.message)
            .map_err(|_| "GUARDIAN_RECEIPT_UNREADABLE")?;
        let fields: Vec<_> = actual.lines().collect();
        if fields.len() != 6 || fields[0] != "NORDOI-R4T-HIGHWATER-V1" {
            return Err("GUARDIAN_RECEIPT_NONCANONICAL");
        }
        let epoch = fields[1].strip_prefix("epoch=")
            .ok_or("GUARDIAN_RECEIPT_NONCANONICAL")?
            .parse::<u64>().map_err(|_| "GUARDIAN_RECEIPT_NONCANONICAL")?;
        let seq = fields[2].strip_prefix("sequence=")
            .ok_or("GUARDIAN_RECEIPT_NONCANONICAL")?
            .parse::<u64>().map_err(|_| "GUARDIAN_RECEIPT_NONCANONICAL")?;
        let head = fields[3].strip_prefix("head_sha256=")
            .ok_or("GUARDIAN_RECEIPT_NONCANONICAL")?;
        let publisher = fields[4].strip_prefix("publisher=")
            .ok_or("GUARDIAN_RECEIPT_NONCANONICAL")?;
        let revoked = fields[5].strip_prefix("revoked=")
            .ok_or("GUARDIAN_RECEIPT_NONCANONICAL")?;
        let canonical = format!(
            "NORDOI-R4T-HIGHWATER-V1\nepoch={epoch}\nsequence={seq}\nhead_sha256={head}\npublisher={publisher}\nrevoked={revoked}\n"
        );
        if actual != canonical || epoch == 0 || seq == 0 || !hex64(head)
            || !valid_key(publisher) || (revoked != "none" && !valid_key(revoked)) {
            return Err("GUARDIAN_RECEIPT_NONCANONICAL");
        }
        if requested.epoch != epoch || requested.seq != seq || requested.digest != head {
            return Err("SIGNED_WITNESS_NOT_CURRENT");
        }
        if requested.publisher != publisher || requested_publisher != publisher
            || requested_publisher == revoked {
            return Err("PUBLISHER_REVOKED_OR_NOT_APPROVED");
        }
        Ok(())
    }

    #[test]
    fn native_r4t_owner_enrollment_requires_separately_supplied_root_pin() {
        let lab = Lab::new();
        let owner = generate_owner_fixture(&lab, "owner-r4t-1");
        let owner_pub = key(&owner);
        let pin = digest(owner_pub.as_bytes());
        let guardian = key(&lab.guardian1);
        let credential = enrollment(&lab, &owner, &guardian, "approved", OWNER_NS);
        assert_eq!(
            check_owner_enrollment(&lab, &owner_pub, None, &guardian, &credential),
            Err("OWNER_TRUST_NOT_ENROLLED")
        );
        assert_eq!(
            check_owner_enrollment(&lab, &owner_pub, Some(&"0".repeat(64)),
                                   &guardian, &credential),
            Err("OWNER_ROOT_NOT_PREAPPROVED")
        );
        assert_eq!(
            check_owner_enrollment(&lab, &owner_pub, Some(&pin),
                                   &guardian, &credential),
            Ok(())
        );
    }

    #[test]
    fn native_r4t_owner_signature_domain_and_guardian_substitution_denied() {
        let lab = Lab::new();
        let owner = generate_owner_fixture(&lab, "owner-r4t-2");
        let root_pub = key(&owner);
        let pin = digest(root_pub.as_bytes());
        let approved = key(&lab.guardian1);
        let other = key(&lab.guardian2);
        let valid = enrollment(&lab, &owner, &approved, "approved", OWNER_NS);
        assert_eq!(
            check_owner_enrollment(&lab, &root_pub, Some(&pin), &other, &valid),
            Err("GUARDIAN_NOT_OWNER_APPROVED")
        );
        let wrong = enrollment(&lab, &owner, &approved, "wrong-domain", WITNESS_NS);
        assert_eq!(
            check_owner_enrollment(&lab, &root_pub, Some(&pin), &approved, &wrong),
            Err("OWNER_ENROLLMENT_SIGNATURE_UNTRUSTED")
        );
        let altered = enrollment(&lab, &owner, &approved, "tampered", OWNER_NS);
        fs::write(&altered.message, b"forged").unwrap();
        assert_eq!(
            check_owner_enrollment(&lab, &root_pub, Some(&pin), &approved, &altered),
            Err("OWNER_ENROLLMENT_SIGNATURE_UNTRUSTED")
        );
    }

    #[test]
    fn native_r4t_signed_highwater_denies_older_chain_after_complete_local_restore() {
        let lab = Lab::new();
        let owner = generate_owner_fixture(&lab, "owner-r4t-3");
        let owner_pub = key(&owner);
        let pin = digest(owner_pub.as_bytes());
        let s3 = lab.issue("r4t-g3", &lab.next(&lab.initial, false, false),
                           1, false, CHECKPOINT_NS);
        let t3 = lab.admit(&lab.initial, &s3).unwrap();
        let s4 = lab.issue("r4t-g4", &lab.next(&t3, false, true),
                           1, false, CHECKPOINT_NS);
        let t4 = lab.admit(&t3, &s4).unwrap();
        let current = signed_highwater(&lab, &owner, &t4, "none", "current", WITNESS_NS);
        assert_eq!(
            admit_with_custodian_receipt(&lab, &t4, &t4.publisher,
                                         Some(&current), &owner_pub, Some(&pin)),
            Ok(())
        );
        // Attacker restores the entire writable NORDOI chain state to t3.
        // The *intact* signed t4 custodian receipt rejects the stale head.
        assert_eq!(
            admit_with_custodian_receipt(&lab, &t3, &t3.publisher,
                                         Some(&current), &owner_pub, Some(&pin)),
            Err("SIGNED_WITNESS_NOT_CURRENT")
        );
    }

    #[test]
    fn native_r4t_revoked_publisher_cannot_use_current_custodian_receipt() {
        let lab = Lab::new();
        let owner = generate_owner_fixture(&lab, "owner-r4t-4");
        let owner_pub = key(&owner);
        let pin = digest(owner_pub.as_bytes());
        let s3 = lab.issue("r4t-r3", &lab.next(&lab.initial, false, false),
                           1, false, CHECKPOINT_NS);
        let t3 = lab.admit(&lab.initial, &s3).unwrap();
        let s4 = lab.issue("r4t-r4", &lab.next(&t3, false, true),
                           1, false, CHECKPOINT_NS);
        let t4 = lab.admit(&t3, &s4).unwrap();
        let receipt = signed_highwater(&lab, &owner, &t4, &t3.publisher,
                                       "revoked", WITNESS_NS);
        assert_eq!(
            admit_with_custodian_receipt(&lab, &t4, &t4.publisher,
                                         Some(&receipt), &owner_pub, Some(&pin)),
            Ok(())
        );
        assert_eq!(
            admit_with_custodian_receipt(&lab, &t4, &t3.publisher,
                                         Some(&receipt), &owner_pub, Some(&pin)),
            Err("PUBLISHER_REVOKED_OR_NOT_APPROVED")
        );
    }

    #[test]
    fn native_r4t_missing_or_forged_witness_and_owner_pin_fail_closed() {
        let lab = Lab::new();
        let owner = generate_owner_fixture(&lab, "owner-r4t-5");
        let owner_pub = key(&owner);
        let pin = digest(owner_pub.as_bytes());
        assert_eq!(
            admit_with_custodian_receipt(&lab, &lab.initial, &lab.initial.publisher,
                                         None, &owner_pub, Some(&pin)),
            Err("GUARDIAN_RECEIPT_UNAVAILABLE")
        );
        let receipt = signed_highwater(&lab, &owner, &lab.initial, "none",
                                       "missing-root", WITNESS_NS);
        assert_eq!(
            admit_with_custodian_receipt(&lab, &lab.initial, &lab.initial.publisher,
                                         Some(&receipt), &owner_pub, None),
            Err("INDEPENDENT_OWNER_PIN_REQUIRED")
        );
        let bad_domain = signed_highwater(&lab, &owner, &lab.initial, "none",
                                          "bad-domain", OWNER_NS);
        assert_eq!(
            admit_with_custodian_receipt(&lab, &lab.initial, &lab.initial.publisher,
                                         Some(&bad_domain), &owner_pub, Some(&pin)),
            Err("GUARDIAN_RECEIPT_SIGNATURE_INVALID")
        );
    }

    #[test]
    fn native_r4t_restored_signed_custodian_receipt_plus_verifier_remains_vulnerable() {
        let lab = Lab::new();
        let owner = generate_owner_fixture(&lab, "owner-r4t-6");
        let root = key(&owner);
        let pin = digest(root.as_bytes());
        let s3 = lab.issue("r4t-old3", &lab.next(&lab.initial, false, false),
                           1, false, CHECKPOINT_NS);
        let t3 = lab.admit(&lab.initial, &s3).unwrap();
        let old = signed_highwater(&lab, &owner, &t3, "none", "old-head", WITNESS_NS);
        let s4 = lab.issue("r4t-new4", &lab.next(&t3, false, false),
                           1, false, CHECKPOINT_NS);
        let t4 = lab.admit(&t3, &s4).unwrap();
        let new_head = signed_highwater(&lab, &owner, &t4, "none",
                                        "new-head", WITNESS_NS);
        assert_eq!(
            admit_with_custodian_receipt(&lab, &t4, &t4.publisher,
                                         Some(&new_head), &root, Some(&pin)),
            Ok(())
        );
        // Counterexample: client and custodian BOTH restored to old signed
        // state; static Ed25519 authenticity alone cannot prove freshness.
        assert_eq!(
            admit_with_custodian_receipt(&lab, &t3, &t3.publisher,
                                         Some(&old), &root, Some(&pin)),
            Ok(())
        );
    }

}

#[cfg(not(target_os = "macos"))]
#[test]
fn portable_target_does_not_claim_administratively_independent_witness() {
    assert_ne!(std::env::consts::OS, "macos");
}

#[test]
fn reference_model_external_host_io_is_default_denied() {
    let (mut model, permit) = Model::bootstrap(
        0x53,
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
