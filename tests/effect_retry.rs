use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use nordoi_kernel::{
    AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, DeadLetteredEffect, DomainRef,
    Effect, EffectBackend, EffectBackendError, EffectBackendReceipt, EffectDeadLetterReason,
    EffectDeliveryFence, EffectDeliveryKey, EffectDeliveryNamespace, EffectDispatchAuthority,
    EffectDispatchRequest, EffectFenceStoreError, EffectJournalCommitReceipt, EffectJournalLease,
    EffectJournalWriterId, EffectRetryCheckpoint, EffectRetryDispatchOutcome, EffectRetryError,
    EffectRetryPolicy, EffectRetryTick, FencedEffectJournalStore, GovernedEffectDispatcher,
    GovernedRetryEffectJournal, InputDeviceId, InputPayload, InputSignal, InputSource,
    InputTargetRef, Instruction, LogicalTime, NairEffectSet, NairProgram, NairReactionAuthority,
    NairReactionStep, NairReactionTrigger, QueuedEffectIntent, ReactionSlot, NAIR_FORMAT_MINOR,
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

fn network_program(code: u32, scopes: &[&str]) -> (NairProgram, NairReactionAuthority) {
    let declared: Vec<Effect> = scopes
        .iter()
        .map(|scope| Effect::Network((*scope).into()))
        .collect();
    let steps = declared
        .iter()
        .cloned()
        .map(|effect| NairReactionStep::EmitEffect { effect })
        .collect();
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "retry-network-reaction".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code },
            },
            action_name: "retry-network-action".into(),
            declared_effects: effects(declared.clone()),
            steps,
        },
        Instruction::Halt,
    ]);

    let mut capabilities = CapabilitySet::new();
    for effect in declared {
        if let Effect::Network(scope) = effect {
            capabilities.allow(Capability::Network(scope));
        }
    }
    let mut authority = NairReactionAuthority::new();
    authority.set(ReactionSlot(0), capabilities);
    (program, authority)
}

fn dispatch_authority(scopes: &[&str]) -> EffectDispatchAuthority {
    let mut authority = EffectDispatchAuthority::new();
    for scope in scopes {
        authority.grant(Capability::Network((*scope).into()));
    }
    authority
}

#[derive(Debug, Default)]
struct SharedState {
    bytes: Option<Vec<u8>>,
    next_fence: u64,
    active: Option<EffectJournalLease>,
    commits: usize,
    fail_commit: bool,
}

#[derive(Debug, Clone, Default)]
struct MemoryFencedStore {
    state: Rc<RefCell<SharedState>>,
}

impl MemoryFencedStore {
    fn state(&self) -> std::cell::Ref<'_, SharedState> {
        self.state.borrow()
    }

    fn state_mut(&self) -> std::cell::RefMut<'_, SharedState> {
        self.state.borrow_mut()
    }

    fn require_active(
        state: &SharedState,
        lease: EffectJournalLease,
    ) -> Result<(), EffectFenceStoreError> {
        if state.active == Some(lease) {
            Ok(())
        } else {
            Err(EffectFenceStoreError::new("stale or inactive fence"))
        }
    }
}

impl FencedEffectJournalStore for MemoryFencedStore {
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
        if state.fail_commit {
            return Err(EffectFenceStoreError::new("synthetic commit failure"));
        }
        state.commits += 1;
        state.bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new(format!(
            "retry-{}-{}",
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
    Success,
    Retryable(&'static str),
    Permanent(&'static str),
}

#[derive(Debug, Default)]
struct ScriptedBackend {
    steps: VecDeque<BackendStep>,
    calls: usize,
    ids: Vec<u64>,
    keys: Vec<Option<EffectDeliveryKey>>,
    fences: Vec<Option<EffectDeliveryFence>>,
}

impl ScriptedBackend {
    fn with_steps(steps: impl IntoIterator<Item = BackendStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
            ..Self::default()
        }
    }
}

impl EffectBackend for ScriptedBackend {
    fn supports(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Network(_))
    }

    fn execute(
        &mut self,
        request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.calls += 1;
        self.ids.push(request.id.value());
        match self.steps.pop_front().unwrap_or(BackendStep::Success) {
            BackendStep::Success => Ok(EffectBackendReceipt::empty()),
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

#[derive(Debug, Default)]
struct UnsupportedBackend;

impl EffectBackend for UnsupportedBackend {
    fn supports(&self, _effect: &Effect) -> bool {
        false
    }

    fn execute(
        &mut self,
        _request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        unreachable!("unsupported backend must not execute")
    }
}

fn policy(max_attempts: u32, initial: u64, max: u64) -> EffectRetryPolicy {
    EffectRetryPolicy::new(max_attempts, initial, max, 0).unwrap()
}

fn boot(scopes: &[&str]) -> AtomicEventLoop {
    let (program, authority) = network_program(7, scopes);
    AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap()
}

fn active_journal(
    seed: u8,
    store: MemoryFencedStore,
    retry_policy: EffectRetryPolicy,
) -> GovernedRetryEffectJournal<MemoryFencedStore> {
    let mut journal =
        GovernedRetryEffectJournal::new(namespace(seed), writer(seed), store, retry_policy);
    journal.acquire().unwrap();
    journal
}

fn stage_one(
    loop_: &mut AtomicEventLoop,
    journal: &mut GovernedRetryEffectJournal<MemoryFencedStore>,
) {
    loop_
        .cycle_to_with_retry_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), journal)
        .unwrap();
}

fn expect_scheduled(outcome: EffectRetryDispatchOutcome) -> (u32, EffectRetryTick) {
    match outcome {
        EffectRetryDispatchOutcome::RetryScheduled {
            failed_attempts,
            next_eligible_tick,
            ..
        } => (failed_attempts, next_eligible_tick),
        other => panic!("expected RetryScheduled, got {other:?}"),
    }
}

fn expect_dead(outcome: EffectRetryDispatchOutcome) -> DeadLetteredEffect {
    match outcome {
        EffectRetryDispatchOutcome::DeadLettered(dead) => dead,
        other => panic!("expected DeadLettered, got {other:?}"),
    }
}

#[test]
fn policy_rejects_zero_attempt_budget() {
    assert!(matches!(
        EffectRetryPolicy::new(0, 1, 8, 0),
        Err(EffectRetryError::InvalidPolicy(_))
    ));
}

#[test]
fn policy_rejects_zero_initial_backoff() {
    assert!(matches!(
        EffectRetryPolicy::new(3, 0, 8, 0),
        Err(EffectRetryError::InvalidPolicy(_))
    ));
}

#[test]
fn policy_rejects_maximum_smaller_than_initial_backoff() {
    assert!(matches!(
        EffectRetryPolicy::new(3, 8, 4, 0),
        Err(EffectRetryError::InvalidPolicy(_))
    ));
}

#[test]
fn retryable_failure_schedules_durable_backoff() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(1, store.clone(), policy(3, 2, 8));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temporary")]);

    let outcome = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(10),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();
    let (attempts, next) = expect_scheduled(outcome);

    assert_eq!(attempts, 1);
    assert_eq!(next, EffectRetryTick(12));
    assert_eq!(loop_.pending_effect_count(), 1);
    assert_eq!(journal.ledger().retry_count(), 1);
    assert!(store.state().bytes.is_some());
}

#[test]
fn delayed_effect_does_not_execute_before_eligibility() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(2, store, policy(3, 3, 9));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend =
        ScriptedBackend::with_steps([BackendStep::Retryable("temporary"), BackendStep::Success]);

    let first = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(5),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert_eq!(expect_scheduled(first).1, EffectRetryTick(8));

    let deferred = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(7),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        deferred,
        EffectRetryDispatchOutcome::Deferred {
            next_eligible_tick: EffectRetryTick(8)
        }
    );
    assert_eq!(backend.calls, 1);
}

#[test]
fn ready_later_intent_bypasses_delayed_older_intent() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(3, store, policy(3, 10, 20));
    let mut loop_ = boot(&["one.example.test", "two.example.test"]);
    stage_one(&mut loop_, &mut journal);
    assert_eq!(loop_.pending_effect_count(), 2);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&[
        "one.example.test",
        "two.example.test",
    ]));
    let mut backend = ScriptedBackend::with_steps([
        BackendStep::Retryable("first blocked"),
        BackendStep::Success,
    ]);

    loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        )
        .unwrap();
    let second = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(2),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();

    assert!(matches!(second, EffectRetryDispatchOutcome::Delivered(_)));
    assert_eq!(backend.ids, vec![1, 2]);
    assert_eq!(loop_.pending_effect_count(), 1);
    assert_eq!(loop_.pending_effects().next().unwrap().id.value(), 1);
}

#[test]
fn exponential_backoff_is_capped() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(4, store, policy(5, 2, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([
        BackendStep::Retryable("a"),
        BackendStep::Retryable("b"),
        BackendStep::Retryable("c"),
    ]);

    let first = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(10),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert_eq!(expect_scheduled(first).1, EffectRetryTick(12));

    let second = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(12),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert_eq!(expect_scheduled(second).1, EffectRetryTick(16));

    let third = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(16),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();
    assert_eq!(expect_scheduled(third).1, EffectRetryTick(20));
}

#[test]
fn deterministic_jitter_is_stable_for_equal_delivery_identity() {
    let policy = EffectRetryPolicy::new(3, 10, 30, 7).unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));

    let mut next_ticks = Vec::new();
    for _ in 0..2 {
        let store = MemoryFencedStore::default();
        let mut journal = active_journal(5, store, policy);
        let mut loop_ = boot(&["api.example.test"]);
        stage_one(&mut loop_, &mut journal);
        let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temporary")]);
        let outcome = loop_
            .dispatch_next_effect_with_retry_journal(
                &mut journal,
                EffectRetryTick(100),
                &dispatcher,
                &mut backend,
            )
            .unwrap()
            .unwrap();
        next_ticks.push(expect_scheduled(outcome).1);
    }

    assert_eq!(next_ticks[0], next_ticks[1]);
}

#[test]
fn permanent_backend_failure_dead_letters_immediately() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(6, store, policy(5, 2, 8));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Permanent("invalid request")]);

    let dead = expect_dead(
        loop_
            .dispatch_next_effect_with_retry_journal(
                &mut journal,
                EffectRetryTick(1),
                &dispatcher,
                &mut backend,
            )
            .unwrap()
            .unwrap(),
    );

    assert_eq!(dead.reason, EffectDeadLetterReason::PermanentBackendFailure);
    assert_eq!(dead.failed_attempts, 1);
    assert_eq!(loop_.pending_effect_count(), 0);
    assert_eq!(journal.ledger().dead_letter_count(), 1);
}

#[test]
fn exhausted_attempt_budget_moves_intent_to_dead_letter() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(7, store, policy(2, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([
        BackendStep::Retryable("first"),
        BackendStep::Retryable("second"),
    ]);

    let first = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();
    let next = expect_scheduled(first).1;
    let dead = expect_dead(
        loop_
            .dispatch_next_effect_with_retry_journal(&mut journal, next, &dispatcher, &mut backend)
            .unwrap()
            .unwrap(),
    );

    assert_eq!(dead.reason, EffectDeadLetterReason::AttemptsExhausted);
    assert_eq!(dead.failed_attempts, 2);
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn dead_letter_redrive_preserves_original_delivery_key() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(8, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend =
        ScriptedBackend::with_steps([BackendStep::Permanent("bad once"), BackendStep::Success]);

    let dead = expect_dead(
        loop_
            .dispatch_next_effect_with_retry_journal(
                &mut journal,
                EffectRetryTick(1),
                &dispatcher,
                &mut backend,
            )
            .unwrap()
            .unwrap(),
    );
    loop_
        .redrive_dead_letter_with_retry_journal(&mut journal, dead.request.id)
        .unwrap();
    let delivered = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(2),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();

    let receipt = match delivered {
        EffectRetryDispatchOutcome::Delivered(receipt) => receipt,
        other => panic!("expected delivery, got {other:?}"),
    };
    assert_eq!(receipt.delivery_key, Some(dead.delivery_key));
    assert_eq!(journal.ledger().dead_letter_count(), 0);
}

#[test]
fn dead_letter_can_be_explicitly_discarded() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(9, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Permanent("bad")]);
    let dead = expect_dead(
        loop_
            .dispatch_next_effect_with_retry_journal(
                &mut journal,
                EffectRetryTick(1),
                &dispatcher,
                &mut backend,
            )
            .unwrap()
            .unwrap(),
    );

    let discarded = loop_
        .discard_dead_letter_with_retry_journal(&mut journal, dead.request.id)
        .unwrap();
    assert_eq!(discarded, dead);
    assert_eq!(journal.ledger().dead_letter_count(), 0);
}

#[test]
fn retry_checkpoint_round_trip_preserves_retry_state() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(10, store.clone(), policy(3, 2, 8));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temp")]);
    loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(4),
            &dispatcher,
            &mut backend,
        )
        .unwrap();

    let bytes = store.state().bytes.clone().unwrap();
    let decoded = EffectRetryCheckpoint::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded.namespace(), namespace(10));
    assert_eq!(decoded.outbox().pending().len(), 1);
    assert_eq!(decoded.ledger().retry_count(), 1);
    assert_eq!(decoded.ledger().last_tick(), EffectRetryTick(4));
}

#[test]
fn certified_k18_checkpoint_migrates_with_empty_retry_ledger() {
    let mut loop_ = boot(&["api.example.test"]);
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7))
        .unwrap();
    let legacy = loop_.effect_checkpoint(namespace(11)).canonical_bytes();
    let decoded = EffectRetryCheckpoint::from_canonical_bytes(&legacy).unwrap();

    assert_eq!(decoded.outbox().pending().len(), 1);
    assert_eq!(decoded.ledger().retry_count(), 0);
    assert_eq!(decoded.ledger().dead_letter_count(), 0);
}

#[test]
fn retry_checkpoint_detects_corruption() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(12, store.clone(), policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let mut bytes = store.state().bytes.clone().unwrap();
    bytes[20] ^= 0x01;

    assert!(matches!(
        EffectRetryCheckpoint::from_canonical_bytes(&bytes),
        Err(EffectRetryError::RetryCheckpointChecksumMismatch { .. })
    ));
}

#[test]
fn retry_tick_cannot_move_backward_after_published_attempt() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(13, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temp")]);
    loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(10),
            &dispatcher,
            &mut backend,
        )
        .unwrap();

    assert!(matches!(
        loop_.dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(9),
            &dispatcher,
            &mut backend,
        ),
        Err(EffectRetryError::RetryTickMovedBackward { .. })
    ));
    assert_eq!(backend.calls, 1);
}

#[test]
fn retry_state_recovers_under_new_fenced_writer() {
    let store = MemoryFencedStore::default();
    let retry_policy = policy(3, 2, 8);
    let mut first = active_journal(14, store.clone(), retry_policy);
    let mut first_loop = boot(&["api.example.test"]);
    stage_one(&mut first_loop, &mut first);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temp")]);
    first_loop
        .dispatch_next_effect_with_retry_journal(
            &mut first,
            EffectRetryTick(5),
            &dispatcher,
            &mut backend,
        )
        .unwrap();

    let mut second =
        GovernedRetryEffectJournal::new(namespace(14), writer(99), store, retry_policy);
    second.acquire().unwrap();
    let mut second_loop = boot(&["api.example.test"]);
    assert!(second_loop
        .recover_effects_from_retry_journal(&mut second)
        .unwrap());

    assert_eq!(second_loop.pending_effect_count(), 1);
    assert_eq!(second.ledger().retry_count(), 1);
    assert_eq!(second.ledger().last_tick(), EffectRetryTick(5));
}

#[test]
fn dead_letter_state_recovers_under_new_writer() {
    let store = MemoryFencedStore::default();
    let retry_policy = policy(3, 1, 4);
    let mut first = active_journal(15, store.clone(), retry_policy);
    let mut first_loop = boot(&["api.example.test"]);
    stage_one(&mut first_loop, &mut first);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Permanent("bad")]);
    first_loop
        .dispatch_next_effect_with_retry_journal(
            &mut first,
            EffectRetryTick(3),
            &dispatcher,
            &mut backend,
        )
        .unwrap();

    let mut second =
        GovernedRetryEffectJournal::new(namespace(15), writer(100), store, retry_policy);
    second.acquire().unwrap();
    let mut second_loop = boot(&["api.example.test"]);
    second_loop
        .recover_effects_from_retry_journal(&mut second)
        .unwrap();

    assert_eq!(second_loop.pending_effect_count(), 0);
    assert_eq!(second.ledger().dead_letter_count(), 1);
}

#[test]
fn stale_writer_is_rejected_before_retry_backend_work() {
    let store = MemoryFencedStore::default();
    let retry_policy = policy(3, 1, 4);
    let mut first = active_journal(16, store.clone(), retry_policy);
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut first);

    let mut second = GovernedRetryEffectJournal::new(namespace(16), writer(2), store, retry_policy);
    second.acquire().unwrap();

    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::default();
    assert!(matches!(
        loop_.dispatch_next_effect_with_retry_journal(
            &mut first,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        ),
        Err(EffectRetryError::Fencing(_))
    ));
    assert_eq!(backend.calls, 0);
}

#[test]
fn retry_state_commit_failure_does_not_publish_local_retry_state() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(17, store.clone(), policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    store.state_mut().fail_commit = true;
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temp")]);

    assert!(matches!(
        loop_.dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        ),
        Err(EffectRetryError::Store(_))
    ));
    assert_eq!(journal.ledger().retry_count(), 0);
    assert_eq!(loop_.pending_effect_count(), 1);
}

#[test]
fn successful_retry_clears_retry_record_and_acknowledges() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(18, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend =
        ScriptedBackend::with_steps([BackendStep::Retryable("temp"), BackendStep::Success]);
    let first = loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        )
        .unwrap()
        .unwrap();
    let next = expect_scheduled(first).1;

    let second = loop_
        .dispatch_next_effect_with_retry_journal(&mut journal, next, &dispatcher, &mut backend)
        .unwrap()
        .unwrap();

    assert!(matches!(second, EffectRetryDispatchOutcome::Delivered(_)));
    assert_eq!(journal.ledger().retry_count(), 0);
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn successful_backend_with_failed_ack_commit_preserves_pending_and_key() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(19, store.clone(), policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    store.state_mut().fail_commit = true;
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Success, BackendStep::Success]);

    assert!(matches!(
        loop_.dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        ),
        Err(EffectRetryError::Store(_))
    ));
    assert_eq!(loop_.pending_effect_count(), 1);
    let first_key = backend.keys[0];

    store.state_mut().fail_commit = false;
    loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        )
        .unwrap();
    assert_eq!(backend.keys[1], first_key);
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn capability_denial_does_not_consume_retry_budget() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(20, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(EffectDispatchAuthority::new());
    let mut backend = ScriptedBackend::default();

    assert!(matches!(
        loop_.dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        ),
        Err(EffectRetryError::Dispatch(_))
    ));
    assert_eq!(journal.ledger().retry_count(), 0);
    assert_eq!(backend.calls, 0);
}

#[test]
fn unsupported_backend_does_not_consume_retry_budget() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(21, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = UnsupportedBackend;

    assert!(matches!(
        loop_.dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        ),
        Err(EffectRetryError::Dispatch(_))
    ));
    assert_eq!(journal.ledger().retry_count(), 0);
}

#[test]
fn retry_scheduling_does_not_change_program_replay_key() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(22, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let replay = loop_.replay_key();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Retryable("temp")]);
    loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        )
        .unwrap();

    assert_eq!(loop_.replay_key(), replay);
}

#[test]
fn dead_letter_transition_does_not_change_live_program_replay_key() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(23, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);
    stage_one(&mut loop_, &mut journal);
    let replay = loop_.replay_key();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Permanent("bad")]);
    loop_
        .dispatch_next_effect_with_retry_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        )
        .unwrap();

    assert_eq!(loop_.replay_key(), replay);
}

#[test]
fn recovery_from_dead_letter_checkpoint_restores_next_intent_sequence() {
    let store = MemoryFencedStore::default();
    let retry_policy = policy(3, 1, 4);
    let mut first = active_journal(24, store.clone(), retry_policy);
    let mut first_loop = boot(&["api.example.test"]);
    stage_one(&mut first_loop, &mut first);
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority(&["api.example.test"]));
    let mut backend = ScriptedBackend::with_steps([BackendStep::Permanent("bad")]);
    first_loop
        .dispatch_next_effect_with_retry_journal(
            &mut first,
            EffectRetryTick(1),
            &dispatcher,
            &mut backend,
        )
        .unwrap();

    let mut second =
        GovernedRetryEffectJournal::new(namespace(24), writer(77), store, retry_policy);
    second.acquire().unwrap();
    let mut second_loop = boot(&["api.example.test"]);
    second_loop
        .recover_effects_from_retry_journal(&mut second)
        .unwrap();
    second_loop
        .cycle_to_with_retry_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut second)
        .unwrap();

    assert_eq!(second_loop.pending_effects().next().unwrap().id.value(), 2);
}

#[test]
fn redrive_requires_existing_dead_letter() {
    let store = MemoryFencedStore::default();
    let mut journal = active_journal(25, store, policy(3, 1, 4));
    let mut loop_ = boot(&["api.example.test"]);

    assert!(matches!(
        loop_.redrive_dead_letter_with_retry_journal(
            &mut journal,
            nordoi_kernel::EffectIntentId(99),
        ),
        Err(EffectRetryError::UnknownDeadLetter(_))
    ));
}

#[test]
fn current_nair_version_is_0_6_after_native_completion_semantics() {
    assert_eq!(NAIR_FORMAT_MINOR, 6);
}

#[test]
fn recovery_rejects_retry_policy_drift() {
    let store = MemoryFencedStore::default();
    let first_policy = policy(3, 1, 4);
    let mut first = active_journal(26, store.clone(), first_policy);
    let mut first_loop = boot(&["api.example.test"]);
    stage_one(&mut first_loop, &mut first);

    let mut second =
        GovernedRetryEffectJournal::new(namespace(26), writer(88), store, policy(9, 2, 16));
    second.acquire().unwrap();
    let mut second_loop = boot(&["api.example.test"]);

    assert_eq!(
        second_loop.recover_effects_from_retry_journal(&mut second),
        Err(EffectRetryError::RetryPolicyMismatch)
    );
    assert_eq!(second_loop.pending_effect_count(), 0);
    assert_eq!(second.ledger().retry_count(), 0);
}
