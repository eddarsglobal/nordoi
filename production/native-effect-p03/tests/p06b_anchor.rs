use ed25519_dalek::{Signer, SigningKey};
use nordoi_kernel::{
    AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, DomainRef, Effect, EffectBackend,
    EffectBackendError, EffectBackendReceipt, EffectDeliveryFence, EffectDeliveryNamespace,
    EffectDispatchAuthority, EffectDispatchRequest, EffectFenceStoreError,
    EffectJournalCommitReceipt, EffectJournalLease, EffectJournalWriterId,
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
    initialize_production_anchor, AnchorOpenMode, AnchorStoreError, MonotonicAnchor,
    ProductionAnchorRecord, RecoveryError,
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
            "nordoi-p06b-{label}-{}-{instant}-{}",
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

#[derive(Default)]
struct AnchorState {
    record: Option<ProductionAnchorRecord>,
    compares: u32,
    fail_on_compare: Option<u32>,
    events: Vec<&'static str>,
}

#[derive(Clone, Default)]
struct SharedAnchor(Rc<RefCell<AnchorState>>);
impl SharedAnchor {
    fn fail_on_compare(&self, compare: u32) {
        self.0.borrow_mut().fail_on_compare = Some(compare);
    }
    fn events(&self) -> Vec<&'static str> {
        self.0.borrow().events.clone()
    }
    fn record(&self) -> Option<ProductionAnchorRecord> {
        self.0.borrow().record
    }
}
impl MonotonicAnchor for SharedAnchor {
    fn read(&mut self) -> Result<Option<ProductionAnchorRecord>, AnchorStoreError> {
        Ok(self.0.borrow().record)
    }
    fn compare_and_advance(
        &mut self,
        expected: Option<ProductionAnchorRecord>,
        next: ProductionAnchorRecord,
    ) -> Result<(), AnchorStoreError> {
        let mut state = self.0.borrow_mut();
        state.compares += 1;
        state.events.push("anchor");
        if state.fail_on_compare == Some(state.compares) || state.record != expected {
            return Err(AnchorStoreError::Conflict);
        }
        match expected {
            None if next.generation() != 1 => return Err(AnchorStoreError::Conflict),
            Some(previous) if next.generation() <= previous.generation() => {
                return Err(AnchorStoreError::Conflict)
            }
            _ => {}
        }
        state.record = Some(next);
        Ok(())
    }
}

struct SigningSource {
    sequence: u64,
}
impl SignedEnvelopeSource for SigningSource {
    fn take(&mut self, request: &EffectDispatchRequest) -> Option<SignedEnvelope> {
        self.sequence += 1;
        let bytes = canonical_dispatch_bytes(request).ok()?;
        let mut envelope = SignedEnvelope {
            root_id: 600,
            epoch: 1,
            sequence: self.sequence,
            intent: Intent::Execute {
                capability: "nordoi/effect/execute".into(),
                resource: "nordoi/dispatch/v1".into(),
                payload_sha256: sha256_payload(&bytes),
            },
            signature: [0; 64],
        };
        envelope.signature = SigningKey::from_bytes(&[41; 32])
            .sign(&envelope.signing_bytes().ok()?)
            .to_bytes();
        Some(envelope)
    }
}

struct Destination {
    hits: Rc<Cell<u32>>,
    events: Rc<RefCell<AnchorState>>,
}
impl EffectBackend for Destination {
    fn supports(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Network(_))
    }
    fn execute(
        &mut self,
        _: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.hits.set(self.hits.get() + 1);
        self.events.borrow_mut().events.push("effect");
        Ok(EffectBackendReceipt::new("p06b destination accepted"))
    }
}

#[derive(Default)]
struct StoreState {
    active: Option<EffectJournalLease>,
    bytes: Option<Vec<u8>>,
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
        Ok(EffectJournalCommitReceipt::new("p06b test checkpoint"))
    }
    fn release(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        let mut state = self.0.borrow_mut();
        Self::require_active(&state, lease)?;
        state.active = None;
        Ok(())
    }
}

fn boot() -> AtomicEventLoop {
    let effect = Effect::Network("anchor.example.test".into());
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "p06b-reaction".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 9 },
            },
            action_name: "p06b-effect".into(),
            declared_effects: NairEffectSet::from_effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect { effect }],
        },
        Instruction::Halt,
    ]);
    let mut caps = CapabilitySet::new();
    caps.allow(Capability::Network("anchor.example.test".into()));
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
                code: 9,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();
    input.drain()
}

type Backend = ProductionEffectBackend<Destination, SigningSource>;
type Profile = FencedProductionEffects<MemoryStore, Destination, SigningSource>;

fn backend(anchor: &SharedAnchor) -> (TempFile, TempFile, Backend, Rc<Cell<u32>>) {
    let trust_file = TempFile::new("trust");
    let delivery_file = TempFile::new("delivery");
    let key = SigningKey::from_bytes(&[41; 32]).verifying_key().to_bytes();
    let trust = DurableTrustGate::initialize(&trust_file.0, &[(600, key, 1)]).unwrap();
    let hits = Rc::new(Cell::new(0));
    let raw = Destination {
        hits: Rc::clone(&hits),
        events: Rc::clone(&anchor.0),
    };
    let guarded = GuardedEffectBackend::new(raw, trust, SigningSource { sequence: 0 });
    let durable = ProductionEffectBackend::initialize(guarded, &delivery_file.0).unwrap();
    (trust_file, delivery_file, durable, hits)
}

fn facade(
    durable: Backend,
    anchor: SharedAnchor,
    mode: AnchorOpenMode,
) -> Result<Profile, RecoveryError> {
    let store = MemoryStore::default();
    let journal = GovernedFencedEffectJournal::new(
        EffectDeliveryNamespace::new([31; 16]),
        EffectJournalWriterId::new([32; 16]),
        store,
    );
    let mut authority = EffectDispatchAuthority::new();
    authority.grant(Capability::Network("anchor.example.test".into()));
    FencedProductionEffects::new(
        boot(),
        journal,
        GovernedEffectDispatcher::new(authority),
        durable,
        anchor,
        mode,
    )
}

fn queue_one(profile: &mut Profile) {
    profile.acquire().unwrap();
    profile.cycle_to(LogicalTime::ZERO, &input()).unwrap();
    assert_eq!(profile.pending_effect_count(), 1);
}

#[test]
fn successful_dispatch_anchors_before_and_after_external_effect() {
    let anchor = SharedAnchor::default();
    let (_trust, _delivery, durable, hits) = backend(&anchor);
    let mut profile = facade(durable, anchor.clone(), AnchorOpenMode::Initialize).unwrap();
    queue_one(&mut profile);
    profile.dispatch_next().unwrap().unwrap();
    assert_eq!(hits.get(), 1);
    assert_eq!(
        anchor.events(),
        vec!["anchor", "anchor", "effect", "anchor"]
    );
    assert_eq!(anchor.record().unwrap().generation(), 3);
}

#[test]
fn pre_effect_anchor_failure_blocks_destination() {
    let anchor = SharedAnchor::default();
    anchor.fail_on_compare(2);
    let (_trust, _delivery, durable, hits) = backend(&anchor);
    let mut profile = facade(durable, anchor.clone(), AnchorOpenMode::Initialize).unwrap();
    queue_one(&mut profile);
    assert!(profile.dispatch_next().is_err());
    assert_eq!(hits.get(), 0);
    assert!(!anchor.events().contains(&"effect"));
}

#[test]
fn post_effect_anchor_failure_never_reexecutes_same_delivery() {
    let anchor = SharedAnchor::default();
    anchor.fail_on_compare(3);
    let (_trust, _delivery, durable, hits) = backend(&anchor);
    let mut profile = facade(durable, anchor.clone(), AnchorOpenMode::Initialize).unwrap();
    queue_one(&mut profile);
    assert!(profile.dispatch_next().is_err());
    assert_eq!(hits.get(), 1);
    assert_eq!(
        anchor.events(),
        vec!["anchor", "anchor", "effect", "anchor"]
    );
    assert!(profile.dispatch_next().is_err());
    assert_eq!(hits.get(), 1);
}

#[test]
fn recovery_mode_rejects_missing_anchor_before_facade_exists() {
    let anchor = SharedAnchor::default();
    let (_trust, _delivery, durable, hits) = backend(&anchor);
    assert!(matches!(
        facade(durable, anchor, AnchorOpenMode::Recover),
        Err(RecoveryError::MissingAnchor)
    ));
    assert_eq!(hits.get(), 0);
}

#[test]
fn initialize_mode_rejects_already_initialized_anchor() {
    let mut anchor = SharedAnchor::default();
    let (_trust, _delivery, durable, hits) = backend(&anchor);
    initialize_production_anchor(&mut anchor, &durable).unwrap();
    assert!(matches!(
        facade(durable, anchor, AnchorOpenMode::Initialize),
        Err(RecoveryError::AlreadyInitialized)
    ));
    assert_eq!(hits.get(), 0);
}

#[test]
fn exact_recovery_then_dispatch_uses_per_dispatch_barriers() {
    let mut anchor = SharedAnchor::default();
    let (_trust, _delivery, durable, hits) = backend(&anchor);
    initialize_production_anchor(&mut anchor, &durable).unwrap();
    let before = anchor.record().unwrap();
    let mut profile = facade(durable, anchor.clone(), AnchorOpenMode::Recover).unwrap();
    queue_one(&mut profile);
    profile.dispatch_next().unwrap().unwrap();
    assert_eq!(hits.get(), 1);
    assert_eq!(
        anchor.record().unwrap().generation(),
        before.generation() + 2
    );
    assert_eq!(
        anchor.events(),
        vec!["anchor", "anchor", "effect", "anchor"]
    );
}
