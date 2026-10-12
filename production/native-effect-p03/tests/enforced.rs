use ed25519_dalek::{Signer, SigningKey};
use nordoi_kernel::{
    AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, DomainRef, Effect, EffectBackend,
    EffectBackendError, EffectBackendReceipt, EffectDeliveryFence, EffectDeliveryNamespace,
    EffectDispatchAuthority, EffectDispatchRequest, EffectFenceStoreError, EffectFencingError,
    EffectJournalCommitReceipt, EffectJournalLease, EffectJournalWriterId, EventLoopError,
    FencedEffectJournalStore, GovernedEffectDispatcher, GovernedFencedEffectJournal, InputBatch,
    InputDeviceId, InputPayload, InputSignal, InputSource, InputTargetRef, Instruction,
    LogicalTime, NairEffectSet, NairProgram, NairReactionAuthority, NairReactionStep,
    NairReactionTrigger, QueuedEffectIntent, ReactionSlot,
};
use nordoi_native_effect_p03::{
    canonical_dispatch_bytes, enforced::FencedProductionEffects,
    production::ProductionEffectBackend, GuardedEffectBackend, SignedEnvelopeSource,
};
use nordoi_native_trust_p01::{durable::DurableTrustGate, sha256_payload, Intent, SignedEnvelope};
use nordoi_rollback_anchor_p06::{
    AnchorOpenMode, AnchorStoreError, MonotonicAnchor, ProductionAnchorRecord,
};
use std::{
    cell::{Cell, RefCell},
    fs,
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NONCE: AtomicU64 = AtomicU64::new(1);
struct TempFile(PathBuf);
impl TempFile {
    fn new(label: &str) -> Self {
        let instant = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "nordoi-p05a-{label}-{}-{instant}-{}",
            std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        )))
    }
}
impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(self.0.with_extension("p02.lock"));
        let _ = fs::remove_file(self.0.with_extension("p04.lock"));
    }
}

struct SigningSource {
    enabled: bool,
    sequence: u64,
}
impl SignedEnvelopeSource for SigningSource {
    fn take(&mut self, request: &EffectDispatchRequest) -> Option<SignedEnvelope> {
        if !self.enabled {
            return None;
        }
        self.sequence += 1;
        let bytes = canonical_dispatch_bytes(request).ok()?;
        let mut envelope = SignedEnvelope {
            root_id: 100,
            epoch: 1,
            sequence: self.sequence,
            intent: Intent::Execute {
                capability: "nordoi/effect/execute".into(),
                resource: "nordoi/dispatch/v1".into(),
                payload_sha256: sha256_payload(&bytes),
            },
            signature: [0; 64],
        };
        envelope.signature = SigningKey::from_bytes(&[24; 32])
            .sign(&envelope.signing_bytes().ok()?)
            .to_bytes();
        Some(envelope)
    }
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

struct UnsafeDestination(Rc<Cell<u32>>);
impl EffectBackend for UnsafeDestination {
    fn supports(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Network(_))
    }
    fn execute(
        &mut self,
        _: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.0.set(self.0.get() + 1);
        Ok(EffectBackendReceipt::new("simulated destination accepted"))
    }
    // Intentionally inherits the kernel's historical contextless fallback.
    // The profile's P0.3/P0.4 layers must guard it.
}

#[derive(Default)]
struct StoreState {
    active: Option<EffectJournalLease>,
    bytes: Option<Vec<u8>>,
    fail_acquire: bool,
}
#[derive(Clone, Default)]
struct MemoryStore(Rc<RefCell<StoreState>>);
impl MemoryStore {
    fn require_active(
        state: &StoreState,
        lease: EffectJournalLease,
    ) -> Result<(), EffectFenceStoreError> {
        if state.active == Some(lease) {
            Ok(())
        } else {
            Err(EffectFenceStoreError::new("stale fence"))
        }
    }
}
impl FencedEffectJournalStore for MemoryStore {
    fn acquire(
        &mut self,
        namespace: EffectDeliveryNamespace,
        writer: EffectJournalWriterId,
    ) -> Result<EffectJournalLease, EffectFenceStoreError> {
        let mut state = self.0.borrow_mut();
        if state.fail_acquire {
            return Err(EffectFenceStoreError::new("lease denied"));
        }
        let lease = EffectJournalLease::new(namespace, writer, EffectDeliveryFence(1));
        state.active = Some(lease);
        Ok(lease)
    }
    fn assert_active(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        Self::require_active(&self.0.borrow(), lease)
    }
    fn load_fenced(
        &mut self,
        lease: EffectJournalLease,
    ) -> Result<Option<Vec<u8>>, EffectFenceStoreError> {
        let state = self.0.borrow();
        Self::require_active(&state, lease)?;
        Ok(state.bytes.clone())
    }
    fn commit_fenced(
        &mut self,
        lease: EffectJournalLease,
        bytes: &[u8],
    ) -> Result<EffectJournalCommitReceipt, EffectFenceStoreError> {
        let mut state = self.0.borrow_mut();
        Self::require_active(&state, lease)?;
        state.bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new("p05a test checkpoint"))
    }
    fn release(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        let mut state = self.0.borrow_mut();
        Self::require_active(&state, lease)?;
        state.active = None;
        Ok(())
    }
}

fn boot() -> AtomicEventLoop {
    let effect = Effect::Network("api.example.test".into());
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "p05a-reaction".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 7 },
            },
            action_name: "p05a-effect".into(),
            declared_effects: NairEffectSet::from_effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect { effect }],
        },
        Instruction::Halt,
    ]);
    let mut caps = CapabilitySet::new();
    caps.allow(Capability::Network("api.example.test".into()));
    let mut authority = NairReactionAuthority::new();
    authority.set(ReactionSlot(0), caps);
    AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap()
}
fn input() -> InputBatch {
    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Keyboard,
            InputDeviceId(1),
            None,
            InputPayload::Key {
                code: 7,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();
    input.drain()
}

type Profile = FencedProductionEffects<MemoryStore, UnsafeDestination, SigningSource>;
fn fixture(enabled: bool) -> (TempFile, TempFile, Profile, Rc<Cell<u32>>, MemoryStore) {
    let trust_file = TempFile::new("trust");
    let delivery_file = TempFile::new("delivery");
    let key = SigningKey::from_bytes(&[24; 32]).verifying_key().to_bytes();
    let trust = DurableTrustGate::initialize(&trust_file.0, &[(100, key, 1)]).unwrap();
    let hits = Rc::new(Cell::new(0));
    let raw = UnsafeDestination(Rc::clone(&hits));
    let guarded = GuardedEffectBackend::new(
        raw,
        trust,
        SigningSource {
            enabled,
            sequence: 0,
        },
    );
    let durable = ProductionEffectBackend::initialize(guarded, &delivery_file.0).unwrap();
    let store = MemoryStore::default();
    let journal = GovernedFencedEffectJournal::new(
        EffectDeliveryNamespace::new([11; 16]),
        EffectJournalWriterId::new([12; 16]),
        store.clone(),
    );
    let mut authority = EffectDispatchAuthority::new();
    authority.grant(Capability::Network("api.example.test".into()));
    let profile = FencedProductionEffects::new(
        boot(),
        journal,
        GovernedEffectDispatcher::new(authority),
        durable,
        MemoryAnchor::default(),
        AnchorOpenMode::Initialize,
    )
    .unwrap();
    (trust_file, delivery_file, profile, hits, store)
}

#[test]
fn facade_owns_production_backend_and_event_loop() {
    let (_trust, _delivery, profile, hits, _store) = fixture(true);
    assert_eq!(profile.pending_effect_count(), 0);
    assert_eq!(hits.get(), 0);
}
#[test]
fn no_lease_denies_cycle_and_dispatch_before_effect() {
    let (_trust, _delivery, mut profile, hits, _store) = fixture(true);
    assert!(matches!(
        profile.cycle_to(LogicalTime::ZERO, &input()),
        Err(EventLoopError::Fencing(EffectFencingError::LeaseRequired))
    ));
    assert!(matches!(
        profile.dispatch_next(),
        Err(EffectFencingError::LeaseRequired)
    ));
    assert_eq!(profile.pending_effect_count(), 0);
    assert_eq!(hits.get(), 0);
}
#[test]
fn fenced_signed_cycle_executes_once_on_real_kernel_path() {
    let (_trust, _delivery, mut profile, hits, _store) = fixture(true);
    profile.acquire().unwrap();
    profile.cycle_to(LogicalTime::ZERO, &input()).unwrap();
    assert_eq!(profile.pending_effect_count(), 1);
    let receipt = profile.dispatch_next().unwrap().unwrap();
    assert!(receipt.delivery_key.is_some());
    assert!(receipt.delivery_fence.is_some());
    assert_eq!(profile.pending_effect_count(), 0);
    assert_eq!(hits.get(), 1);
    assert!(profile.dispatch_next().unwrap().is_none());
    assert_eq!(hits.get(), 1);
    profile.release().unwrap();
}
#[test]
fn fenced_cycle_without_signature_never_calls_raw_backend() {
    let (_trust, _delivery, mut profile, hits, _store) = fixture(false);
    profile.acquire().unwrap();
    profile.cycle_to(LogicalTime::ZERO, &input()).unwrap();
    assert!(profile.dispatch_next().is_err());
    assert_eq!(profile.pending_effect_count(), 1);
    assert_eq!(hits.get(), 0);
    assert!(profile.dispatch_next().is_err());
    assert_eq!(hits.get(), 0);
}
#[test]
fn failed_lease_acquisition_blocks_all_work() {
    let (_trust, _delivery, mut profile, hits, store) = fixture(true);
    store.0.borrow_mut().fail_acquire = true;
    assert!(profile.acquire().is_err());
    assert!(profile.cycle_to(LogicalTime::ZERO, &input()).is_err());
    assert_eq!(hits.get(), 0);
}
#[test]
fn release_blocks_further_dispatch_and_cycle() {
    let (_trust, _delivery, mut profile, hits, _store) = fixture(true);
    profile.acquire().unwrap();
    profile.release().unwrap();
    assert!(matches!(
        profile.dispatch_next(),
        Err(EffectFencingError::LeaseRequired)
    ));
    assert!(profile.cycle_to(LogicalTime::ZERO, &input()).is_err());
    assert_eq!(hits.get(), 0);
}
#[test]
fn profile_recovery_requires_a_live_lease() {
    let (_trust, _delivery, mut profile, hits, _store) = fixture(true);
    assert!(matches!(
        profile.recover(),
        Err(EffectFencingError::LeaseRequired)
    ));
    assert_eq!(hits.get(), 0);
}
