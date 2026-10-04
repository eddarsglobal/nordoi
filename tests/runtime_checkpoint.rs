use std::{cell::RefCell, rc::Rc};

use nordoi_kernel::{
    AtomSlot, AtomicEventLoop, Capability, CapabilitySet, DirtyMask, DomainId, DomainRef, Effect,
    EffectAttemptId, EffectAuditDispatchOutcome, EffectBackend, EffectBackendError,
    EffectBackendReceipt, EffectCompletion, EffectCompletionBatch, EffectCompletionError,
    EffectCompletionOutcome, EffectCompletionProjection, EffectCompletionProjectionValue,
    EffectCompletionSequence, EffectCompletionSourceId, EffectDeliveryFence, EffectDeliveryKey,
    EffectDeliveryNamespace, EffectDispatchAuthority, EffectFenceStoreError,
    EffectJournalCommitReceipt, EffectJournalLease, EffectJournalWriterId, EffectRetryPolicy,
    EffectRetryTick, EventLoopError, FencedEffectJournalStore, FencedRuntimeCheckpointStore,
    GovernedAuditedEffectJournal, GovernedEffectDispatcher, InputBatch, InputBridgeSlot,
    InputDeviceId, InputEvent, InputPayload, InputSequence, InputSignal, InputSource, InputTarget,
    InputTargetRef, Instruction, LogicalDuration, LogicalTime, NairEffectSet, NairProgram,
    NairReactionAuthority, NairReactionStep, NairReactionTrigger, ReactionSlot, RegisterId,
    RenderNodeSlot, RenderPrimitive, RenderSpace, RuntimeCheckpointCommitReceipt,
    RuntimeCheckpointError, RuntimeCheckpointStoreError, RuntimeError, RuntimeSemanticCheckpoint,
    Value,
};

const SOURCE: EffectCompletionSourceId = EffectCompletionSourceId(9);

fn namespace(seed: u8) -> EffectDeliveryNamespace {
    EffectDeliveryNamespace::new([seed; 16])
}

fn writer(seed: u8) -> EffectJournalWriterId {
    EffectJournalWriterId::new([seed; 16])
}

fn policy() -> EffectRetryPolicy {
    EffectRetryPolicy::new(3, 5, 20, 0).unwrap()
}

fn key_batch(sequence: u64, pressed: bool, code: u32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(sequence),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed,
                repeat: false,
            },
        }],
    }
}

fn interactive_program() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateRenderNode {
            dst: RenderNodeSlot(0),
            primitive: RenderPrimitive::Text,
            space: RenderSpace::Screen,
        },
        Instruction::BindRenderAtom {
            atom: AtomSlot(0),
            node: RenderNodeSlot(0),
            dirty: DirtyMask::CONTENT,
        },
        Instruction::RenderFlush,
        Instruction::CreateInputBridge {
            dst: InputBridgeSlot(0),
            domain: DomainRef::Root,
        },
        Instruction::BindInputAtom {
            bridge: InputBridgeSlot(0),
            atom: AtomSlot(0),
            source: Some(InputSource::Keyboard),
            device: None,
            target: InputTargetRef::Global,
            signal: InputSignal::KeyPressed { code: 13 },
        },
        Instruction::ApplyInput {
            bridge: InputBridgeSlot(0),
        },
        Instruction::RenderFlush,
        Instruction::Halt,
    ])
}

fn effect_program() -> (NairProgram, NairReactionAuthority) {
    let effect = Effect::Network("api.example.test".into());
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Text("initial".into()),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "runtime-checkpoint-effect".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 7 },
            },
            action_name: "runtime-checkpoint-action".into(),
            declared_effects: NairEffectSet::from_effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect { effect }],
        },
        Instruction::Halt,
    ]);
    let mut capabilities = CapabilitySet::new();
    capabilities.allow(Capability::Network("api.example.test".into()));
    let mut authority = NairReactionAuthority::new();
    authority.set(ReactionSlot(0), capabilities);
    (program, authority)
}

fn dispatcher() -> GovernedEffectDispatcher {
    let mut authority = EffectDispatchAuthority::new();
    authority.grant(Capability::Network("api.example.test".into()));
    GovernedEffectDispatcher::new(authority)
}

#[derive(Debug, Default)]
struct StoreState {
    effect_bytes: Option<Vec<u8>>,
    runtime_bytes: Option<Vec<u8>>,
    next_fence: u64,
    active: Option<EffectJournalLease>,
    effect_commits: usize,
    bundle_commits: usize,
    fail_bundle: bool,
}

#[derive(Debug, Clone, Default)]
struct MemoryRuntimeStore {
    state: Rc<RefCell<StoreState>>,
}

impl MemoryRuntimeStore {
    fn require_active(
        state: &StoreState,
        lease: EffectJournalLease,
    ) -> Result<(), EffectFenceStoreError> {
        if state.active == Some(lease) {
            Ok(())
        } else {
            Err(EffectFenceStoreError::new("stale or inactive fence"))
        }
    }

    fn fail_next_bundle(&self) {
        self.state.borrow_mut().fail_bundle = true;
    }

    fn runtime_bytes(&self) -> Option<Vec<u8>> {
        self.state.borrow().runtime_bytes.clone()
    }

    fn effect_bytes(&self) -> Option<Vec<u8>> {
        self.state.borrow().effect_bytes.clone()
    }

    fn tamper_runtime(&self) {
        let mut state = self.state.borrow_mut();
        let bytes = state.runtime_bytes.as_mut().unwrap();
        bytes[40] ^= 0x5a;
    }

    fn clear_runtime(&self) {
        self.state.borrow_mut().runtime_bytes = None;
    }

    fn bundle_commits(&self) -> usize {
        self.state.borrow().bundle_commits
    }
}

impl FencedEffectJournalStore for MemoryRuntimeStore {
    fn acquire(
        &mut self,
        namespace: EffectDeliveryNamespace,
        writer: EffectJournalWriterId,
    ) -> Result<EffectJournalLease, EffectFenceStoreError> {
        let mut state = self.state.borrow_mut();
        state.next_fence = state
            .next_fence
            .checked_add(1)
            .ok_or_else(|| EffectFenceStoreError::new("fence exhausted"))?;
        let lease =
            EffectJournalLease::new(namespace, writer, EffectDeliveryFence(state.next_fence));
        state.active = Some(lease);
        Ok(lease)
    }

    fn assert_active(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        Self::require_active(&self.state.borrow(), lease)
    }

    fn load_fenced(
        &mut self,
        lease: EffectJournalLease,
    ) -> Result<Option<Vec<u8>>, EffectFenceStoreError> {
        let state = self.state.borrow();
        Self::require_active(&state, lease)?;
        Ok(state.effect_bytes.clone())
    }

    fn commit_fenced(
        &mut self,
        lease: EffectJournalLease,
        bytes: &[u8],
    ) -> Result<EffectJournalCommitReceipt, EffectFenceStoreError> {
        let mut state = self.state.borrow_mut();
        Self::require_active(&state, lease)?;
        state.effect_commits += 1;
        state.effect_bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new(format!(
            "effect-{}-{}",
            lease.fence.0, state.effect_commits
        )))
    }

    fn release(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        let mut state = self.state.borrow_mut();
        Self::require_active(&state, lease)?;
        state.active = None;
        Ok(())
    }
}

impl FencedRuntimeCheckpointStore for MemoryRuntimeStore {
    fn load_runtime_fenced(
        &mut self,
        lease: EffectJournalLease,
    ) -> Result<Option<Vec<u8>>, RuntimeCheckpointStoreError> {
        let state = self.state.borrow();
        if state.active != Some(lease) {
            return Err(RuntimeCheckpointStoreError::new("stale runtime fence"));
        }
        Ok(state.runtime_bytes.clone())
    }

    fn commit_effect_and_runtime_fenced(
        &mut self,
        lease: EffectJournalLease,
        effect_checkpoint: &[u8],
        runtime_checkpoint: &[u8],
    ) -> Result<RuntimeCheckpointCommitReceipt, RuntimeCheckpointStoreError> {
        let mut state = self.state.borrow_mut();
        if state.active != Some(lease) {
            return Err(RuntimeCheckpointStoreError::new("stale runtime fence"));
        }
        if state.fail_bundle {
            state.fail_bundle = false;
            return Err(RuntimeCheckpointStoreError::new("synthetic bundle failure"));
        }
        state.bundle_commits += 1;
        state.effect_bytes = Some(effect_checkpoint.to_vec());
        state.runtime_bytes = Some(runtime_checkpoint.to_vec());
        Ok(RuntimeCheckpointCommitReceipt::new(format!(
            "bundle-{}-{}",
            lease.fence.0, state.bundle_commits
        )))
    }
}

#[derive(Debug)]
struct SuccessBackend(&'static str);

impl EffectBackend for SuccessBackend {
    fn supports(&self, _effect: &Effect) -> bool {
        true
    }

    fn execute(
        &mut self,
        _request: &nordoi_kernel::QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        Ok(EffectBackendReceipt::new(self.0))
    }
}

fn active_journal(
    seed: u8,
    writer_seed: u8,
    store: MemoryRuntimeStore,
) -> GovernedAuditedEffectJournal<MemoryRuntimeStore> {
    let mut journal =
        GovernedAuditedEffectJournal::new(namespace(seed), writer(writer_seed), store, policy());
    journal.acquire().unwrap();
    journal
}

fn boot_effect() -> AtomicEventLoop {
    let (program, authority) = effect_program();
    AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap()
}

fn delivered_fixture(
    seed: u8,
) -> (
    MemoryRuntimeStore,
    GovernedAuditedEffectJournal<MemoryRuntimeStore>,
    AtomicEventLoop,
    EffectDeliveryKey,
    EffectAttemptId,
) {
    delivered_fixture_with_reference(seed, "remote-ok")
}

fn delivered_fixture_with_reference(
    seed: u8,
    backend_reference: &'static str,
) -> (
    MemoryRuntimeStore,
    GovernedAuditedEffectJournal<MemoryRuntimeStore>,
    AtomicEventLoop,
    EffectDeliveryKey,
    EffectAttemptId,
) {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(seed, 1, store.clone());
    let mut event_loop = boot_effect();
    event_loop
        .cycle_to_with_runtime_checkpoint(LogicalTime::ZERO, &key_batch(1, true, 7), &mut journal)
        .unwrap();
    let mut backend = SuccessBackend(backend_reference);
    let outcome = event_loop
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(),
            &mut backend,
        )
        .unwrap()
        .unwrap();
    let EffectAuditDispatchOutcome::Delivered { attempt, receipt } = outcome else {
        panic!("expected delivered effect")
    };
    (
        store,
        journal,
        event_loop,
        receipt.delivery_key.unwrap(),
        attempt,
    )
}

fn configure_projection(event_loop: &mut AtomicEventLoop, ns: EffectDeliveryNamespace) {
    let atom = event_loop.snapshot().unwrap()[&AtomSlot(0)].id;
    event_loop
        .grant_effect_completion_source(SOURCE, ns)
        .unwrap();
    event_loop
        .register_effect_completion_projection(EffectCompletionProjection::new(
            SOURCE,
            ns,
            DomainId(0),
            atom,
            EffectCompletionProjectionValue::OutcomeValue,
        ))
        .unwrap();
}

fn completion(
    sequence: u64,
    key: EffectDeliveryKey,
    attempt: EffectAttemptId,
    value: &str,
) -> EffectCompletionBatch {
    EffectCompletionBatch::new(vec![EffectCompletion::new(
        SOURCE,
        EffectCompletionSequence(sequence),
        key,
        attempt,
        EffectCompletionOutcome::success(value),
    )])
}

#[test]
fn runtime_checkpoint_round_trip_is_byte_stable() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(1, 1, store);
    let mut event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    event_loop
        .cycle_to_with_runtime_checkpoint(LogicalTime(5), &key_batch(1, true, 13), &mut journal)
        .unwrap();
    let bytes = event_loop
        .semantic_checkpoint(&journal)
        .unwrap()
        .canonical_bytes()
        .unwrap();
    let decoded = RuntimeSemanticCheckpoint::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded.canonical_bytes().unwrap(), bytes);
    assert_eq!(decoded.cycle(), 1);
    assert_eq!(decoded.logical_time(), LogicalTime(5));
}

#[test]
fn durable_cycle_commits_effect_and_runtime_together() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(2, 1, store.clone());
    let mut event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    event_loop
        .cycle_to_with_runtime_checkpoint(LogicalTime(5), &key_batch(1, true, 13), &mut journal)
        .unwrap();
    assert_eq!(store.bundle_commits(), 1);
    assert!(store.effect_bytes().is_some());
    assert!(store.runtime_bytes().is_some());
}

#[test]
fn failed_bundle_commit_publishes_no_runtime_progress() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(3, 1, store.clone());
    let mut event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    let replay = event_loop.replay_key();
    store.fail_next_bundle();
    assert!(event_loop
        .cycle_to_with_runtime_checkpoint(LogicalTime(5), &key_batch(1, true, 13), &mut journal)
        .is_err());
    assert_eq!(event_loop.cycle_index(), 0);
    assert_eq!(event_loop.logical_time(), LogicalTime::ZERO);
    assert_eq!(event_loop.replay_key(), replay);
    assert_eq!(
        event_loop.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(false)
    );
    assert!(store.effect_bytes().is_none());
    assert!(store.runtime_bytes().is_none());
}

#[test]
fn recovery_restores_atom_time_cycle_and_replay_exactly() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(4, 1, store.clone());
    let mut original = AtomicEventLoop::boot(&interactive_program()).unwrap();
    original
        .cycle_to_with_runtime_checkpoint(LogicalTime(7), &key_batch(1, true, 13), &mut journal)
        .unwrap();
    let replay = original.replay_key();
    let version = original.snapshot().unwrap()[&AtomSlot(0)].version;

    let mut recovered_journal = active_journal(4, 2, store);
    let mut recovered = AtomicEventLoop::boot(&interactive_program()).unwrap();
    let report = recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap()
        .unwrap();
    assert_eq!(report.cycle, 1);
    assert_eq!(recovered.cycle_index(), 1);
    assert_eq!(recovered.logical_time(), LogicalTime(7));
    assert_eq!(recovered.replay_key(), replay);
    assert_eq!(
        recovered.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(true)
    );
    assert_eq!(recovered.snapshot().unwrap()[&AtomSlot(0)].version, version);
}

#[test]
fn recovered_runtime_preserves_cross_tick_input_sequence() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(5, 1, store.clone());
    let mut original = AtomicEventLoop::boot(&interactive_program()).unwrap();
    original
        .cycle_to_with_runtime_checkpoint(LogicalTime(1), &key_batch(5, true, 13), &mut journal)
        .unwrap();

    let mut recovered_journal = active_journal(5, 2, store);
    let mut recovered = AtomicEventLoop::boot(&interactive_program()).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap();
    assert!(matches!(
        recovered.cycle_to(LogicalTime(2), &key_batch(5, false, 13)),
        Err(EventLoopError::Runtime(RuntimeError::Input(_)))
    ));
    assert_eq!(recovered.cycle_index(), 1);
}

#[test]
fn recovered_timer_continues_occurrence_sequence() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(6, 1, store.clone());
    let mut original = AtomicEventLoop::boot(&interactive_program()).unwrap();
    let timer = original
        .schedule_repeating_at(LogicalTime(5), LogicalDuration(5))
        .unwrap();
    original
        .cycle_to_with_runtime_checkpoint(LogicalTime(12), &InputBatch::default(), &mut journal)
        .unwrap();
    assert_eq!(original.timer_snapshot(timer).unwrap().occurrences, 2);

    let mut recovered_journal = active_journal(6, 2, store);
    let mut recovered = AtomicEventLoop::boot(&interactive_program()).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap();
    assert_eq!(
        recovered.timer_snapshot(timer).unwrap().next_deadline,
        LogicalTime(15)
    );
    let report = recovered
        .cycle_to(LogicalTime(16), &InputBatch::default())
        .unwrap();
    assert_eq!(report.time.fires.len(), 1);
    assert_eq!(report.time.fires[0].occurrence, 3);
}

#[test]
fn recovered_trace_continues_with_identical_replay_identity() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(7, 1, store.clone());
    let mut original = AtomicEventLoop::boot(&interactive_program()).unwrap();
    original
        .schedule_repeating_at(LogicalTime(4), LogicalDuration(4))
        .unwrap();
    original
        .cycle_to_with_runtime_checkpoint(LogicalTime(5), &key_batch(1, true, 13), &mut journal)
        .unwrap();

    let mut recovered_journal = active_journal(7, 2, store);
    let mut recovered = AtomicEventLoop::boot(&interactive_program()).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap();

    let a = original
        .cycle_to(LogicalTime(9), &key_batch(2, false, 13))
        .unwrap();
    let b = recovered
        .cycle_to(LogicalTime(9), &key_batch(2, false, 13))
        .unwrap();
    assert_eq!(a.replay_key, b.replay_key);
    assert_eq!(a.runtime.final_atoms, b.runtime.final_atoms);
    assert_eq!(a.time, b.time);
}

#[test]
fn render_revision_continues_after_recovery() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(8, 1, store.clone());
    let mut original = AtomicEventLoop::boot(&interactive_program()).unwrap();
    original
        .cycle_to_with_runtime_checkpoint(LogicalTime(1), &key_batch(1, true, 13), &mut journal)
        .unwrap();

    let mut recovered_journal = active_journal(8, 2, store);
    let mut recovered = AtomicEventLoop::boot(&interactive_program()).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap();
    let a = original
        .cycle_to(LogicalTime(2), &key_batch(2, false, 13))
        .unwrap();
    let b = recovered
        .cycle_to(LogicalTime(2), &key_batch(2, false, 13))
        .unwrap();
    let a_revision = a.runtime.frame.unwrap().batch.updates[0].revision;
    let b_revision = b.runtime.frame.unwrap().batch.updates[0].revision;
    assert_eq!(a_revision, b_revision);
}

#[test]
fn recovery_is_bootstrap_only() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(9, 1, store.clone());
    let mut source = AtomicEventLoop::boot(&interactive_program()).unwrap();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime(1), &key_batch(1, true, 13), &mut journal)
        .unwrap();
    let mut takeover = active_journal(9, 2, store);
    let mut already_running = AtomicEventLoop::boot(&interactive_program()).unwrap();
    already_running
        .cycle_to(LogicalTime(1), &InputBatch::default())
        .unwrap();
    assert!(matches!(
        already_running.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::RecoveryAfterCycleStarted { .. }
        ))
    ));
}

#[test]
fn empty_store_recovery_is_none() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(10, 1, store);
    let mut event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    assert!(event_loop
        .recover_runtime_from_audited_journal(&mut journal)
        .unwrap()
        .is_none());
}

#[test]
fn effect_only_checkpoint_without_runtime_half_fails_closed() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(11, 1, store.clone());
    let event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    journal
        .checkpoint(&nordoi_kernel::AtomicEffectOutbox::new())
        .unwrap();
    assert!(store.effect_bytes().is_some());
    assert!(store.runtime_bytes().is_none());

    let mut takeover = active_journal(11, 2, store);
    let mut recovered = event_loop.clone();
    assert!(matches!(
        recovered.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::RecoveryBundleIncomplete
        ))
    ));
}

#[test]
fn tampered_runtime_checkpoint_is_rejected() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(12, 1, store.clone());
    let mut source = AtomicEventLoop::boot(&interactive_program()).unwrap();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime(1), &key_batch(1, true, 13), &mut journal)
        .unwrap();
    store.tamper_runtime();
    let mut takeover = active_journal(12, 2, store);
    let mut recovered = AtomicEventLoop::boot(&interactive_program()).unwrap();
    assert!(matches!(
        recovered.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::DigestMismatch { .. }
        ))
    ));
}

#[test]
fn different_program_cannot_recover_checkpoint() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(13, 1, store.clone());
    let mut source = AtomicEventLoop::boot(&interactive_program()).unwrap();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime(1), &key_batch(1, true, 13), &mut journal)
        .unwrap();
    let different = NairProgram::from_instructions(vec![Instruction::Halt]);
    let mut takeover = active_journal(13, 2, store);
    let mut recovered = AtomicEventLoop::boot(&different).unwrap();
    assert!(matches!(
        recovered.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::ProgramMismatch
        ))
    ));
}

#[test]
fn fire_budget_mismatch_is_rejected() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(14, 1, store.clone());
    let mut source = AtomicEventLoop::boot_with_fire_budget(&interactive_program(), 17).unwrap();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime(1), &InputBatch::default(), &mut journal)
        .unwrap();
    let mut takeover = active_journal(14, 2, store);
    let mut recovered = AtomicEventLoop::boot_with_fire_budget(&interactive_program(), 18).unwrap();
    assert!(matches!(
        recovered.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::FireBudgetMismatch { .. }
        ))
    ));
}

#[test]
fn stale_writer_cannot_publish_runtime_bundle() {
    let store = MemoryRuntimeStore::default();
    let mut stale = active_journal(15, 1, store.clone());
    let _takeover = active_journal(15, 2, store);
    let mut event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    assert!(event_loop
        .cycle_to_with_runtime_checkpoint(LogicalTime(1), &key_batch(1, true, 13), &mut stale)
        .is_err());
    assert_eq!(event_loop.cycle_index(), 0);
}

#[test]
fn newer_writer_can_recover_same_runtime_checkpoint() {
    let store = MemoryRuntimeStore::default();
    let mut first = active_journal(16, 1, store.clone());
    let mut source = AtomicEventLoop::boot(&interactive_program()).unwrap();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime(3), &key_batch(1, true, 13), &mut first)
        .unwrap();
    let first_fence = first.lease().unwrap().fence;
    let mut takeover = active_journal(16, 2, store);
    assert!(takeover.lease().unwrap().fence > first_fence);
    let mut recovered = AtomicEventLoop::boot(&interactive_program()).unwrap();
    assert!(recovered
        .recover_runtime_from_audited_journal(&mut takeover)
        .unwrap()
        .is_some());
    assert_eq!(recovered.logical_time(), LogicalTime(3));
}

#[test]
fn later_effect_audit_descendant_does_not_invalidate_runtime_checkpoint() {
    let (store, journal, event_loop, _key, _attempt) = delivered_fixture(17);
    assert_eq!(journal.audit_ledger().len(), 2);
    let mut takeover = active_journal(17, 2, store);
    let mut recovered = boot_effect();
    recovered
        .recover_runtime_from_audited_journal(&mut takeover)
        .unwrap();
    assert_eq!(recovered.cycle_index(), event_loop.cycle_index());
    assert_eq!(recovered.pending_effect_count(), 0);
    assert_eq!(takeover.audit_ledger().len(), 2);
}

#[test]
fn effect_intent_frontier_detects_non_durable_later_cycle() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(18, 1, store.clone());
    let mut source = boot_effect();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime::ZERO, &InputBatch::default(), &mut journal)
        .unwrap();
    source
        .cycle_to_with_audited_effect_journal(
            LogicalTime::ZERO,
            &key_batch(1, true, 7),
            &mut journal,
        )
        .unwrap();
    let mut takeover = active_journal(18, 2, store);
    let mut recovered = boot_effect();
    assert!(matches!(
        recovered.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::EffectIntentSequenceMismatch { .. }
        ))
    ));
}

#[test]
fn shorter_audit_history_than_runtime_prefix_is_rejected() {
    let (store, mut journal, mut event_loop, _key, _attempt) = delivered_fixture(19);
    event_loop
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let runtime_bytes = store.runtime_bytes().unwrap();

    let clean_store = MemoryRuntimeStore::default();
    let mut clean_journal = active_journal(19, 3, clean_store.clone());
    let mut clean_loop = boot_effect();
    clean_loop
        .cycle_to_with_runtime_checkpoint(
            LogicalTime::ZERO,
            &key_batch(1, true, 7),
            &mut clean_journal,
        )
        .unwrap();
    let shorter_effect = clean_store.effect_bytes().unwrap();
    {
        let mut state = store.state.borrow_mut();
        state.effect_bytes = Some(shorter_effect);
        state.runtime_bytes = Some(runtime_bytes);
    }

    let mut takeover = active_journal(19, 4, store);
    let mut recovered = boot_effect();
    assert!(matches!(
        recovered.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::AuditHistoryTooShort { .. }
        ))
    ));
}

#[test]
fn divergent_valid_audit_prefix_is_rejected() {
    let (store_a, mut journal_a, mut loop_a, _key_a, _attempt_a) = delivered_fixture(20);
    loop_a
        .checkpoint_runtime_with_audited_journal(&mut journal_a)
        .unwrap();
    let runtime_bytes = store_a.runtime_bytes().unwrap();

    let (store_b, _journal_b, _loop_b, _key_b, _attempt_b) =
        delivered_fixture_with_reference(20, "different-valid-branch");
    let effect_b = store_b.effect_bytes().unwrap();
    assert_ne!(effect_b, store_a.effect_bytes().unwrap());
    {
        let mut state = store_a.state.borrow_mut();
        state.effect_bytes = Some(effect_b);
        state.runtime_bytes = Some(runtime_bytes);
    }

    let mut takeover = active_journal(20, 5, store_a);
    let mut recovered = boot_effect();
    assert!(matches!(
        recovered.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::AuditPrefixMismatch { .. }
        ))
    ));
}

#[test]
fn completion_consumption_and_nam_state_survive_recovery() {
    let (store, mut journal, mut event_loop, key, attempt) = delivered_fixture(21);
    configure_projection(&mut event_loop, key.namespace);
    event_loop
        .cycle_to_with_runtime_checkpoint_and_completions(
            LogicalTime::ZERO,
            &InputBatch::default(),
            &completion(1, key, attempt, "done"),
            &mut journal,
        )
        .unwrap();
    assert_eq!(
        event_loop.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Text("done".into())
    );

    let mut takeover = active_journal(21, 2, store);
    let mut recovered = boot_effect();
    configure_projection(&mut recovered, key.namespace);
    recovered
        .recover_runtime_from_audited_journal(&mut takeover)
        .unwrap();
    assert_eq!(
        recovered.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Text("done".into())
    );
    assert!(recovered.has_effect_completion(key));
    assert!(matches!(
        recovered.cycle_to_with_effect_completions(
            LogicalTime::ZERO,
            &InputBatch::default(),
            &completion(2, key, attempt, "duplicate"),
            &mut takeover,
        ),
        Err(EventLoopError::Runtime(RuntimeError::Completion(
            EffectCompletionError::DeliveryAlreadyCompleted(_)
        )))
    ));
}

#[test]
fn checkpoint_runtime_can_refresh_current_state_without_new_cycle() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(22, 1, store.clone());
    let mut event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    let receipt = event_loop
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    assert!(receipt.reference.is_some());
    assert_eq!(store.bundle_commits(), 1);
    let decoded =
        RuntimeSemanticCheckpoint::from_canonical_bytes(&store.runtime_bytes().unwrap()).unwrap();
    assert_eq!(decoded.cycle(), 0);
}

#[test]
fn runtime_checkpoint_digest_detects_single_byte_corruption() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(23, 1, store.clone());
    let mut event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    event_loop
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let mut bytes = store.runtime_bytes().unwrap();
    bytes[12] ^= 0x01;
    assert!(matches!(
        RuntimeSemanticCheckpoint::from_canonical_bytes(&bytes),
        Err(RuntimeCheckpointError::DigestMismatch { .. })
    ));
}

#[test]
fn missing_runtime_half_after_effect_only_advance_remains_fail_closed() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(24, 1, store.clone());
    let mut event_loop = AtomicEventLoop::boot(&interactive_program()).unwrap();
    event_loop
        .cycle_to_with_runtime_checkpoint(LogicalTime(1), &InputBatch::default(), &mut journal)
        .unwrap();
    store.clear_runtime();
    let mut takeover = active_journal(24, 2, store);
    let mut recovered = AtomicEventLoop::boot(&interactive_program()).unwrap();
    assert!(matches!(
        recovered.recover_runtime_from_audited_journal(&mut takeover),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::RecoveryBundleIncomplete
        ))
    ));
}

#[test]
fn existing_runtime_bundle_requires_recovery_before_overwrite() {
    let store = MemoryRuntimeStore::default();
    let mut first = active_journal(25, 1, store.clone());
    let mut source = AtomicEventLoop::boot(&interactive_program()).unwrap();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime(3), &key_batch(1, true, 13), &mut first)
        .unwrap();

    let mut takeover = active_journal(25, 2, store);
    let mut fresh = AtomicEventLoop::boot(&interactive_program()).unwrap();
    assert!(matches!(
        fresh.cycle_to_with_runtime_checkpoint(
            LogicalTime(4),
            &key_batch(1, false, 13),
            &mut takeover,
        ),
        Err(EventLoopError::RuntimeCheckpoint(
            RuntimeCheckpointError::RecoveryRequired
        ))
    ));
    assert_eq!(fresh.cycle_index(), 0);
    assert_eq!(fresh.logical_time(), LogicalTime::ZERO);
}
