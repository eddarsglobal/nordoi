//! G0.3 R4N — independently enrolled ED25519 root, SSHSIG manifests,
//! delegated release signer rotation, revocation and monotonic rollback lab.
//! Native macOS test fixture only. No NORDOI production authority is granted.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;
use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod native {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};

    const POLICY_NS: &str = "nordoi-r4n-policy@eddarsglobal";
    const RELEASE_NS: &str = "nordoi-r4n-release@eddarsglobal";
    const ROOT_PRINCIPAL: &str = "nordoi-r4n-root";
    const RELEASE_PRINCIPAL: &str = "nordoi-r4n-release";
    static SERIAL: AtomicU64 = AtomicU64::new(1);

    fn valid_public_key(s: &str) -> bool {
        let words: Vec<&str> = s.split(' ').collect();
        words.len() == 2
            && words[0] == "ssh-ed25519"
            && words[1].len() >= 40
            && words[1]
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'+' || c == b'/' || c == b'=')
    }

    fn pubkey(path: &Path) -> String {
        let raw = fs::read_to_string(path).expect("read freshly generated public key");
        let words: Vec<&str> = raw.split_whitespace().collect();
        assert!(words.len() >= 2, "public key tokens");
        let key = format!("{} {}", words[0], words[1]);
        assert!(valid_public_key(&key), "independent Ed25519 public key");
        key
    }

    fn cdhash_valid(s: &str) -> bool {
        s.len() == 40
            && s.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Policy {
        pub epoch: u64,
        pub delegate: String,
        pub revoked: String,
    }
    impl Policy {
        fn encode(&self) -> String {
            format!(
                "NORDOI-R4N-POLICY-V1\nepoch={}\ndelegate={}\nrevoked={}\n",
                self.epoch, self.delegate, self.revoked
            )
        }
        fn parse(s: &str) -> Option<Self> {
            let lines: Vec<&str> = s.lines().collect();
            if lines.len() != 4 || lines[0] != "NORDOI-R4N-POLICY-V1" {
                return None;
            }
            let epoch = lines[1].strip_prefix("epoch=")?.parse::<u64>().ok()?;
            let delegate = lines[2].strip_prefix("delegate=")?.to_string();
            let revoked = lines[3].strip_prefix("revoked=")?.to_string();
            if epoch == 0 || !valid_public_key(&delegate) {
                return None;
            }
            if revoked != "none" && !valid_public_key(&revoked) {
                return None;
            }
            let p = Self {
                epoch,
                delegate,
                revoked,
            };
            (p.encode() == s).then_some(p)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Release {
        pub epoch: u64,
        pub seq: u64,
        pub cdhash: String,
    }
    impl Release {
        fn encode(&self) -> String {
            format!(
                "NORDOI-R4N-RELEASE-V1\nepoch={}\nseq={}\ncdhash={}\n",
                self.epoch, self.seq, self.cdhash
            )
        }
        fn parse(s: &str) -> Option<Self> {
            let lines: Vec<&str> = s.lines().collect();
            if lines.len() != 4 || lines[0] != "NORDOI-R4N-RELEASE-V1" {
                return None;
            }
            let epoch = lines[1].strip_prefix("epoch=")?.parse::<u64>().ok()?;
            let seq = lines[2].strip_prefix("seq=")?.parse::<u64>().ok()?;
            let cdhash = lines[3].strip_prefix("cdhash=")?.to_string();
            if epoch == 0 || seq == 0 || !cdhash_valid(&cdhash) {
                return None;
            }
            let r = Self { epoch, seq, cdhash };
            (r.encode() == s).then_some(r)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Custody {
        pub epoch: u64,
        pub seq: u64,
        pub active: String,
        pub revoked: String,
    }
    impl Custody {
        pub fn empty() -> Self {
            Self {
                epoch: 0,
                seq: 0,
                active: "none".to_string(),
                revoked: "none".to_string(),
            }
        }
        fn encode(&self) -> String {
            format!(
                "NORDOI-R4N-CUSTODY-V1\nepoch={}\nseq={}\nactive={}\nrevoked={}\n",
                self.epoch, self.seq, self.active, self.revoked
            )
        }
        pub fn parse(s: &str) -> Option<Self> {
            let lines: Vec<&str> = s.lines().collect();
            if lines.len() != 5 || lines[0] != "NORDOI-R4N-CUSTODY-V1" {
                return None;
            }
            let epoch = lines[1].strip_prefix("epoch=")?.parse().ok()?;
            let seq = lines[2].strip_prefix("seq=")?.parse().ok()?;
            let active = lines[3].strip_prefix("active=")?.to_string();
            let revoked = lines[4].strip_prefix("revoked=")?.to_string();
            if epoch == 0 {
                if seq != 0 || active != "none" || revoked != "none" {
                    return None;
                }
            } else if seq == 0 || !valid_public_key(&active) {
                return None;
            }
            if revoked != "none" && !valid_public_key(&revoked) {
                return None;
            }
            let value = Self {
                epoch,
                seq,
                active,
                revoked,
            };
            (value.encode() == s).then_some(value)
        }
        pub fn save_test_only(&self, path: &Path) {
            // This is NOT tamper-resistant storage: a malicious filesystem
            // owner can restore an older valid-looking cursor.
            let temp = path.with_extension("temporary");
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)
                .expect("fresh private cursor staging file");
            file.write_all(self.encode().as_bytes())
                .expect("write complete research cursor");
            file.sync_all().expect("sync temporary state");
            fs::rename(&temp, path).expect("atomic rename research cursor");
        }
    }

    pub struct Bundle {
        pub msg: PathBuf,
        pub sig: PathBuf,
    }

    pub struct Lab {
        dir: PathBuf,
        signer: PathBuf,
        verifier: PathBuf,
        root: PathBuf,
        delegate1: PathBuf,
        delegate2: PathBuf,
        rogue: PathBuf,
        root_allowed: PathBuf,
        good: PathBuf,
        bad: PathBuf,
        good_hash: String,
        bad_hash: String,
    }

    fn execute(mut cmd: Command, stdin: Option<&[u8]>) -> std::process::Output {
        if stdin.is_some() {
            cmd.stdin(Stdio::piped());
        }
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("invoke installed macOS research tool");
        if let Some(data) = stdin {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(data)
                .expect("bounded signature input");
        }
        child
            .wait_with_output()
            .expect("finish installed macOS research tool")
    }

    fn sign_message(message: &Path, private_key: &Path, namespace: &str) -> Bundle {
        let mut cmd = Command::new("/usr/bin/ssh-keygen");
        cmd.args(["-Y", "sign", "-f"])
            .arg(private_key)
            .args(["-n", namespace])
            .arg(message);
        let out = execute(cmd, None);
        assert!(
            out.status.success(),
            "signer fixture SSHSIG: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let sig = PathBuf::from(format!("{}.sig", message.display()));
        assert!(sig.is_file(), "OpenSSH must create detached signature");
        Bundle {
            msg: message.to_path_buf(),
            sig,
        }
    }

    fn verify_message(bundle: &Bundle, allowed: &Path, principal: &str, namespace: &str) -> bool {
        let data = match fs::read(&bundle.msg) {
            Ok(v) if v.len() < 2048 => v,
            _ => return false,
        };
        let mut cmd = Command::new("/usr/bin/ssh-keygen");
        cmd.args(["-Y", "verify", "-f"])
            .arg(allowed)
            .args(["-I", principal, "-n", namespace, "-s"])
            .arg(&bundle.sig);
        execute(cmd, Some(&data)).status.success()
    }

    impl Lab {
        pub fn new() -> Self {
            let counter = SERIAL.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("nordoi-r4n-{}-{counter}", std::process::id()));
            fs::create_dir(&dir).expect("new temporary directory");
            let dir = fs::canonicalize(dir).expect("canonical temporary directory");
            let signer = dir.join("signer_private_fixture");
            let verifier = dir.join("independent_verifier_fixture");
            fs::create_dir(&signer).expect("signer directory");
            fs::create_dir(&verifier).expect("verifier directory");
            let root = signer.join("root");
            let delegate1 = signer.join("release1");
            let delegate2 = signer.join("release2");
            let rogue = signer.join("rogue");
            for (key, comment) in [
                (&root, "r4n-test-root"),
                (&delegate1, "r4n-test-delegate1"),
                (&delegate2, "r4n-test-delegate2"),
                (&rogue, "r4n-test-rogue"),
            ] {
                let mut cmd = Command::new("/usr/bin/ssh-keygen");
                cmd.args(["-q", "-t", "ed25519", "-N", "", "-C", comment, "-f"])
                    .arg(key);
                let r = execute(cmd, None);
                assert!(r.status.success(), "generate disposable Ed25519 key");
            }
            let root_allowed = verifier.join("root_allowed_signers");
            fs::write(
                &root_allowed,
                format!("{ROOT_PRINCIPAL} {}\n", pubkey(&root.with_extension("pub"))),
            )
            .expect("enroll public trust root outside signed manifest");
            let good = dir.join("approved_binary");
            let bad = dir.join("different_binary");
            let mut lab = Self {
                dir,
                signer,
                verifier,
                root,
                delegate1,
                delegate2,
                rogue,
                root_allowed,
                good,
                bad,
                good_hash: String::new(),
                bad_hash: String::new(),
            };
            lab.make_signed_code();
            lab
        }

        fn make_signed_code(&mut self) {
            for (name, path) in [
                ("approved_code_v1", &self.good),
                ("changed_code_same_identifier", &self.bad),
            ] {
                let src = self.dir.join(format!("{name}.c"));
                fs::write(
                    &src,
                    format!(
                        "#include <stdio.h>\nint main(void) {{ puts(\"{name}\"); return 0; }}\n"
                    ),
                )
                .expect("bounded disposable native source");
                let mut cc = Command::new("/usr/bin/cc");
                cc.args(["-O0", "-x", "c"]).arg(&src).arg("-o").arg(path);
                assert!(
                    execute(cc, None).status.success(),
                    "build real Mach-O executable"
                );
                let mut cs = Command::new("/usr/bin/codesign");
                cs.args([
                    "--force",
                    "--sign",
                    "-",
                    "--identifier",
                    "org.eddarsglobal.nordoi.r4n.authorized-code",
                ])
                .arg(path);
                assert!(
                    execute(cs, None).status.success(),
                    "sign disposable Mach-O code"
                );
            }
            self.good_hash = Self::code_hash(&self.good);
            self.bad_hash = Self::code_hash(&self.bad);
            assert_ne!(
                self.good_hash, self.bad_hash,
                "different signed code directories"
            );
        }

        pub fn code_hash(path: &Path) -> String {
            let mut verify = Command::new("/usr/bin/codesign");
            verify.args(["--verify", "--strict"]).arg(path);
            assert!(
                execute(verify, None).status.success(),
                "valid code signature"
            );
            let mut display = Command::new("/usr/bin/codesign");
            display.args(["--display", "--verbose=4"]).arg(path);
            let out = execute(display, None);
            assert!(out.status.success(), "real signed code identity");
            let text = String::from_utf8_lossy(&out.stderr);
            let hash = text
                .lines()
                .find_map(|line| line.trim().strip_prefix("CDHash="))
                .expect("actual Apple CDHash required")
                .to_ascii_lowercase();
            assert!(cdhash_valid(&hash), "canonical 20-byte CodeDirectory hash");
            hash
        }

        pub fn approved_hash(&self) -> &str {
            &self.good_hash
        }
        pub fn modified_hash(&self) -> &str {
            &self.bad_hash
        }

        pub fn delegate_key(&self, id: u8) -> &Path {
            match id {
                1 => &self.delegate1,
                2 => &self.delegate2,
                3 => &self.rogue,
                _ => panic!("only known disposable fixture signer"),
            }
        }

        pub fn delegate_public(&self, id: u8) -> String {
            pubkey(&self.delegate_key(id).with_extension("pub"))
        }

        pub fn root_public(&self) -> String {
            pubkey(&self.root.with_extension("pub"))
        }

        pub fn policy(&self, id: &str, data: &Policy, signer_rogue: bool) -> Bundle {
            let path = self.signer.join(format!("{id}.policy"));
            fs::write(&path, data.encode()).expect("write test policy");
            sign_message(
                &path,
                if signer_rogue {
                    &self.rogue
                } else {
                    &self.root
                },
                POLICY_NS,
            )
        }

        pub fn release(&self, id: &str, data: &Release, signer_id: u8, namespace: &str) -> Bundle {
            let path = self.signer.join(format!("{id}.release"));
            fs::write(&path, data.encode()).expect("write test release");
            sign_message(&path, self.delegate_key(signer_id), namespace)
        }

        pub fn root_trust(&self) -> &Path {
            &self.root_allowed
        }

        pub fn verify(
            &self,
            policy: &Bundle,
            release: &Bundle,
            previous: &Custody,
            native_cdhash: &str,
        ) -> Result<Custody, &'static str> {
            // Verifier inputs intentionally exclude all private signing keys.
            if !verify_message(policy, self.root_trust(), ROOT_PRINCIPAL, POLICY_NS) {
                return Err("ROOT_NOT_AUTHENTICATED");
            }
            let p = Policy::parse(&fs::read_to_string(&policy.msg).map_err(|_| "POLICY_READ")?)
                .ok_or("POLICY_NONCANONICAL")?;
            if p.delegate == p.revoked {
                return Err("ACTIVE_KEY_ALREADY_REVOKED");
            }
            if previous.epoch == 0 {
                if p.epoch != 1 || p.revoked != "none" {
                    return Err("UNENROLLED_EPOCH");
                }
            } else if p.epoch == previous.epoch {
                if p.delegate != previous.active || p.revoked != previous.revoked {
                    return Err("SAME_EPOCH_AUTHORITY_MUTATION");
                }
            } else if p.epoch == previous.epoch + 1 {
                // One-step root-authorized key rotation with predecessor
                // explicitly revoked. No silently trusted candidate keys.
                if p.delegate == previous.active || p.revoked != previous.active {
                    return Err("UNAUTHORIZED_KEY_ROTATION");
                }
            } else {
                return Err("EPOCH_REPLAY_OR_GAP");
            }
            let allowed = self.verifier.join("release_allowed_signers");
            fs::write(&allowed, format!("{RELEASE_PRINCIPAL} {}\n", p.delegate))
                .map_err(|_| "RELEASE_PUBLIC_ENROLLMENT")?;
            if !verify_message(release, &allowed, RELEASE_PRINCIPAL, RELEASE_NS) {
                return Err("DELEGATE_SIGNATURE_NOT_AUTHENTICATED");
            }
            let r = Release::parse(&fs::read_to_string(&release.msg).map_err(|_| "RELEASE_READ")?)
                .ok_or("RELEASE_NONCANONICAL")?;
            if r.epoch != p.epoch || r.seq <= previous.seq {
                return Err("RELEASE_REPLAY_OR_WRONG_EPOCH");
            }
            if !cdhash_valid(native_cdhash) || r.cdhash != native_cdhash {
                return Err("CODE_VERSION_NOT_AUTHORIZED");
            }
            Ok(Custody {
                epoch: p.epoch,
                seq: r.seq,
                active: p.delegate,
                revoked: p.revoked,
            })
        }

        pub fn policy_v1(&self) -> Policy {
            Policy {
                epoch: 1,
                delegate: self.delegate_public(1),
                revoked: "none".into(),
            }
        }
        pub fn policy_v2(&self) -> Policy {
            Policy {
                epoch: 2,
                delegate: self.delegate_public(2),
                revoked: self.delegate_public(1),
            }
        }
        pub fn release_v1(&self) -> Release {
            Release {
                epoch: 1,
                seq: 1,
                cdhash: self.good_hash.clone(),
            }
        }
        pub fn base(&self) -> (Bundle, Bundle, Custody) {
            let p = self.policy("p1", &self.policy_v1(), false);
            let r = self.release("r1", &self.release_v1(), 1, RELEASE_NS);
            let c = self
                .verify(&p, &r, &Custody::empty(), self.approved_hash())
                .expect("real root + delegate + code admission");
            (p, r, c)
        }

        pub fn assert_no_private_signer_in_verifier(&self) {
            let allowed =
                fs::read_to_string(self.root_trust()).expect("read independent public-only anchor");
            assert!(allowed.starts_with(ROOT_PRINCIPAL));
            assert!(allowed.contains("ssh-ed25519 "));
            assert!(!allowed.contains("PRIVATE KEY"));
            assert!(!self.verifier.join("root").exists());
            assert!(!self.verifier.join("release1").exists());
            assert!(
                self.root.is_file(),
                "signer private root present only in signer fixture"
            );
        }
    }
    impl Drop for Lab {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("delete only disposable fixture");
        }
    }

    #[test]
    fn macos_root_and_delegate_sign_real_version_cdhash() {
        let lab = Lab::new();
        lab.assert_no_private_signer_in_verifier();
        let (_p, _r, c) = lab.base();
        assert_eq!(c.epoch, 1);
        assert_eq!(c.seq, 1);
        assert_eq!(c.active, lab.delegate_public(1));
        assert!(cdhash_valid(lab.approved_hash()));
    }

    #[test]
    fn macos_changed_real_code_and_modified_release_fail_closed() {
        let lab = Lab::new();
        let (p, r, c) = lab.base();
        assert_ne!(lab.approved_hash(), lab.modified_hash());
        assert_eq!(
            lab.verify(&p, &r, &Custody::empty(), lab.modified_hash()),
            Err("CODE_VERSION_NOT_AUTHORIZED")
        );
        // Tamper with signed manifest bytes; do not replace original signature.
        fs::write(
            &r.msg,
            Release {
                epoch: 1,
                seq: 1,
                cdhash: lab.modified_hash().to_string(),
            }
            .encode(),
        )
        .unwrap();
        assert_eq!(
            lab.verify(&p, &r, &Custody::empty(), lab.modified_hash()),
            Err("DELEGATE_SIGNATURE_NOT_AUTHENTICATED")
        );
        assert_eq!(c.seq, 1, "previously admitted custody state unchanged");
    }

    #[test]
    fn macos_rogue_root_cannot_authorize_modified_code() {
        let lab = Lab::new();
        let policy = lab.policy("rogue-policy", &lab.policy_v1(), true);
        let release = lab.release("rogue-release", &lab.release_v1(), 1, RELEASE_NS);
        assert_eq!(
            lab.verify(&policy, &release, &Custody::empty(), lab.approved_hash()),
            Err("ROOT_NOT_AUTHENTICATED")
        );
    }

    #[test]
    fn macos_wrong_delegate_and_cross_namespace_signature_fail_closed() {
        let lab = Lab::new();
        let p = lab.policy("p1", &lab.policy_v1(), false);
        let release = lab.release("signed-by-rogue", &lab.release_v1(), 3, RELEASE_NS);
        assert_eq!(
            lab.verify(&p, &release, &Custody::empty(), lab.approved_hash()),
            Err("DELEGATE_SIGNATURE_NOT_AUTHENTICATED")
        );
        let wrong_namespace = lab.release("wrong-domain", &lab.release_v1(), 1, POLICY_NS);
        assert_eq!(
            lab.verify(&p, &wrong_namespace, &Custody::empty(), lab.approved_hash()),
            Err("DELEGATE_SIGNATURE_NOT_AUTHENTICATED")
        );
    }

    #[test]
    fn macos_root_authorized_rotation_revokes_old_release_signer() {
        let lab = Lab::new();
        let (_p1, _r1, c1) = lab.base();
        let p2 = lab.policy("p2", &lab.policy_v2(), false);
        let r2 = lab.release(
            "r2",
            &Release {
                epoch: 2,
                seq: 2,
                cdhash: lab.approved_hash().into(),
            },
            2,
            RELEASE_NS,
        );
        let c2 = lab
            .verify(&p2, &r2, &c1, lab.approved_hash())
            .expect("root-approved new delegate, old signer explicitly revoked");
        assert_eq!(c2.epoch, 2);
        assert_eq!(c2.revoked, lab.delegate_public(1));
        let old_delegate = lab.release(
            "old-delegate",
            &Release {
                epoch: 2,
                seq: 3,
                cdhash: lab.approved_hash().into(),
            },
            1,
            RELEASE_NS,
        );
        assert_eq!(
            lab.verify(&p2, &old_delegate, &c2, lab.approved_hash()),
            Err("DELEGATE_SIGNATURE_NOT_AUTHENTICATED")
        );
    }

    #[test]
    fn macos_sequence_replay_and_pre_rotation_policy_are_denied() {
        let lab = Lab::new();
        let (p1, r1, c1) = lab.base();
        assert_eq!(
            lab.verify(&p1, &r1, &c1, lab.approved_hash()),
            Err("RELEASE_REPLAY_OR_WRONG_EPOCH")
        );
        let p2 = lab.policy("p2", &lab.policy_v2(), false);
        let r2 = lab.release(
            "r2",
            &Release {
                epoch: 2,
                seq: 2,
                cdhash: lab.approved_hash().into(),
            },
            2,
            RELEASE_NS,
        );
        let c2 = lab.verify(&p2, &r2, &c1, lab.approved_hash()).unwrap();
        assert_eq!(
            lab.verify(&p1, &r1, &c2, lab.approved_hash()),
            Err("EPOCH_REPLAY_OR_GAP")
        );
        let save = lab.verifier.join("monotonic_state_fixture");
        c2.save_test_only(&save);
        let recovered = Custody::parse(&fs::read_to_string(&save).unwrap())
            .expect("monotonic cursor recovered across test restart");
        assert_eq!(recovered, c2);
        assert_eq!(
            lab.verify(&p1, &r1, &recovered, lab.approved_hash()),
            Err("EPOCH_REPLAY_OR_GAP")
        );
    }

    #[test]
    fn macos_unsigned_rotation_and_reactivated_revoked_key_are_denied() {
        let lab = Lab::new();
        let (_, _, c1) = lab.base();
        let wrong_rotation = Policy {
            epoch: 2,
            delegate: lab.delegate_public(2),
            revoked: "none".into(),
        };
        let p2 = lab.policy("wrong-rotation", &wrong_rotation, false);
        let r2 = lab.release(
            "r2",
            &Release {
                epoch: 2,
                seq: 2,
                cdhash: lab.approved_hash().into(),
            },
            2,
            RELEASE_NS,
        );
        assert_eq!(
            lab.verify(&p2, &r2, &c1, lab.approved_hash()),
            Err("UNAUTHORIZED_KEY_ROTATION")
        );
        let unsigned = lab.policy("unsigned-revocation", &lab.policy_v2(), false);
        fs::write(&unsigned.msg, lab.policy_v1().encode()).unwrap();
        assert_eq!(
            lab.verify(&unsigned, &r2, &c1, lab.approved_hash()),
            Err("ROOT_NOT_AUTHENTICATED")
        );
    }

    #[test]
    fn macos_noncanonical_input_and_corrupt_custody_fail_closed() {
        let lab = Lab::new();
        assert!(Policy::parse("NORDOI-R4N-POLICY-V1\nepoch=01\n").is_none());
        assert!(Release::parse("NORDOI-R4N-RELEASE-V1\nepoch=1\nseq=1\ncdhash=evil\n").is_none());
        assert!(Custody::parse("NORDOI-R4N-CUSTODY-V1\nepoch=1\nseq=1\n").is_none());
        assert!(Custody::parse(
            "NORDOI-R4N-CUSTODY-V1\nepoch=0\nseq=1\nactive=none\nrevoked=none\n"
        )
        .is_none());
        assert!(!cdhash_valid(lab.approved_hash().to_uppercase().as_str()));
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn non_macos_does_not_claim_openssh_or_macos_code_signature_validation() {
    assert_ne!(std::env::consts::OS, "macos");
}

#[test]
fn reference_external_host_io_remains_default_denied() {
    let (mut model, permit) = Model::bootstrap(
        0x4e,
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
