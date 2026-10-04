use std::{cell::RefCell, rc::Rc};

use nordoi_kernel::{
    AtomSlot, AtomUpgradeRule, AtomicEventLoop, DomainRef, EffectDeliveryFence,
    EffectDeliveryNamespace, EffectFenceStoreError, EffectJournalCommitReceipt, EffectJournalLease,
    EffectJournalWriterId, EffectRetryPolicy, EventLoopError, FencedEffectJournalStore,
    FencedRuntimeCheckpointStore, GovernedAuditedEffectJournal, InputBatch, Instruction,
    LogicalDuration, LogicalTime, NairCompletionAuthority, NairProgram, NairReactionAuthority,
    ProgramEpoch, RegisterId, RuntimeCheckpointCommitReceipt, RuntimeCheckpointStoreError,
    RuntimeSemanticCheckpoint, RuntimeTimerAwareUpgradePlan, RuntimeTimerUpgradePlan,
    RuntimeUpgradeAuthority, RuntimeUpgradeError, RuntimeUpgradePlan, TimerSlot, TimerUpgradeRule,
    Value, NAIR_FORMAT_MINOR,
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
        state.effect_bytes = Some(effect_checkpoint.to_vec());
        state.runtime_bytes = Some(runtime_checkpoint.to_vec());
        Ok(RuntimeCheckpointCommitReceipt::new(format!(
            "bundle-{}",
            lease.fence.0
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

fn program_with_timers(timer_instructions: Vec<Instruction>, atom: AtomSlot) -> NairProgram {
    let mut instructions = vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(7),
        },
        Instruction::CreateAtom {
            dst: atom,
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
    ];
    instructions.extend(timer_instructions);
    instructions.push(Instruction::Halt);
    NairProgram::from_instructions(instructions)
}

fn source_once(deadline: u64) -> NairProgram {
    program_with_timers(
        vec![Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(deadline),
        }],
        AtomSlot(0),
    )
}

fn source_repeating(first: u64, interval: u64) -> NairProgram {
    program_with_timers(
        vec![Instruction::ScheduleTimerRepeatingAt {
            dst: TimerSlot(0),
            first_deadline: LogicalTime(first),
            interval: LogicalDuration(interval),
        }],
        AtomSlot(0),
    )
}

fn source_two_timers() -> NairProgram {
    program_with_timers(
        vec![
            Instruction::ScheduleTimerOnceAt {
                dst: TimerSlot(0),
                deadline: LogicalTime(20),
            },
            Instruction::ScheduleTimerOnceAt {
                dst: TimerSlot(1),
                deadline: LogicalTime(30),
            },
        ],
        AtomSlot(0),
    )
}

fn target_once(deadline: u64) -> NairProgram {
    program_with_timers(
        vec![Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(5),
            deadline: LogicalTime(deadline),
        }],
        AtomSlot(7),
    )
}

fn target_repeating(first: u64, interval: u64) -> NairProgram {
    program_with_timers(
        vec![Instruction::ScheduleTimerRepeatingAt {
            dst: TimerSlot(5),
            first_deadline: LogicalTime(first),
            interval: LogicalDuration(interval),
        }],
        AtomSlot(7),
    )
}

fn target_two_once(first: u64, second: u64) -> NairProgram {
    program_with_timers(
        vec![
            Instruction::ScheduleTimerOnceAt {
                dst: TimerSlot(4),
                deadline: LogicalTime(first),
            },
            Instruction::ScheduleTimerOnceAt {
                dst: TimerSlot(5),
                deadline: LogicalTime(second),
            },
        ],
        AtomSlot(7),
    )
}

fn target_cancelled_once(deadline: u64) -> NairProgram {
    program_with_timers(
        vec![
            Instruction::ScheduleTimerOnceAt {
                dst: TimerSlot(5),
                deadline: LogicalTime(deadline),
            },
            Instruction::CancelTimer {
                timer: TimerSlot(5),
            },
        ],
        AtomSlot(7),
    )
}

fn atom_plan(source: &AtomicEventLoop, target: &NairProgram) -> RuntimeUpgradePlan {
    let target_hash = AtomicEventLoop::boot(target).unwrap().program_hash();
    RuntimeUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        [AtomUpgradeRule::Copy {
            source: AtomSlot(0),
            target: AtomSlot(7),
        }],
    )
    .unwrap()
}

fn timer_plan(
    source: &AtomicEventLoop,
    target: &NairProgram,
    rules: impl IntoIterator<Item = TimerUpgradeRule>,
) -> RuntimeTimerUpgradePlan {
    let target_hash = AtomicEventLoop::boot(target).unwrap().program_hash();
    RuntimeTimerUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        rules,
    )
    .unwrap()
}

fn authority_for(plan: &RuntimeUpgradePlan) -> RuntimeUpgradeAuthority {
    let mut authority = RuntimeUpgradeAuthority::new();
    authority.grant(plan.source_program_hash(), plan.target_program_hash());
    authority
}

fn empty_authorities() -> (NairReactionAuthority, NairCompletionAuthority) {
    (NairReactionAuthority::new(), NairCompletionAuthority::new())
}

fn durable_source(
    seed: u8,
    program: &NairProgram,
    time: LogicalTime,
) -> (
    MemoryRuntimeStore,
    GovernedAuditedEffectJournal<MemoryRuntimeStore>,
    AtomicEventLoop,
) {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(seed, 1, store.clone());
    let mut event_loop = AtomicEventLoop::boot(program).unwrap();
    event_loop
        .cycle_to_with_runtime_checkpoint(time, &InputBatch::default(), &mut journal)
        .unwrap();
    (store, journal, event_loop)
}

fn upgrade_with_timers(
    source: &mut AtomicEventLoop,
    target: &NairProgram,
    atom_plan: &RuntimeUpgradePlan,
    timer_plan: &RuntimeTimerUpgradePlan,
    journal: &mut GovernedAuditedEffectJournal<MemoryRuntimeStore>,
) -> nordoi_kernel::RuntimeUpgradeReport {
    let authority = authority_for(atom_plan);
    let migration = RuntimeTimerAwareUpgradePlan::new(atom_plan, timer_plan).unwrap();
    let (reactions, completions) = empty_authorities();
    source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            target,
            &reactions,
            &completions,
            &authority,
            &migration,
            journal,
        )
        .unwrap()
}

#[test]
fn timer_plan_rejects_duplicate_source_disposition() {
    let result = RuntimeTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [
            TimerUpgradeRule::DropSource {
                source: TimerSlot(0),
            },
            TimerUpgradeRule::Carry {
                source: TimerSlot(0),
                target: TimerSlot(5),
            },
        ],
    );
    assert_eq!(
        result,
        Err(RuntimeUpgradeError::DuplicateSourceTimerDisposition(
            TimerSlot(0)
        ))
    );
}

#[test]
fn timer_plan_rejects_duplicate_target_disposition() {
    let result = RuntimeTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [
            TimerUpgradeRule::Carry {
                source: TimerSlot(0),
                target: TimerSlot(5),
            },
            TimerUpgradeRule::KeepTargetDefault {
                target: TimerSlot(5),
            },
        ],
    );
    assert_eq!(
        result,
        Err(RuntimeUpgradeError::DuplicateTargetTimerDisposition(
            TimerSlot(5)
        ))
    );
}

#[test]
fn timer_plan_hash_is_stable_under_rule_order() {
    let a = RuntimeTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(3),
        [
            TimerUpgradeRule::Carry {
                source: TimerSlot(0),
                target: TimerSlot(5),
            },
            TimerUpgradeRule::KeepTargetDefault {
                target: TimerSlot(6),
            },
        ],
    )
    .unwrap();
    let b = RuntimeTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(3),
        [
            TimerUpgradeRule::KeepTargetDefault {
                target: TimerSlot(6),
            },
            TimerUpgradeRule::Carry {
                source: TimerSlot(0),
                target: TimerSlot(5),
            },
        ],
    )
    .unwrap();
    assert_eq!(a.canonical_bytes(), b.canonical_bytes());
    assert_eq!(a.plan_hash(), b.plan_hash());
}

#[test]
fn legacy_k115_upgrade_path_still_rejects_pending_timer() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(1, &source_program, LogicalTime(10));
    let target = target_once(30);
    let plan = atom_plan(&source, &target);
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
fn one_shot_timer_carries_remaining_deadline_to_target_slot() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(2, &source_program, LogicalTime(10));
    let target = target_once(100);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let report = upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let target_id = source.native_timer_id(TimerSlot(5)).unwrap();
    let snapshot = source.timer_snapshot(target_id).unwrap();
    assert_eq!(snapshot.next_deadline, LogicalTime(20));
    assert_eq!(snapshot.interval, None);
    assert_eq!(snapshot.occurrences, 0);
    assert_eq!(report.migrated_timers, 1);
    assert_eq!(report.dropped_timers, 0);
    assert_eq!(report.defaulted_timers, 0);
}

#[test]
fn repeating_timer_carries_deadline_interval_and_occurrence_count() {
    let source_program = source_repeating(5, 5);
    let (_store, mut journal, mut source) = durable_source(3, &source_program, LogicalTime(12));
    let target = target_repeating(50, 5);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let target_id = source.native_timer_id(TimerSlot(5)).unwrap();
    let snapshot = source.timer_snapshot(target_id).unwrap();
    assert_eq!(snapshot.next_deadline, LogicalTime(15));
    assert_eq!(snapshot.interval, Some(LogicalDuration(5)));
    assert_eq!(snapshot.occurrences, 2);
}

#[test]
fn carried_timer_uses_target_timer_identity() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(4, &source_program, LogicalTime(10));
    let source_id = source.native_timer_id(TimerSlot(0)).unwrap();
    let target = target_two_once(200, 100);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [
            TimerUpgradeRule::KeepTargetDefault {
                target: TimerSlot(4),
            },
            TimerUpgradeRule::Carry {
                source: TimerSlot(0),
                target: TimerSlot(5),
            },
        ],
    );
    upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let target_id = source.native_timer_id(TimerSlot(5)).unwrap();
    assert_ne!(source_id, target_id);
    let snapshot = source.timer_snapshot(target_id).unwrap();
    assert_eq!(snapshot.id, target_id);
    assert_eq!(snapshot.next_deadline, LogicalTime(20));
}

#[test]
fn carried_cancelled_source_timer_cancels_target_timer() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(5, &source_program, LogicalTime(10));
    let source_id = source.native_timer_id(TimerSlot(0)).unwrap();
    assert!(source.cancel_timer(source_id));
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let target_id = source.native_timer_id(TimerSlot(5)).unwrap();
    assert!(source.timer_snapshot(target_id).is_err());
    assert_eq!(source.pending_timers(), 0);
}

#[test]
fn drop_source_and_keep_target_default_are_explicit() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(6, &source_program, LogicalTime(10));
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [
            TimerUpgradeRule::DropSource {
                source: TimerSlot(0),
            },
            TimerUpgradeRule::KeepTargetDefault {
                target: TimerSlot(5),
            },
        ],
    );
    let report = upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let target_id = source.native_timer_id(TimerSlot(5)).unwrap();
    assert_eq!(
        source.timer_snapshot(target_id).unwrap().next_deadline,
        LogicalTime(30)
    );
    assert_eq!(report.migrated_timers, 0);
    assert_eq!(report.dropped_timers, 1);
    assert_eq!(report.defaulted_timers, 1);
}

#[test]
fn unknown_source_timer_slot_is_rejected() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(7, &source_program, LogicalTime(10));
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(99),
            target: TimerSlot(5),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::UnknownSourceTimerSlot(TimerSlot(99)))
    );
}

#[test]
fn unknown_target_timer_slot_is_rejected() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(8, &source_program, LogicalTime(10));
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(99),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::UnknownTargetTimerSlot(TimerSlot(99)))
    );
}

#[test]
fn every_source_native_timer_slot_requires_disposition() {
    let source_program = source_two_timers();
    let (_store, mut journal, mut source) = durable_source(9, &source_program, LogicalTime(10));
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::MissingSourceTimerDisposition(
            TimerSlot(1)
        ))
    );
}

#[test]
fn every_target_native_timer_slot_requires_disposition() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(10, &source_program, LogicalTime(10));
    let target = target_two_once(30, 40);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::MissingTargetTimerDisposition(
            TimerSlot(4)
        ))
    );
}

#[test]
fn dynamic_pending_source_timer_is_rejected() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(11, &source_program, LogicalTime(10));
    source.schedule_once_at(LogicalTime(40)).unwrap();
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::DynamicSourceTimerUnsupported { timer: 2 })
    );
}

#[test]
fn timer_kind_mismatch_is_rejected() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(12, &source_program, LogicalTime(10));
    let target = target_repeating(30, 5);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::TimerKindMismatch {
            source: TimerSlot(0),
            target: TimerSlot(5),
        })
    );
}

#[test]
fn repeating_timer_interval_mismatch_is_rejected() {
    let source_program = source_repeating(5, 5);
    let (_store, mut journal, mut source) = durable_source(13, &source_program, LogicalTime(12));
    let target = target_repeating(30, 7);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::TimerIntervalMismatch {
            source: TimerSlot(0),
            target: TimerSlot(5),
            source_interval: 5,
            target_interval: 7,
        })
    );
}

#[test]
fn active_source_cannot_carry_into_bootstrap_cancelled_target() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(14, &source_program, LogicalTime(10));
    let target = target_cancelled_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EventLoopError::Upgrade(RuntimeUpgradeError::TargetTimerInactive(TimerSlot(5)))
    );
}

#[test]
fn target_default_timer_in_preserved_past_is_rejected() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(15, &source_program, LogicalTime(10));
    let target = target_once(5);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [
            TimerUpgradeRule::DropSource {
                source: TimerSlot(0),
            },
            TimerUpgradeRule::KeepTargetDefault {
                target: TimerSlot(5),
            },
        ],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    let error = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
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
fn carried_timer_can_replace_target_bootstrap_deadline_in_preserved_past() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(16, &source_program, LogicalTime(10));
    let target = target_once(5);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let target_id = source.native_timer_id(TimerSlot(5)).unwrap();
    assert_eq!(
        source.timer_snapshot(target_id).unwrap().next_deadline,
        LogicalTime(20)
    );
}

#[test]
fn timer_plan_source_program_must_match_atom_plan() {
    let source = AtomicEventLoop::boot(&source_once(20)).unwrap();
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = RuntimeTimerUpgradePlan::new(
        [9; 32],
        atoms.target_program_hash(),
        source.program_epoch(),
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    )
    .unwrap();
    assert_eq!(
        RuntimeTimerAwareUpgradePlan::new(&atoms, &timers),
        Err(RuntimeUpgradeError::TimerPlanSourceProgramMismatch)
    );
}

#[test]
fn timer_plan_target_program_must_match_atom_plan() {
    let source = AtomicEventLoop::boot(&source_once(20)).unwrap();
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = RuntimeTimerUpgradePlan::new(
        source.program_hash(),
        [8; 32],
        source.program_epoch(),
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    )
    .unwrap();
    assert_eq!(
        RuntimeTimerAwareUpgradePlan::new(&atoms, &timers),
        Err(RuntimeUpgradeError::TimerPlanTargetProgramMismatch)
    );
}

#[test]
fn timer_plan_epoch_must_match_atom_plan() {
    let source = AtomicEventLoop::boot(&source_once(20)).unwrap();
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = RuntimeTimerUpgradePlan::new(
        source.program_hash(),
        atoms.target_program_hash(),
        ProgramEpoch(99),
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    )
    .unwrap();
    assert_eq!(
        RuntimeTimerAwareUpgradePlan::new(&atoms, &timers),
        Err(RuntimeUpgradeError::TimerPlanSourceEpochMismatch {
            expected: 0,
            actual: 99,
        })
    );
}

#[test]
fn different_timer_migration_semantics_change_replay_identity() {
    fn run(carry_to: TimerSlot, default: TimerSlot) -> nordoi_kernel::EventLoopReplayKey {
        let source_program = source_once(20);
        let (_store, mut journal, mut source) =
            durable_source(20, &source_program, LogicalTime(10));
        let target = target_two_once(30, 40);
        let atoms = atom_plan(&source, &target);
        let timers = timer_plan(
            &source,
            &target,
            [
                TimerUpgradeRule::Carry {
                    source: TimerSlot(0),
                    target: carry_to,
                },
                TimerUpgradeRule::KeepTargetDefault { target: default },
            ],
        );
        upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
        source.replay_key()
    }
    assert_ne!(
        run(TimerSlot(4), TimerSlot(5)),
        run(TimerSlot(5), TimerSlot(4))
    );
}

#[test]
fn different_timer_migration_semantics_change_upgrade_lineage() {
    fn run(carry_to: TimerSlot, default: TimerSlot) -> nordoi_kernel::RuntimeUpgradeHash {
        let source_program = source_once(20);
        let (_store, mut journal, mut source) =
            durable_source(21, &source_program, LogicalTime(10));
        let target = target_two_once(30, 40);
        let atoms = atom_plan(&source, &target);
        let timers = timer_plan(
            &source,
            &target,
            [
                TimerUpgradeRule::Carry {
                    source: TimerSlot(0),
                    target: carry_to,
                },
                TimerUpgradeRule::KeepTargetDefault { target: default },
            ],
        );
        upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
        source.upgrade_chain_root()
    }
    assert_ne!(
        run(TimerSlot(4), TimerSlot(5)),
        run(TimerSlot(5), TimerSlot(4))
    );
}

#[test]
fn lineage_record_commits_composite_atom_and_timer_plan_hash() {
    let source_program = source_once(20);
    let (store, mut journal, mut source) = durable_source(22, &source_program, LogicalTime(10));
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let report = upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let checkpoint =
        RuntimeSemanticCheckpoint::from_canonical_bytes(&store.runtime_bytes().unwrap()).unwrap();
    assert_eq!(
        checkpoint.last_upgrade().unwrap().plan_hash,
        report.composite_plan_hash
    );
    assert_eq!(report.plan_hash, atoms.plan_hash());
    assert_eq!(report.timer_plan_hash, Some(timers.plan_hash()));
}

#[test]
fn failed_timer_aware_upgrade_commit_keeps_source_live_state() {
    let source_program = source_repeating(5, 5);
    let (store, mut journal, mut source) = durable_source(23, &source_program, LogicalTime(12));
    let old_hash = source.program_hash();
    let old_epoch = source.program_epoch();
    let old_replay = source.replay_key();
    let source_id = source.native_timer_id(TimerSlot(0)).unwrap();
    let old_timer = source.timer_snapshot(source_id).unwrap();
    let target = target_repeating(30, 5);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let authority = authority_for(&atoms);
    let (reactions, completions) = empty_authorities();
    store.fail_next_bundle();
    assert!(source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &reactions,
            &completions,
            &authority,
            &RuntimeTimerAwareUpgradePlan::new(&atoms, &timers).unwrap(),
            &mut journal,
        )
        .is_err());
    assert_eq!(source.program_hash(), old_hash);
    assert_eq!(source.program_epoch(), old_epoch);
    assert_eq!(source.replay_key(), old_replay);
    assert_eq!(source.timer_snapshot(source_id).unwrap(), old_timer);
}

#[test]
fn migrated_timer_state_survives_crash_recovery() {
    let source_program = source_repeating(5, 5);
    let (store, mut journal, mut source) = durable_source(24, &source_program, LogicalTime(12));
    let target = target_repeating(30, 5);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let target_id = source.native_timer_id(TimerSlot(5)).unwrap();
    let expected = source.timer_snapshot(target_id).unwrap();

    let mut recovered_journal = active_journal(24, 2, store);
    let mut recovered = AtomicEventLoop::boot(&target).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap()
        .unwrap();
    let recovered_id = recovered.native_timer_id(TimerSlot(5)).unwrap();
    assert_eq!(recovered_id, target_id);
    assert_eq!(recovered.timer_snapshot(recovered_id).unwrap(), expected);
    assert_eq!(recovered.program_epoch(), ProgramEpoch(1));
}

#[test]
fn migrated_timer_fires_with_target_binding_and_next_occurrence() {
    let source_program = source_repeating(5, 5);
    let (_store, mut journal, mut source) = durable_source(25, &source_program, LogicalTime(12));
    let target = target_repeating(30, 5);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let target_id = source.native_timer_id(TimerSlot(5)).unwrap();
    let cycle = source
        .cycle_to(LogicalTime(15), &InputBatch::default())
        .unwrap();
    assert_eq!(cycle.time.fires.len(), 1);
    assert_eq!(cycle.time.fires[0].timer, target_id);
    assert_eq!(cycle.time.fires[0].occurrence, 3);
}

#[test]
fn cancelled_dynamic_timer_identity_frontier_survives_upgrade() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(26, &source_program, LogicalTime(10));
    let dynamic = source.schedule_once_at(LogicalTime(50)).unwrap();
    assert_eq!(dynamic.0, 2);
    assert!(source.cancel_timer(dynamic));
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    let next = source.schedule_once_at(LogicalTime(60)).unwrap();
    assert_eq!(next.0, 3);
}

#[test]
fn cancelled_target_default_remains_cancelled() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(27, &source_program, LogicalTime(10));
    let target = target_cancelled_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [
            TimerUpgradeRule::DropSource {
                source: TimerSlot(0),
            },
            TimerUpgradeRule::KeepTargetDefault {
                target: TimerSlot(5),
            },
        ],
    );
    upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    assert_eq!(source.pending_timers(), 0);
}

#[test]
fn identical_timer_aware_upgrade_traces_are_deterministic() {
    fn run() -> (
        nordoi_kernel::EventLoopReplayKey,
        nordoi_kernel::RuntimeUpgradeHash,
    ) {
        let source_program = source_repeating(5, 5);
        let (_store, mut journal, mut source) =
            durable_source(28, &source_program, LogicalTime(12));
        let target = target_repeating(30, 5);
        let atoms = atom_plan(&source, &target);
        let timers = timer_plan(
            &source,
            &target,
            [TimerUpgradeRule::Carry {
                source: TimerSlot(0),
                target: TimerSlot(5),
            }],
        );
        upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
        (source.replay_key(), source.upgrade_chain_root())
    }
    assert_eq!(run(), run());
}

#[test]
fn timer_aware_upgrade_advances_program_epoch_once() {
    let source_program = source_once(20);
    let (_store, mut journal, mut source) = durable_source(29, &source_program, LogicalTime(10));
    let target = target_once(30);
    let atoms = atom_plan(&source, &target);
    let timers = timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let report = upgrade_with_timers(&mut source, &target, &atoms, &timers, &mut journal);
    assert_eq!(report.source_epoch, ProgramEpoch(0));
    assert_eq!(report.target_epoch, ProgramEpoch(1));
    assert_eq!(source.program_epoch(), ProgramEpoch(1));
}

#[test]
fn nair_version_remains_0_6_in_k116() {
    assert_eq!(NAIR_FORMAT_MINOR, 6);
}
