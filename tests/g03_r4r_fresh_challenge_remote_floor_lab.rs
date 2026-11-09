//! NORDOI G0.3 R4R — dual-authority SSHSIG quorum adversarial experiment.
//! A single repository signer cannot supersede an independently pinned witness
//! in the bounded *model*. Both keys are generated in one test environment:
//! actual independent administrative custody is NOT established.
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

    static SERIAL: AtomicU64 = AtomicU64::new(1);
    const RELEASE_NS: &str = "nordoi-r4r-repository-release@eddarsglobal";
    const WITNESS_NS: &str = "nordoi-r4r-independent-checkpoint@eddarsglobal";
    const RELEASE_PRINCIPAL: &str = "nordoi-r4r-publisher";
    const WITNESS_PRINCIPAL: &str = "nordoi-r4r-witness";
    const EVIDENCE_COMMIT: &str = "ff8abe01cc1a527dd84b463dec1736d20a5391a4";
    const EVIDENCE_BRANCH: &str = "refs/heads/evidence/g03-r4p-external-signed-checkpoint-20261109";
    const GITHUB_URL: &str = "https://github.com/eddarsglobal/nordoi.git";

    fn run(mut cmd: Command, input: Option<&[u8]>) -> Output {
        if input.is_some() {
            cmd.stdin(Stdio::piped());
        }
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("native fixture tool must execute");
        if let Some(data) = input {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(data)
                .expect("bounded SSHSIG data");
        }
        child.wait_with_output().expect("native fixture tool exit")
    }

    fn pubkey(private: &Path) -> String {
        let path = PathBuf::from(format!("{}.pub", private.display()));
        let data = fs::read_to_string(path).expect("read Ed25519 public key");
        let tokens: Vec<_> = data.split_whitespace().collect();
        assert!(tokens.len() >= 2 && tokens[0] == "ssh-ed25519");
        assert!(tokens[1].len() >= 40);
        assert!(tokens[1]
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || v == b'+' || v == b'/' || v == b'='));
        format!("{} {}", tokens[0], tokens[1])
    }

    fn valid_sha(s: &str) -> bool {
        s.len() == 40
            && s.bytes()
                .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Cursor {
        epoch: u64,
        seq: u64,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Release {
        cursor: Cursor,
        evidence_commit: String,
    }

    impl Release {
        fn encode(&self) -> String {
            format!(
                "NORDOI-R4R-RELEASE-V1\nepoch={}\nseq={}\nevidence_commit={}\n",
                self.cursor.epoch, self.cursor.seq, self.evidence_commit
            )
        }

        fn parse(raw: &str) -> Option<Self> {
            let lines: Vec<_> = raw.lines().collect();
            if lines.len() != 4 || lines[0] != "NORDOI-R4R-RELEASE-V1" {
                return None;
            }
            let v = Self {
                cursor: Cursor {
                    epoch: lines[1].strip_prefix("epoch=")?.parse().ok()?,
                    seq: lines[2].strip_prefix("seq=")?.parse().ok()?,
                },
                evidence_commit: lines[3].strip_prefix("evidence_commit=")?.into(),
            };
            (v.cursor.epoch > 0
                && v.cursor.seq > 0
                && valid_sha(&v.evidence_commit)
                && v.encode() == raw)
                .then_some(v)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct IndependentHead {
        release: Release,
        approved_publisher: String,
    }

    impl IndependentHead {
        fn encode(&self) -> String {
            format!(
                "NORDOI-R4R-EXTERNAL-V1\nepoch={}\nseq={}\nevidence_commit={}\npublisher={}\n",
                self.release.cursor.epoch,
                self.release.cursor.seq,
                self.release.evidence_commit,
                self.approved_publisher
            )
        }

        fn parse(raw: &str) -> Option<Self> {
            let lines: Vec<_> = raw.lines().collect();
            if lines.len() != 5 || lines[0] != "NORDOI-R4R-EXTERNAL-V1" {
                return None;
            }
            let v = Self {
                release: Release {
                    cursor: Cursor {
                        epoch: lines[1].strip_prefix("epoch=")?.parse().ok()?,
                        seq: lines[2].strip_prefix("seq=")?.parse().ok()?,
                    },
                    evidence_commit: lines[3].strip_prefix("evidence_commit=")?.into(),
                },
                approved_publisher: lines[4].strip_prefix("publisher=")?.into(),
            };
            (v.release.cursor.epoch > 0
                && v.release.cursor.seq > 0
                && valid_sha(&v.release.evidence_commit)
                && v.approved_publisher.starts_with("ssh-ed25519 ")
                && v.approved_publisher.len() > 60
                && !v.approved_publisher.contains('\n')
                && v.encode() == raw)
                .then_some(v)
        }
    }

    #[derive(Clone, Debug)]
    pub struct Signed {
        msg: PathBuf,
        signature: PathBuf,
    }

    fn sign(path: &Path, key: &Path, namespace: &str) -> Signed {
        let mut cmd = Command::new("/usr/bin/ssh-keygen");
        cmd.args(["-Y", "sign", "-f"])
            .arg(key)
            .args(["-n", namespace])
            .arg(path);
        let out = run(cmd, None);
        assert!(
            out.status.success(),
            "real macOS Ed25519 SSHSIG signer: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let signature = PathBuf::from(format!("{}.sig", path.display()));
        assert!(signature.is_file());
        Signed {
            msg: path.to_owned(),
            signature,
        }
    }

    fn verify(signed: &Signed, key: &str, principal: &str, namespace: &str) -> bool {
        let payload = match fs::read(&signed.msg) {
            Ok(v) if v.len() <= 2048 => v,
            _ => return false,
        };
        let allowed = signed.msg.with_extension("allowed_signers");
        if fs::write(&allowed, format!("{principal} {key}\n")).is_err() {
            return false;
        }
        let mut cmd = Command::new("/usr/bin/ssh-keygen");
        cmd.args(["-Y", "verify", "-f"])
            .arg(allowed)
            .args(["-I", principal, "-n", namespace, "-s"])
            .arg(&signed.signature);
        run(cmd, Some(&payload)).status.success()
    }

    pub struct Lab {
        dir: PathBuf,
        repo_signer: PathBuf,
        external_signer: PathBuf,
        repo_key: PathBuf,
        repo_new_key: PathBuf,
        fake_repo_key: PathBuf,
        witness_key: PathBuf,
        fake_witness_key: PathBuf,
        verifier_pinned_witness: String,
    }

    impl Lab {
        pub fn new() -> Self {
            let id = SERIAL.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("nordoi-r4r-quorum-{}-{id}", std::process::id()));
            fs::create_dir(&dir).expect("new R4R test-only fixture");
            let dir = fs::canonicalize(dir).expect("canonical fixture");
            let repo_signer = dir.join("repository_admin_fixture");
            let external_signer = dir.join("separate_witness_fixture");
            fs::create_dir(&repo_signer).unwrap();
            fs::create_dir(&external_signer).unwrap();
            let repo_key = repo_signer.join("publisher");
            let repo_new_key = repo_signer.join("publisher_next");
            let fake_repo_key = repo_signer.join("malicious_publisher");
            let witness_key = external_signer.join("witness");
            let fake_witness_key = repo_signer.join("malicious_witness");
            for (key, comment) in [
                (&repo_key, "r4r-repo-publisher"),
                (&repo_new_key, "r4r-publisher-next"),
                (&fake_repo_key, "r4r-compromised-repo-key"),
                (&witness_key, "r4r-independent-fixture-witness"),
                (&fake_witness_key, "r4r-forged-external-witness"),
            ] {
                let mut cmd = Command::new("/usr/bin/ssh-keygen");
                cmd.args(["-q", "-t", "ed25519", "-N", "", "-C", comment, "-f"])
                    .arg(key);
                assert!(
                    run(cmd, None).status.success(),
                    "generate disposable fixture-only Ed25519 key"
                );
            }
            let verifier_pinned_witness = pubkey(&witness_key);
            Self {
                dir,
                repo_signer,
                external_signer,
                repo_key,
                repo_new_key,
                fake_repo_key,
                witness_key,
                fake_witness_key,
                verifier_pinned_witness,
            }
        }

        pub fn version(&self, epoch: u64, seq: u64) -> Release {
            Release {
                cursor: Cursor { epoch, seq },
                evidence_commit: EVIDENCE_COMMIT.to_owned(),
            }
        }

        pub fn publisher_key(&self, next: bool) -> String {
            pubkey(if next {
                &self.repo_new_key
            } else {
                &self.repo_key
            })
        }

        pub fn signer_public(&self) -> String {
            self.verifier_pinned_witness.clone()
        }

        pub fn publish(&self, name: &str, release: &Release, signer: usize) -> Signed {
            assert!(name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
            let path = self.repo_signer.join(format!("{name}.release"));
            fs::write(&path, release.encode()).unwrap();
            let key = match signer {
                0 => &self.repo_key,
                1 => &self.repo_new_key,
                2 => &self.fake_repo_key,
                _ => panic!("invalid research signer"),
            };
            sign(&path, key, RELEASE_NS)
        }

        pub fn attest(
            &self,
            name: &str,
            release: &Release,
            approved_publisher: &str,
            forged: bool,
            namespace: &str,
        ) -> Signed {
            assert!(name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
            let path = self.external_signer.join(format!("{name}.checkpoint"));
            fs::write(
                &path,
                IndependentHead {
                    release: release.clone(),
                    approved_publisher: approved_publisher.to_owned(),
                }
                .encode(),
            )
            .unwrap();
            sign(
                &path,
                if forged {
                    &self.fake_witness_key
                } else {
                    &self.witness_key
                },
                namespace,
            )
        }

        pub fn install_external_head(&self, signed: &Signed) {
            let file = self.external_signer.join("active_external");
            let sig = self.external_signer.join("active_external.sig");
            fs::copy(&signed.msg, file).unwrap();
            fs::copy(&signed.signature, sig).unwrap();
        }

        fn external_now(&self) -> Option<Signed> {
            let msg = self.external_signer.join("active_external");
            let signature = self.external_signer.join("active_external.sig");
            (msg.is_file() && signature.is_file()).then_some(Signed { msg, signature })
        }

        pub fn verify_admission(
            &self,
            signed_release: &Signed,
            local: &[Cursor; 2],
        ) -> Result<Cursor, &'static str> {
            // The verifier trusts its *pre-enrolled* external public key,
            // never a public key returned by the repository admin.
            let external = self.external_now().ok_or("EXTERNAL_WITNESS_REQUIRED")?;
            if !verify(
                &external,
                &self.verifier_pinned_witness,
                WITNESS_PRINCIPAL,
                WITNESS_NS,
            ) {
                return Err("EXTERNAL_SIGNATURE_NOT_TRUSTED");
            }
            let head = IndependentHead::parse(
                &fs::read_to_string(&external.msg).map_err(|_| "EXTERNAL_HEAD_UNREADABLE")?,
            )
            .ok_or("EXTERNAL_HEAD_NONCANONICAL")?;
            if head.release.evidence_commit != EVIDENCE_COMMIT {
                return Err("EXTERNAL_EVIDENCE_COMMIT_NOT_PINNED");
            }
            if local[0] != local[1] {
                return Err("LOCAL_COPIES_DIVERGED");
            }
            if local[0] != head.release.cursor {
                return Err("LOCAL_STATE_NOT_LATEST");
            }
            if !verify(
                signed_release,
                &head.approved_publisher,
                RELEASE_PRINCIPAL,
                RELEASE_NS,
            ) {
                return Err("REPOSITORY_RELEASE_SIGNER_NOT_APPROVED");
            }
            let requested = Release::parse(
                &fs::read_to_string(&signed_release.msg).map_err(|_| "RELEASE_UNREADABLE")?,
            )
            .ok_or("RELEASE_NONCANONICAL")?;
            if requested != head.release {
                return Err("RELEASE_NOT_CURRENTLY_WITNESSED");
            }
            Ok(requested.cursor)
        }

        pub fn remove_current(&self) {
            let (msg, sig) = (
                self.external_signer.join("active_external"),
                self.external_signer.join("active_external.sig"),
            );
            let _ = fs::remove_file(msg);
            let _ = fs::remove_file(sig);
        }

        pub fn corrupt_current(&self) {
            let p = self.external_signer.join("active_external");
            let mut v = fs::read(&p).unwrap();
            v.push(b'!');
            fs::write(p, v).unwrap();
        }

        pub fn prove_real_r4p_github_ref(&self) {
            let mut cmd = Command::new("/usr/bin/git");
            cmd.env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_TERMINAL_PROMPT", "0")
                .args(["ls-remote", GITHUB_URL, EVIDENCE_BRANCH]);
            let out = run(cmd, None);
            assert!(
                out.status.success(),
                "actual pinned R4P GitHub ref reachable"
            );
            let expected = format!("{EVIDENCE_COMMIT}\t{EVIDENCE_BRANCH}\n");
            assert_eq!(
                String::from_utf8(out.stdout).unwrap(),
                expected,
                "GitHub evidence ref must not be rewritten"
            );
        }

        pub fn restore_both_local_to_old(&self) -> [Cursor; 2] {
            // Adversary model: local write authority + repo signer, but
            // NO access to external signer private key or its active head.
            [Cursor { epoch: 1, seq: 1 }; 2]
        }
    }

    impl Drop for Lab {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("remove only disposable R4R fixture");
        }
    }

    #[test]
    fn native_live_github_ref_and_distinct_ed25519_authority_keys() {
        let lab = Lab::new();
        lab.prove_real_r4p_github_ref();
        assert_ne!(lab.publisher_key(false), lab.signer_public());
        assert_ne!(lab.publisher_key(true), lab.publisher_key(false));
        let v2 = lab.version(2, 2);
        let a2 = lab.publish("approved-a2", &v2, 0);
        let b2 = lab.attest(
            "independent-b2",
            &v2,
            &lab.publisher_key(false),
            false,
            WITNESS_NS,
        );
        lab.install_external_head(&b2);
        assert_eq!(lab.verify_admission(&a2, &[v2.cursor; 2]), Ok(v2.cursor));
    }

    #[test]
    fn native_compromised_repo_and_both_local_cursors_restored_fail_quorum() {
        let lab = Lab::new();
        let v2 = lab.version(2, 2);
        let v1 = lab.version(1, 1);
        let b2 = lab.attest("b2", &v2, &lab.publisher_key(false), false, WITNESS_NS);
        lab.install_external_head(&b2);
        let a1 = lab.publish("attacker-a1", &v1, 0);
        assert_eq!(
            lab.verify_admission(&a1, &lab.restore_both_local_to_old()),
            Err("LOCAL_STATE_NOT_LATEST")
        );
        let a2 = lab.publish("good-a2", &v2, 0);
        assert_eq!(
            lab.verify_admission(&a2, &lab.restore_both_local_to_old()),
            Err("LOCAL_STATE_NOT_LATEST")
        );
    }

    #[test]
    fn native_repo_admin_can_sign_fictitious_future_epoch_but_cannot_approve_it() {
        let lab = Lab::new();
        let v2 = lab.version(2, 2);
        let v3 = lab.version(3, 3);
        let b2 = lab.attest("b2", &v2, &lab.publisher_key(false), false, WITNESS_NS);
        lab.install_external_head(&b2);
        let a3 = lab.publish("malicious-a3", &v3, 0);
        assert_eq!(
            lab.verify_admission(&a3, &[v2.cursor; 2]),
            Err("RELEASE_NOT_CURRENTLY_WITNESSED")
        );
    }

    #[test]
    fn native_repo_signer_substitution_is_not_externally_authorized() {
        let lab = Lab::new();
        let v2 = lab.version(2, 2);
        let b2 = lab.attest("b2", &v2, &lab.publisher_key(false), false, WITNESS_NS);
        lab.install_external_head(&b2);
        let forged = lab.publish("malicious-signer", &v2, 2);
        assert_eq!(
            lab.verify_admission(&forged, &[v2.cursor; 2]),
            Err("REPOSITORY_RELEASE_SIGNER_NOT_APPROVED")
        );
    }

    #[test]
    fn native_fake_external_signer_and_wrong_namespace_are_rejected() {
        let lab = Lab::new();
        let v2 = lab.version(2, 2);
        let a2 = lab.publish("a2", &v2, 0);
        let forged = lab.attest(
            "forged-b2",
            &v2,
            &lab.publisher_key(false),
            true,
            WITNESS_NS,
        );
        lab.install_external_head(&forged);
        assert_eq!(
            lab.verify_admission(&a2, &[v2.cursor; 2]),
            Err("EXTERNAL_SIGNATURE_NOT_TRUSTED")
        );
        let wrong = lab.attest(
            "wrong-domain",
            &v2,
            &lab.publisher_key(false),
            false,
            RELEASE_NS,
        );
        lab.install_external_head(&wrong);
        assert_eq!(
            lab.verify_admission(&a2, &[v2.cursor; 2]),
            Err("EXTERNAL_SIGNATURE_NOT_TRUSTED")
        );
    }

    #[test]
    fn native_missing_corrupted_or_stale_external_head_never_falls_back_to_github() {
        let lab = Lab::new();
        let v2 = lab.version(2, 2);
        let a2 = lab.publish("a2", &v2, 0);
        assert_eq!(
            lab.verify_admission(&a2, &[v2.cursor; 2]),
            Err("EXTERNAL_WITNESS_REQUIRED")
        );
        let b2 = lab.attest("b2", &v2, &lab.publisher_key(false), false, WITNESS_NS);
        lab.install_external_head(&b2);
        lab.corrupt_current();
        assert_eq!(
            lab.verify_admission(&a2, &[v2.cursor; 2]),
            Err("EXTERNAL_SIGNATURE_NOT_TRUSTED")
        );
        lab.remove_current();
        assert_eq!(
            lab.verify_admission(&a2, &[v2.cursor; 2]),
            Err("EXTERNAL_WITNESS_REQUIRED")
        );
    }

    #[test]
    fn native_witness_endorsed_publisher_rotation_revokes_old_key() {
        let lab = Lab::new();
        let v3 = lab.version(3, 3);
        let b3 = lab.attest("b3", &v3, &lab.publisher_key(true), false, WITNESS_NS);
        lab.install_external_head(&b3);
        let approved_next = lab.publish("approved-next", &v3, 1);
        assert_eq!(
            lab.verify_admission(&approved_next, &[v3.cursor; 2]),
            Ok(v3.cursor)
        );
        let former_key = lab.publish("formerly-allowed", &v3, 0);
        assert_eq!(
            lab.verify_admission(&former_key, &[v3.cursor; 2]),
            Err("REPOSITORY_RELEASE_SIGNER_NOT_APPROVED")
        );
    }

    #[test]
    fn native_two_local_copies_must_match_latest_witness_head() {
        let lab = Lab::new();
        let v2 = lab.version(2, 2);
        let b2 = lab.attest("b2", &v2, &lab.publisher_key(false), false, WITNESS_NS);
        lab.install_external_head(&b2);
        let a2 = lab.publish("a2", &v2, 0);
        assert_eq!(
            lab.verify_admission(&a2, &[v2.cursor, Cursor { epoch: 1, seq: 1 }]),
            Err("LOCAL_COPIES_DIVERGED")
        );
    }

    #[test]
    fn native_if_second_admin_current_head_is_restored_too_rollback_is_possible() {
        let lab = Lab::new();
        let v2 = lab.version(2, 2);
        let b2 = lab.attest("b2", &v2, &lab.publisher_key(false), false, WITNESS_NS);
        lab.install_external_head(&b2);
        let a2 = lab.publish("a2", &v2, 0);
        assert_eq!(lab.verify_admission(&a2, &[v2.cursor; 2]), Ok(v2.cursor));
        let v1 = lab.version(1, 1);
        let b1 = lab.attest("old-b1", &v1, &lab.publisher_key(false), false, WITNESS_NS);
        let a1 = lab.publish("old-a1", &v1, 0);
        // Negative finding: if the independent checkpoint authority's
        // *current signed view* is rolled back, static old valid signatures
        // provide NO hardware-protected monotonicity guarantee.
        lab.install_external_head(&b1);
        assert_eq!(lab.verify_admission(&a1, &[v1.cursor; 2]), Ok(v1.cursor));
    }

    // R4R: fixed, public-only R4P authenticated remote floor.
    // A current B SSHSIG response is insufficient if B's current state is
    // itself restored: the independent Git-hosted floor must also agree.
    const R4R_REMOTE_URL: &str = "https://github.com/eddarsglobal/nordoi.git";
    const R4R_REMOTE_REF: &str = "refs/heads/evidence/g03-r4p-external-signed-checkpoint-20261109";
    const R4R_REMOTE_HEAD: &str = "ff8abe01cc1a527dd84b463dec1736d20a5391a4";
    const R4R_REMOTE_PUBLIC: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIE1i2sBt6ulBFM32PSg6dUeMGG1EBAj3URQ0XUaskrAx";
    const R4R_REMOTE_NS: &str = "nordoi-r4p-external-witness@eddarsglobal";
    const R4R_REMOTE_ID: &str = "nordoi-r4p-remote";
    const R4R_FRESH_NS: &str = "nordoi-r4r-freshness@eddarsglobal";
    const R4R_REMOTE_DIR: &str = "research/r4p_external_witness/";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct AuthenticatedFloor {
        latest: Cursor,
    }

    fn r4r_git(cmd: &mut Command) -> Result<Output, &'static str> {
        cmd.env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ASKPASS", "/usr/bin/false");
        let out = cmd.output().map_err(|_| "REMOTE_GIT_UNAVAILABLE")?;
        if !out.status.success() {
            return Err("REMOTE_GIT_NOT_VERIFIED");
        }
        Ok(out)
    }

    fn r4r_read(repo: &Path, name: &str) -> Result<Vec<u8>, &'static str> {
        let mut cmd = Command::new("/usr/bin/git");
        cmd.arg("-C")
            .arg(repo)
            .arg("show")
            .arg(format!("FETCH_HEAD:{R4R_REMOTE_DIR}{name}"));
        let result = r4r_git(&mut cmd)?;
        if result.stdout.is_empty() || result.stdout.len() > 4096 {
            return Err("REMOTE_FILE_BOUNDS");
        }
        Ok(result.stdout)
    }

    fn r4r_live_floor(lab: &Lab) -> Result<AuthenticatedFloor, &'static str> {
        // No cached ref or network response: actual HTTPS ls-remote and
        // temporary Git fetch are repeated by the post-rollback test.
        let mut ls = Command::new("/usr/bin/git");
        ls.args(["ls-remote", R4R_REMOTE_URL, R4R_REMOTE_REF]);
        let remote = r4r_git(&mut ls)?;
        if remote.stdout != format!("{R4R_REMOTE_HEAD}\t{R4R_REMOTE_REF}\n").as_bytes() {
            return Err("REMOTE_REF_CHANGED");
        }
        let seq = SERIAL.fetch_add(1, Ordering::Relaxed);
        let repo = lab.dir.join(format!("r4r-remote-floor-{seq}"));
        fs::create_dir(&repo).map_err(|_| "REMOTE_CLONE_INIT")?;
        let mut init = Command::new("/usr/bin/git");
        init.args(["init", "-q"]).arg(&repo);
        r4r_git(&mut init)?;
        let mut fetch = Command::new("/usr/bin/git");
        fetch.arg("-C").arg(&repo).args([
            "fetch",
            "--depth=1",
            "--no-tags",
            R4R_REMOTE_URL,
            R4R_REMOTE_REF,
        ]);
        r4r_git(&mut fetch)?;
        let mut rev = Command::new("/usr/bin/git");
        rev.arg("-C").arg(&repo).args(["rev-parse", "FETCH_HEAD"]);
        let observed = r4r_git(&mut rev)?;
        if observed.stdout != format!("{R4R_REMOTE_HEAD}\n").as_bytes() {
            return Err("REMOTE_FETCHED_HEAD_CHANGED");
        }
        let actual_root = r4r_read(&repo, "root_public.txt")?;
        if actual_root != format!("{R4R_REMOTE_PUBLIC}\n").as_bytes() {
            return Err("REMOTE_ROOT_SUBSTITUTED");
        }
        let message = r4r_read(&repo, "witness_v2.txt")?;
        let expected =
            b"NORDOI-R4P-EXTERNAL-WITNESS-V1\nproject=eddarsglobal/nordoi\nepoch=2\nsequence=2\n";
        if message != expected {
            return Err("REMOTE_WITNESS_NONCANONICAL");
        }
        let signature = r4r_read(&repo, "witness_v2.txt.sig")?;
        let msg = repo.join("authenticated_remote_floor.msg");
        let sig = repo.join("authenticated_remote_floor.msg.sig");
        fs::write(&msg, message).map_err(|_| "REMOTE_MESSAGE_WRITE")?;
        fs::write(&sig, signature).map_err(|_| "REMOTE_SIG_WRITE")?;
        if !verify(
            &Signed {
                msg,
                signature: sig,
            },
            R4R_REMOTE_PUBLIC,
            R4R_REMOTE_ID,
            R4R_REMOTE_NS,
        ) {
            return Err("REMOTE_SIGNATURE_NOT_TRUSTED");
        }
        Ok(AuthenticatedFloor {
            latest: Cursor { epoch: 2, seq: 2 },
        })
    }

    fn r4r_nonce() -> String {
        use std::io::Read;
        let mut bytes = [0_u8; 32];
        fs::File::open("/dev/urandom")
            .expect("native random source")
            .read_exact(&mut bytes)
            .expect("full 256-bit research nonce");
        let mut nonce = String::with_capacity(64);
        for byte in bytes {
            use std::fmt::Write as _;
            write!(&mut nonce, "{byte:02x}").expect("write canonical nonce");
        }
        nonce
    }

    impl Lab {
        fn r4r_challenge_response(&self, name: &str, nonce: &str, cursor: Cursor) -> Signed {
            assert!(name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
            assert_eq!(nonce.len(), 64);
            let path = self.external_signer.join(format!("{name}.freshness"));
            fs::write(
                &path,
                format!(
                    "NORDOI-R4R-FRESHNESS-V1\nepoch={}\nseq={}\nnonce={nonce}\n",
                    cursor.epoch, cursor.seq,
                ),
            )
            .expect("ephemeral signer challenge receipt");
            sign(&path, &self.witness_key, R4R_FRESH_NS)
        }

        fn r4r_admit(
            &self,
            release: &Signed,
            local: &[Cursor; 2],
            floor: Option<&AuthenticatedFloor>,
            nonce: &str,
            response: &Signed,
        ) -> Result<Cursor, &'static str> {
            // A remote high-water proof is mandatory. No offline fallback.
            let remote = floor.ok_or("REMOTE_FLOOR_REQUIRED")?;
            let candidate = self.verify_admission(release, local)?;
            if candidate != remote.latest {
                return Err("REMOTE_HIGH_WATER_MISMATCH");
            }
            if !verify(
                response,
                &self.verifier_pinned_witness,
                WITNESS_PRINCIPAL,
                R4R_FRESH_NS,
            ) {
                return Err("FRESHNESS_SIGNATURE_INVALID");
            }
            let observed =
                fs::read_to_string(&response.msg).map_err(|_| "FRESHNESS_MESSAGE_MISSING")?;
            let wanted = format!(
                "NORDOI-R4R-FRESHNESS-V1\nepoch={}\nseq={}\nnonce={nonce}\n",
                candidate.epoch, candidate.seq,
            );
            if observed != wanted {
                return Err("FRESHNESS_NONCE_OR_STATE_MISMATCH");
            }
            Ok(candidate)
        }
    }

    #[test]
    fn native_r4r_live_signed_remote_floor_and_unique_nonce_admit_v2() {
        let lab = Lab::new();
        let current = lab.version(2, 2);
        let a2 = lab.publish("r4r-a2", &current, 0);
        let b2 = lab.attest(
            "r4r-b2",
            &current,
            &lab.publisher_key(false),
            false,
            WITNESS_NS,
        );
        lab.install_external_head(&b2);
        let floor = r4r_live_floor(&lab).expect("live signed GitHub epoch 2 floor");
        let nonce = r4r_nonce();
        let token = lab.r4r_challenge_response("r4r-fresh-accepted", &nonce, current.cursor);
        assert_eq!(
            lab.r4r_admit(&a2, &[current.cursor; 2], Some(&floor), &nonce, &token),
            Ok(current.cursor)
        );
    }

    #[test]
    fn native_r4r_restored_B_with_fresh_valid_signature_fails_remote_high_water() {
        let lab = Lab::new();
        let previous = lab.version(1, 1);
        let latest = lab.version(2, 2);
        let b2 = lab.attest(
            "r4r-b2",
            &latest,
            &lab.publisher_key(false),
            false,
            WITNESS_NS,
        );
        lab.install_external_head(&b2);
        let a2 = lab.publish("r4r-before-rollback", &latest, 0);
        assert_eq!(
            lab.verify_admission(&a2, &[latest.cursor; 2]),
            Ok(latest.cursor)
        );

        // Malicious restoration of B's current view AND the two local files.
        let b1 = lab.attest(
            "r4r-restored-b1",
            &previous,
            &lab.publisher_key(false),
            false,
            WITNESS_NS,
        );
        lab.install_external_head(&b1);
        let a1 = lab.publish("r4r-replayed-a1", &previous, 0);
        assert_eq!(
            lab.verify_admission(&a1, &[previous.cursor; 2]),
            Ok(previous.cursor),
            "R4R intentionally admits B's genuinely signed old current head"
        );
        // Even if the compromised B can sign a FRESH nonce for old state,
        // the out-of-device signed, pinned GitHub epoch-2 floor refuses it.
        let nonce = r4r_nonce();
        let freshly_signed_old =
            lab.r4r_challenge_response("r4r-fresh-old", &nonce, previous.cursor);
        let remote_after_rollback =
            r4r_live_floor(&lab).expect("new live GitHub fetch AFTER malicious restore");
        assert_eq!(
            lab.r4r_admit(
                &a1,
                &[previous.cursor; 2],
                Some(&remote_after_rollback),
                &nonce,
                &freshly_signed_old
            ),
            Err("REMOTE_HIGH_WATER_MISMATCH")
        );
    }

    #[test]
    fn native_r4r_replayed_nonce_cannot_satisfy_new_challenge() {
        let lab = Lab::new();
        let latest = lab.version(2, 2);
        let b2 = lab.attest(
            "r4r-b2",
            &latest,
            &lab.publisher_key(false),
            false,
            WITNESS_NS,
        );
        lab.install_external_head(&b2);
        let a2 = lab.publish("r4r-a2", &latest, 0);
        let nonce = r4r_nonce();
        let different_nonce = r4r_nonce();
        assert_ne!(nonce, different_nonce);
        let old_token = lab.r4r_challenge_response("r4r-replay", &nonce, latest.cursor);
        // The only valid floor object is produced by a signed+commit-pinned fetch;
        // here we use a fixed offline authenticated floor snapshot fixture for
        // narrow nonce tests, not a new network freshness claim.
        let fixed_floor = AuthenticatedFloor {
            latest: latest.cursor,
        };
        assert_eq!(
            lab.r4r_admit(
                &a2,
                &[latest.cursor; 2],
                Some(&fixed_floor),
                &different_nonce,
                &old_token
            ),
            Err("FRESHNESS_NONCE_OR_STATE_MISMATCH")
        );
    }

    #[test]
    fn native_r4r_tampered_or_wrong_domain_freshness_denied() {
        let lab = Lab::new();
        let latest = lab.version(2, 2);
        let b2 = lab.attest(
            "r4r-b2",
            &latest,
            &lab.publisher_key(false),
            false,
            WITNESS_NS,
        );
        lab.install_external_head(&b2);
        let a2 = lab.publish("r4r-a2", &latest, 0);
        let floor = AuthenticatedFloor {
            latest: latest.cursor,
        };
        let nonce = r4r_nonce();
        let altered = lab.r4r_challenge_response("r4r-corrupt", &nonce, latest.cursor);
        fs::write(&altered.msg, b"forged").unwrap();
        assert_eq!(
            lab.r4r_admit(&a2, &[latest.cursor; 2], Some(&floor), &nonce, &altered),
            Err("FRESHNESS_SIGNATURE_INVALID")
        );
        let domain_path = lab.external_signer.join("r4r-wrong-domain.freshness");
        fs::write(
            &domain_path,
            format!("NORDOI-R4R-FRESHNESS-V1\nepoch=2\nseq=2\nnonce={nonce}\n"),
        )
        .unwrap();
        let wrong_domain = sign(&domain_path, &lab.witness_key, RELEASE_NS);
        assert_eq!(
            lab.r4r_admit(
                &a2,
                &[latest.cursor; 2],
                Some(&floor),
                &nonce,
                &wrong_domain
            ),
            Err("FRESHNESS_SIGNATURE_INVALID")
        );
    }

    #[test]
    fn native_r4r_network_unavailable_never_falls_back_to_local_quorum() {
        let lab = Lab::new();
        let latest = lab.version(2, 2);
        let b2 = lab.attest(
            "r4r-b2",
            &latest,
            &lab.publisher_key(false),
            false,
            WITNESS_NS,
        );
        lab.install_external_head(&b2);
        let a2 = lab.publish("r4r-a2", &latest, 0);
        let nonce = r4r_nonce();
        let signed = lab.r4r_challenge_response("r4r-offline", &nonce, latest.cursor);
        assert_eq!(
            lab.r4r_admit(&a2, &[latest.cursor; 2], None, &nonce, &signed),
            Err("REMOTE_FLOOR_REQUIRED")
        );
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn non_macos_does_not_claim_independent_admin_or_native_sshsig_checks() {
    assert_ne!(std::env::consts::OS, "macos");
}

#[test]
fn reference_model_still_never_grants_external_host_io() {
    let (mut model, permit) = Model::bootstrap(
        0x51,
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
