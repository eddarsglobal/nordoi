use ed25519_dalek::{Signer, SigningKey};
use nordoi_kernel::{
    Effect, EffectBackend, EffectBackendError, EffectBackendReceipt, EffectDeliveryFence,
    EffectDeliveryKey, EffectDeliveryNamespace, EffectDispatchRequest, EffectIntentId,
    QueuedEffectIntent, ReactionId,
};
use nordoi_native_effect_p03::production::{DeliveryState, ProductionEffectBackend};
use nordoi_native_effect_p03::{
    canonical_dispatch_bytes, GuardedEffectBackend, SignedEnvelopeSource,
};
use nordoi_native_trust_p01::{durable::DurableTrustGate, sha256_payload, Intent, SignedEnvelope};
use std::cell::Cell;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SERIAL: AtomicU64 = AtomicU64::new(1);
struct Scratch(PathBuf);
impl Scratch {
    fn new(prefix: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "nordoi-p04-{prefix}-{}-{stamp}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed),
        )))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(self.0.with_extension("p02.lock"));
        let _ = fs::remove_file(self.0.with_extension("p04.lock"));
    }
}
fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[24; 32])
}
fn roots() -> [(u64, [u8; 32], u64); 1] {
    [(100, signing_key().verifying_key().to_bytes(), 1)]
}
fn request(id: u64, fence: u64) -> EffectDispatchRequest {
    let id = EffectIntentId(id);
    EffectDispatchRequest {
        queued: QueuedEffectIntent {
            id,
            cycle: 2,
            ordinal: 0,
            intent: nordoi_kernel::EffectIntent {
                reaction: ReactionId(3),
                action_name: "real-network-call".into(),
                effect: Effect::Network("example.invalid".into()),
            },
        },
        delivery_key: Some(EffectDeliveryKey::new(
            EffectDeliveryNamespace::new([11; 16]),
            id,
        )),
        delivery_fence: Some(EffectDeliveryFence(fence)),
    }
}
fn signed(request: &EffectDispatchRequest, sequence: u64) -> SignedEnvelope {
    let payload = canonical_dispatch_bytes(request).unwrap();
    let mut envelope = SignedEnvelope {
        root_id: 100,
        epoch: 1,
        sequence,
        intent: Intent::Execute {
            capability: "nordoi/effect/execute".into(),
            resource: "nordoi/dispatch/v1".into(),
            payload_sha256: sha256_payload(&payload),
        },
        signature: [0; 64],
    };
    envelope.signature = signing_key()
        .sign(&envelope.signing_bytes().unwrap())
        .to_bytes();
    envelope
}
struct Envelopes(VecDeque<SignedEnvelope>);
impl SignedEnvelopeSource for Envelopes {
    fn take(&mut self, _: &EffectDispatchRequest) -> Option<SignedEnvelope> {
        self.0.pop_front()
    }
}
#[derive(Clone)]
struct Destination {
    calls: Rc<Cell<u32>>,
    fail: bool,
}
impl EffectBackend for Destination {
    fn supports(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Network(_))
    }
    fn execute(
        &mut self,
        _: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            Err(EffectBackendError::new("uncertain delivery"))
        } else {
            Ok(EffectBackendReceipt::new("acknowledged-by-test-host"))
        }
    }
}
fn guarded(
    trust: DurableTrustGate,
    envelopes: Vec<SignedEnvelope>,
    fail: bool,
) -> (GuardedEffectBackend<Destination, Envelopes>, Rc<Cell<u32>>) {
    let calls = Rc::new(Cell::new(0));
    let backend = Destination {
        calls: Rc::clone(&calls),
        fail,
    };
    (
        GuardedEffectBackend::new(backend, trust, Envelopes(envelopes.into())),
        calls,
    )
}
fn fixture(
    envelopes: Vec<SignedEnvelope>,
    fail: bool,
) -> (
    Scratch,
    Scratch,
    ProductionEffectBackend<Destination, Envelopes>,
    Rc<Cell<u32>>,
) {
    let journal = Scratch::new("trust");
    let ledger = Scratch::new("delivery");
    let trust = DurableTrustGate::initialize(&journal.0, &roots()).unwrap();
    let (backend, calls) = guarded(trust, envelopes, fail);
    let production = ProductionEffectBackend::initialize(backend, &ledger.0).unwrap();
    (journal, ledger, production, calls)
}
#[test]
fn trait_is_the_kernel_effect_backend_contract() {
    fn accepts_backend<T: EffectBackend>(_: &T) {}
    let (_t, _l, backend, _calls) = fixture(vec![], false);
    accepts_backend(&backend);
}
#[test]
fn contextless_execution_denies_without_ledger_entry() {
    let r = request(1, 1);
    let (_t, _l, mut backend, calls) = fixture(vec![], false);
    assert!(backend.execute(&r.queued).is_err());
    assert_eq!(backend.status(&r), None);
    assert_eq!(calls.get(), 0);
}
#[test]
fn valid_signed_dispatch_is_locally_acknowledged() {
    let r = request(11, 3);
    let (_t, _l, mut backend, calls) = fixture(vec![signed(&r, 1)], false);
    assert!(backend.execute_with_context(&r).is_ok());
    assert_eq!(backend.status(&r), Some(DeliveryState::LocallyAcknowledged));
    assert_eq!(calls.get(), 1);
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 1);
}
#[test]
fn unknown_backend_outcome_stays_in_doubt() {
    let r = request(12, 3);
    let (_t, _l, mut backend, calls) = fixture(vec![signed(&r, 1)], true);
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(backend.status(&r), Some(DeliveryState::InDoubt));
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 1);
}
#[test]
fn new_signed_sequence_cannot_reauthorize_same_delivery() {
    let r = request(13, 3);
    let (_t, _l, mut backend, calls) = fixture(vec![signed(&r, 1), signed(&r, 2)], true);
    assert!(backend.execute_with_context(&r).is_err());
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 1);
}
#[test]
fn changed_fence_cannot_reauthorize_same_delivery_identity() {
    let r = request(14, 3);
    let newer = request(14, 4);
    let (_t, _l, mut backend, calls) = fixture(vec![signed(&r, 1), signed(&newer, 2)], false);
    backend.execute_with_context(&r).unwrap();
    assert!(backend.execute_with_context(&newer).is_err());
    assert_eq!(calls.get(), 1);
}
#[test]
fn different_delivery_ids_can_each_execute() {
    let a = request(15, 3);
    let b = request(16, 3);
    let (_t, _l, mut backend, calls) = fixture(vec![signed(&a, 1), signed(&b, 2)], false);
    backend.execute_with_context(&a).unwrap();
    backend.execute_with_context(&b).unwrap();
    assert_eq!(calls.get(), 2);
}
#[test]
fn missing_signature_reserves_conservative_in_doubt_without_effect() {
    let r = request(17, 3);
    let (_t, _l, mut backend, calls) = fixture(vec![], false);
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(backend.status(&r), Some(DeliveryState::InDoubt));
    assert_eq!(calls.get(), 0);
}
#[test]
fn malformed_delivery_context_is_rejected_before_reservation() {
    let mut r = request(18, 3);
    r.delivery_fence = None;
    let (_t, _l, mut backend, calls) = fixture(vec![], false);
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(backend.status(&r), None);
    assert_eq!(calls.get(), 0);
}
#[test]
fn restart_retains_ack_and_denies_reissued_signed_delivery() {
    let r = request(19, 3);
    let (journal, ledger, mut backend, calls) = fixture(vec![signed(&r, 1)], false);
    backend.execute_with_context(&r).unwrap();
    assert_eq!(calls.get(), 1);
    drop(backend);
    let trust = DurableTrustGate::open(&journal.0, &roots()).unwrap();
    let (guard, after_calls) = guarded(trust, vec![signed(&r, 2)], false);
    let mut restored = ProductionEffectBackend::open(guard, &ledger.0).unwrap();
    assert_eq!(
        restored.status(&r),
        Some(DeliveryState::LocallyAcknowledged)
    );
    assert!(restored.execute_with_context(&r).is_err());
    assert_eq!(after_calls.get(), 0);
}
#[test]
fn restart_retains_in_doubt_and_denies_retry() {
    let r = request(20, 3);
    let (journal, ledger, mut backend, _) = fixture(vec![signed(&r, 1)], true);
    assert!(backend.execute_with_context(&r).is_err());
    drop(backend);
    let trust = DurableTrustGate::open(&journal.0, &roots()).unwrap();
    let (guard, after_calls) = guarded(trust, vec![signed(&r, 2)], false);
    let mut restored = ProductionEffectBackend::open(guard, &ledger.0).unwrap();
    assert_eq!(restored.status(&r), Some(DeliveryState::InDoubt));
    assert!(restored.execute_with_context(&r).is_err());
    assert_eq!(after_calls.get(), 0);
}
#[test]
fn corrupt_ledger_fails_closed_on_reopen() {
    let r = request(21, 3);
    let (journal, ledger, mut backend, _) = fixture(vec![signed(&r, 1)], false);
    backend.execute_with_context(&r).unwrap();
    drop(backend);
    let mut bytes = fs::read(&ledger.0).unwrap();
    *bytes.last_mut().unwrap() ^= 0x01;
    fs::write(&ledger.0, bytes).unwrap();
    let trust = DurableTrustGate::open(&journal.0, &roots()).unwrap();
    let (guard, _) = guarded(trust, vec![], false);
    assert!(ProductionEffectBackend::open(guard, &ledger.0).is_err());
}
#[test]
fn torn_ledger_fails_closed_on_reopen() {
    let r = request(22, 3);
    let (journal, ledger, mut backend, _) = fixture(vec![signed(&r, 1)], false);
    backend.execute_with_context(&r).unwrap();
    drop(backend);
    let mut bytes = fs::read(&ledger.0).unwrap();
    bytes.pop();
    fs::write(&ledger.0, bytes).unwrap();
    let trust = DurableTrustGate::open(&journal.0, &roots()).unwrap();
    let (guard, _) = guarded(trust, vec![], false);
    assert!(ProductionEffectBackend::open(guard, &ledger.0).is_err());
}
#[test]
fn second_ledger_owner_is_denied() {
    let (_journal, ledger, _first, _calls) = fixture(vec![], false);
    let trust_two_path = Scratch::new("trust-two");
    let trust_two = DurableTrustGate::initialize(&trust_two_path.0, &roots()).unwrap();
    let (guard, _) = guarded(trust_two, vec![], false);
    assert!(ProductionEffectBackend::open(guard, &ledger.0).is_err());
}
#[test]
fn missing_ledger_is_not_implicitly_initialized() {
    let trust_path = Scratch::new("trust-missing");
    let ledger_path = Scratch::new("missing");
    let trust = DurableTrustGate::initialize(&trust_path.0, &roots()).unwrap();
    let (guard, _) = guarded(trust, vec![], false);
    assert!(ProductionEffectBackend::open(guard, &ledger_path.0).is_err());
    assert!(!ledger_path.0.exists());
}
