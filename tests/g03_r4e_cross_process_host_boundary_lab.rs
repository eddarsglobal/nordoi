//! G0.3 R4E: real, distinct OS child process with bounded parent-host IPC.
//!
//! Research-only integration tests: NO sandbox or nonforgeable OS authority.
//! The child inherits ambient OS user rights; a malicious child could bypass
//! these IPC checks with direct system calls. No production .noi executor.
//! Parent owns Model, File, path and NAM. Disk and NAM are NOT atomic together.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;

use nordoi_kernel::AtomicKernel;
use reference::{
    Command, GrantId, Model, ModelError, Receipt, ResourceEffect, ResourceId, ScopeBudget,
    ScopeOutcome, ScopeReport, TaskId,
};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command as OsCommand, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const CORRELATION: &str = "r4e-fixed-fixture-20261108";
const MAX_TRANSCRIPT_BYTES: usize = 4096;
const FAILURE_CODE: u16 = 222;
static NEXT: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scenario {
    Valid,
    Cancel,
    FakeCancelSuccess,
    MissingGrant,
    RequestHostWrite,
    ForgedCorrelation,
    DuplicateReply,
    Overflow,
    Panic,
    EarlyExit,
    Stall,
    ExcessOutput,
}
impl Scenario {
    fn command(self) -> &'static str {
        match self {
            Self::Valid | Self::MissingGrant => "GO|20|22",
            Self::Cancel => "CANCEL",
            Self::FakeCancelSuccess => "CANCEL_FAKE",
            Self::RequestHostWrite => "HOST_WRITE",
            Self::ForgedCorrelation => "FORGE",
            Self::DuplicateReply => "DUPLICATE",
            Self::Overflow => "GO|9223372036854775807|1",
            Self::Panic => "PANIC",
            Self::EarlyExit => "EARLY",
            Self::Stall => "STALL",
            Self::ExcessOutput => "FLOOD",
        }
    }
    fn is_cancel(self) -> bool {
        matches!(self, Self::Cancel | Self::FakeCancelSuccess)
    }
}

/// This is an ignored libtest entry. A parent test executes the SAME pinned
/// test binary in a real, separate OS process using exact filtered arguments.
/// Only a fixed arithmetic operation is supported. No path is passed to child.
#[test]
#[ignore = "executed only by controlled R4E subprocess test launcher"]
fn r4e_child_entry() {
    if env::var("NORDOI_R4E_CHILD_MODE").ok().as_deref() != Some("FIXED_V1") {
        return;
    }
    if env::var("NORDOI_R4E_EARLY").ok().as_deref() == Some("1") {
        return;
    }
    let id = env::var("NORDOI_R4E_CORRELATION").expect("test-only child correlation");
    eprintln!("R4E|READY|{id}");
    let mut line = String::new();
    BufReader::new(io::stdin().lock())
        .read_line(&mut line)
        .expect("single parent command");
    let command = line.trim_end_matches(['\n', '\r']);
    match command {
        "GO|20|22" | "GO|9223372036854775807|1" => {
            let mut parts = command.split('|');
            assert_eq!(parts.next(), Some("GO"));
            let a: i64 = parts.next().unwrap().parse().unwrap();
            let b: i64 = parts.next().unwrap().parse().unwrap();
            match a.checked_add(b) {
                Some(value) => eprintln!("R4E|RESULT|{id}|{value}"),
                None => eprintln!("R4E|OVERFLOW|{id}"),
            }
        }
        "CANCEL" => eprintln!("R4E|CANCELLED|{id}"),
        "CANCEL_FAKE" => eprintln!("R4E|RESULT|{id}|42"),
        "HOST_WRITE" => eprintln!("R4E|REQUEST_HOST_WRITE|{id}"),
        "FORGE" => eprintln!("R4E|RESULT|wrong-id|42"),
        "DUPLICATE" => {
            eprintln!("R4E|RESULT|{id}|42");
            eprintln!("R4E|RESULT|{id}|42");
        }
        "PANIC" => panic!("R4E deliberate child-process panic"),
        "STALL" => loop {
            thread::sleep(Duration::from_millis(20));
        },
        "FLOOD" => {
            for _ in 0..100 {
                eprintln!("R4E|UNSOLICITED|{id}|AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
            }
        }
        _ => eprintln!("R4E|INVALID|{id}"),
    }
}

struct Scratch {
    dir: PathBuf,
}
impl Scratch {
    fn new() -> Self {
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!("nordoi-r4e-{}-{serial}", std::process::id()));
        fs::create_dir(&dir).expect("create one new disposable directory");
        Self { dir }
    }
    fn file(&self) -> PathBuf {
        self.dir.join("parent-only.txt")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("remove only this new test directory");
    }
}

#[derive(Debug)]
struct ProcessWitness {
    frames: Vec<String>,
    exit_ok: bool,
    killed: bool,
    started: usize,
}
fn reap_with_timeout(child: &mut Child, limit: Duration) -> (ExitStatus, bool) {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("test subprocess status") {
            return (status, false);
        }
        if start.elapsed() >= limit {
            let _ = child.kill();
            return (child.wait().expect("reap killed test subprocess"), true);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

/// Reader is bounded in byte count; every read has a global stderr cap.
/// On timeout the parent kills/reaps the OS child, then joins this reader.
/// Correlation is ONLY a test-run marker, not cryptographic authentication.
fn run_child(scenario: Scenario, model: &mut Model, task: TaskId) -> ProcessWitness {
    let exe = env::current_exe().expect("test binary identity");
    let mut cmd = OsCommand::new(exe);
    cmd.args([
        "--exact",
        "r4e_child_entry",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ])
    .env("NORDOI_R4E_CHILD_MODE", "FIXED_V1")
    .env("NORDOI_R4E_CORRELATION", CORRELATION)
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::piped());
    if scenario == Scenario::EarlyExit {
        cmd.env("NORDOI_R4E_EARLY", "1");
    }
    let mut child = cmd.spawn().expect("start fixed test child executable");
    let stderr = child.stderr.take().expect("owned child stderr pipe");
    let (sender, receiver) = mpsc::channel::<String>();
    let reader = thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut count = 0_usize;
        loop {
            let mut line = String::new();
            let n = match reader.read_line(&mut line) {
                Ok(n) => n,
                Err(_) => break,
            };
            if n == 0 {
                break;
            }
            count = count.saturating_add(n);
            if count > MAX_TRANSCRIPT_BYTES {
                let _ = sender.send("R4E|LIMIT".to_string());
                break;
            }
            let frame = line.trim_end_matches(['\n', '\r']);
            if frame.starts_with("R4E|") && sender.send(frame.to_string()).is_err() {
                break;
            }
        }
        let _ = sender.send("R4E|EOF".to_string());
    });

    let ready = format!("R4E|READY|{CORRELATION}");
    let first = receiver.recv_timeout(Duration::from_secs(10));
    let mut frames = Vec::new();
    let mut must_kill = false;
    if let Ok(frame) = first {
        if frame == ready {
            frames.push(frame);
            if scenario.is_cancel() {
                model
                    .apply(Command::RequestCancel { task })
                    .expect("parent-side R0.2 cancellation request");
            }
            let mut stdin = child.stdin.take().expect("owned child stdin");
            if writeln!(stdin, "{}", scenario.command()).is_err() {
                must_kill = true;
            }
            drop(stdin);
            if !must_kill {
                let limit = if matches!(scenario, Scenario::Stall) {
                    Duration::from_millis(350)
                } else {
                    Duration::from_secs(10)
                };
                match receiver.recv_timeout(limit) {
                    Ok(result) => frames.push(result),
                    Err(_) => must_kill = true,
                }
            }
        } else {
            frames.push(frame);
        }
    } else {
        must_kill = true;
    }
    if must_kill {
        let _ = child.kill();
    }
    let (status, timed_out) = reap_with_timeout(
        &mut child,
        if matches!(scenario, Scenario::ExcessOutput) {
            Duration::from_secs(2)
        } else {
            Duration::from_secs(10)
        },
    );
    reader.join().expect("join bounded reader after child exit");
    frames.extend(receiver.try_iter().filter(|frame| frame != "R4E|EOF"));
    ProcessWitness {
        frames,
        exit_ok: status.success(),
        killed: must_kill || timed_out,
        started: 1,
    }
}

struct RunEvidence {
    report: ScopeReport,
    bytes: Vec<u8>,
    nam_version: u64,
    replay_ok: bool,
    granted: bool,
    child: ProcessWitness,
}
/// The parent alone holds model authority, native File and NAM. Completion
/// of the real child, strict two-frame protocol and explicit modeled grant
/// must ALL succeed before it makes any native write.
fn execute(scenario: Scenario, grant_write: bool) -> RunEvidence {
    let (mut model, permit) = Model::bootstrap(
        0x2026_1108,
        ScopeBudget {
            tasks: 1,
            resources: 1,
            children: 0,
        },
        64,
    );
    let scope = model.root();
    let (resource, grant): (Option<ResourceId>, Option<GrantId>) = if grant_write {
        model
            .apply(Command::DeclareEffect {
                scope,
                effect: ResourceEffect::Write,
            })
            .unwrap();
        let grant = model
            .host_grant(&permit, scope, ResourceEffect::Write)
            .unwrap();
        let resource = match model
            .apply(Command::Acquire {
                scope,
                grant,
                effect: ResourceEffect::Write,
            })
            .unwrap()
        {
            Receipt::Resource(id) => id,
            other => panic!("unexpected modeled resource {other:?}"),
        };
        (Some(resource), Some(grant))
    } else {
        (None, None)
    };
    let task = match model.apply(Command::DeclareTask { scope }).unwrap() {
        Receipt::Task(id) => id,
        other => panic!("unexpected modeled task {other:?}"),
    };
    model.apply(Command::Admit { task }).unwrap();
    model.apply(Command::Start { task }).unwrap();

    let scratch = Scratch::new();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(scratch.file())
        .expect("only parent creates actual file");
    let child = run_child(scenario, &mut model, task);

    let correct = vec![
        format!("R4E|READY|{CORRELATION}"),
        format!("R4E|RESULT|{CORRELATION}|42"),
    ];
    let cancelled = vec![
        format!("R4E|READY|{CORRELATION}"),
        format!("R4E|CANCELLED|{CORRELATION}"),
    ];
    let success = child.exit_ok
        && !child.killed
        && !scenario.is_cancel()
        && child.frames == correct
        && grant_write;
    let acknowledged_cancel =
        child.exit_ok && !child.killed && scenario == Scenario::Cancel && child.frames == cancelled;
    if success {
        model.apply(Command::Succeed { task }).unwrap();
    } else if acknowledged_cancel {
        model.apply(Command::AcknowledgeCancel { task }).unwrap();
    } else {
        model
            .apply(Command::Fail {
                task,
                code: FAILURE_CODE,
            })
            .unwrap();
    }
    model.apply(Command::Join { task }).unwrap();
    if success {
        // This is a deliberately narrow model-to-host fixture convention;
        // R0.2 ExternalIo grants are still unsupported.
        let grant = grant.expect("parent-owned modeled grant");
        model
            .apply(Command::Use {
                scope,
                resource: resource.expect("explicit resource"),
                grant,
                effect: ResourceEffect::Write,
            })
            .unwrap();
        file.write_all(b"42\n").expect("bounded native test write");
    }
    drop(file);
    if let Some(resource) = resource {
        model.apply(Command::Release { scope, resource }).unwrap();
    }
    let report = match model.apply(Command::Close { scope }).unwrap() {
        Receipt::Closed(report) => report,
        other => panic!("unexpected model close {other:?}"),
    };
    let replay_ok = model.replay_checked().is_ok();
    let bytes = fs::read(scratch.file()).unwrap();

    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    if success && report.outcome == ScopeOutcome::Succeeded && replay_ok {
        assert_eq!(kernel.set(atom, 42_i64), Ok(true));
    }
    RunEvidence {
        report,
        bytes,
        nam_version: kernel.version(atom).unwrap(),
        replay_ok,
        granted: grant_write,
        child,
    }
}

#[test]
fn real_child_process_computation_requires_explicit_parent_grant() {
    let result = execute(Scenario::Valid, true);
    assert_eq!(result.child.started, 1);
    assert_eq!(result.child.frames.len(), 2);
    assert_eq!(result.report.outcome, ScopeOutcome::Succeeded);
    assert_eq!(result.bytes, b"42\n");
    assert_eq!(result.nam_version, 1);
    assert!(result.replay_ok && result.granted);
}

#[test]
fn correct_child_result_without_grant_cannot_write_or_commit_nam() {
    let result = execute(Scenario::MissingGrant, false);
    assert!(result.child.exit_ok);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert!(result.bytes.is_empty());
    assert_eq!(result.nam_version, 0);
}

#[test]
fn live_child_ready_then_parent_cancel_acknowledgement_blocks_io() {
    let result = execute(Scenario::Cancel, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Cancelled);
    assert_eq!(result.bytes, b"");
    assert_eq!(result.nam_version, 0);
    assert!(result.child.exit_ok);
    assert!(result.replay_ok);
}

#[test]
fn malicious_success_after_cancel_request_is_denied() {
    let result = execute(Scenario::FakeCancelSuccess, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert!(result.bytes.is_empty());
    assert_eq!(result.nam_version, 0);
}

#[test]
fn unsolicited_os_file_write_request_is_not_an_authorized_frame() {
    let result = execute(Scenario::RequestHostWrite, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert_eq!(result.bytes, b"");
    assert_eq!(result.nam_version, 0);
}

#[test]
fn forged_test_correlation_marker_is_denied() {
    let result = execute(Scenario::ForgedCorrelation, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert_eq!(result.bytes, b"");
    assert_eq!(result.nam_version, 0);
}

#[test]
fn duplicate_success_frames_are_denied() {
    let result = execute(Scenario::DuplicateReply, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert!(result.bytes.is_empty());
    assert_eq!(result.nam_version, 0);
}

#[test]
fn checked_arithmetic_overflow_cannot_commit() {
    let result = execute(Scenario::Overflow, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert_eq!(result.nam_version, 0);
    assert!(result.bytes.is_empty());
}

#[test]
fn actual_child_process_panic_is_not_parent_success() {
    let result = execute(Scenario::Panic, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert!(!result.child.exit_ok);
    assert_eq!(result.nam_version, 0);
    assert!(result.bytes.is_empty());
}

#[test]
fn child_early_exit_before_handshake_fails_closed() {
    let result = execute(Scenario::EarlyExit, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert_eq!(result.nam_version, 0);
    assert!(result.bytes.is_empty());
}

#[test]
fn unresponsive_child_is_terminated_and_reaped() {
    let result = execute(Scenario::Stall, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert!(result.child.killed);
    assert_eq!(result.nam_version, 0);
    assert!(result.bytes.is_empty());
}

#[test]
fn excessive_child_protocol_output_is_bounded_and_denied() {
    let result = execute(Scenario::ExcessOutput, true);
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert_eq!(result.nam_version, 0);
    assert!(result.bytes.is_empty());
    assert!(result.replay_ok);
}

#[test]
fn unsupported_real_external_io_effect_remains_default_denied() {
    let (mut model, permit) = Model::bootstrap(
        99,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        16,
    );
    let scope = model.root();
    assert_eq!(
        model.host_grant(&permit, scope, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(model.event_count(), 0);
}

#[test]
fn disposable_native_file_is_removed_at_parent_scope_exit() {
    let file_path;
    {
        let scratch = Scratch::new();
        file_path = scratch.file();
        let file = File::create(&file_path).unwrap();
        drop(file);
        assert!(file_path.exists());
    }
    assert!(!file_path.exists());
}
