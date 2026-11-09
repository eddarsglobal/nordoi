//! G0.3 R4P — external GitHub-hosted, SSHSIG-authenticated checkpoint.
//! The remote Git commit+public signer are pinned independently of local state.
//! Research only: GitHub repo admin is NOT an independent governance authority.
#![allow(dead_code)]
#[path = "../src/resource_task_r02.rs"]
mod reference;
use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod native {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        OnceLock,
    };

    const URL: &str = "https://github.com/eddarsglobal/nordoi.git";
    const BRANCH: &str = "refs/heads/evidence/g03-r4p-external-signed-checkpoint-20261109";
    const EXPECTED_HEAD: &str = "ff8abe01cc1a527dd84b463dec1736d20a5391a4";
    const ROOT: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIE1i2sBt6ulBFM32PSg6dUeMGG1EBAj3URQ0XUaskrAx";
    const PRINCIPAL: &str = "nordoi-r4p-remote";
    const NAMESPACE: &str = "nordoi-r4p-external-witness@eddarsglobal";
    const DIR: &str = "research/r4p_external_witness/";
    static ID: AtomicU64 = AtomicU64::new(1);
    static EVIDENCE: OnceLock<Result<Remote, String>> = OnceLock::new();

    #[derive(Debug, Clone)]
    struct Remote {
        head: String,
        public: String,
        new_msg: Vec<u8>,
        new_sig: Vec<u8>,
        old_msg: Vec<u8>,
        old_sig: Vec<u8>,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Cursor {
        epoch: u64,
        seq: u64,
    }
    fn parse_cursor(bytes: &[u8]) -> Result<Cursor, &'static str> {
        let raw = std::str::from_utf8(bytes).map_err(|_| "INVALID_UTF8")?;
        let lines: Vec<_> = raw.lines().collect();
        if lines.len() != 4
            || lines[0] != "NORDOI-R4P-EXTERNAL-WITNESS-V1"
            || lines[1] != "project=eddarsglobal/nordoi"
        {
            return Err("NONCANONICAL_REMOTE_WITNESS");
        }
        let epoch = lines[2]
            .strip_prefix("epoch=")
            .ok_or("MISSING_EPOCH")?
            .parse::<u64>()
            .map_err(|_| "INVALID_EPOCH")?;
        let seq = lines[3]
            .strip_prefix("sequence=")
            .ok_or("MISSING_SEQ")?
            .parse::<u64>()
            .map_err(|_| "INVALID_SEQ")?;
        let expected=format!(
            "NORDOI-R4P-EXTERNAL-WITNESS-V1\nproject=eddarsglobal/nordoi\nepoch={epoch}\nsequence={seq}\n"
        );
        if raw != expected || epoch == 0 || seq == 0 {
            return Err("NONCANONICAL_REMOTE_WITNESS");
        }
        Ok(Cursor { epoch, seq })
    }
    fn git(cmd: &mut Command) -> Result<Output, String> {
        cmd.env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ASKPASS", "/usr/bin/false");
        let r = cmd.output().map_err(|e| format!("NO_GIT_EXECUTION:{e}"))?;
        if !r.status.success() {
            return Err(format!(
                "GIT_FAILED:{:?}:{}",
                r.status.code(),
                String::from_utf8_lossy(&r.stderr)
                    .chars()
                    .take(300)
                    .collect::<String>()
            ));
        }
        Ok(r)
    }
    struct Temporary(PathBuf);
    impl Temporary {
        fn new(label: &str) -> Self {
            let n = ID.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("nordoi-r4p-{label}-{}-{n}", std::process::id()));
            fs::create_dir(&dir).expect("fresh R4P temporary directory");
            Self(fs::canonicalize(dir).expect("canonical temporary fixture"))
        }
    }
    impl Drop for Temporary {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove only research fixture");
        }
    }
    fn read_git_file(repo: &Path, file: &str) -> Result<Vec<u8>, String> {
        let mut cmd = Command::new("/usr/bin/git");
        cmd.arg("-C")
            .arg(repo)
            .arg("show")
            .arg(format!("FETCH_HEAD:{DIR}{file}"));
        let out = git(&mut cmd)?;
        if out.stdout.is_empty() || out.stdout.len() > 4096 {
            return Err("UNBOUNDED_OR_MISSING_REMOTE_FILE".into());
        }
        Ok(out.stdout)
    }
    fn fetch_live() -> Result<Remote, String> {
        let tmp = Temporary::new("fetch");
        let mut lookup = Command::new("/usr/bin/git");
        lookup.args(["ls-remote", URL, BRANCH]);
        let output = git(&mut lookup)?;
        let listing = String::from_utf8(output.stdout).map_err(|_| "BAD_REMOTE_HEAD_UTF8")?;
        let lines: Vec<&str> = listing.lines().collect();
        if lines.len() != 1 {
            return Err("REMOTE_REF_NOT_UNIQUE".into());
        }
        let (sha, refname) = lines[0].split_once('\t').ok_or("BAD_REMOTE_REF_FORMAT")?;
        if refname != BRANCH || sha != EXPECTED_HEAD {
            return Err("REMOTE_REF_NOT_PINNED".into());
        }
        let mut init = Command::new("/usr/bin/git");
        init.args(["init", "-q"]).arg(&tmp.0);
        git(&mut init)?;
        let mut fetch = Command::new("/usr/bin/git");
        fetch
            .arg("-C")
            .arg(&tmp.0)
            .args(["fetch", "--depth=1", "--no-tags", URL, BRANCH]);
        git(&mut fetch)?;
        let mut verify = Command::new("/usr/bin/git");
        verify
            .arg("-C")
            .arg(&tmp.0)
            .args(["rev-parse", "FETCH_HEAD"]);
        let after = git(&mut verify)?;
        if String::from_utf8_lossy(&after.stdout).trim() != EXPECTED_HEAD {
            return Err("REMOTE_CHANGED_DURING_FETCH".into());
        }
        let remote = Remote {
            head: sha.to_owned(),
            public: String::from_utf8(read_git_file(&tmp.0, "root_public.txt")?)
                .map_err(|_| "ROOT_PUBLIC_NOT_UTF8")?,
            old_msg: read_git_file(&tmp.0, "witness_v1.txt")?,
            old_sig: read_git_file(&tmp.0, "witness_v1.txt.sig")?,
            new_msg: read_git_file(&tmp.0, "witness_v2.txt")?,
            new_sig: read_git_file(&tmp.0, "witness_v2.txt.sig")?,
        };
        if remote.public != format!("{ROOT}\n") {
            return Err("REMOTE_ROOT_KEY_NOT_PINNED".into());
        }
        if parse_cursor(&remote.old_msg).map_err(|e| e.to_string())?
            != (Cursor { epoch: 1, seq: 1 })
            || parse_cursor(&remote.new_msg).map_err(|e| e.to_string())?
                != (Cursor { epoch: 2, seq: 2 })
        {
            return Err("REMOTE_CHECKPOINT_SEQUENCES_NOT_PINNED".into());
        }
        Ok(remote)
    }
    fn evidence() -> &'static Remote {
        EVIDENCE
            .get_or_init(fetch_live)
            .as_ref()
            .unwrap_or_else(|e| panic!("REAL_EXTERNAL_GITHUB_WITNESS_UNAVAILABLE:{e}"))
    }
    fn verify_signature(msg: &[u8], sig: &[u8], root: &str, namespace: &str) -> bool {
        let tmp = Temporary::new("sshsig");
        let allowed = tmp.0.join("allowed_signers");
        let signature = tmp.0.join("checkpoint.sig");
        if fs::write(&allowed, format!("{PRINCIPAL} {root}\n")).is_err()
            || fs::write(&signature, sig).is_err()
        {
            return false;
        }
        let mut cmd = Command::new("/usr/bin/ssh-keygen");
        cmd.args(["-Y", "verify", "-f"])
            .arg(&allowed)
            .args(["-I", PRINCIPAL, "-n", namespace, "-s"])
            .arg(&signature)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(_) => return false,
        };
        if child.stdin.take().unwrap().write_all(msg).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        child
            .wait_with_output()
            .is_ok_and(|out| out.status.success())
    }
    fn verified_remote(remote: &Remote) -> Result<Cursor, &'static str> {
        if remote.head != EXPECTED_HEAD {
            return Err("REMOTE_COMMIT_MISMATCH");
        }
        if remote.public != format!("{ROOT}\n") {
            return Err("REMOTE_ROOT_SUBSTITUTED");
        }
        if !verify_signature(&remote.new_msg, &remote.new_sig, ROOT, NAMESPACE) {
            return Err("REMOTE_SIGNATURE_NOT_AUTHENTICATED");
        }
        let head = parse_cursor(&remote.new_msg)?;
        if head != (Cursor { epoch: 2, seq: 2 }) {
            return Err("REMOTE_ADVANCE_NOT_PINNED");
        }
        Ok(head)
    }
    fn admit(
        remote: Option<&Remote>,
        local: &[Option<Cursor>; 2],
        request: Cursor,
    ) -> Result<(), &'static str> {
        let remote = remote.ok_or("EXTERNAL_WITNESS_REQUIRED")?;
        let latest = verified_remote(remote)?;
        let a = local[0].ok_or("LOCAL_CURSOR_MISSING")?;
        let b = local[1].ok_or("LOCAL_WITNESS_MISSING")?;
        if a != b {
            return Err("LOCAL_WITNESSES_DIVERGED");
        }
        if a != latest {
            return Err("REMOTE_AND_LOCAL_HEAD_DIVERGED");
        }
        if request != latest {
            return Err("REQUEST_NOT_REMOTE_APPROVED");
        }
        Ok(())
    }
    fn write_both(cur: Cursor) -> Temporary {
        let dir = Temporary::new("cursor");
        for file in ["local_state", "secondary_state"] {
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(dir.0.join(file))
                .expect("test-only cursor");
            f.write_all(format!("epoch={}\nsequence={}\n", cur.epoch, cur.seq).as_bytes())
                .expect("write temporary cursor");
            f.sync_all().expect("sync test-only cursor");
        }
        dir
    }
    fn read_both(path: &Path) -> [Option<Cursor>; 2] {
        let mut states = [None, None];
        for (i, name) in ["local_state", "secondary_state"].iter().enumerate() {
            let s = match fs::read_to_string(path.join(name)) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let v: Vec<_> = s.lines().collect();
            if v.len() != 2 {
                continue;
            }
            if let (Some(a), Some(b)) =
                (v[0].strip_prefix("epoch="), v[1].strip_prefix("sequence="))
            {
                if let (Ok(epoch), Ok(seq)) = (a.parse::<u64>(), b.parse::<u64>()) {
                    states[i] = Some(Cursor { epoch, seq });
                }
            }
        }
        states
    }
    #[test]
    fn macos_remote_head_and_pinned_ed25519_signer_verified_from_github() {
        let r = evidence();
        assert_eq!(r.head, EXPECTED_HEAD);
        assert_eq!(verified_remote(r), Ok(Cursor { epoch: 2, seq: 2 }));
        assert!(
            verify_signature(&r.old_msg, &r.old_sig, ROOT, NAMESPACE),
            "the old replay is authentic despite being obsolete"
        );
    }
    #[test]
    fn macos_genuine_external_witness_detects_both_local_copies_restored() {
        let remote = evidence();
        let v2 = Cursor { epoch: 2, seq: 2 };
        let v1 = Cursor { epoch: 1, seq: 1 };
        let temp = write_both(v2);
        assert_eq!(admit(Some(remote), &read_both(&temp.0), v2), Ok(()));
        // Attacker restores BOTH writable states: R4O demonstrably allowed
        // the formerly signed old version. The live remote V2 is unchanged.
        for file in ["local_state", "secondary_state"] {
            fs::write(temp.0.join(file), b"epoch=1\nsequence=1\n").unwrap();
        }
        // Recontact the actual GitHub branch AFTER the malicious restore.
        // Never infer the remote's current head only from a process cache.
        let reloaded = fetch_live().expect("fresh external GitHub witness after local restore");
        assert_eq!(verified_remote(&reloaded), Ok(v2));
        assert_eq!(
            admit(Some(&reloaded), &read_both(&temp.0), v1),
            Err("REMOTE_AND_LOCAL_HEAD_DIVERGED")
        );
        assert_eq!(
            admit(Some(&reloaded), &read_both(&temp.0), v2),
            Err("REMOTE_AND_LOCAL_HEAD_DIVERGED")
        );
    }
    #[test]
    fn macos_rollback_of_only_one_copy_fails_without_repair() {
        let r = evidence();
        let cur = Cursor { epoch: 2, seq: 2 };
        let dir = write_both(cur);
        fs::write(dir.0.join("secondary_state"), b"epoch=1\nsequence=1\n").unwrap();
        assert_eq!(
            admit(Some(r), &read_both(&dir.0), cur),
            Err("LOCAL_WITNESSES_DIVERGED")
        );
    }
    #[test]
    fn macos_genuinely_signed_v1_is_stale_even_if_both_local_states_are_v1() {
        let r = evidence();
        assert!(verify_signature(&r.old_msg, &r.old_sig, ROOT, NAMESPACE));
        let old = parse_cursor(&r.old_msg).unwrap();
        let dir = write_both(old);
        assert_eq!(
            admit(Some(r), &read_both(&dir.0), old),
            Err("REMOTE_AND_LOCAL_HEAD_DIVERGED")
        );
    }
    #[test]
    fn macos_remote_signer_and_signature_tampering_denied() {
        let r = evidence().clone();
        let mut fake = r.clone();
        fake.public = "ssh-ed25519 AAAA\n".into();
        assert_eq!(verified_remote(&fake), Err("REMOTE_ROOT_SUBSTITUTED"));
        let mut corrupted = r.clone();
        corrupted.new_msg.push(b'\n');
        assert_eq!(
            verified_remote(&corrupted),
            Err("REMOTE_SIGNATURE_NOT_AUTHENTICATED")
        );
        let mut fake_signature = r;
        fake_signature.new_sig[30] ^= 1;
        assert_eq!(
            verified_remote(&fake_signature),
            Err("REMOTE_SIGNATURE_NOT_AUTHENTICATED")
        );
    }
    #[test]
    fn macos_cross_namespace_fails_even_with_correct_ed25519_root() {
        let r = evidence();
        assert!(!verify_signature(
            &r.new_msg,
            &r.new_sig,
            ROOT,
            "nordoi-r4p-incorrect-namespace"
        ));
    }
    #[test]
    fn macos_remote_branch_rewrite_or_wrong_head_fails_closed() {
        let mut r = evidence().clone();
        r.head = "0000000000000000000000000000000000000000".into();
        assert_eq!(verified_remote(&r), Err("REMOTE_COMMIT_MISMATCH"));
    }
    #[test]
    fn macos_no_network_or_missing_remote_witness_is_never_admitted() {
        let v2 = Cursor { epoch: 2, seq: 2 };
        let dir = write_both(v2);
        assert_eq!(
            admit(None, &read_both(&dir.0), v2),
            Err("EXTERNAL_WITNESS_REQUIRED")
        );
        let r = evidence();
        assert_eq!(
            admit(Some(r), &[None, Some(v2)], v2),
            Err("LOCAL_CURSOR_MISSING")
        );
        assert_eq!(
            admit(Some(r), &[Some(v2), None], v2),
            Err("LOCAL_WITNESS_MISSING")
        );
    }
    #[test]
    fn macos_local_state_ahead_of_remote_is_not_authorized() {
        let r = evidence();
        let ahead = Cursor { epoch: 3, seq: 3 };
        let dir = write_both(ahead);
        assert_eq!(
            admit(Some(r), &read_both(&dir.0), ahead),
            Err("REMOTE_AND_LOCAL_HEAD_DIVERGED")
        );
    }
}
#[cfg(not(target_os = "macos"))]
#[test]
fn portable_target_does_not_claim_live_external_macos_antirollback() {
    assert_ne!(std::env::consts::OS, "macos");
}
#[test]
fn reference_model_external_io_remains_unsupported() {
    let (mut model, permit) = Model::bootstrap(
        0x50,
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
