use std::{cell::RefCell, rc::Rc};

use nordoi_kernel::{
    AtomSlot, AtomUpgradeRule, AtomicEventLoop, DomainRef, EffectDeliveryFence,
    EffectDeliveryNamespace, EffectFenceStoreError, EffectJournalCommitReceipt, EffectJournalLease,
    EffectJournalWriterId, EffectRetryPolicy, EventLoopError, FencedEffectJournalStore,
    FencedRuntimeCheckpointStore, GovernedAuditedEffectJournal, InputBatch, InputBridgeSlot,
    InputDeviceId, InputEvent, InputPayload, InputSequence, InputSignal, InputSource, InputTarget,
    InputTargetRef, Instruction, LogicalTime, NairCompletionAuthority, NairProgram,
    NairReactionAuthority, ProgramEpoch, RegisterId, RuntimeCheckpointCommitReceipt,
    RuntimeCheckpointStoreError, RuntimeError, RuntimeSemanticCheckpoint, RuntimeUpgradeAuthority,
    RuntimeUpgradeError, RuntimeUpgradeHash, RuntimeUpgradePlan, TimerSlot, Value,
    NAIR_FORMAT_MINOR,
};

fn namespace(seed: u8) -> EffectDeliveryNamespace {
    EffectDeliveryNamespace::new([seed; 16])
}

fn writer(seed: u8) -> EffectJournalWriterId {
    EffectJournalWriterId::new([seed; 16])
}

fn policy() -> EffectRetryPolicy {
    EffectRetryPolicy::new(3, 5, 20, 0).unwrap()
}

#[derive(Debug, Default)]
struct StoreState {
    effect_bytes: Option<Vec<u8>>,
    runtime_bytes: Option<Vec<u8>>,
    next_fence: u64,
    active: Option<EffectJournalLease>,
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

    fn bundle_commits(&self) -> usize {
        self.state.borrow().bundle_commits
    }

    fn runtime_bytes(&self) -> Option<Vec<u8>> {
        self.state.borrow().runtime_bytes.clone()
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
        state.effect_bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new(format!(
            "effect-{}",
            lease.fence.0
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

fn source_program() -> NairProgram {
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
        Instruction::Halt,
    ])
}

fn target_program() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Text("fresh".into()),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(7),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(8),
            owner: DomainRef::Root,
            value: RegisterId(1),
        },
        Instruction::Halt,
    ])
}

fn target_program_two() -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Text("second".into()),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(20),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(21),
            owner: DomainRef::Root,
            value: RegisterId(1),
        },
        Instruction::Halt,
    ])
}

fn target_program_with_timer(deadline: u64) -> NairProgram {
    let mut instructions = target_program().instructions().to_vec();
    instructions.pop();
    instructions.push(Instruction::ScheduleTimerOnceAt {
        dst: TimerSlot(0),
        deadline: LogicalTime(deadline),
    });
    instructions.push(Instruction::Halt);
    NairProgram::from_instructions(instructions)
}

fn key_batch(sequence: u64) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(sequence),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: 13,
                pressed: true,
                repeat: false,
            },
        }],
    }
}

fn copy_plan(source: &AtomicEventLoop, target: &NairProgram) -> RuntimeUpgradePlan {
    let target_hash = AtomicEventLoop::boot(target).unwrap().program_hash();
    RuntimeUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        [
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap()
}

fn authority_for(plan: &RuntimeUpgradePlan) -> RuntimeUpgradeAuthority {
    let mut authority = RuntimeUpgradeAuthority::new();
    authority.grant(plan.source_program_hash(), plan.target_program_hash());
    authority
}

fn durable_source(
    seed: u8,
    time: LogicalTime,
) -> (
    MemoryRuntimeStore,
    GovernedAuditedEffectJournal<MemoryRuntimeStore>,
    AtomicEventLoop,
) {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(seed, 1, store.clone());
    let mut event_loop = AtomicEventLoop::boot(&source_program()).unwrap();
    event_loop
        .cycle_to_with_runtime_checkpoint(time, &key_batch(1), &mut journal)
        .unwrap();
    (store, journal, event_loop)
}

fn empty_authorities() -> (NairReactionAuthority, NairCompletionAuthority) {
    (NairReactionAuthority::new(), NairCompletionAuthority::new())
}

#[test]
fn plan_rejects_identical_program_hashes() {
    let hash = [1; 32];
    assert_eq!(
        RuntimeUpgradePlan::new(hash, hash, ProgramEpoch(0), []),
        Err(RuntimeUpgradeError::SameProgram)
    );
}

#[test]
fn plan_rejects_duplicate_source_disposition() {
    let result = RuntimeUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [
            AtomUpgradeRule::DropSource {
                source: AtomSlot(0),
            },
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(7),
            },
        ],
    );
    assert_eq!(
        result,
        Err(RuntimeUpgradeError::DuplicateSourceAtomDisposition(
            AtomSlot(0)
        ))
    );
}

#[test]
fn plan_rejects_duplicate_target_disposition() {
    let result = RuntimeUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(7),
            },
        ],
    );
    assert_eq!(
        result,
        Err(RuntimeUpgradeError::DuplicateTargetAtomDisposition(
            AtomSlot(7)
        ))
    );
}

#[test]
fn upgrade_requires_existing_durable_runtime_lineage() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(1, 1, store);
    let mut source = AtomicEventLoop::boot(&source_program()).unwrap();
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::UpgradeRequiresDurableRuntimeBinding)
    );
}

#[test]
fn unauthorized_upgrade_is_rejected() {
    let (_store, mut journal, mut source) = durable_source(2, LogicalTime::ZERO);
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &RuntimeUpgradeAuthority::new(),
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::UnauthorizedTransition)
    );
}

#[test]
fn revoked_upgrade_authority_is_rejected() {
    let (_store, mut journal, mut source) = durable_source(3, LogicalTime::ZERO);
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let mut authority = authority_for(&plan);
    authority.revoke(plan.source_program_hash(), plan.target_program_hash());
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::UnauthorizedTransition)
    );
}

#[test]
fn source_program_hash_must_match_live_runtime() {
    let (_store, mut journal, mut source) = durable_source(4, LogicalTime::ZERO);
    let target = target_program();
    let target_hash = AtomicEventLoop::boot(&target).unwrap().program_hash();
    let plan = RuntimeUpgradePlan::new(
        [9; 32],
        target_hash,
        source.program_epoch(),
        [
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap();
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::SourceProgramMismatch)
    );
}

#[test]
fn target_program_hash_must_match_booted_target() {
    let (_store, mut journal, mut source) = durable_source(5, LogicalTime::ZERO);
    let target = target_program();
    let plan = RuntimeUpgradePlan::new(
        source.program_hash(),
        [8; 32],
        source.program_epoch(),
        [
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap();
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::TargetProgramMismatch)
    );
}

#[test]
fn source_epoch_must_match_live_runtime() {
    let (_store, mut journal, mut source) = durable_source(6, LogicalTime::ZERO);
    let target = target_program();
    let target_hash = AtomicEventLoop::boot(&target).unwrap().program_hash();
    let plan = RuntimeUpgradePlan::new(
        source.program_hash(),
        target_hash,
        ProgramEpoch(9),
        [
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap();
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::SourceEpochMismatch {
            expected: 9,
            actual: 0,
        })
    );
}

#[test]
fn every_source_atom_requires_explicit_disposition() {
    let (_store, mut journal, mut source) = durable_source(7, LogicalTime::ZERO);
    let target = target_program();
    let target_hash = AtomicEventLoop::boot(&target).unwrap().program_hash();
    let plan = RuntimeUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        [
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap();
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::MissingSourceAtomDisposition(AtomSlot(
            0
        )))
    );
}

#[test]
fn every_target_atom_requires_copy_or_explicit_default() {
    let (_store, mut journal, mut source) = durable_source(8, LogicalTime::ZERO);
    let target = target_program();
    let target_hash = AtomicEventLoop::boot(&target).unwrap().program_hash();
    let plan = RuntimeUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        [AtomUpgradeRule::Copy {
            source: AtomSlot(0),
            target: AtomSlot(7),
        }],
    )
    .unwrap();
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::MissingTargetAtomDisposition(AtomSlot(
            8
        )))
    );
}

#[test]
fn unknown_source_atom_fails_closed() {
    let (_store, mut journal, mut source) = durable_source(9, LogicalTime::ZERO);
    let target = target_program();
    let target_hash = AtomicEventLoop::boot(&target).unwrap().program_hash();
    let plan = RuntimeUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        [
            AtomUpgradeRule::DropSource {
                source: AtomSlot(0),
            },
            AtomUpgradeRule::Copy {
                source: AtomSlot(99),
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap();
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::UnknownSourceAtom(AtomSlot(99)))
    );
}

#[test]
fn unknown_target_atom_fails_closed() {
    let (_store, mut journal, mut source) = durable_source(10, LogicalTime::ZERO);
    let target = target_program();
    let target_hash = AtomicEventLoop::boot(&target).unwrap().program_hash();
    let plan = RuntimeUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        [
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(99),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap();
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::UnknownTargetAtom(AtomSlot(99)))
    );
}

#[test]
fn copy_and_keep_default_migrate_state_deterministically() {
    let (_store, mut journal, mut source) = durable_source(11, LogicalTime::ZERO);
    assert_eq!(
        source.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(true)
    );
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let report = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap();
    let snapshot = source.snapshot().unwrap();
    assert_eq!(snapshot[&AtomSlot(7)].value, Value::Bool(true));
    assert_eq!(snapshot[&AtomSlot(8)].value, Value::Text("fresh".into()));
    assert_eq!(report.migrated_atoms, 1);
    assert_eq!(report.defaulted_atoms, 1);
    assert_eq!(report.dropped_atoms, 0);
}

#[test]
fn explicit_drop_allows_source_state_to_be_removed() {
    let (_store, mut journal, mut source) = durable_source(12, LogicalTime::ZERO);
    let target = target_program();
    let target_hash = AtomicEventLoop::boot(&target).unwrap().program_hash();
    let plan = RuntimeUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        [
            AtomUpgradeRule::DropSource {
                source: AtomSlot(0),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap();
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let report = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap();
    let snapshot = source.snapshot().unwrap();
    assert_eq!(snapshot[&AtomSlot(7)].value, Value::Bool(false));
    assert_eq!(report.dropped_atoms, 1);
    assert_eq!(report.migrated_atoms, 0);
}

#[test]
fn upgrade_preserves_cycle_and_logical_time() {
    let (_store, mut journal, mut source) = durable_source(13, LogicalTime(5));
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let report = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap();
    assert_eq!(report.cycle, 1);
    assert_eq!(source.cycle_index(), 1);
    assert_eq!(report.logical_time, LogicalTime(5));
    assert_eq!(source.logical_time(), LogicalTime(5));
}

#[test]
fn upgrade_changes_replay_identity_and_program_epoch() {
    let (_store, mut journal, mut source) = durable_source(14, LogicalTime::ZERO);
    let replay = source.replay_key();
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let report = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap();
    assert_ne!(source.replay_key(), replay);
    assert_eq!(source.program_epoch(), ProgramEpoch(1));
    assert_eq!(report.target_epoch, ProgramEpoch(1));
    assert_ne!(source.upgrade_chain_root(), RuntimeUpgradeHash::ZERO);
}

#[test]
fn identical_upgrade_traces_produce_identical_replay_and_lineage() {
    fn run() -> (nordoi_kernel::EventLoopReplayKey, RuntimeUpgradeHash) {
        let (_store, mut journal, mut source) = durable_source(15, LogicalTime::ZERO);
        let target = target_program();
        let plan = copy_plan(&source, &target);
        let authority = authority_for(&plan);
        let (reactions, completions) = empty_authorities();
        source
            .upgrade_program_with_runtime_checkpoint(
                &target,
                &reactions,
                &completions,
                &authority,
                &plan,
                &mut journal,
            )
            .unwrap();
        (source.replay_key(), source.upgrade_chain_root())
    }
    assert_eq!(run(), run());
}

#[test]
fn different_atom_migration_plan_changes_replay_identity() {
    fn run(copy_target: AtomSlot, default_target: AtomSlot) -> nordoi_kernel::EventLoopReplayKey {
        let (_store, mut journal, mut source) = durable_source(16, LogicalTime::ZERO);
        let target = target_program();
        let target_hash = AtomicEventLoop::boot(&target).unwrap().program_hash();
        let plan = RuntimeUpgradePlan::new(
            source.program_hash(),
            target_hash,
            source.program_epoch(),
            [
                AtomUpgradeRule::Copy {
                    source: AtomSlot(0),
                    target: copy_target,
                },
                AtomUpgradeRule::KeepTargetDefault {
                    target: default_target,
                },
            ],
        )
        .unwrap();
        let authority = authority_for(&plan);
        let (reactions, completions) = empty_authorities();
        source
            .upgrade_program_with_runtime_checkpoint(
                &target,
                &reactions,
                &completions,
                &authority,
                &plan,
                &mut journal,
            )
            .unwrap();
        source.replay_key()
    }
    assert_ne!(run(AtomSlot(7), AtomSlot(8)), run(AtomSlot(8), AtomSlot(7)));
}

#[test]
fn failed_upgrade_bundle_commit_publishes_no_live_upgrade() {
    let (store, mut journal, mut source) = durable_source(17, LogicalTime::ZERO);
    let old_hash = source.program_hash();
    let old_epoch = source.program_epoch();
    let old_replay = source.replay_key();
    let old_snapshot = source.snapshot().unwrap();
    let old_runtime_bytes = store.runtime_bytes().unwrap();
    let old_commits = store.bundle_commits();
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    store.fail_next_bundle();
    assert!(source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .is_err());
    assert_eq!(source.program_hash(), old_hash);
    assert_eq!(source.program_epoch(), old_epoch);
    assert_eq!(source.replay_key(), old_replay);
    assert_eq!(source.snapshot().unwrap(), old_snapshot);
    assert_eq!(store.runtime_bytes().unwrap(), old_runtime_bytes);
    assert_eq!(store.bundle_commits(), old_commits);
}

#[test]
fn pending_source_timer_blocks_upgrade() {
    let (_store, mut journal, mut source) = durable_source(18, LogicalTime::ZERO);
    source.schedule_once_at(LogicalTime(20)).unwrap();
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::PendingTimersUnsupported { count: 1 })
    );
}

#[test]
fn target_timer_cannot_start_before_upgrade_logical_time() {
    let (_store, mut journal, mut source) = durable_source(19, LogicalTime(10));
    let target = target_program_with_timer(5);
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::TargetTimerDeadlineBeforeUpgradeTime {
            timer: 1,
            deadline: 5,
            logical_time: 10,
        })
    );
}

#[test]
fn future_target_timer_is_allowed_and_uses_preserved_logical_time() {
    let (_store, mut journal, mut source) = durable_source(20, LogicalTime(10));
    let target = target_program_with_timer(15);
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap();
    assert_eq!(source.logical_time(), LogicalTime(10));
    assert_eq!(source.pending_timers(), 1);
}

#[test]
fn recovered_target_continues_from_upgraded_epoch_and_state() {
    let (store, mut journal, mut source) = durable_source(21, LogicalTime::ZERO);
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap();

    let mut recovered_journal = active_journal(21, 2, store);
    let mut recovered = AtomicEventLoop::boot(&target).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap()
        .unwrap();
    assert_eq!(recovered.program_epoch(), ProgramEpoch(1));
    assert_eq!(recovered.upgrade_chain_root(), source.upgrade_chain_root());
    assert_eq!(
        recovered.snapshot().unwrap()[&AtomSlot(7)].value,
        Value::Bool(true)
    );
    assert_eq!(recovered.replay_key(), source.replay_key());
}

#[test]
fn second_upgrade_increments_epoch_and_extends_lineage() {
    let (_store, mut journal, mut source) = durable_source(22, LogicalTime::ZERO);
    let target = target_program();
    let plan1 = copy_plan(&source, &target);
    let authority1 = authority_for(&plan1);
    let (reactions, completions) = empty_authorities();
    source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority1,
            &plan1,
            &mut journal,
        )
        .unwrap();
    let first_root = source.upgrade_chain_root();

    let target2 = target_program_two();
    let target2_hash = AtomicEventLoop::boot(&target2).unwrap().program_hash();
    let plan2 = RuntimeUpgradePlan::new(
        source.program_hash(),
        target2_hash,
        source.program_epoch(),
        [
            AtomUpgradeRule::Copy {
                source: AtomSlot(7),
                target: AtomSlot(20),
            },
            AtomUpgradeRule::DropSource {
                source: AtomSlot(8),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(21),
            },
        ],
    )
    .unwrap();
    let authority2 = authority_for(&plan2);
    source
        .upgrade_program_with_runtime_checkpoint(
            &target2,
            &reactions,
            &completions,
            &authority2,
            &plan2,
            &mut journal,
        )
        .unwrap();
    assert_eq!(source.program_epoch(), ProgramEpoch(2));
    assert_ne!(source.upgrade_chain_root(), first_root);
    assert_eq!(
        source.snapshot().unwrap()[&AtomSlot(20)].value,
        Value::Bool(true)
    );
}

#[test]
fn last_upgrade_record_is_persisted_in_runtime_checkpoint() {
    let (store, mut journal, mut source) = durable_source(23, LogicalTime::ZERO);
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    let report = source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap();
    let decoded =
        RuntimeSemanticCheckpoint::from_canonical_bytes(&store.runtime_bytes().unwrap()).unwrap();
    let record = decoded.last_upgrade().unwrap();
    assert_eq!(record.source_program_hash, report.source_program_hash);
    assert_eq!(record.target_program_hash, report.target_program_hash);
    assert_eq!(record.plan_hash, report.plan_hash);
    assert_eq!(record.source_checkpoint_hash, report.source_checkpoint_hash);
}

#[test]
fn input_sequence_frontier_survives_program_upgrade() {
    let (_store, mut journal, mut source) = durable_source(24, LogicalTime::ZERO);
    let target = target_program();
    let plan = copy_plan(&source, &target);
    let authority = authority_for(&plan);
    let (reactions, completions) = empty_authorities();
    source
        .upgrade_program_with_runtime_checkpoint(
            &target,
            &reactions,
            &completions,
            &authority,
            &plan,
            &mut journal,
        )
        .unwrap();
    let error = source
        .cycle_to(LogicalTime::ZERO, &key_batch(1))
        .unwrap_err();
    assert!(matches!(
        error,
        EventLoopError::Runtime(RuntimeError::Input(_))
    ));
}

#[test]
fn plan_hash_is_stable_under_rule_input_order() {
    let source_hash = [1; 32];
    let target_hash = [2; 32];
    let a = RuntimeUpgradePlan::new(
        source_hash,
        target_hash,
        ProgramEpoch(0),
        [
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(7),
            },
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
        ],
    )
    .unwrap();
    let b = RuntimeUpgradePlan::new(
        source_hash,
        target_hash,
        ProgramEpoch(0),
        [
            AtomUpgradeRule::KeepTargetDefault {
                target: AtomSlot(8),
            },
            AtomUpgradeRule::Copy {
                source: AtomSlot(0),
                target: AtomSlot(7),
            },
        ],
    )
    .unwrap();
    assert_eq!(a.plan_hash(), b.plan_hash());
    assert_eq!(a.canonical_bytes(), b.canonical_bytes());
}

#[test]
fn delivery_namespace_changes_lineage_but_not_upgrade_replay_semantics() {
    fn run(seed: u8) -> (nordoi_kernel::EventLoopReplayKey, RuntimeUpgradeHash) {
        let (_store, mut journal, mut source) = durable_source(seed, LogicalTime::ZERO);
        let target = target_program();
        let plan = copy_plan(&source, &target);
        let authority = authority_for(&plan);
        let (reactions, completions) = empty_authorities();
        source
            .upgrade_program_with_runtime_checkpoint(
                &target,
                &reactions,
                &completions,
                &authority,
                &plan,
                &mut journal,
            )
            .unwrap();
        (source.replay_key(), source.upgrade_chain_root())
    }
    let first = run(30);
    let second = run(31);
    assert_eq!(first.0, second.0);
    assert_ne!(first.1, second.1);
}

#[test]
fn nair_version_remains_0_6_in_k115() {
    assert_eq!(NAIR_FORMAT_MINOR, 6);
}
