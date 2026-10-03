use std::{cell::RefCell, rc::Rc};

use nordoi_kernel::{
    AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, DomainRef, Effect, EffectBackend,
    EffectBackendError, EffectBackendReceipt, EffectDeliveryFence, EffectDeliveryKey,
    EffectDeliveryNamespace, EffectDispatchAuthority, EffectDispatchRequest, EffectFenceStoreError,
    EffectFencingError, EffectJournalCommitReceipt, EffectJournalLease, EffectJournalWriterId,
    EffectOutboxCheckpoint, FencedEffectJournalStore, GovernedEffectDispatcher,
    GovernedFencedEffectJournal, InputDeviceId, InputPayload, InputSignal, InputSource,
    InputTargetRef, Instruction, LogicalTime, NairEffectSet, NairProgram, NairReactionAuthority,
    NairReactionStep, NairReactionTrigger, QueuedEffectIntent, ReactionSlot,
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

fn network_program(code: u32, scope: &str) -> (NairProgram, NairReactionAuthority) {
    let effect = Effect::Network(scope.into());
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "fenced-network-reaction".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code },
            },
            action_name: "fenced-network-action".into(),
            declared_effects: effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect { effect }],
        },
        Instruction::Halt,
    ]);

    let mut capabilities = CapabilitySet::new();
    capabilities.allow(Capability::Network(scope.into()));
    let mut authority = NairReactionAuthority::new();
    authority.set(ReactionSlot(0), capabilities);
    (program, authority)
}

fn dispatch_authority(scope: &str) -> EffectDispatchAuthority {
    let mut authority = EffectDispatchAuthority::new();
    authority.grant(Capability::Network(scope.into()));
    authority
}

#[derive(Debug, Default)]
struct SharedState {
    bytes: Option<Vec<u8>>,
    next_fence: u64,
    active: Option<EffectJournalLease>,
    commits: usize,
    fail_commit: bool,
    fail_assert: bool,
    force_zero: bool,
    force_namespace: Option<EffectDeliveryNamespace>,
    force_writer: Option<EffectJournalWriterId>,
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
        let lease = EffectJournalLease::new(
            state.force_namespace.unwrap_or(namespace),
            state.force_writer.unwrap_or(writer),
            EffectDeliveryFence(if state.force_zero {
                0
            } else {
                state.next_fence
            }),
        );
        state.active = Some(lease);
        Ok(lease)
    }

    fn assert_active(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        let state = self.state.borrow();
        if state.fail_assert {
            return Err(EffectFenceStoreError::new("synthetic assert failure"));
        }
        Self::require_active(&state, lease)
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
            return Err(EffectFenceStoreError::new(
                "synthetic fenced commit failure",
            ));
        }
        state.commits += 1;
        state.bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new(format!(
            "fenced-{}-{}",
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
struct FenceAwareBackend {
    calls: usize,
    keys: Vec<Option<EffectDeliveryKey>>,
    fences: Vec<Option<EffectDeliveryFence>>,
}

impl EffectBackend for FenceAwareBackend {
    fn supports(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Network(_))
    }

    fn execute(
        &mut self,
        _request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.calls += 1;
        Ok(EffectBackendReceipt::empty())
    }

    fn execute_with_context(
        &mut self,
        request: &EffectDispatchRequest,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.calls += 1;
        self.keys.push(request.delivery_key);
        self.fences.push(request.delivery_fence);
        Ok(EffectBackendReceipt::new(format!("call-{}", self.calls)))
    }
}

fn boot() -> AtomicEventLoop {
    let (program, authority) = network_program(7, "api.example.test");
    AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap()
}

#[test]
fn acquire_returns_nonzero_fence() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(1), writer(1), store);
    let lease = journal.acquire().unwrap();
    assert_eq!(lease.fence, EffectDeliveryFence(1));
}

#[test]
fn same_journal_cannot_acquire_twice_without_release() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(2), writer(1), store);
    journal.acquire().unwrap();
    assert_eq!(journal.acquire(), Err(EffectFencingError::LeaseAlreadyHeld));
}

#[test]
fn takeover_increments_fence_and_supersedes_old_writer() {
    let store = MemoryFencedStore::default();
    let mut first = GovernedFencedEffectJournal::new(namespace(3), writer(1), store.clone());
    let mut second = GovernedFencedEffectJournal::new(namespace(3), writer(2), store);
    let a = first.acquire().unwrap();
    let b = second.acquire().unwrap();
    assert!(b.fence > a.fence);
    assert!(matches!(
        first.assert_active(),
        Err(EffectFencingError::Store(_))
    ));
    assert_eq!(second.assert_active().unwrap(), b);
}

#[test]
fn stale_writer_cannot_overwrite_newer_checkpoint() {
    let store = MemoryFencedStore::default();
    let mut first = GovernedFencedEffectJournal::new(namespace(4), writer(1), store.clone());
    let mut second = GovernedFencedEffectJournal::new(namespace(4), writer(2), store.clone());
    first.acquire().unwrap();
    second.acquire().unwrap();
    second
        .checkpoint(&nordoi_kernel::AtomicEffectOutbox::new())
        .unwrap();
    let committed = store.state().bytes.clone().unwrap();
    assert!(matches!(
        first.checkpoint(&nordoi_kernel::AtomicEffectOutbox::new()),
        Err(EffectFencingError::Store(_))
    ));
    assert_eq!(store.state().bytes.as_ref().unwrap(), &committed);
}

#[test]
fn checkpoint_requires_active_lease() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(5), writer(1), store);
    assert_eq!(
        journal.checkpoint(&nordoi_kernel::AtomicEffectOutbox::new()),
        Err(EffectFencingError::LeaseRequired)
    );
}

#[test]
fn fenced_cycle_requires_lease_and_does_not_publish() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(6), writer(1), store);
    let mut loop_ = boot();
    let before = loop_.replay_key();
    assert!(matches!(
        loop_.cycle_to_with_fenced_effect_journal(
            LogicalTime::ZERO,
            &keyboard_batch(7),
            &mut journal
        ),
        Err(nordoi_kernel::EventLoopError::Fencing(
            EffectFencingError::LeaseRequired
        ))
    ));
    assert_eq!(loop_.cycle_index(), 0);
    assert_eq!(loop_.replay_key(), before);
}

#[test]
fn fenced_cycle_persists_before_publication() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(7), writer(1), store.clone());
    journal.acquire().unwrap();
    let mut loop_ = boot();
    loop_
        .cycle_to_with_fenced_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();
    assert_eq!(loop_.pending_effect_count(), 1);
    let bytes = store.state().bytes.clone().unwrap();
    assert_eq!(
        EffectOutboxCheckpoint::from_canonical_bytes(&bytes)
            .unwrap()
            .pending()
            .len(),
        1
    );
}

#[test]
fn takeover_can_recover_committed_pending_effect() {
    let store = MemoryFencedStore::default();
    let mut first = GovernedFencedEffectJournal::new(namespace(8), writer(1), store.clone());
    first.acquire().unwrap();
    let mut source = boot();
    source
        .cycle_to_with_fenced_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut first)
        .unwrap();
    let mut second = GovernedFencedEffectJournal::new(namespace(8), writer(2), store);
    second.acquire().unwrap();
    let mut recovered = boot();
    assert!(recovered
        .recover_effects_from_fenced_journal(&mut second)
        .unwrap());
    assert_eq!(recovered.pending_effect_count(), 1);
}

#[test]
fn stale_writer_cycle_is_rejected_before_candidate_publication() {
    let store = MemoryFencedStore::default();
    let mut first = GovernedFencedEffectJournal::new(namespace(9), writer(1), store.clone());
    let mut second = GovernedFencedEffectJournal::new(namespace(9), writer(2), store);
    first.acquire().unwrap();
    second.acquire().unwrap();
    let mut loop_ = boot();
    assert!(matches!(
        loop_.cycle_to_with_fenced_effect_journal(
            LogicalTime::ZERO,
            &keyboard_batch(7),
            &mut first
        ),
        Err(nordoi_kernel::EventLoopError::Fencing(
            EffectFencingError::Store(_)
        ))
    ));
    assert_eq!(loop_.cycle_index(), 0);
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn fenced_dispatch_exposes_key_and_current_fence() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(10), writer(1), store);
    let lease = journal.acquire().unwrap();
    let mut loop_ = boot();
    loop_
        .cycle_to_with_fenced_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = FenceAwareBackend::default();
    let receipt = loop_
        .dispatch_next_effect_with_fenced_journal(&mut journal, &dispatcher, &mut backend)
        .unwrap()
        .unwrap();
    let key = EffectDeliveryKey::new(namespace(10), receipt.request.id);
    assert_eq!(receipt.delivery_key, Some(key));
    assert_eq!(receipt.delivery_fence, Some(lease.fence));
    assert_eq!(backend.keys, vec![Some(key)]);
    assert_eq!(backend.fences, vec![Some(lease.fence)]);
}

#[test]
fn stale_writer_is_rejected_before_backend_execution() {
    let store = MemoryFencedStore::default();
    let mut first = GovernedFencedEffectJournal::new(namespace(11), writer(1), store.clone());
    first.acquire().unwrap();
    let mut loop_ = boot();
    loop_
        .cycle_to_with_fenced_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut first)
        .unwrap();
    let mut second = GovernedFencedEffectJournal::new(namespace(11), writer(2), store);
    second.acquire().unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = FenceAwareBackend::default();
    assert!(matches!(
        loop_.dispatch_next_effect_with_fenced_journal(&mut first, &dispatcher, &mut backend),
        Err(EffectFencingError::Store(_))
    ));
    assert_eq!(backend.calls, 0);
    assert_eq!(loop_.pending_effect_count(), 1);
}

#[test]
fn takeover_preserves_delivery_key_but_changes_fence() {
    let store = MemoryFencedStore::default();
    let mut first = GovernedFencedEffectJournal::new(namespace(12), writer(1), store.clone());
    let first_lease = first.acquire().unwrap();
    let mut loop_ = boot();
    loop_
        .cycle_to_with_fenced_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut first)
        .unwrap();
    let pending = loop_.pending_effects().next().unwrap().id;
    let mut second = GovernedFencedEffectJournal::new(namespace(12), writer(2), store);
    let second_lease = second.acquire().unwrap();
    let mut recovered = boot();
    recovered
        .recover_effects_from_fenced_journal(&mut second)
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = FenceAwareBackend::default();
    recovered
        .dispatch_next_effect_with_fenced_journal(&mut second, &dispatcher, &mut backend)
        .unwrap();
    assert_eq!(
        backend.keys[0],
        Some(EffectDeliveryKey::new(namespace(12), pending))
    );
    assert_eq!(backend.fences[0], Some(second_lease.fence));
    assert!(second_lease.fence > first_lease.fence);
}

#[test]
fn failed_fenced_ack_commit_preserves_pending_for_retry() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(13), writer(1), store.clone());
    let lease = journal.acquire().unwrap();
    let mut loop_ = boot();
    loop_
        .cycle_to_with_fenced_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();
    store.state_mut().fail_commit = true;
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = FenceAwareBackend::default();
    assert!(matches!(
        loop_.dispatch_next_effect_with_fenced_journal(&mut journal, &dispatcher, &mut backend),
        Err(EffectFencingError::Store(_))
    ));
    assert_eq!(loop_.pending_effect_count(), 1);
    let key = backend.keys[0];
    assert_eq!(backend.fences[0], Some(lease.fence));
    store.state_mut().fail_commit = false;
    loop_
        .dispatch_next_effect_with_fenced_journal(&mut journal, &dispatcher, &mut backend)
        .unwrap();
    assert_eq!(backend.keys[1], key);
    assert_eq!(backend.fences[1], Some(lease.fence));
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn release_removes_local_lease_and_blocks_future_operations() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(14), writer(1), store);
    let lease = journal.acquire().unwrap();
    assert_eq!(journal.release().unwrap(), lease);
    assert_eq!(journal.lease(), None);
    assert_eq!(
        journal.assert_active(),
        Err(EffectFencingError::LeaseRequired)
    );
}

#[test]
fn zero_fence_from_host_is_rejected() {
    let store = MemoryFencedStore::default();
    store.state_mut().force_zero = true;
    let mut journal = GovernedFencedEffectJournal::new(namespace(15), writer(1), store);
    assert_eq!(
        journal.acquire(),
        Err(EffectFencingError::InvalidFence(EffectDeliveryFence(0)))
    );
}

#[test]
fn mismatched_namespace_from_host_is_rejected() {
    let store = MemoryFencedStore::default();
    store.state_mut().force_namespace = Some(namespace(99));
    let mut journal = GovernedFencedEffectJournal::new(namespace(16), writer(1), store);
    assert!(matches!(
        journal.acquire(),
        Err(EffectFencingError::LeaseNamespaceMismatch { .. })
    ));
}

#[test]
fn mismatched_writer_from_host_is_rejected() {
    let store = MemoryFencedStore::default();
    store.state_mut().force_writer = Some(writer(99));
    let mut journal = GovernedFencedEffectJournal::new(namespace(17), writer(1), store);
    assert!(matches!(
        journal.acquire(),
        Err(EffectFencingError::LeaseWriterMismatch { .. })
    ));
}

#[test]
fn explicit_store_assert_failure_blocks_cycle() {
    let store = MemoryFencedStore::default();
    let mut journal = GovernedFencedEffectJournal::new(namespace(18), writer(1), store.clone());
    journal.acquire().unwrap();
    store.state_mut().fail_assert = true;
    let mut loop_ = boot();
    assert!(matches!(
        loop_.cycle_to_with_fenced_effect_journal(
            LogicalTime::ZERO,
            &keyboard_batch(7),
            &mut journal
        ),
        Err(nordoi_kernel::EventLoopError::Fencing(
            EffectFencingError::Store(_)
        ))
    ));
    assert_eq!(loop_.cycle_index(), 0);
}

#[test]
fn fencing_epoch_does_not_change_recovered_replay_identity() {
    let store = MemoryFencedStore::default();
    let mut first_journal =
        GovernedFencedEffectJournal::new(namespace(19), writer(1), store.clone());
    first_journal.acquire().unwrap();
    let mut source = boot();
    source
        .cycle_to_with_fenced_effect_journal(
            LogicalTime::ZERO,
            &keyboard_batch(7),
            &mut first_journal,
        )
        .unwrap();

    let mut second_journal =
        GovernedFencedEffectJournal::new(namespace(19), writer(2), store.clone());
    second_journal.acquire().unwrap();
    let mut a = boot();
    a.recover_effects_from_fenced_journal(&mut second_journal)
        .unwrap();

    let mut third_journal = GovernedFencedEffectJournal::new(namespace(19), writer(3), store);
    third_journal.acquire().unwrap();
    let mut b = boot();
    b.recover_effects_from_fenced_journal(&mut third_journal)
        .unwrap();

    assert_eq!(a.replay_key(), b.replay_key());
}
