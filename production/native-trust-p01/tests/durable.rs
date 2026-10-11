use ed25519_dalek::{Signer, SigningKey};
use nordoi_native_trust_p01::{
    durable::{DurableError, DurableTrustGate},
    sha256_payload, GateError, Intent, SignedEnvelope,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
struct TempJournal(PathBuf);
impl TempJournal {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "nordoi-p02-{}-{}-{}.bin",
            std::process::id(),
            stamp,
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        )))
    }
}
impl Drop for TempJournal {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(self.0.with_extension("p02.lock"));
    }
}
fn key(byte: u8) -> SigningKey {
    SigningKey::from_bytes(&[byte; 32])
}
fn roots() -> Vec<(u64, [u8; 32], u64)> {
    vec![
        (10, key(7).verifying_key().to_bytes(), 1),
        (20, key(8).verifying_key().to_bytes(), 1),
    ]
}
fn signed(key: &SigningKey, id: u64, epoch: u64, seq: u64, intent: Intent) -> SignedEnvelope {
    let mut e = SignedEnvelope {
        root_id: id,
        epoch,
        sequence: seq,
        intent,
        signature: [0; 64],
    };
    e.signature = key.sign(&e.signing_bytes().unwrap()).to_bytes();
    e
}
fn execute(key: &SigningKey, id: u64, epoch: u64, seq: u64) -> SignedEnvelope {
    signed(
        key,
        id,
        epoch,
        seq,
        Intent::Execute {
            capability: "file.write".into(),
            resource: "workspace/report".into(),
            payload_sha256: sha256_payload(b"hello"),
        },
    )
}
fn admit(g: &mut DurableTrustGate, e: &SignedEnvelope) -> Result<(), DurableError> {
    g.verify_execution(e, "file.write", "workspace/report", b"hello")
        .map(|_| ())
}

#[test]
fn initialization_is_explicit_and_missing_journal_fails_closed() {
    let p = TempJournal::new();
    assert!(matches!(
        DurableTrustGate::open(&p.0, &roots()),
        Err(DurableError::MissingJournal)
    ));
    let gate = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    assert_eq!(gate.root_status(10), Some((1, 0, false)));
    drop(gate);
    assert!(matches!(
        DurableTrustGate::initialize(&p.0, &roots()),
        Err(DurableError::JournalExists)
    ));
}
#[test]
fn signed_execution_is_fsynced_and_replayed_after_restart() {
    let p = TempJournal::new();
    let mut g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    admit(&mut g, &execute(&key(7), 10, 1, 1)).unwrap();
    assert!(g.journal_bytes() > 64);
    drop(g);
    let mut recovered = DurableTrustGate::open(&p.0, &roots()).unwrap();
    assert_eq!(recovered.root_status(10), Some((1, 1, false)));
    assert!(matches!(
        admit(&mut recovered, &execute(&key(7), 10, 1, 1)),
        Err(DurableError::Crypto(GateError::WrongSequence))
    ));
    admit(&mut recovered, &execute(&key(7), 10, 1, 2)).unwrap();
}
#[test]
fn key_rotation_survives_restart_and_old_key_is_rejected() {
    let p = TempJournal::new();
    let mut g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    g.rotate(&signed(
        &key(7),
        10,
        1,
        1,
        Intent::Rotate {
            next_epoch: 2,
            next_public_key: key(9).verifying_key().to_bytes(),
        },
    ))
    .unwrap();
    drop(g);
    let mut g = DurableTrustGate::open(&p.0, &roots()).unwrap();
    assert_eq!(g.root_status(10), Some((2, 1, false)));
    assert!(matches!(
        admit(&mut g, &execute(&key(7), 10, 1, 2)),
        Err(DurableError::Crypto(GateError::WrongEpoch))
    ));
    admit(&mut g, &execute(&key(9), 10, 2, 2)).unwrap();
}
#[test]
fn signed_revocation_survives_restart_without_affecting_second_root() {
    let p = TempJournal::new();
    let mut g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    g.revoke(&signed(&key(7), 10, 1, 1, Intent::Revoke))
        .unwrap();
    drop(g);
    let mut g = DurableTrustGate::open(&p.0, &roots()).unwrap();
    assert_eq!(g.root_status(10), Some((1, 1, true)));
    assert!(matches!(
        admit(&mut g, &execute(&key(7), 10, 1, 2)),
        Err(DurableError::Crypto(GateError::Revoked))
    ));
    admit(&mut g, &execute(&key(8), 20, 1, 1)).unwrap();
}
#[test]
fn corruption_fails_closed_before_recovery() {
    let p = TempJournal::new();
    let mut g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    admit(&mut g, &execute(&key(7), 10, 1, 1)).unwrap();
    drop(g);
    let mut bytes = fs::read(&p.0).unwrap();
    let at = bytes.len() - 40;
    bytes[at] ^= 0x80;
    fs::write(&p.0, bytes).unwrap();
    assert!(matches!(
        DurableTrustGate::open(&p.0, &roots()),
        Err(DurableError::Corrupt)
    ));
}
#[test]
fn torn_last_record_is_detected() {
    let p = TempJournal::new();
    let mut g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    admit(&mut g, &execute(&key(7), 10, 1, 1)).unwrap();
    drop(g);
    let mut bytes = fs::read(&p.0).unwrap();
    bytes.truncate(bytes.len() - 7);
    fs::write(&p.0, bytes).unwrap();
    assert!(matches!(
        DurableTrustGate::open(&p.0, &roots()),
        Err(DurableError::Corrupt)
    ));
}
#[test]
fn incorrect_trust_anchor_cannot_replay_journal() {
    let p = TempJournal::new();
    let g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    drop(g);
    let altered = vec![
        (10, key(9).verifying_key().to_bytes(), 1),
        (20, key(8).verifying_key().to_bytes(), 1),
    ];
    assert!(matches!(
        DurableTrustGate::open(&p.0, &altered),
        Err(DurableError::BootstrapMismatch)
    ));
}
#[test]
fn concurrent_writer_is_blocked_by_exclusive_lock() {
    let p = TempJournal::new();
    let g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    assert!(matches!(
        DurableTrustGate::open(&p.0, &roots()),
        Err(DurableError::Busy)
    ));
    drop(g);
    assert!(DurableTrustGate::open(&p.0, &roots()).is_ok());
}
#[test]
fn invalid_signature_does_not_append_or_consume_sequence() {
    let p = TempJournal::new();
    let mut g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    let before = g.journal_bytes();
    let forged = execute(&key(8), 10, 1, 1);
    assert!(matches!(
        admit(&mut g, &forged),
        Err(DurableError::Crypto(GateError::BadSignature))
    ));
    assert_eq!(g.journal_bytes(), before);
    admit(&mut g, &execute(&key(7), 10, 1, 1)).unwrap();
}
#[test]
fn changed_signed_resource_does_not_persist() {
    let p = TempJournal::new();
    let mut g = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    let before = g.journal_bytes();
    let req = execute(&key(7), 10, 1, 1);
    assert!(matches!(
        g.verify_execution(&req, "file.write", "workspace/other", b"hello"),
        Err(DurableError::Crypto(GateError::WrongScope))
    ));
    assert_eq!(g.journal_bytes(), before);
}
