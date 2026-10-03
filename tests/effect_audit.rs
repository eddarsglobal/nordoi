use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use nordoi_kernel::{
    AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, Effect, EffectAuditCheckpoint,
    EffectAuditDispatchOutcome, EffectAuditError, EffectAuditEvent, EffectBackend,
    EffectBackendError, EffectBackendReceipt, EffectDeliveryFence, EffectDeliveryKey,
    EffectDeliveryNamespace, EffectDispatchAuthority, EffectDispatchError, EffectDispatchRequest,
    EffectFenceStoreError, EffectJournalCommitReceipt, EffectJournalLease, EffectJournalWriterId,
    EffectRetryPolicy, EffectRetryTick, FencedEffectJournalStore, GovernedAuditedEffectJournal,
    GovernedEffectDispatcher, GovernedRetryEffectJournal, InputDeviceId, InputPayload, InputSignal,
    InputSource, InputTargetRef, Instruction, LogicalTime, NairEffectSet, NairProgram,
    NairReactionAuthority, NairReactionStep, NairReactionTrigger, QueuedEffectIntent, ReactionSlot,
};

fn effects(effects: impl IntoIterator<Item = Effect>) -> NairEffectSet {
    NairEffectSet::from_effects(effects)
}

fn namespace(seed: u8) -> EffectDeliveryNamespace {
    EffectDeliveryNamespace::new([seed; 16])
}
fn writer(seed: u8) -> EffectJournalWriterId {
    EffectJournalWriterId::new([seed; 16])
}
fn policy() -> EffectRetryPolicy {
    EffectRetryPolicy::new(3, 5, 20, 0).unwrap()
}

fn keyboard_batch(code: u32) -> nordoi_kernel::InputBatch {
    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Keyboard,
            InputDeviceId(1),
            None,
            InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();
    input.drain()
}

fn program() -> (NairProgram, NairReactionAuthority) {
    let effect = Effect::Network("api.example.test".into());
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "audit-reaction".into(),
            domain: nordoi_kernel::DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 7 },
            },
            action_name: "audit-action".into(),
            declared_effects: effects([effect.clone()]),
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

fn boot() -> AtomicEventLoop {
    let (program, authority) = program();
    AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap()
}

fn dispatcher(allowed: bool) -> GovernedEffectDispatcher {
    let mut authority = EffectDispatchAuthority::new();
    if allowed {
        authority.grant(Capability::Network("api.example.test".into()));
    }
    GovernedEffectDispatcher::new(authority)
}

#[derive(Debug, Default)]
struct StoreState {
    bytes: Option<Vec<u8>>,
    next_fence: u64,
    active: Option<EffectJournalLease>,
    commits: usize,
    fail_next_commit: bool,
}

#[derive(Debug, Clone, Default)]
struct MemoryStore {
    state: Rc<RefCell<StoreState>>,
}

impl MemoryStore {
    fn snapshot(&self) -> std::cell::Ref<'_, StoreState> {
        self.state.borrow()
    }
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
}

impl FencedEffectJournalStore for MemoryStore {
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
        Ok(state.bytes.clone())
    }
    fn commit_fenced(
        &mut self,
        lease: EffectJournalLease,
        bytes: &[u8],
    ) -> Result<EffectJournalCommitReceipt, EffectFenceStoreError> {
        let mut state = self.state.borrow_mut();
        Self::require_active(&state, lease)?;
        if state.fail_next_commit {
            state.fail_next_commit = false;
            return Err(EffectFenceStoreError::new("synthetic commit failure"));
        }
        state.commits += 1;
        state.bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new(format!(
            "audit-{}-{}",
            lease.fence.0, state.commits
        )))
    }
    fn release(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        let mut state = self.state.borrow_mut();
        Self::require_active(&state, lease)?;
        state.active = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum BackendStep {
    Success(Option<&'static str>),
    Retryable(&'static str),
    Permanent(&'static str),
}

#[derive(Debug, Default)]
struct ScriptedBackend {
    steps: VecDeque<BackendStep>,
    calls: usize,
    keys: Vec<Option<EffectDeliveryKey>>,
    fences: Vec<Option<EffectDeliveryFence>>,
    observed_commits: Vec<usize>,
    shared_store: Option<Rc<RefCell<StoreState>>>,
    fail_commit_on_execute: bool,
    supports: bool,
}

impl ScriptedBackend {
    fn with_steps(steps: impl IntoIterator<Item = BackendStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
            supports: true,
            ..Self::default()
        }
    }
    fn unsupported() -> Self {
        Self {
            supports: false,
            ..Self::default()
        }
    }
    fn observe_store(mut self, store: &MemoryStore) -> Self {
        self.shared_store = Some(store.state.clone());
        self
    }
    fn fail_final_commit(mut self, store: &MemoryStore) -> Self {
        self.shared_store = Some(store.state.clone());
        self.fail_commit_on_execute = true;
        self
    }
}

impl EffectBackend for ScriptedBackend {
    fn supports(&self, _effect: &Effect) -> bool {
        self.supports
    }
    fn execute(
        &mut self,
        request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.calls += 1;
        if let Some(shared) = &self.shared_store {
            self.observed_commits.push(shared.borrow().commits);
            if self.fail_commit_on_execute {
                shared.borrow_mut().fail_next_commit = true;
            }
        }
        let _ = request;
        match self.steps.pop_front().unwrap_or(BackendStep::Success(None)) {
            BackendStep::Success(reference) => Ok(reference
                .map_or_else(EffectBackendReceipt::empty, |value| {
                    EffectBackendReceipt::new(value)
                })),
            BackendStep::Retryable(message) => Err(EffectBackendError::new(message)),
            BackendStep::Permanent(message) => Err(EffectBackendError::permanent(message)),
        }
    }
    fn execute_with_context(
        &mut self,
        request: &EffectDispatchRequest,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.keys.push(request.delivery_key);
        self.fences.push(request.delivery_fence);
        self.execute(&request.queued)
    }
}

fn active_journal(
    seed: u8,
    writer_seed: u8,
    store: MemoryStore,
) -> GovernedAuditedEffectJournal<MemoryStore> {
    let mut journal =
        GovernedAuditedEffectJournal::new(namespace(seed), writer(writer_seed), store, policy());
    journal.acquire().unwrap();
    journal
}

fn stage_one(loop_: &mut AtomicEventLoop, journal: &mut GovernedAuditedEffectJournal<MemoryStore>) {
    loop_
        .cycle_to_with_audited_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), journal)
        .unwrap();
    assert_eq!(loop_.pending_effect_count(), 1);
}

fn create_in_doubt(
    seed: u8,
) -> (
    MemoryStore,
    GovernedAuditedEffectJournal<MemoryStore>,
    AtomicEventLoop,
    EffectDeliveryKey,
    EffectDeliveryFence,
) {
    let store = MemoryStore::default();
    let mut journal = active_journal(seed, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(Some("remote-ok"))])
        .fail_final_commit(&store);
    let result = loop_.dispatch_next_effect_with_audited_journal(
        &mut journal,
        EffectRetryTick(10),
        &dispatcher(true),
        &mut backend,
    );
    assert!(matches!(result, Err(EffectAuditError::Store(_))));
    let open = journal.in_doubt_attempt().unwrap().clone();
    (
        store,
        journal,
        loop_,
        open.delivery_key,
        open.delivery_fence,
    )
}

#[test]
fn audit_checkpoint_migrates_legacy_retry_checkpoint() {
    let store = MemoryStore::default();
    let mut legacy =
        GovernedRetryEffectJournal::new(namespace(1), writer(1), store.clone(), policy());
    legacy.acquire().unwrap();
    let mut source = boot();
    source
        .cycle_to_with_retry_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut legacy)
        .unwrap();
    legacy.release().unwrap();

    let mut audited = active_journal(1, 2, store);
    let mut recovered = boot();
    assert!(recovered
        .recover_effects_from_audited_journal(&mut audited)
        .unwrap());
    assert_eq!(recovered.pending_effect_count(), 1);
    assert!(audited.audit_ledger().is_empty());
}

#[test]
fn audit_checkpoint_round_trip_is_byte_stable() {
    let store = MemoryStore::default();
    let mut journal = active_journal(2, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(Some("receipt-1"))]);
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap();
    let bytes = store.snapshot().bytes.clone().unwrap();
    let decoded = EffectAuditCheckpoint::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded.canonical_bytes(), bytes);
    assert_eq!(decoded.audit().len(), 2);
}

#[test]
fn tampered_audit_checkpoint_is_rejected() {
    let store = MemoryStore::default();
    let mut journal = active_journal(3, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(None)]);
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap();
    let mut bytes = store.snapshot().bytes.clone().unwrap();
    bytes[32] ^= 0x5a;
    assert!(matches!(
        EffectAuditCheckpoint::from_canonical_bytes(&bytes),
        Err(EffectAuditError::AuditCheckpointDigestMismatch { .. })
    ));
}

#[test]
fn attempt_is_persisted_before_backend_execution() {
    let store = MemoryStore::default();
    let mut journal = active_journal(4, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend =
        ScriptedBackend::with_steps([BackendStep::Success(None)]).observe_store(&store);
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap();
    assert_eq!(backend.observed_commits, vec![2]);
    assert_eq!(store.snapshot().commits, 3);
}

#[test]
fn successful_dispatch_records_prepared_then_delivered() {
    let store = MemoryStore::default();
    let mut journal = active_journal(5, 1, store);
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(Some("ok"))]);
    let outcome = loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert!(matches!(
        outcome,
        EffectAuditDispatchOutcome::Delivered { .. }
    ));
    assert!(matches!(
        &journal.audit_ledger().records()[0].event,
        EffectAuditEvent::AttemptPrepared(_)
    ));
    assert!(matches!(
        &journal.audit_ledger().records()[1].event,
        EffectAuditEvent::AttemptDelivered { .. }
    ));
    assert!(journal.in_doubt_attempt().is_none());
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn retryable_failure_records_retry_schedule() {
    let store = MemoryStore::default();
    let mut journal = active_journal(6, 1, store);
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temporary")]);
    let outcome = loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(10),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert!(matches!(
        outcome,
        EffectAuditDispatchOutcome::RetryScheduled {
            next_eligible_tick: EffectRetryTick(15),
            ..
        }
    ));
    assert!(matches!(
        &journal.audit_ledger().records()[1].event,
        EffectAuditEvent::AttemptRetryScheduled { .. }
    ));
    assert_eq!(loop_.pending_effect_count(), 1);
}

#[test]
fn permanent_failure_records_dead_letter() {
    let store = MemoryStore::default();
    let mut journal = active_journal(7, 1, store);
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Permanent("invalid")]);
    let outcome = loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert!(matches!(
        outcome,
        EffectAuditDispatchOutcome::DeadLettered { .. }
    ));
    assert!(matches!(
        &journal.audit_ledger().records()[1].event,
        EffectAuditEvent::AttemptDeadLettered { .. }
    ));
    assert_eq!(journal.retry_ledger().dead_letter_count(), 1);
}

#[test]
fn deferred_retry_creates_no_new_audit_record() {
    let store = MemoryStore::default();
    let mut journal = active_journal(8, 1, store);
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([
        BackendStep::Retryable("temporary"),
        BackendStep::Success(None),
    ]);
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(10),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap();
    let before = journal.audit_ledger().len();
    let outcome = loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(11),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        outcome,
        EffectAuditDispatchOutcome::Deferred {
            next_eligible_tick: EffectRetryTick(15)
        }
    );
    assert_eq!(journal.audit_ledger().len(), before);
    assert_eq!(backend.calls, 1);
}

#[test]
fn capability_denial_happens_before_attempt_prepare() {
    let store = MemoryStore::default();
    let mut journal = active_journal(9, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let commits = store.snapshot().commits;
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(None)]);
    let result = loop_.dispatch_next_effect_with_audited_journal(
        &mut journal,
        EffectRetryTick(1),
        &dispatcher(false),
        &mut backend,
    );
    assert!(matches!(
        result,
        Err(EffectAuditError::Dispatch(
            EffectDispatchError::CapabilityDenied(_)
        ))
    ));
    assert!(journal.audit_ledger().is_empty());
    assert_eq!(store.snapshot().commits, commits);
    assert_eq!(backend.calls, 0);
}

#[test]
fn unsupported_backend_happens_before_attempt_prepare() {
    let store = MemoryStore::default();
    let mut journal = active_journal(10, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let commits = store.snapshot().commits;
    let mut backend = ScriptedBackend::unsupported();
    let result = loop_.dispatch_next_effect_with_audited_journal(
        &mut journal,
        EffectRetryTick(1),
        &dispatcher(true),
        &mut backend,
    );
    assert!(matches!(
        result,
        Err(EffectAuditError::Dispatch(
            EffectDispatchError::BackendUnsupported { .. }
        ))
    ));
    assert!(journal.audit_ledger().is_empty());
    assert_eq!(store.snapshot().commits, commits);
    assert_eq!(backend.calls, 0);
}

#[test]
fn failed_prepare_commit_prevents_external_execution() {
    let store = MemoryStore::default();
    let mut journal = active_journal(11, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    store.state.borrow_mut().fail_next_commit = true;
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(None)]);
    let result = loop_.dispatch_next_effect_with_audited_journal(
        &mut journal,
        EffectRetryTick(1),
        &dispatcher(true),
        &mut backend,
    );
    assert!(matches!(result, Err(EffectAuditError::Store(_))));
    assert_eq!(backend.calls, 0);
    assert!(journal.audit_ledger().is_empty());
    assert_eq!(loop_.pending_effect_count(), 1);
}

#[test]
fn failed_terminal_commit_leaves_attempt_in_doubt() {
    let (_store, journal, loop_, _, _) = create_in_doubt(12);
    assert_eq!(journal.audit_ledger().len(), 1);
    assert!(journal.in_doubt_attempt().is_some());
    assert_eq!(loop_.pending_effect_count(), 1);
}

#[test]
fn in_doubt_attempt_blocks_automatic_redispatch() {
    let (_store, mut journal, mut loop_, _, _) = create_in_doubt(13);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(None)]);
    let result = loop_.dispatch_next_effect_with_audited_journal(
        &mut journal,
        EffectRetryTick(11),
        &dispatcher(true),
        &mut backend,
    );
    assert!(matches!(
        result,
        Err(EffectAuditError::InDoubtAttemptExists(_))
    ));
    assert_eq!(backend.calls, 0);
}

#[test]
fn recovery_detects_persisted_in_doubt_attempt() {
    let (store, _journal, _loop, key, fence) = create_in_doubt(14);
    let mut recovered_journal = active_journal(14, 2, store);
    let mut recovered_loop = boot();
    assert!(recovered_loop
        .recover_effects_from_audited_journal(&mut recovered_journal)
        .unwrap());
    let open = recovered_journal.in_doubt_attempt().unwrap();
    assert_eq!(open.delivery_key, key);
    assert_eq!(open.delivery_fence, fence);
    assert_eq!(recovered_loop.pending_effect_count(), 1);
}

#[test]
fn explicit_retry_authorization_closes_in_doubt_without_execution() {
    let (_store, mut journal, mut loop_, _, _) = create_in_doubt(15);
    let request = loop_
        .authorize_in_doubt_effect_retry(&mut journal, EffectRetryTick(10))
        .unwrap();
    assert_eq!(request.id.value(), 1);
    assert!(journal.in_doubt_attempt().is_none());
    assert_eq!(loop_.pending_effect_count(), 1);
    assert!(matches!(
        &journal.audit_ledger().records().last().unwrap().event,
        EffectAuditEvent::InDoubtRetryAuthorized { .. }
    ));
}

#[test]
fn assume_delivered_resolution_removes_pending_intent() {
    let (_store, mut journal, mut loop_, _, _) = create_in_doubt(16);
    let request = loop_
        .resolve_in_doubt_effect_as_delivered(
            &mut journal,
            EffectRetryTick(10),
            Some("reconciled".into()),
        )
        .unwrap();
    assert_eq!(request.id.value(), 1);
    assert!(journal.in_doubt_attempt().is_none());
    assert_eq!(loop_.pending_effect_count(), 0);
    assert!(matches!(
        &journal.audit_ledger().records().last().unwrap().event,
        EffectAuditEvent::InDoubtAssumedDelivered { .. }
    ));
}

#[test]
fn takeover_changes_fence_but_preserves_delivery_key() {
    let (store, _journal, _loop, key, old_fence) = create_in_doubt(17);
    let mut journal = active_journal(17, 2, store);
    let new_fence = journal.lease().unwrap().fence;
    assert!(new_fence.0 > old_fence.0);
    let mut loop_ = boot();
    loop_
        .recover_effects_from_audited_journal(&mut journal)
        .unwrap();
    loop_
        .authorize_in_doubt_effect_retry(&mut journal, EffectRetryTick(10))
        .unwrap();
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(None)]);
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(10),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap();
    assert_eq!(backend.keys, vec![Some(key)]);
    assert_eq!(backend.fences, vec![Some(new_fence)]);
}

#[test]
fn dead_letter_redrive_is_audited() {
    let store = MemoryStore::default();
    let mut journal = active_journal(18, 1, store);
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Permanent("bad")]);
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap();
    loop_
        .redrive_dead_letter_with_audited_journal(&mut journal, nordoi_kernel::EffectIntentId(1))
        .unwrap();
    assert_eq!(loop_.pending_effect_count(), 1);
    assert!(matches!(
        &journal.audit_ledger().records().last().unwrap().event,
        EffectAuditEvent::DeadLetterRedriven { .. }
    ));
}

#[test]
fn audit_history_survives_later_cycle_checkpoint() {
    let store = MemoryStore::default();
    let mut journal = active_journal(19, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temporary")]);
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap();
    assert_eq!(journal.audit_ledger().len(), 2);
    let mut later_input = keyboard_batch(99);
    later_input.events[0].sequence = nordoi_kernel::InputSequence(2);
    loop_
        .cycle_to_with_audited_effect_journal(LogicalTime(1), &later_input, &mut journal)
        .unwrap();
    let bytes = store.snapshot().bytes.clone().unwrap();
    let checkpoint = EffectAuditCheckpoint::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(checkpoint.audit().len(), 2);
    assert_eq!(
        checkpoint.audit().root_hash(),
        journal.audit_ledger().root_hash()
    );
}

#[test]
fn audit_dispatch_does_not_change_event_loop_replay_identity() {
    let store = MemoryStore::default();
    let mut journal = active_journal(20, 1, store);
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let before = loop_.replay_key();
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success(None)]);
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(true),
            &mut backend,
        )
        .unwrap();
    assert_eq!(loop_.replay_key(), before);
}
