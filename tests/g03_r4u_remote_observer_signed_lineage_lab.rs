//! G0.3 R4U: actual GitHub-hosted signed multi-epoch observer log, bounded
//! replay/fork experiments. SAME account owns both research signer roles.
//! No production guardian, independent administration, or native host grant.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;
use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod native {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Command, Output, Stdio};
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        OnceLock,
    };

    const URL: &str = "https://github.com/eddarsglobal/nordoi.git";
    const REF: &str = "refs/heads/evidence/g03-r4u-signed-observer-ledger-20261109";
    const COMMIT: &str = "15adf075050a233705b3bad6eae62b6f9e6f946d";
    const PACK_PATH: &str = "research/r4u_signed_observer_ledger/evidence_pack.txt";
    const PACK_SHA: &str = "692809e55aed8ad188208e04653d3bf4fdb9945aa7fa661db6ee22fe8100400d";
    const GUARDIAN: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHfClkGUTZWf18i0YPkUCj5R+jSwE2eF3sGASALCLvL6";
    const OBSERVER: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGRMxyrMD2G9xpR3YPmCgtp9cinas6Rxz7oiuYMoS91f";
    const GUARDIAN_NS: &str = "nordoi-r4u-guardian-log@eddarsglobal";
    const OBSERVER_NS: &str = "nordoi-r4u-observer-highwater@eddarsglobal";
    const GUARDIAN_PRINCIPAL: &str = "nordoi-r4u-guardian";
    const OBSERVER_PRINCIPAL: &str = "nordoi-r4u-observer";
    const GENESIS_SHA: &str = "f69ab4ba094c555b0064e2128e1a90458abb82a2ea652e1a149fa217069e7aa8";
    const E3_SHA: &str = "11e000dc42cf66554c46d68a385959d0098bc516925b6bf33afa5bbb18f5b3c8";
    const E4_SHA: &str = "65b71123804659cea8b334c783f51d7e63ad3c8283057c22e432a8a54f6c9d1c";
    const FORK_SHA: &str = "989377b80da386e7344aefa514614853da8e832b1bec826681dce96ffa218e0a";
    static SERIAL: AtomicU64 = AtomicU64::new(1);
    static REMOTE: OnceLock<Result<Evidence, String>> = OnceLock::new();

    #[derive(Clone, Debug)]
    struct Evidence {
        head: String,
        pack: String,
    }

    struct Temporary(PathBuf);
    impl Temporary {
        fn new(label: &str) -> Self {
            let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
            let name = format!("nordoi-r4u-{label}-{}-{serial}", std::process::id());
            let dir = std::env::temp_dir().join(name);
            fs::create_dir(&dir).expect("create fresh R4U disposable fixture");
            Self(fs::canonicalize(dir).expect("absolute R4U fixture path"))
        }
    }
    impl Drop for Temporary {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("clean only the new R4U fixture");
        }
    }

    fn command(mut cmd: Command, payload: Option<&[u8]>) -> Result<Output, String> {
        if payload.is_some() {
            cmd.stdin(Stdio::piped());
        }
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("HOST_EXEC_UNAVAILABLE:{e}"))?;
        if let Some(bytes) = payload {
            child
                .stdin
                .take()
                .ok_or("NO_STDIN")?
                .write_all(bytes)
                .map_err(|_| "HOST_STDIN_FAILED")?;
        }
        child
            .wait_with_output()
            .map_err(|_| "HOST_PROCESS_FAILED".into())
    }
    fn git(cmd: &mut Command) -> Result<Output, String> {
        cmd.env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ASKPASS", "/usr/bin/false");
        let result = cmd.output().map_err(|_| "GIT_EXEC_UNAVAILABLE")?;
        if !result.status.success() {
            return Err(format!("GIT_FAIL:{:?}", result.status.code()));
        }
        Ok(result)
    }
    fn sha(bytes: &[u8]) -> String {
        let mut cmd = Command::new("/usr/bin/shasum");
        cmd.args(["-a", "256"]);
        let output = command(cmd, Some(bytes)).expect("native SHA256 available");
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .unwrap()
            .split_ascii_whitespace()
            .next()
            .unwrap()
            .to_owned()
    }

    fn fetch_live() -> Result<Evidence, String> {
        let mut lookup = Command::new("/usr/bin/git");
        lookup.args(["ls-remote", URL, REF]);
        let remote = git(&mut lookup)?;
        if remote.stdout != format!("{COMMIT}\t{REF}\n").as_bytes() {
            return Err("REMOTE_BRANCH_HEAD_CHANGED".into());
        }
        let temp = Temporary::new("fetch");
        let mut init = Command::new("/usr/bin/git");
        init.args(["init", "-q"]).arg(&temp.0);
        git(&mut init)?;
        let mut fetch = Command::new("/usr/bin/git");
        fetch
            .arg("-C")
            .arg(&temp.0)
            .args(["fetch", "--depth=1", "--no-tags", URL, REF]);
        git(&mut fetch)?;
        let mut refcheck = Command::new("/usr/bin/git");
        refcheck
            .arg("-C")
            .arg(&temp.0)
            .args(["rev-parse", "FETCH_HEAD"]);
        if git(&mut refcheck)?.stdout != format!("{COMMIT}\n").as_bytes() {
            return Err("FETCH_CHANGED_DURING_READ".into());
        }
        let mut read = Command::new("/usr/bin/git");
        read.arg("-C")
            .arg(&temp.0)
            .arg("show")
            .arg(format!("FETCH_HEAD:{PACK_PATH}"));
        let out = git(&mut read)?;
        if out.stdout.is_empty() || out.stdout.len() > 8192 {
            return Err("EXTERNAL_EVIDENCE_TOO_LARGE_OR_MISSING".into());
        }
        let pack = String::from_utf8(out.stdout).map_err(|_| "REMOTE_EVIDENCE_NON_UTF8")?;
        Ok(Evidence {
            head: COMMIT.to_owned(),
            pack,
        })
    }

    fn evidence() -> &'static Evidence {
        REMOTE
            .get_or_init(fetch_live)
            .as_ref()
            .unwrap_or_else(|err| panic!("R4U_REAL_GITHUB_WITNESS_UNAVAILABLE:{err}"))
    }

    fn section<'a>(pack: &'a str, key: &str) -> Result<&'a str, &'static str> {
        let start = format!("[{key}]\n");
        let end = format!("[/{key}]\n");
        if pack.matches(start.as_str()).count() != 1 || pack.matches(end.as_str()).count() != 1 {
            return Err("EVIDENCE_SECTION_NOT_UNIQUE");
        }
        let (_, tail) = pack
            .split_once(start.as_str())
            .ok_or("EVIDENCE_SECTION_MISSING")?;
        let (part, _) = tail
            .split_once(end.as_str())
            .ok_or("EVIDENCE_SECTION_NOT_CLOSED")?;
        if !part.ends_with('\n') || part.is_empty() {
            return Err("EVIDENCE_SECTION_NONCANONICAL");
        }
        Ok(part)
    }

    fn sshsig(
        payload: &str,
        signature: &str,
        public: &str,
        principal: &str,
        namespace: &str,
    ) -> bool {
        let temp = Temporary::new("sshsig");
        let p = temp.0.join("payload.txt");
        let s = temp.0.join("payload.sig");
        let allowed = temp.0.join("allowed_signers");
        if fs::write(&p, payload).is_err()
            || fs::write(&s, signature).is_err()
            || fs::write(&allowed, format!("{principal} {public}\n")).is_err()
        {
            return false;
        }
        let mut verify = Command::new("/usr/bin/ssh-keygen");
        verify
            .args(["-Y", "verify", "-f"])
            .arg(&allowed)
            .args(["-I", principal, "-n", namespace, "-s"])
            .arg(&s);
        command(verify, Some(payload.as_bytes())).is_ok_and(|o| o.status.success())
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Record {
        epoch: u64,
        seq: u64,
        parent: String,
        policy: String,
    }

    fn parse_record(bytes: &str) -> Result<Record, &'static str> {
        let lines: Vec<_> = bytes.lines().collect();
        if lines.len() != 6
            || lines[0] != "NORDOI-R4U-LOG-V1"
            || lines[1] != "project=eddarsglobal/nordoi"
        {
            return Err("LOG_RECORD_NONCANONICAL");
        }
        let epoch = lines[2]
            .strip_prefix("epoch=")
            .ok_or("LOG_EPOCH_MISSING")?
            .parse::<u64>()
            .map_err(|_| "LOG_EPOCH_INVALID")?;
        let seq = lines[3]
            .strip_prefix("sequence=")
            .ok_or("LOG_SEQUENCE_MISSING")?
            .parse::<u64>()
            .map_err(|_| "LOG_SEQUENCE_INVALID")?;
        let parent = lines[4]
            .strip_prefix("previous_sha256=")
            .ok_or("LOG_PARENT_MISSING")?
            .to_owned();
        let policy = lines[5]
            .strip_prefix("policy=")
            .ok_or("LOG_POLICY_MISSING")?
            .to_owned();
        let expected = format!(
            "NORDOI-R4U-LOG-V1\nproject=eddarsglobal/nordoi\nepoch={epoch}\nsequence={seq}\nprevious_sha256={parent}\npolicy={policy}\n"
        );
        if bytes != expected
            || epoch == 0
            || seq == 0
            || parent.len() != 64
            || !parent
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || !matches!(policy.as_str(), "accepted" | "revoked")
        {
            return Err("LOG_RECORD_NONCANONICAL");
        }
        Ok(Record {
            epoch,
            seq,
            parent,
            policy,
        })
    }
    fn check_link(first: &Record, first_bytes: &str, second: &Record) -> Result<(), &'static str> {
        if second.epoch != first.epoch + 1 || second.seq != first.seq + 1 {
            return Err("LOG_NONMONOTONIC");
        }
        if second.parent != sha(first_bytes.as_bytes()) {
            return Err("LOG_PREDECESSOR_MISMATCH");
        }
        Ok(())
    }

    fn accepted_remote(pack: &str) -> Result<Record, &'static str> {
        if !pack.starts_with("NORDOI-R4U-SIGNED-EVIDENCE-PACK-V1\n") {
            return Err("BAD_PACK_VERSION");
        }
        let guardian = section(pack, "guardian_public")?.trim_end();
        let observer = section(pack, "observer_public")?.trim_end();
        if guardian != GUARDIAN || observer != OBSERVER {
            return Err("EVIDENCE_SIGNER_NOT_PINNED");
        }
        let v3 = section(pack, "epoch3")?;
        let v4 = section(pack, "epoch4")?;
        let first = parse_record(v3)?;
        let latest = parse_record(v4)?;
        if first.epoch != 3
            || first.seq != 3
            || first.parent != GENESIS_SHA
            || first.policy != "accepted"
            || sha(v3.as_bytes()) != E3_SHA
        {
            return Err("GENESIS_OR_EPOCH3_INVALID");
        }
        check_link(&first, v3, &latest)?;
        if latest.epoch != 4
            || latest.seq != 4
            || latest.policy != "accepted"
            || sha(v4.as_bytes()) != E4_SHA
        {
            return Err("LATEST_REMOTE_RECORD_MISMATCH");
        }
        for (stage, contents) in [("epoch3", v3), ("epoch4", v4)] {
            let guardian_sig = section(pack, &format!("{stage}_guardian_sig"))?;
            let observer_sig = section(pack, &format!("{stage}_observer_sig"))?;
            if !sshsig(
                contents,
                guardian_sig,
                GUARDIAN,
                GUARDIAN_PRINCIPAL,
                GUARDIAN_NS,
            ) {
                return Err("GUARDIAN_SIGNATURE_UNTRUSTED");
            }
            if !sshsig(
                contents,
                observer_sig,
                OBSERVER,
                OBSERVER_PRINCIPAL,
                OBSERVER_NS,
            ) {
                return Err("OBSERVER_SIGNATURE_UNTRUSTED");
            }
        }
        let fork = section(pack, "epoch4_fork")?;
        let fork_record = parse_record(fork)?;
        let fork_sig = section(pack, "epoch4_fork_guardian_sig")?;
        if fork_record.epoch != 4
            || fork_record.seq != 4
            || fork_record.parent != E3_SHA
            || fork_record.policy != "revoked"
            || sha(fork.as_bytes()) != FORK_SHA
            || !sshsig(fork, fork_sig, GUARDIAN, GUARDIAN_PRINCIPAL, GUARDIAN_NS)
        {
            return Err("FORK_EVIDENCE_NOT_AUTHENTIC");
        }
        Ok(latest)
    }

    fn trusted_head(remote: Option<&Evidence>) -> Result<Record, &'static str> {
        let e = remote.ok_or("OFF_DEVICE_OBSERVER_REQUIRED")?;
        if e.head != COMMIT || sha(e.pack.as_bytes()) != PACK_SHA {
            return Err("EVIDENCE_COMMIT_OR_CONTENT_NOT_PINNED");
        }
        accepted_remote(&e.pack)
    }

    fn admission(
        remote: Option<&Evidence>,
        local: [Record; 2],
        requested: &Record,
    ) -> Result<(), &'static str> {
        let latest = trusted_head(remote)?;
        if local[0] != local[1] {
            return Err("LOCAL_STATE_DIVERGED");
        }
        if local[0] != latest || *requested != latest {
            return Err("OBSERVER_HIGHWATER_DENIES_OLD_OR_FORK");
        }
        Ok(())
    }

    #[test]
    fn native_actual_github_ref_fetch_signatures_and_chained_epoch4_pass() {
        let evidence = evidence();
        assert_eq!(evidence.head, COMMIT);
        assert_eq!(sha(evidence.pack.as_bytes()), PACK_SHA);
        let v4 = trusted_head(Some(evidence)).unwrap();
        assert_eq!((v4.epoch, v4.seq), (4, 4));
    }

    #[test]
    fn native_after_restoring_both_local_and_old_signed_receipts_recontact_remote() {
        let e = evidence();
        let current = trusted_head(Some(e)).unwrap();
        let old = parse_record(section(&e.pack, "epoch3").unwrap()).unwrap();
        assert_eq!(admission(Some(e), [current.clone(), current.clone()], &current), Ok(()));
        assert_eq!((old.epoch, old.seq), (3, 3));
        assert!(sshsig(
            section(&e.pack, "epoch3").unwrap(),
            section(&e.pack, "epoch3_guardian_sig").unwrap(),
            GUARDIAN,
            GUARDIAN_PRINCIPAL,
            GUARDIAN_NS
        ));
        assert!(sshsig(
            section(&e.pack, "epoch3").unwrap(),
            section(&e.pack, "epoch3_observer_sig").unwrap(),
            OBSERVER,
            OBSERVER_PRINCIPAL,
            OBSERVER_NS
        ));
        // Both writable local copies and the two authentic old signatures
        // are restored. Re-fetch the GitHub ref and both signatures NOW.
        let refreshed = fetch_live().expect("fresh external read after rollback");
        assert_eq!(
            admission(Some(&refreshed), [old.clone(), old.clone()], &old),
            Err("OBSERVER_HIGHWATER_DENIES_OLD_OR_FORK")
        );
    }

    #[test]
    fn native_guardian_signed_same_epoch_fork_is_denied_by_observer_pin() {
        let e = evidence();
        let fork = section(&e.pack, "epoch4_fork").unwrap();
        let revoked = parse_record(fork).unwrap();
        let v4 = trusted_head(Some(e)).unwrap();
        assert_eq!((revoked.epoch, revoked.seq), (4, 4));
        assert!(sshsig(
            fork,
            section(&e.pack, "epoch4_fork_guardian_sig").unwrap(),
            GUARDIAN,
            GUARDIAN_PRINCIPAL,
            GUARDIAN_NS
        ));
        assert!(!sshsig(
            fork,
            section(&e.pack, "epoch4_observer_sig").unwrap(),
            OBSERVER,
            OBSERVER_PRINCIPAL,
            OBSERVER_NS
        ));
        assert_eq!(
            admission(Some(e), [revoked.clone(), revoked.clone()], &revoked),
            Err("OBSERVER_HIGHWATER_DENIES_OLD_OR_FORK")
        );
        assert_ne!(v4, revoked);
    }

    #[test]
    fn native_observer_signature_corruption_fails_native_ssh_verify() {
        let e = evidence();
        let v4 = section(&e.pack, "epoch4").unwrap();
        let mut sig = section(&e.pack, "epoch4_observer_sig").unwrap().to_owned();
        sig.replace_range(35..36, "X");
        assert!(!sshsig(v4, &sig, OBSERVER, OBSERVER_PRINCIPAL, OBSERVER_NS));
    }

    #[test]
    fn native_guardian_signature_replay_on_changed_checkpoint_fails() {
        let e = evidence();
        let v4 = section(&e.pack, "epoch4").unwrap();
        let corrupted = v4.replace("policy=accepted", "policy=revoked");
        assert!(!sshsig(
            &corrupted,
            section(&e.pack, "epoch4_guardian_sig").unwrap(),
            GUARDIAN,
            GUARDIAN_PRINCIPAL,
            GUARDIAN_NS
        ));
    }

    #[test]
    fn native_observer_cross_namespace_signature_rejected() {
        let e = evidence();
        let msg = section(&e.pack, "epoch4").unwrap();
        let sig = section(&e.pack, "epoch4_observer_sig").unwrap();
        assert!(!sshsig(msg, sig, OBSERVER, OBSERVER_PRINCIPAL, GUARDIAN_NS));
    }

    #[test]
    fn native_signer_substitution_and_head_pin_mismatch_fail_closed() {
        let e = evidence();
        let mut changed = e.clone();
        changed.head = "0000000000000000000000000000000000000000".into();
        assert_eq!(
            trusted_head(Some(&changed)),
            Err("EVIDENCE_COMMIT_OR_CONTENT_NOT_PINNED")
        );
        let mut wrong_key = e.clone();
        wrong_key.pack = wrong_key.pack.replacen(GUARDIAN, OBSERVER, 1);
        assert_eq!(
            trusted_head(Some(&wrong_key)),
            Err("EVIDENCE_COMMIT_OR_CONTENT_NOT_PINNED")
        );
        assert_eq!(
            accepted_remote(&wrong_key.pack),
            Err("EVIDENCE_SIGNER_NOT_PINNED")
        );
    }

    #[test]
    fn native_duplicate_or_missing_signed_observer_sections_fail_closed() {
        let e = evidence();
        let mut duplicate = e.pack.clone();
        duplicate.push_str("[epoch4_observer_sig]\nforged\n[/epoch4_observer_sig]\n");
        assert_eq!(
            accepted_remote(&duplicate),
            Err("EVIDENCE_SECTION_NOT_UNIQUE")
        );
        let missing = e
            .pack
            .replace("[epoch4_observer_sig]", "[missing_observer_sig]");
        assert_eq!(
            accepted_remote(&missing),
            Err("EVIDENCE_SECTION_NOT_UNIQUE")
        );
    }

    #[test]
    fn native_sha256_predecessor_rewrite_or_epoch_gap_rejected() {
        let e = evidence();
        let first_str = section(&e.pack, "epoch3").unwrap();
        let next_str = section(&e.pack, "epoch4").unwrap();
        let first = parse_record(first_str).unwrap();
        let mut next = parse_record(next_str).unwrap();
        next.parent = "0".repeat(64);
        assert_eq!(
            check_link(&first, first_str, &next),
            Err("LOG_PREDECESSOR_MISMATCH")
        );
        next.parent = E3_SHA.into();
        next.seq = 7;
        assert_eq!(
            check_link(&first, first_str, &next),
            Err("LOG_NONMONOTONIC")
        );
    }

    #[test]
    fn native_missing_remote_or_divergent_local_copies_fail_closed() {
        let e = evidence();
        let v4 = trusted_head(Some(e)).unwrap();
        let old = parse_record(section(&e.pack, "epoch3").unwrap()).unwrap();
        assert_eq!(
            admission(None, [v4.clone(), v4.clone()], &v4),
            Err("OFF_DEVICE_OBSERVER_REQUIRED")
        );
        assert_eq!(
            admission(Some(e), [v4.clone(), old], &v4),
            Err("LOCAL_STATE_DIVERGED")
        );
    }

    #[test]
    fn native_old_authentic_observer_receipt_stays_authentic_but_obsolete() {
        let e = evidence();
        let old = section(&e.pack, "epoch3").unwrap();
        let s = section(&e.pack, "epoch3_observer_sig").unwrap();
        assert!(sshsig(old, s, OBSERVER, OBSERVER_PRINCIPAL, OBSERVER_NS));
        let previously_valid = parse_record(old).unwrap();
        assert_eq!(
            admission(Some(e), [previously_valid.clone(), previously_valid.clone()], &previously_valid),
            Err("OBSERVER_HIGHWATER_DENIES_OLD_OR_FORK")
        );
    }
}
#[cfg(not(target_os = "macos"))]
#[test]
fn portable_targets_do_not_claim_actual_independent_guardian_custody() {
    assert_ne!(std::env::consts::OS, "macos");
}
#[test]
fn reference_kernel_host_external_io_remains_default_denied() {
    let (mut model, permit) = Model::bootstrap(
        0x54,
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
