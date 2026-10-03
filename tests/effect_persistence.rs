use nordoi_kernel::{
    AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, DomainRef, Effect, EffectBackend,
    EffectBackendError, EffectBackendReceipt, EffectDeliveryKey, EffectDeliveryNamespace,
    EffectDispatchAuthority, EffectDispatchError, EffectDispatchRequest,
    EffectJournalCommitReceipt, EffectJournalStore, EffectJournalStoreError,
    EffectOutboxCheckpoint, EffectPersistenceError, GovernedEffectDispatcher,
    GovernedEffectJournal, InputDeviceId, InputPayload, InputSignal, InputSource, InputTargetRef,
    Instruction, LogicalTime, NairEffectSet, NairProgram, NairReactionAuthority, NairReactionStep,
    NairReactionTrigger, QueuedEffectIntent, ReactionSlot,
};

fn effects(effects: impl IntoIterator<Item = Effect>) -> NairEffectSet {
    NairEffectSet::from_effects(effects)
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
            name: "persistent-network-reaction".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code },
            },
            action_name: "persistent-network-action".into(),
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

fn namespace(seed: u8) -> EffectDeliveryNamespace {
    EffectDeliveryNamespace::new([seed; 16])
}

#[derive(Debug, Clone, Default)]
struct MemoryJournalStore {
    bytes: Option<Vec<u8>>,
    commits: usize,
    fail_commit: bool,
    fail_load: bool,
}

impl EffectJournalStore for MemoryJournalStore {
    fn load(&mut self) -> Result<Option<Vec<u8>>, EffectJournalStoreError> {
        if self.fail_load {
            Err(EffectJournalStoreError::new("synthetic load failure"))
        } else {
            Ok(self.bytes.clone())
        }
    }

    fn commit(
        &mut self,
        bytes: &[u8],
    ) -> Result<EffectJournalCommitReceipt, EffectJournalStoreError> {
        if self.fail_commit {
            return Err(EffectJournalStoreError::new("synthetic commit failure"));
        }
        self.commits += 1;
        self.bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new(format!(
            "commit-{}",
            self.commits
        )))
    }
}

#[derive(Debug, Default)]
struct ContextBackend {
    fail: bool,
    seen: Vec<QueuedEffectIntent>,
    keys: Vec<Option<EffectDeliveryKey>>,
}

impl EffectBackend for ContextBackend {
    fn supports(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Network(_))
    }

    fn execute(
        &mut self,
        request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.seen.push(request.clone());
        if self.fail {
            Err(EffectBackendError::new("synthetic backend failure"))
        } else {
            Ok(EffectBackendReceipt::new(format!(
                "legacy-{}",
                request.id.value()
            )))
        }
    }

    fn execute_with_context(
        &mut self,
        request: &EffectDispatchRequest,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.keys.push(request.delivery_key);
        self.seen.push(request.queued.clone());
        if self.fail {
            Err(EffectBackendError::new("synthetic backend failure"))
        } else {
            Ok(EffectBackendReceipt::new(format!(
                "context-{}",
                request.queued.id.value()
            )))
        }
    }
}

fn loop_with_one_pending_effect() -> AtomicEventLoop {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7))
        .unwrap();
    loop_
}

#[test]
fn checkpoint_round_trip_preserves_pending_state() {
    let loop_ = loop_with_one_pending_effect();
    let checkpoint = loop_.effect_checkpoint(namespace(1));
    let decoded =
        EffectOutboxCheckpoint::from_canonical_bytes(&checkpoint.canonical_bytes()).unwrap();

    assert_eq!(decoded.namespace(), namespace(1));
    assert_eq!(decoded.next_intent_id(), 2);
    assert_eq!(decoded.pending(), checkpoint.pending());
}

#[test]
fn checkpoint_encoding_is_byte_stable() {
    let loop_ = loop_with_one_pending_effect();
    let checkpoint = loop_.effect_checkpoint(namespace(2));
    let first = checkpoint.canonical_bytes();
    let second = checkpoint.canonical_bytes();
    assert_eq!(first, second);
}

#[test]
fn checkpoint_detects_single_byte_corruption() {
    let loop_ = loop_with_one_pending_effect();
    let checkpoint = loop_.effect_checkpoint(namespace(3));
    let mut bytes = checkpoint.canonical_bytes();
    bytes[20] ^= 0x01;

    assert!(matches!(
        EffectOutboxCheckpoint::from_canonical_bytes(&bytes),
        Err(EffectPersistenceError::ChecksumMismatch { .. })
    ));
}

#[test]
fn checkpoint_rejects_truncated_input() {
    assert_eq!(
        EffectOutboxCheckpoint::from_canonical_bytes(&[1, 2, 3, 4]),
        Err(EffectPersistenceError::Truncated)
    );
}

#[test]
fn journal_store_load_failure_is_reported() {
    let store = MemoryJournalStore {
        fail_load: true,
        ..MemoryJournalStore::default()
    };
    let mut journal = GovernedEffectJournal::new(namespace(4), store);
    assert!(matches!(
        journal.recover(),
        Err(EffectPersistenceError::Store(_))
    ));
}

#[test]
fn journal_commit_then_recover_preserves_checkpoint() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut journal = GovernedEffectJournal::new(namespace(5), MemoryJournalStore::default());
    loop_
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();

    let recovered = journal.recover().unwrap().unwrap();
    assert_eq!(recovered.pending().len(), 1);
    assert_eq!(recovered.pending()[0].id.value(), 1);
    assert_eq!(journal.store().commits, 1);
}

#[test]
fn empty_store_recovery_is_a_noop() {
    let mut journal = GovernedEffectJournal::new(namespace(6), MemoryJournalStore::default());
    assert!(journal.recover().unwrap().is_none());
}

#[test]
fn journal_rejects_namespace_mismatch() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut first = GovernedEffectJournal::new(namespace(7), MemoryJournalStore::default());
    loop_
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut first)
        .unwrap();
    let store = first.into_store();
    let mut second = GovernedEffectJournal::new(namespace(8), store);

    assert!(matches!(
        second.recover(),
        Err(EffectPersistenceError::NamespaceMismatch { .. })
    ));
}

#[test]
fn cycle_with_journal_persists_candidate_before_publication() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut journal = GovernedEffectJournal::new(namespace(9), MemoryJournalStore::default());

    let report = loop_
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();

    assert_eq!(report.effects.len(), 1);
    assert_eq!(loop_.cycle_index(), 1);
    assert_eq!(loop_.pending_effect_count(), 1);
    let persisted =
        EffectOutboxCheckpoint::from_canonical_bytes(journal.store().bytes.as_deref().unwrap())
            .unwrap();
    assert_eq!(persisted.pending().len(), 1);
}

#[test]
fn journal_store_failure_aborts_cycle_publication() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let initial_replay = loop_.replay_key();
    let store = MemoryJournalStore {
        fail_commit: true,
        ..MemoryJournalStore::default()
    };
    let mut journal = GovernedEffectJournal::new(namespace(10), store);

    assert!(matches!(
        loop_.cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal),
        Err(nordoi_kernel::EventLoopError::Persistence(
            EffectPersistenceError::Store(_)
        ))
    ));
    assert_eq!(loop_.cycle_index(), 0);
    assert_eq!(loop_.pending_effect_count(), 0);
    assert_eq!(loop_.replay_key(), initial_replay);
}

#[test]
fn recovered_checkpoint_restores_pending_effect_before_first_cycle() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut first = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut journal = GovernedEffectJournal::new(namespace(11), MemoryJournalStore::default());
    first
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();

    let store = journal.into_store();
    let mut recovered_journal = GovernedEffectJournal::new(namespace(11), store);
    let mut second = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    assert!(second
        .recover_effects_from_journal(&mut recovered_journal)
        .unwrap());
    assert_eq!(second.pending_effect_count(), 1);
    assert_eq!(second.pending_effects().next().unwrap().id.value(), 1);
}

#[test]
fn recovery_changes_replay_identity_explicitly() {
    let (program, authority) = network_program(7, "api.example.test");
    let fresh = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut source = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    source
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7))
        .unwrap();
    let checkpoint = source.effect_checkpoint(namespace(12));
    let mut recovered =
        AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    recovered.restore_effect_checkpoint(&checkpoint).unwrap();

    assert_ne!(fresh.replay_key(), recovered.replay_key());
}

#[test]
fn recovery_after_first_cycle_is_rejected() {
    let mut loop_ = loop_with_one_pending_effect();
    let checkpoint = loop_.effect_checkpoint(namespace(13));
    assert_eq!(
        loop_.restore_effect_checkpoint(&checkpoint),
        Err(EffectPersistenceError::RecoveryAfterCycleStarted { cycle: 1 })
    );
}

#[test]
fn recovered_outbox_continues_monotonic_intent_ids() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut first = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut journal = GovernedEffectJournal::new(namespace(14), MemoryJournalStore::default());
    first
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();

    let store = journal.into_store();
    let mut journal = GovernedEffectJournal::new(namespace(14), store);
    let mut second = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    second.recover_effects_from_journal(&mut journal).unwrap();
    let report = second
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();

    assert_eq!(report.effects.enqueued[0].id.value(), 2);
    assert_eq!(second.pending_effect_count(), 2);
}

#[test]
fn durable_dispatch_exposes_stable_delivery_key() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut journal = GovernedEffectJournal::new(namespace(15), MemoryJournalStore::default());
    loop_
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = ContextBackend::default();

    let receipt = loop_
        .dispatch_next_effect_with_journal(&mut journal, &dispatcher, &mut backend)
        .unwrap()
        .unwrap();

    let expected = EffectDeliveryKey::new(namespace(15), receipt.request.id);
    assert_eq!(receipt.delivery_key, Some(expected));
    assert_eq!(backend.keys, vec![Some(expected)]);
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn failed_ack_checkpoint_preserves_pending_and_retry_key() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut journal = GovernedEffectJournal::new(namespace(16), MemoryJournalStore::default());
    loop_
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();
    journal.store_mut().fail_commit = true;
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = ContextBackend::default();

    assert!(matches!(
        loop_.dispatch_next_effect_with_journal(&mut journal, &dispatcher, &mut backend),
        Err(EffectPersistenceError::Store(_))
    ));
    assert_eq!(loop_.pending_effect_count(), 1);
    let first_key = backend.keys[0].unwrap();

    journal.store_mut().fail_commit = false;
    loop_
        .dispatch_next_effect_with_journal(&mut journal, &dispatcher, &mut backend)
        .unwrap();
    assert_eq!(backend.keys[1], Some(first_key));
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn durable_dispatch_persists_acknowledgement() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut journal = GovernedEffectJournal::new(namespace(17), MemoryJournalStore::default());
    loop_
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = ContextBackend::default();

    loop_
        .dispatch_next_effect_with_journal(&mut journal, &dispatcher, &mut backend)
        .unwrap();
    let checkpoint =
        EffectOutboxCheckpoint::from_canonical_bytes(journal.store().bytes.as_deref().unwrap())
            .unwrap();
    assert!(checkpoint.pending().is_empty());
    assert_eq!(checkpoint.next_intent_id(), 2);
}

#[test]
fn backend_failure_does_not_persist_acknowledgement() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut journal = GovernedEffectJournal::new(namespace(18), MemoryJournalStore::default());
    loop_
        .cycle_to_with_effect_journal(LogicalTime::ZERO, &keyboard_batch(7), &mut journal)
        .unwrap();
    let before = journal.store().bytes.clone();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = ContextBackend {
        fail: true,
        ..ContextBackend::default()
    };

    assert!(matches!(
        loop_.dispatch_next_effect_with_journal(&mut journal, &dispatcher, &mut backend),
        Err(EffectPersistenceError::Dispatch(
            EffectDispatchError::BackendFailed { .. }
        ))
    ));
    assert_eq!(loop_.pending_effect_count(), 1);
    assert_eq!(journal.store().bytes, before);
}

#[test]
fn different_namespaces_produce_different_delivery_keys() {
    let intent = nordoi_kernel::EffectIntentId(42);
    assert_ne!(
        EffectDeliveryKey::new(namespace(19), intent),
        EffectDeliveryKey::new(namespace(20), intent)
    );
}
