use std::{cell::RefCell, rc::Rc};

use nordoi_kernel::{
    AtomSlot, AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, DomainId, Effect,
    EffectAttemptId, EffectAuditDispatchOutcome, EffectBackend, EffectBackendError,
    EffectBackendReceipt, EffectCompletion, EffectCompletionBatch, EffectCompletionError,
    EffectCompletionOutcome, EffectCompletionProjection, EffectCompletionProjectionValue,
    EffectCompletionSequence, EffectCompletionSourceId, EffectDeliveryFence, EffectDeliveryKey,
    EffectDeliveryNamespace, EffectDispatchAuthority, EffectFenceStoreError,
    EffectJournalCommitReceipt, EffectJournalLease, EffectJournalWriterId, EffectRetryPolicy,
    EffectRetryTick, EventLoopError, FencedEffectJournalStore, GovernedAuditedEffectJournal,
    GovernedEffectDispatcher, InputBatch, InputDeviceId, InputPayload, InputSignal, InputSource,
    InputTargetRef, Instruction, LogicalTime, NairEffectSet, NairProgram, NairReactionAuthority,
    NairReactionStep, NairReactionTrigger, QueuedEffectIntent, ReactionSlot, RegisterId,
    RuntimeError, Value,
};

const SOURCE: EffectCompletionSourceId = EffectCompletionSourceId(1);

fn namespace(seed: u8) -> EffectDeliveryNamespace {
    EffectDeliveryNamespace::new([seed; 16])
}

fn writer(seed: u8) -> EffectJournalWriterId {
    EffectJournalWriterId::new([seed; 16])
}

fn policy() -> EffectRetryPolicy {
    EffectRetryPolicy::new(3, 5, 20, 0).unwrap()
}

fn effects(effects: impl IntoIterator<Item = Effect>) -> NairEffectSet {
    NairEffectSet::from_effects(effects)
}

fn keyboard_batch() -> nordoi_kernel::InputBatch {
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

fn program() -> (NairProgram, NairReactionAuthority) {
    let effect = Effect::Network("api.example.test".into());
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Text("initial".into()),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: nordoi_kernel::DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "completion-reaction".into(),
            domain: nordoi_kernel::DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 7 },
            },
            action_name: "completion-action".into(),
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

fn dispatcher() -> GovernedEffectDispatcher {
    let mut authority = EffectDispatchAuthority::new();
    authority.grant(Capability::Network("api.example.test".into()));
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
    fn commits(&self) -> usize {
        self.state.borrow().commits
    }

    fn fail_next_commit(&self) {
        self.state.borrow_mut().fail_next_commit = true;
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
            return Err(EffectFenceStoreError::new(
                "synthetic completion checkpoint failure",
            ));
        }
        state.commits += 1;
        state.bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new(format!(
            "completion-{}-{}",
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

#[derive(Debug, Default)]
struct SuccessBackend;

impl EffectBackend for SuccessBackend {
    fn supports(&self, _effect: &Effect) -> bool {
        true
    }

    fn execute(
        &mut self,
        _request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        Ok(EffectBackendReceipt::new("remote-ok"))
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

fn delivered_fixture(
    seed: u8,
) -> (
    MemoryStore,
    GovernedAuditedEffectJournal<MemoryStore>,
    AtomicEventLoop,
    EffectDeliveryKey,
    EffectAttemptId,
) {
    let store = MemoryStore::default();
    let mut journal = active_journal(seed, 1, store.clone());
    let mut event_loop = boot();
    event_loop
        .cycle_to_with_audited_effect_journal(LogicalTime::ZERO, &keyboard_batch(), &mut journal)
        .unwrap();
    let mut backend = SuccessBackend;
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
        panic!("expected delivered audit outcome")
    };
    let key = receipt
        .delivery_key
        .expect("audited delivery must have a key");
    (store, journal, event_loop, key, attempt)
}

fn configure_projection(
    event_loop: &mut AtomicEventLoop,
    ns: EffectDeliveryNamespace,
    value: EffectCompletionProjectionValue,
) {
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
            value,
        ))
        .unwrap();
}

fn completion(
    sequence: u64,
    key: EffectDeliveryKey,
    attempt: EffectAttemptId,
    outcome: EffectCompletionOutcome,
) -> EffectCompletionBatch {
    EffectCompletionBatch::new(vec![EffectCompletion::new(
        SOURCE,
        EffectCompletionSequence(sequence),
        key,
        attempt,
        outcome,
    )])
}

#[test]
fn delivered_completion_reenters_nam_before_runtime_flush() {
    let (_store, mut journal, mut event_loop, key, attempt) = delivered_fixture(41);
    configure_projection(
        &mut event_loop,
        key.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    let report = event_loop
        .cycle_to_with_effect_completions(
            LogicalTime::ZERO,
            &InputBatch::default(),
            &completion(
                1,
                key,
                attempt,
                EffectCompletionOutcome::success("complete"),
            ),
            &mut journal,
        )
        .unwrap();
    assert_eq!(report.completions.accepted, 1);
    assert_eq!(report.completions.changed_atoms, 1);
    assert!(report.runtime.frame.is_some());
    assert_eq!(
        event_loop.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Text("complete".into())
    );
}

#[test]
fn completion_is_part_of_deterministic_replay_identity() {
    let (_store_a, mut journal_a, mut a, key_a, attempt_a) = delivered_fixture(42);
    let (_store_b, mut journal_b, mut b, key_b, attempt_b) = delivered_fixture(42);
    configure_projection(
        &mut a,
        key_a.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    configure_projection(
        &mut b,
        key_b.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    a.cycle_to_with_effect_completions(
        LogicalTime::ZERO,
        &InputBatch::default(),
        &completion(
            1,
            key_a,
            attempt_a,
            EffectCompletionOutcome::success("same"),
        ),
        &mut journal_a,
    )
    .unwrap();
    b.cycle_to_with_effect_completions(
        LogicalTime::ZERO,
        &InputBatch::default(),
        &completion(
            1,
            key_b,
            attempt_b,
            EffectCompletionOutcome::success("same"),
        ),
        &mut journal_b,
    )
    .unwrap();
    assert_eq!(a.replay_key(), b.replay_key());
}

#[test]
fn different_completion_outcome_changes_replay_identity() {
    let (_store_a, mut journal_a, mut a, key_a, attempt_a) = delivered_fixture(43);
    let (_store_b, mut journal_b, mut b, key_b, attempt_b) = delivered_fixture(43);
    configure_projection(
        &mut a,
        key_a.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    configure_projection(
        &mut b,
        key_b.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    a.cycle_to_with_effect_completions(
        LogicalTime::ZERO,
        &InputBatch::default(),
        &completion(1, key_a, attempt_a, EffectCompletionOutcome::success("ok")),
        &mut journal_a,
    )
    .unwrap();
    b.cycle_to_with_effect_completions(
        LogicalTime::ZERO,
        &InputBatch::default(),
        &completion(
            1,
            key_b,
            attempt_b,
            EffectCompletionOutcome::failure("error"),
        ),
        &mut journal_b,
    )
    .unwrap();
    assert_ne!(a.replay_key(), b.replay_key());
}

#[test]
fn duplicate_delivery_completion_aborts_whole_cycle_publication() {
    let (_store, mut journal, mut event_loop, key, attempt) = delivered_fixture(44);
    configure_projection(
        &mut event_loop,
        key.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    event_loop
        .cycle_to_with_effect_completions(
            LogicalTime::ZERO,
            &InputBatch::default(),
            &completion(1, key, attempt, EffectCompletionOutcome::success("first")),
            &mut journal,
        )
        .unwrap();
    let before_cycle = event_loop.cycle_index();
    let before_replay = event_loop.replay_key();
    let before_value = event_loop.snapshot().unwrap()[&AtomSlot(0)].value.clone();
    let result = event_loop.cycle_to_with_effect_completions(
        LogicalTime::ZERO,
        &InputBatch::default(),
        &completion(2, key, attempt, EffectCompletionOutcome::success("second")),
        &mut journal,
    );
    assert!(matches!(
        result,
        Err(EventLoopError::Runtime(RuntimeError::Completion(
            EffectCompletionError::DeliveryAlreadyCompleted(_)
        )))
    ));
    assert_eq!(event_loop.cycle_index(), before_cycle);
    assert_eq!(event_loop.replay_key(), before_replay);
    assert_eq!(
        event_loop.snapshot().unwrap()[&AtomSlot(0)].value,
        before_value
    );
}

#[test]
fn unauthorized_completion_aborts_without_runtime_progress() {
    let (_store, mut journal, mut event_loop, key, attempt) = delivered_fixture(45);
    let atom = event_loop.snapshot().unwrap()[&AtomSlot(0)].id;
    event_loop
        .register_effect_completion_projection(EffectCompletionProjection::new(
            SOURCE,
            key.namespace,
            DomainId(0),
            atom,
            EffectCompletionProjectionValue::OutcomeValue,
        ))
        .unwrap();
    let before = event_loop.replay_key();
    let result = event_loop.cycle_to_with_effect_completions(
        LogicalTime::ZERO,
        &InputBatch::default(),
        &completion(1, key, attempt, EffectCompletionOutcome::success("blocked")),
        &mut journal,
    );
    assert!(matches!(
        result,
        Err(EventLoopError::Runtime(RuntimeError::Completion(
            EffectCompletionError::UnauthorizedSource { .. }
        )))
    ));
    assert_eq!(event_loop.replay_key(), before);
}

#[test]
fn stale_effect_journal_writer_cannot_accept_completion() {
    let (store, mut stale, mut event_loop, key, attempt) = delivered_fixture(46);
    configure_projection(
        &mut event_loop,
        key.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    let mut takeover = active_journal(46, 2, store);
    assert!(takeover.lease().unwrap().fence > stale.lease().unwrap().fence);
    let before = event_loop.replay_key();
    assert!(event_loop
        .cycle_to_with_effect_completions(
            LogicalTime::ZERO,
            &InputBatch::default(),
            &completion(1, key, attempt, EffectCompletionOutcome::success("blocked")),
            &mut stale,
        )
        .is_err());
    assert_eq!(event_loop.replay_key(), before);
    takeover.release().unwrap();
}

#[test]
fn audited_cycle_with_completion_checkpoints_before_publication() {
    let (store, mut journal, mut event_loop, key, attempt) = delivered_fixture(47);
    configure_projection(
        &mut event_loop,
        key.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    let before_commits = store.commits();
    let report = event_loop
        .cycle_to_with_audited_effect_journal_and_completions(
            LogicalTime::ZERO,
            &InputBatch::default(),
            &completion(
                1,
                key,
                attempt,
                EffectCompletionOutcome::success("durable-cycle"),
            ),
            &mut journal,
        )
        .unwrap();
    assert_eq!(store.commits(), before_commits + 1);
    assert_eq!(report.completions.accepted, 1);
    assert_eq!(
        event_loop.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Text("durable-cycle".into())
    );
}

#[test]
fn failed_audited_checkpoint_aborts_completion_publication() {
    let (store, mut journal, mut event_loop, key, attempt) = delivered_fixture(48);
    configure_projection(
        &mut event_loop,
        key.namespace,
        EffectCompletionProjectionValue::OutcomeValue,
    );
    let before_cycle = event_loop.cycle_index();
    let before_replay = event_loop.replay_key();
    let before_value = event_loop.snapshot().unwrap()[&AtomSlot(0)].value.clone();
    store.fail_next_commit();
    let result = event_loop.cycle_to_with_audited_effect_journal_and_completions(
        LogicalTime::ZERO,
        &InputBatch::default(),
        &completion(
            1,
            key,
            attempt,
            EffectCompletionOutcome::success("must-not-publish"),
        ),
        &mut journal,
    );
    assert!(result.is_err());
    assert_eq!(event_loop.cycle_index(), before_cycle);
    assert_eq!(event_loop.replay_key(), before_replay);
    assert_eq!(
        event_loop.snapshot().unwrap()[&AtomSlot(0)].value,
        before_value
    );
    assert!(!event_loop.has_effect_completion(key));
}
