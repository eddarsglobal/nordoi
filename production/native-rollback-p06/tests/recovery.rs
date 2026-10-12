use ed25519_dalek::{Signer, SigningKey};
use nordoi_kernel::{
    Effect, EffectBackend, EffectBackendError, EffectBackendReceipt, EffectDeliveryFence,
    EffectDeliveryKey, EffectDeliveryNamespace, EffectDispatchRequest, EffectIntentId,
    QueuedEffectIntent, ReactionId,
};
use nordoi_native_effect_p03::production::{DeliveryLedgerWitness, ProductionEffectBackend};
use nordoi_native_effect_p03::{
    canonical_dispatch_bytes, GuardedEffectBackend, SignedEnvelopeSource,
};
use nordoi_native_trust_p01::{
    durable::{DurableTrustGate, TrustJournalWitness},
    sha256_payload, Intent, SignedEnvelope,
};
use nordoi_rollback_anchor_p06::{
    initialize_production_anchor, verify_production_recovery, AnchorStoreError, ChainWitness,
    MonotonicAnchor, ProductionAnchorRecord, ProductionWitnessSource, RecoveryError,
};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
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
            "nordoi-p06a-{prefix}-{}-{stamp}-{}",
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
    SigningKey::from_bytes(&[31; 32])
}

fn roots() -> [(u64, [u8; 32], u64); 1] {
    [(700, signing_key().verifying_key().to_bytes(), 1)]
}

fn request(id: u64) -> EffectDispatchRequest {
    let id = EffectIntentId(id);
    EffectDispatchRequest {
        queued: QueuedEffectIntent {
            id,
            cycle: 9,
            ordinal: 0,
            intent: nordoi_kernel::EffectIntent {
                reaction: ReactionId(12),
                action_name: "p06a-network-call".into(),
                effect: Effect::Network("example.invalid".into()),
            },
        },
        delivery_key: Some(EffectDeliveryKey::new(
            EffectDeliveryNamespace::new([19; 16]),
            id,
        )),
        delivery_fence: Some(EffectDeliveryFence(5)),
    }
}

fn signed(request: &EffectDispatchRequest, sequence: u64) -> SignedEnvelope {
    let payload = canonical_dispatch_bytes(request).unwrap();
    let mut envelope = SignedEnvelope {
        root_id: 700,
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

struct Destination;

impl EffectBackend for Destination {
    fn supports(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Network(_))
    }

    fn execute(
        &mut self,
        _: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        Ok(EffectBackendReceipt::new("p06a-test-ack"))
    }
}

type Backend = ProductionEffectBackend<Destination, Envelopes>;

struct BackendWitnessSource<'a>(&'a Backend);

impl ProductionWitnessSource for BackendWitnessSource<'_> {
    type Error = ();

    fn trust_witness(&self) -> ChainWitness {
        let witness = self.0.trust_witness();
        ChainWitness::from_parts(witness.bytes(), witness.digest())
    }

    fn contains_trust_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error> {
        self.0
            .contains_trust_witness(TrustJournalWitness::from_parts(
                witness.position(),
                witness.digest(),
            ))
            .map_err(|_| ())
    }

    fn delivery_witness(&self) -> ChainWitness {
        let witness = self.0.delivery_witness();
        ChainWitness::from_parts(witness.bytes(), witness.digest())
    }

    fn contains_delivery_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error> {
        self.0
            .contains_delivery_witness(DeliveryLedgerWitness::from_parts(
                witness.position(),
                witness.digest(),
            ))
            .map_err(|_| ())
    }
}

fn guarded(
    trust: DurableTrustGate,
    envelopes: Vec<SignedEnvelope>,
) -> GuardedEffectBackend<Destination, Envelopes> {
    GuardedEffectBackend::new(Destination, trust, Envelopes(envelopes.into()))
}

fn initialize_backend(envelopes: Vec<SignedEnvelope>) -> (Scratch, Scratch, Backend) {
    let trust_path = Scratch::new("trust");
    let ledger_path = Scratch::new("delivery");
    let trust = DurableTrustGate::initialize(&trust_path.0, &roots()).unwrap();
    let backend =
        ProductionEffectBackend::initialize(guarded(trust, envelopes), &ledger_path.0).unwrap();
    (trust_path, ledger_path, backend)
}

fn reopen_backend(trust_path: &Scratch, ledger_path: &Scratch) -> Backend {
    let trust = DurableTrustGate::open(&trust_path.0, &roots()).unwrap();
    ProductionEffectBackend::open(guarded(trust, vec![]), &ledger_path.0).unwrap()
}

#[derive(Default)]
struct MemoryAnchor {
    record: Option<ProductionAnchorRecord>,
}

impl MonotonicAnchor for MemoryAnchor {
    fn read(&mut self) -> Result<Option<ProductionAnchorRecord>, AnchorStoreError> {
        Ok(self.record)
    }

    fn compare_and_advance(
        &mut self,
        expected: Option<ProductionAnchorRecord>,
        next: ProductionAnchorRecord,
    ) -> Result<(), AnchorStoreError> {
        if self.record != expected {
            return Err(AnchorStoreError::Conflict);
        }
        match expected {
            None if next.generation() != 1 => return Err(AnchorStoreError::Conflict),
            Some(previous) if next.generation() <= previous.generation() => {
                return Err(AnchorStoreError::Conflict)
            }
            _ => {}
        }
        self.record = Some(next);
        Ok(())
    }
}

#[test]
fn explicit_initialization_then_exact_recovery_passes() {
    let (_trust, _ledger, backend) = initialize_backend(vec![]);
    let mut anchor = MemoryAnchor::default();
    let initialized =
        initialize_production_anchor(&mut anchor, &BackendWitnessSource(&backend)).unwrap();
    assert_eq!(initialized.generation(), 1);
    assert_eq!(
        verify_production_recovery(&mut anchor, &BackendWitnessSource(&backend)),
        Ok(initialized)
    );
}

#[test]
fn missing_anchor_on_recovery_fails_closed() {
    let (_trust, _ledger, backend) = initialize_backend(vec![]);
    let mut anchor = MemoryAnchor::default();
    assert_eq!(
        verify_production_recovery(&mut anchor, &BackendWitnessSource(&backend)),
        Err(RecoveryError::MissingAnchor)
    );
}

#[test]
fn valid_local_extension_advances_anchor_only_from_verified_prefix() {
    let r = request(1);
    let (_trust, _ledger, mut backend) = initialize_backend(vec![signed(&r, 1)]);
    let mut anchor = MemoryAnchor::default();
    let first = initialize_production_anchor(&mut anchor, &BackendWitnessSource(&backend)).unwrap();
    backend.execute_with_context(&r).unwrap();
    let advanced =
        verify_production_recovery(&mut anchor, &BackendWitnessSource(&backend)).unwrap();
    assert_eq!(advanced.generation(), first.generation() + 1);
    assert!(advanced.trust().position() > first.trust().position());
    assert!(advanced.delivery().position() > first.delivery().position());
}

#[test]
fn restoring_older_valid_trust_journal_is_detected() {
    let r = request(2);
    let (trust_path, ledger_path, mut backend) = initialize_backend(vec![signed(&r, 1)]);
    let old_trust = fs::read(&trust_path.0).unwrap();
    let mut anchor = MemoryAnchor::default();
    initialize_production_anchor(&mut anchor, &BackendWitnessSource(&backend)).unwrap();
    backend.execute_with_context(&r).unwrap();
    verify_production_recovery(&mut anchor, &BackendWitnessSource(&backend)).unwrap();
    drop(backend);

    fs::write(&trust_path.0, old_trust).unwrap();
    let restored = reopen_backend(&trust_path, &ledger_path);
    assert_eq!(
        verify_production_recovery(&mut anchor, &BackendWitnessSource(&restored)),
        Err(RecoveryError::TrustRollback)
    );
}

#[test]
fn restoring_older_valid_delivery_ledger_is_detected() {
    let r = request(3);
    let (trust_path, ledger_path, mut backend) = initialize_backend(vec![signed(&r, 1)]);
    let old_ledger = fs::read(&ledger_path.0).unwrap();
    let mut anchor = MemoryAnchor::default();
    initialize_production_anchor(&mut anchor, &BackendWitnessSource(&backend)).unwrap();
    backend.execute_with_context(&r).unwrap();
    verify_production_recovery(&mut anchor, &BackendWitnessSource(&backend)).unwrap();
    drop(backend);

    fs::write(&ledger_path.0, old_ledger).unwrap();
    let restored = reopen_backend(&trust_path, &ledger_path);
    assert_eq!(
        verify_production_recovery(&mut anchor, &BackendWitnessSource(&restored)),
        Err(RecoveryError::DeliveryRollback)
    );
}
