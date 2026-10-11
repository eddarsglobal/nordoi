use ed25519_dalek::{Signer, SigningKey};
use nordoi_kernel::{
    Effect, EffectBackend, EffectBackendError, EffectBackendReceipt, EffectDeliveryFence,
    EffectDeliveryKey, EffectDeliveryNamespace, EffectDispatchRequest, EffectIntentId,
    QueuedEffectIntent, ReactionId,
};
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

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct TempJournal(PathBuf);
impl TempJournal {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "nordoi-p03-{}-{}-{}.bin",
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
fn key() -> SigningKey {
    SigningKey::from_bytes(&[7; 32])
}
fn roots() -> [(u64, [u8; 32], u64); 1] {
    [(10, key().verifying_key().to_bytes(), 1)]
}
fn request() -> EffectDispatchRequest {
    let id = EffectIntentId(47);
    EffectDispatchRequest {
        queued: QueuedEffectIntent {
            id,
            cycle: 2,
            ordinal: 0,
            intent: nordoi_kernel::EffectIntent {
                reaction: ReactionId(3),
                action_name: "network-call".into(),
                effect: Effect::Network("api.example.test".into()),
            },
        },
        delivery_key: Some(EffectDeliveryKey::new(
            EffectDeliveryNamespace::new([9; 16]),
            id,
        )),
        delivery_fence: Some(EffectDeliveryFence(3)),
    }
}
fn signed(req: &EffectDispatchRequest, sequence: u64) -> SignedEnvelope {
    let payload = canonical_dispatch_bytes(req).unwrap();
    let mut envelope = SignedEnvelope {
        root_id: 10,
        epoch: 1,
        sequence,
        intent: Intent::Execute {
            capability: "nordoi/effect/execute".into(),
            resource: "nordoi/dispatch/v1".into(),
            payload_sha256: sha256_payload(&payload),
        },
        signature: [0; 64],
    };
    envelope.signature = key().sign(&envelope.signing_bytes().unwrap()).to_bytes();
    envelope
}
struct Envelopes(VecDeque<SignedEnvelope>);
impl SignedEnvelopeSource for Envelopes {
    fn take(&mut self, _request: &EffectDispatchRequest) -> Option<SignedEnvelope> {
        self.0.pop_front()
    }
}
#[derive(Clone)]
struct CountingBackend {
    calls: Rc<Cell<u32>>,
    fail: bool,
}
impl EffectBackend for CountingBackend {
    fn supports(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Network(_))
    }
    fn execute(
        &mut self,
        _request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            Err(EffectBackendError::new("simulated uncertain delivery"))
        } else {
            Ok(EffectBackendReceipt::new("tested"))
        }
    }
}
fn fixture(
    envelopes: Vec<SignedEnvelope>,
    fail: bool,
) -> (
    TempJournal,
    GuardedEffectBackend<CountingBackend, Envelopes>,
    Rc<Cell<u32>>,
) {
    let p = TempJournal::new();
    let trust = DurableTrustGate::initialize(&p.0, &roots()).unwrap();
    let calls = Rc::new(Cell::new(0));
    let backend = CountingBackend {
        calls: Rc::clone(&calls),
        fail,
    };
    (
        p,
        GuardedEffectBackend::new(backend, trust, Envelopes(envelopes.into())),
        calls,
    )
}
#[test]
fn real_kernel_effect_backend_trait_is_implemented() {
    fn accepts_kernel_backend<B: EffectBackend>(_value: &B) {}
    let (_path, backend, _calls) = fixture(vec![], false);
    accepts_kernel_backend(&backend);
}
#[test]
fn no_envelope_denies_before_backend_execution() {
    let (_path, mut backend, calls) = fixture(vec![], false);
    assert!(backend.execute_with_context(&request()).is_err());
    assert_eq!(calls.get(), 0);
    assert_eq!(backend.root_status(10), Some((1, 0, false)));
}
#[test]
fn legacy_contextless_execution_always_denies() {
    let (_path, mut backend, calls) = fixture(vec![], false);
    assert!(backend.execute(&request().queued).is_err());
    assert_eq!(calls.get(), 0);
}
#[test]
fn signed_correct_delivery_executes_once() {
    let r = request();
    let (_path, mut backend, calls) = fixture(vec![signed(&r, 1)], false);
    backend.execute_with_context(&r).unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(backend.root_status(10), Some((1, 1, false)));
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 1);
}
#[test]
fn wrong_signature_denies_and_does_not_consume() {
    let r = request();
    let mut envelope = signed(&r, 1);
    envelope.signature[9] ^= 2;
    let (_path, mut backend, calls) = fixture(vec![envelope], false);
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 0);
    assert_eq!(backend.root_status(10), Some((1, 0, false)));
}
#[test]
fn signed_action_replacement_denies() {
    let r = request();
    let mut altered = r.clone();
    altered.queued.intent.action_name = "other-action".into();
    let (_path, mut backend, calls) = fixture(vec![signed(&r, 1)], false);
    assert!(backend.execute_with_context(&altered).is_err());
    assert_eq!(calls.get(), 0);
}
#[test]
fn signed_scope_replacement_denies() {
    let r = request();
    let mut altered = r.clone();
    altered.queued.intent.effect = Effect::Network("attacker.example.test".into());
    let (_path, mut backend, calls) = fixture(vec![signed(&r, 1)], false);
    assert!(backend.execute_with_context(&altered).is_err());
    assert_eq!(calls.get(), 0);
}
#[test]
fn delivery_namespace_or_fence_replacement_denies() {
    let r = request();
    for case in 0..2 {
        let mut altered = r.clone();
        if case == 0 {
            altered.delivery_key = Some(EffectDeliveryKey::new(
                EffectDeliveryNamespace::new([5; 16]),
                r.queued.id,
            ));
        } else {
            altered.delivery_fence = Some(EffectDeliveryFence(4));
        }
        let (_path, mut backend, calls) = fixture(vec![signed(&r, 1)], false);
        assert!(backend.execute_with_context(&altered).is_err());
        assert_eq!(calls.get(), 0);
    }
}
#[test]
fn mismatched_delivery_key_denies_even_with_signed_payload() {
    let mut r = request();
    r.delivery_key = Some(EffectDeliveryKey::new(
        EffectDeliveryNamespace::new([9; 16]),
        EffectIntentId(12),
    ));
    assert!(canonical_dispatch_bytes(&r).is_err());
    let (_path, mut backend, calls) = fixture(vec![], false);
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 0);
}
#[test]
fn missing_fence_or_key_denies() {
    for case in 0..2 {
        let mut r = request();
        if case == 0 {
            r.delivery_key = None;
        } else {
            r.delivery_fence = None;
        }
        let (_path, mut backend, calls) = fixture(vec![], false);
        assert!(backend.execute_with_context(&r).is_err());
        assert_eq!(calls.get(), 0);
    }
}
#[test]
fn internal_effect_denied_even_with_valid_delivery_key() {
    let mut r = request();
    r.queued.intent.effect = Effect::StateWrite;
    assert!(canonical_dispatch_bytes(&r).is_err());
    let (_path, mut backend, calls) = fixture(vec![], false);
    assert!(!backend.supports(&Effect::StateWrite));
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 0);
}
#[test]
fn backend_failure_never_refunds_consumed_sequence() {
    let r = request();
    let (_path, mut backend, calls) = fixture(vec![signed(&r, 1), signed(&r, 1)], true);
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 1);
    assert_eq!(backend.root_status(10), Some((1, 1, false)));
    assert!(backend.execute_with_context(&r).is_err());
    assert_eq!(calls.get(), 1);
}
#[test]
fn journal_restart_rejects_old_envelope() {
    let r = request();
    let (path, mut backend, calls) = fixture(vec![signed(&r, 1)], false);
    backend.execute_with_context(&r).unwrap();
    assert_eq!(calls.get(), 1);
    drop(backend);
    let recovered = DurableTrustGate::open(&path.0, &roots()).unwrap();
    let new_calls = Rc::new(Cell::new(0));
    let mut after = GuardedEffectBackend::new(
        CountingBackend {
            calls: Rc::clone(&new_calls),
            fail: false,
        },
        recovered,
        Envelopes(vec![signed(&r, 1)].into()),
    );
    assert!(after.execute_with_context(&r).is_err());
    assert_eq!(new_calls.get(), 0);
}
#[test]
fn existing_kernel_dispatch_entry_denies_legacy_path() {
    use nordoi_kernel::{
        AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, DomainRef,
        EffectDispatchAuthority, GovernedEffectDispatcher, InputDeviceId, InputPayload,
        InputSignal, InputSource, InputTargetRef, Instruction, LogicalTime, NairEffectSet,
        NairProgram, NairReactionAuthority, NairReactionStep, NairReactionTrigger, ReactionSlot,
    };
    let effect = Effect::Network("api.example.test".into());
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "signed-effect-boundary".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 7 },
            },
            action_name: "network-call".into(),
            declared_effects: NairEffectSet::from_effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect { effect }],
        },
        Instruction::Halt,
    ]);
    let mut capabilities = CapabilitySet::new();
    capabilities.allow(Capability::Network("api.example.test".into()));
    let mut reaction_authority = NairReactionAuthority::new();
    reaction_authority.set(ReactionSlot(0), capabilities);
    let mut loop_ =
        AtomicEventLoop::boot_with_reaction_authority(&program, &reaction_authority).unwrap();
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
    loop_.cycle_to(LogicalTime::ZERO, &input.drain()).unwrap();
    let mut dispatcher_authority = EffectDispatchAuthority::new();
    dispatcher_authority.grant(Capability::Network("api.example.test".into()));
    let dispatcher = GovernedEffectDispatcher::new(dispatcher_authority);
    let (_path, mut guarded, calls) = fixture(vec![], false);
    assert!(loop_
        .dispatch_next_effect(&dispatcher, &mut guarded)
        .is_err());
    assert_eq!(loop_.pending_effect_count(), 1);
    assert_eq!(calls.get(), 0);
}

#[test]
fn canonical_binding_is_stable_and_sensitive_to_payload_identity() {
    let r = request();
    let once = canonical_dispatch_bytes(&r).unwrap();
    assert_eq!(once, canonical_dispatch_bytes(&r).unwrap());
    let mut changed = r.clone();
    changed.queued.cycle += 1;
    assert_ne!(once, canonical_dispatch_bytes(&changed).unwrap());
    changed = r.clone();
    changed.queued.ordinal += 1;
    assert_ne!(once, canonical_dispatch_bytes(&changed).unwrap());
    changed = r.clone();
    changed.queued.intent.reaction = ReactionId(5);
    assert_ne!(once, canonical_dispatch_bytes(&changed).unwrap());
}
