use std::{cell::RefCell, rc::Rc};

use nordoi_kernel::EffectFenceStoreError;
use nordoi_kernel::{
    AtomSlot, AtomUpgradeRule, AtomicEventLoop, DynamicTimerUpgradeRule, EffectDeliveryFence,
    EffectDeliveryNamespace, EffectJournalCommitReceipt, EffectJournalLease, EffectJournalWriterId,
    EffectRetryPolicy, FencedEffectJournalStore, FencedRuntimeCheckpointStore,
    GovernedAuditedEffectJournal, InputBatch, Instruction, LogicalDuration, LogicalTime,
    NairCompletionAuthority, NairProgram, NairReactionAuthority, ProgramEpoch, RegisterId,
    RuntimeAllTimerUpgradePlan, RuntimeCheckpointCommitReceipt, RuntimeCheckpointStoreError,
    RuntimeDynamicTimerUpgradePlan, RuntimeTimerUpgradePlan, RuntimeUpgradeAuthority,
    RuntimeUpgradeError, RuntimeUpgradePlan, TimerId, TimerSlot, TimerUpgradeRule, Value,
    NAIR_FORMAT_MINOR,
};

#[derive(Debug, Default)]
struct StoreState {
    active: Option<EffectJournalLease>,
    next_fence: u64,
    effect_bytes: Option<Vec<u8>>,
    runtime_bytes: Option<Vec<u8>>,
    fail_bundle: bool,
}

#[derive(Debug, Clone, Default)]
struct MemoryRuntimeStore {
    state: Rc<RefCell<StoreState>>,
}

impl MemoryRuntimeStore {
    fn fail_next_bundle(&self) {
        self.state.borrow_mut().fail_bundle = true;
    }

    fn require_active(
        state: &StoreState,
        lease: EffectJournalLease,
    ) -> Result<(), EffectFenceStoreError> {
        if state.active != Some(lease) {
            return Err(EffectFenceStoreError::new("stale fence"));
        }
        Ok(())
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

fn namespace(seed: u8) -> EffectDeliveryNamespace {
    EffectDeliveryNamespace([seed; 16])
}

fn writer(seed: u8) -> EffectJournalWriterId {
    EffectJournalWriterId([seed; 16])
}

fn policy() -> EffectRetryPolicy {
    EffectRetryPolicy::new(4, 2, 32, 0).unwrap()
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

fn atom_program(slot: AtomSlot) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(7),
        },
        Instruction::CreateAtom {
            dst: slot,
            owner: nordoi_kernel::DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::Halt,
    ])
}

fn target_with_native_timer(deadline: u64, cancel: bool) -> NairProgram {
    let mut instructions = vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(7),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(7),
            owner: nordoi_kernel::DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(5),
            deadline: LogicalTime(deadline),
        },
    ];
    if cancel {
        instructions.push(Instruction::CancelTimer {
            timer: TimerSlot(5),
        });
    }
    instructions.push(Instruction::Halt);
    NairProgram::from_instructions(instructions)
}

fn source_with_native_timer(deadline: u64) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(7),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: nordoi_kernel::DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(deadline),
        },
        Instruction::Halt,
    ])
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

fn empty_native_timer_plan(
    source: &AtomicEventLoop,
    target: &NairProgram,
) -> RuntimeTimerUpgradePlan {
    let target_hash = AtomicEventLoop::boot(target).unwrap().program_hash();
    RuntimeTimerUpgradePlan::new(
        source.program_hash(),
        target_hash,
        source.program_epoch(),
        [],
    )
    .unwrap()
}

fn native_timer_plan(
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

fn dynamic_plan(
    source: &AtomicEventLoop,
    target: &NairProgram,
    rules: impl IntoIterator<Item = DynamicTimerUpgradeRule>,
) -> RuntimeDynamicTimerUpgradePlan {
    let target_hash = AtomicEventLoop::boot(target).unwrap().program_hash();
    RuntimeDynamicTimerUpgradePlan::new(
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

fn full_plan(
    atom: &RuntimeUpgradePlan,
    native: &RuntimeTimerUpgradePlan,
    dynamic: &RuntimeDynamicTimerUpgradePlan,
) -> RuntimeAllTimerUpgradePlan {
    RuntimeAllTimerUpgradePlan::new(atom, native, dynamic).unwrap()
}

fn durable_dynamic_source(
    seed: u8,
    repeating: bool,
) -> (
    MemoryRuntimeStore,
    GovernedAuditedEffectJournal<MemoryRuntimeStore>,
    AtomicEventLoop,
    TimerId,
) {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(seed, 1, store.clone());
    let source_program = atom_program(AtomSlot(0));
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    let timer = if repeating {
        source
            .schedule_repeating_at(LogicalTime(20), LogicalDuration(5))
            .unwrap()
    } else {
        source.schedule_once_at(LogicalTime(20)).unwrap()
    };
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    (store, journal, source, timer)
}

fn upgrade_all(
    source: &mut AtomicEventLoop,
    target: &NairProgram,
    migration: &RuntimeAllTimerUpgradePlan,
    journal: &mut GovernedAuditedEffectJournal<MemoryRuntimeStore>,
) -> nordoi_kernel::RuntimeAllTimerUpgradeReport {
    let authority = authority_for(migration.atom_plan());
    source
        .upgrade_program_with_runtime_checkpoint_and_dynamic_timers(
            target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            migration,
            journal,
        )
        .unwrap()
}

#[test]
fn dynamic_plan_rejects_duplicate_source_disposition() {
    let result = RuntimeDynamicTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [
            DynamicTimerUpgradeRule::DropSource { source: TimerId(3) },
            DynamicTimerUpgradeRule::Carry {
                source: TimerId(3),
                target: TimerId(4),
            },
        ],
    );
    assert_eq!(
        result,
        Err(RuntimeUpgradeError::DuplicateDynamicSourceTimerDisposition(
            TimerId(3)
        ))
    );
}

#[test]
fn dynamic_plan_rejects_duplicate_target_disposition() {
    let result = RuntimeDynamicTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [
            DynamicTimerUpgradeRule::Carry {
                source: TimerId(3),
                target: TimerId(8),
            },
            DynamicTimerUpgradeRule::Carry {
                source: TimerId(4),
                target: TimerId(8),
            },
        ],
    );
    assert_eq!(
        result,
        Err(RuntimeUpgradeError::DuplicateDynamicTargetTimerDisposition(
            TimerId(8)
        ))
    );
}

#[test]
fn dynamic_plan_rejects_zero_target_identity() {
    let result = RuntimeDynamicTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [DynamicTimerUpgradeRule::Carry {
            source: TimerId(3),
            target: TimerId(0),
        }],
    );
    assert_eq!(
        result,
        Err(RuntimeUpgradeError::InvalidDynamicTargetTimerId(TimerId(0)))
    );
}

#[test]
fn dynamic_plan_rejects_exhausted_target_identity() {
    let result = RuntimeDynamicTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [DynamicTimerUpgradeRule::Carry {
            source: TimerId(3),
            target: TimerId(u64::MAX),
        }],
    );
    assert_eq!(
        result,
        Err(RuntimeUpgradeError::DynamicTargetTimerIdentityExhausted(
            TimerId(u64::MAX)
        ))
    );
}

#[test]
fn dynamic_plan_hash_is_stable_under_rule_order() {
    let a = RuntimeDynamicTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(4),
        [
            DynamicTimerUpgradeRule::Carry {
                source: TimerId(7),
                target: TimerId(20),
            },
            DynamicTimerUpgradeRule::DropSource { source: TimerId(8) },
        ],
    )
    .unwrap();
    let b = RuntimeDynamicTimerUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(4),
        [
            DynamicTimerUpgradeRule::DropSource { source: TimerId(8) },
            DynamicTimerUpgradeRule::Carry {
                source: TimerId(7),
                target: TimerId(20),
            },
        ],
    )
    .unwrap();
    assert_eq!(a.canonical_bytes(), b.canonical_bytes());
    assert_eq!(a.plan_hash(), b.plan_hash());
}

#[test]
fn all_timer_plan_rejects_dynamic_source_program_mismatch() {
    let atom = RuntimeUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [AtomUpgradeRule::DropSource {
            source: AtomSlot(0),
        }],
    )
    .unwrap();
    let native = RuntimeTimerUpgradePlan::new([1; 32], [2; 32], ProgramEpoch(0), []).unwrap();
    let dynamic =
        RuntimeDynamicTimerUpgradePlan::new([3; 32], [2; 32], ProgramEpoch(0), []).unwrap();
    assert_eq!(
        RuntimeAllTimerUpgradePlan::new(&atom, &native, &dynamic),
        Err(RuntimeUpgradeError::DynamicTimerPlanSourceProgramMismatch)
    );
}

#[test]
fn all_timer_plan_rejects_dynamic_target_program_mismatch() {
    let atom = RuntimeUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(0),
        [AtomUpgradeRule::DropSource {
            source: AtomSlot(0),
        }],
    )
    .unwrap();
    let native = RuntimeTimerUpgradePlan::new([1; 32], [2; 32], ProgramEpoch(0), []).unwrap();
    let dynamic =
        RuntimeDynamicTimerUpgradePlan::new([1; 32], [3; 32], ProgramEpoch(0), []).unwrap();
    assert_eq!(
        RuntimeAllTimerUpgradePlan::new(&atom, &native, &dynamic),
        Err(RuntimeUpgradeError::DynamicTimerPlanTargetProgramMismatch)
    );
}

#[test]
fn all_timer_plan_rejects_dynamic_source_epoch_mismatch() {
    let atom = RuntimeUpgradePlan::new(
        [1; 32],
        [2; 32],
        ProgramEpoch(4),
        [AtomUpgradeRule::DropSource {
            source: AtomSlot(0),
        }],
    )
    .unwrap();
    let native = RuntimeTimerUpgradePlan::new([1; 32], [2; 32], ProgramEpoch(4), []).unwrap();
    let dynamic =
        RuntimeDynamicTimerUpgradePlan::new([1; 32], [2; 32], ProgramEpoch(3), []).unwrap();
    assert_eq!(
        RuntimeAllTimerUpgradePlan::new(&atom, &native, &dynamic),
        Err(RuntimeUpgradeError::DynamicTimerPlanSourceEpochMismatch {
            expected: 4,
            actual: 3,
        })
    );
}

#[test]
fn k116_timer_api_still_rejects_pending_dynamic_timer() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(40, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let migration = nordoi_kernel::RuntimeTimerAwareUpgradePlan::new(&atom, &native).unwrap();
    let authority = authority_for(&atom);
    let err = source
        .upgrade_program_with_runtime_checkpoint_and_timers(
            &target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            &migration,
            &mut journal,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::EventLoopError::Upgrade(RuntimeUpgradeError::DynamicSourceTimerUnsupported {
            timer
        }) if timer == dynamic_id.0
    ));
}

#[test]
fn dynamic_one_shot_can_preserve_exact_identity() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(41, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: dynamic_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let report = upgrade_all(&mut source, &target, &migration, &mut journal);
    let timer = source.timer_snapshot(dynamic_id).unwrap();
    assert_eq!(timer.next_deadline, LogicalTime(20));
    assert_eq!(timer.interval, None);
    assert_eq!(
        report.dynamic_timer_mappings.get(&dynamic_id),
        Some(&dynamic_id)
    );
    assert_eq!(report.migrated_dynamic_timers, 1);
}

#[test]
fn dynamic_one_shot_can_remap_to_fresh_identity() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(42, false);
    let target = atom_program(AtomSlot(7));
    let target_id = TimerId(2);
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: target_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let report = upgrade_all(&mut source, &target, &migration, &mut journal);
    assert!(source.timer_snapshot(dynamic_id).is_err());
    assert_eq!(
        source.timer_snapshot(target_id).unwrap().next_deadline,
        LogicalTime(20)
    );
    assert_eq!(
        report.dynamic_timer_mappings.get(&dynamic_id),
        Some(&target_id)
    );
}

#[test]
fn dynamic_repeating_timer_preserves_interval_deadline_and_occurrence() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(43, 1, store);
    let source_program = atom_program(AtomSlot(0));
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    let dynamic_id = source
        .schedule_repeating_at(LogicalTime(10), LogicalDuration(5))
        .unwrap();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime(12), &InputBatch::default(), &mut journal)
        .unwrap();
    assert_eq!(source.timer_snapshot(dynamic_id).unwrap().occurrences, 1);

    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: dynamic_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    let timer = source.timer_snapshot(dynamic_id).unwrap();
    assert_eq!(timer.next_deadline, LogicalTime(15));
    assert_eq!(timer.interval, Some(LogicalDuration(5)));
    assert_eq!(timer.occurrences, 1);
}

#[test]
fn dynamic_timer_can_be_dropped_explicitly() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(44, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::DropSource { source: dynamic_id }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let report = upgrade_all(&mut source, &target, &migration, &mut journal);
    assert!(source.timer_snapshot(dynamic_id).is_err());
    assert_eq!(report.dropped_dynamic_timers, 1);
    assert!(report.dynamic_timer_mappings.is_empty());
}

#[test]
fn missing_dynamic_source_disposition_fails_closed() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(45, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(&source, &target, []);
    let migration = full_plan(&atom, &native, &dynamic);
    let authority = authority_for(&atom);
    let err = source
        .upgrade_program_with_runtime_checkpoint_and_dynamic_timers(
            &target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            &migration,
            &mut journal,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::EventLoopError::Upgrade(
            RuntimeUpgradeError::MissingDynamicSourceTimerDisposition(id)
        ) if id == dynamic_id
    ));
}

#[test]
fn unknown_dynamic_source_timer_is_rejected() {
    let (_store, mut journal, mut source, _dynamic_id) = durable_dynamic_source(46, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::DropSource {
            source: TimerId(99),
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let authority = authority_for(&atom);
    let err = source
        .upgrade_program_with_runtime_checkpoint_and_dynamic_timers(
            &target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            &migration,
            &mut journal,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::EventLoopError::Upgrade(RuntimeUpgradeError::UnknownDynamicSourceTimer(
            TimerId(99)
        ))
    ));
}

#[test]
fn native_source_timer_cannot_be_claimed_as_dynamic() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(47, 1, store);
    let source_program = source_with_native_timer(20);
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let native_id = source.native_timer_id(TimerSlot(0)).unwrap();
    let target = target_with_native_timer(30, false);
    let atom = atom_plan(&source, &target);
    let native = native_timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::DropSource { source: native_id }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let authority = authority_for(&atom);
    let err = source
        .upgrade_program_with_runtime_checkpoint_and_dynamic_timers(
            &target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            &migration,
            &mut journal,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::EventLoopError::Upgrade(RuntimeUpgradeError::DynamicSourceTimerIsNative(id))
            if id == native_id
    ));
}

#[test]
fn dynamic_target_cannot_collide_with_active_native_target_identity() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(48, false);
    let target = target_with_native_timer(30, false);
    let target_native_id = AtomicEventLoop::boot(&target)
        .unwrap()
        .native_timer_id(TimerSlot(5))
        .unwrap();
    let atom = atom_plan(&source, &target);
    let native = native_timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::KeepTargetDefault {
            target: TimerSlot(5),
        }],
    );
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: target_native_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let authority = authority_for(&atom);
    let err = source
        .upgrade_program_with_runtime_checkpoint_and_dynamic_timers(
            &target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            &migration,
            &mut journal,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::EventLoopError::Upgrade(
            RuntimeUpgradeError::DynamicTargetTimerConflictsWithNative(id)
        ) if id == target_native_id
    ));
}

#[test]
fn dynamic_target_cannot_collide_with_cancelled_native_target_identity() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(49, false);
    let target = target_with_native_timer(30, true);
    let target_native_id = AtomicEventLoop::boot(&target)
        .unwrap()
        .native_timer_id(TimerSlot(5))
        .unwrap();
    let atom = atom_plan(&source, &target);
    let native = native_timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::KeepTargetDefault {
            target: TimerSlot(5),
        }],
    );
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: target_native_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let authority = authority_for(&atom);
    let err = source
        .upgrade_program_with_runtime_checkpoint_and_dynamic_timers(
            &target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            &migration,
            &mut journal,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::EventLoopError::Upgrade(
            RuntimeUpgradeError::DynamicTargetTimerConflictsWithNative(id)
        ) if id == target_native_id
    ));
}

#[test]
fn remapped_dynamic_identity_below_frontier_is_rejected() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(51, 1, store);
    let source_program = atom_program(AtomSlot(0));
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    let first = source.schedule_once_at(LogicalTime(20)).unwrap();
    let second = source.schedule_once_at(LogicalTime(30)).unwrap();
    source.cancel_timer(second);
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: first,
            target: second,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let authority = authority_for(&atom);
    let err = source
        .upgrade_program_with_runtime_checkpoint_and_dynamic_timers(
            &target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            &migration,
            &mut journal,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        nordoi_kernel::EventLoopError::Upgrade(
            RuntimeUpgradeError::DynamicTargetTimerIdentityNotFresh { source, target, minimum }
        ) if source == first && target == second && minimum == 3
    ));
}

#[test]
fn preserved_dynamic_identity_is_allowed_below_frontier() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(52, 1, store);
    let source_program = atom_program(AtomSlot(0));
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    let first = source.schedule_once_at(LogicalTime(20)).unwrap();
    let canceled = source.schedule_once_at(LogicalTime(30)).unwrap();
    source.cancel_timer(canceled);
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: first,
            target: first,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    assert_eq!(
        source.timer_snapshot(first).unwrap().next_deadline,
        LogicalTime(20)
    );
    assert_eq!(
        source.schedule_once_at(LogicalTime(40)).unwrap(),
        TimerId(3)
    );
}

#[test]
fn fresh_remap_advances_timer_allocation_frontier() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(53, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: TimerId(50),
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    assert_eq!(
        source.schedule_once_at(LogicalTime(40)).unwrap(),
        TimerId(51)
    );
}

#[test]
fn cancelled_dynamic_timer_requires_no_disposition_but_frontier_survives() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(54, 1, store);
    let source_program = atom_program(AtomSlot(0));
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    let canceled = source.schedule_once_at(LogicalTime(20)).unwrap();
    assert!(source.cancel_timer(canceled));
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(&source, &target, []);
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    assert_eq!(
        source.schedule_once_at(LogicalTime(30)).unwrap(),
        TimerId(2)
    );
}

#[test]
fn multiple_dynamic_timers_migrate_with_explicit_mapping() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(55, 1, store);
    let source_program = atom_program(AtomSlot(0));
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    let first = source.schedule_once_at(LogicalTime(20)).unwrap();
    let second = source
        .schedule_repeating_at(LogicalTime(25), LogicalDuration(5))
        .unwrap();
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [
            DynamicTimerUpgradeRule::Carry {
                source: first,
                target: first,
            },
            DynamicTimerUpgradeRule::Carry {
                source: second,
                target: TimerId(10),
            },
        ],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let report = upgrade_all(&mut source, &target, &migration, &mut journal);
    assert_eq!(report.migrated_dynamic_timers, 2);
    assert_eq!(
        source.timer_snapshot(first).unwrap().next_deadline,
        LogicalTime(20)
    );
    assert_eq!(
        source.timer_snapshot(TimerId(10)).unwrap().interval,
        Some(LogicalDuration(5))
    );
}

#[test]
fn different_dynamic_mapping_changes_upgrade_replay_identity() {
    fn run(seed: u8, target_id: TimerId) -> u64 {
        let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(seed, false);
        let target = atom_program(AtomSlot(7));
        let atom = atom_plan(&source, &target);
        let native = empty_native_timer_plan(&source, &target);
        let dynamic = dynamic_plan(
            &source,
            &target,
            [DynamicTimerUpgradeRule::Carry {
                source: dynamic_id,
                target: target_id,
            }],
        );
        let migration = full_plan(&atom, &native, &dynamic);
        upgrade_all(&mut source, &target, &migration, &mut journal);
        source.replay_key().value()
    }
    assert_ne!(run(56, TimerId(1)), run(57, TimerId(2)));
}

#[test]
fn identical_dynamic_upgrade_traces_produce_equal_replay_identity() {
    fn run(seed: u8) -> u64 {
        let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(seed, false);
        let target = atom_program(AtomSlot(7));
        let atom = atom_plan(&source, &target);
        let native = empty_native_timer_plan(&source, &target);
        let dynamic = dynamic_plan(
            &source,
            &target,
            [DynamicTimerUpgradeRule::Carry {
                source: dynamic_id,
                target: dynamic_id,
            }],
        );
        let migration = full_plan(&atom, &native, &dynamic);
        upgrade_all(&mut source, &target, &migration, &mut journal);
        source.replay_key().value()
    }
    assert_eq!(run(58), run(59));
}

#[test]
fn report_exposes_dynamic_and_composite_plan_hashes() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(60, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: dynamic_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let report = upgrade_all(&mut source, &target, &migration, &mut journal);
    assert_eq!(report.upgrade.plan_hash, atom.plan_hash());
    assert_eq!(report.upgrade.timer_plan_hash, Some(native.plan_hash()));
    assert_eq!(report.dynamic_timer_plan_hash, dynamic.plan_hash());
    assert_eq!(
        report.upgrade.composite_plan_hash,
        migration.composite_plan_hash()
    );
    assert_eq!(
        source.last_upgrade().unwrap().plan_hash,
        migration.composite_plan_hash()
    );
}

#[test]
fn failed_bundle_commit_preserves_source_dynamic_timer_and_program() {
    let (store, mut journal, mut source, dynamic_id) = durable_dynamic_source(61, false);
    let source_program_hash = source.program_hash();
    let source_replay = source.replay_key();
    let source_snapshot = source.timer_snapshot(dynamic_id).unwrap();
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: TimerId(2),
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let authority = authority_for(&atom);
    store.fail_next_bundle();
    assert!(source
        .upgrade_program_with_runtime_checkpoint_and_dynamic_timers(
            &target,
            &NairReactionAuthority::new(),
            &NairCompletionAuthority::new(),
            &authority,
            &migration,
            &mut journal,
        )
        .is_err());
    assert_eq!(source.program_hash(), source_program_hash);
    assert_eq!(source.replay_key(), source_replay);
    assert_eq!(source.timer_snapshot(dynamic_id).unwrap(), source_snapshot);
}

#[test]
fn recovered_target_restores_preserved_dynamic_timer() {
    let (store, mut journal, mut source, dynamic_id) = durable_dynamic_source(62, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: dynamic_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    let expected_replay = source.replay_key();
    journal.release().unwrap();

    let mut recovered_journal = active_journal(62, 2, store);
    let mut recovered = AtomicEventLoop::boot(&target).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap()
        .unwrap();
    assert_eq!(recovered.replay_key(), expected_replay);
    assert_eq!(
        recovered.timer_snapshot(dynamic_id).unwrap().next_deadline,
        LogicalTime(20)
    );
}

#[test]
fn recovered_target_restores_remapped_dynamic_timer() {
    let (store, mut journal, mut source, dynamic_id) = durable_dynamic_source(63, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let target_id = TimerId(9);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: target_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    journal.release().unwrap();

    let mut recovered_journal = active_journal(63, 2, store);
    let mut recovered = AtomicEventLoop::boot(&target).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap()
        .unwrap();
    assert!(recovered.timer_snapshot(dynamic_id).is_err());
    assert_eq!(
        recovered.timer_snapshot(target_id).unwrap().next_deadline,
        LogicalTime(20)
    );
}

#[test]
fn recovered_remapped_one_shot_fires_under_target_identity() {
    let (store, mut journal, mut source, dynamic_id) = durable_dynamic_source(64, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let target_id = TimerId(7);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: target_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    journal.release().unwrap();

    let mut recovered_journal = active_journal(64, 2, store);
    let mut recovered = AtomicEventLoop::boot(&target).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap()
        .unwrap();
    let report = recovered
        .cycle_to(LogicalTime(20), &InputBatch::default())
        .unwrap();
    assert_eq!(report.time.fires.len(), 1);
    assert_eq!(report.time.fires[0].timer, target_id);
}

#[test]
fn recovered_remapped_repeating_timer_continues_occurrence_sequence() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(65, 1, store.clone());
    let source_program = atom_program(AtomSlot(0));
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    let dynamic_id = source
        .schedule_repeating_at(LogicalTime(10), LogicalDuration(5))
        .unwrap();
    source
        .cycle_to_with_runtime_checkpoint(LogicalTime(12), &InputBatch::default(), &mut journal)
        .unwrap();
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let target_id = TimerId(8);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: target_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    journal.release().unwrap();

    let mut recovered_journal = active_journal(65, 2, store);
    let mut recovered = AtomicEventLoop::boot(&target).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap()
        .unwrap();
    let report = recovered
        .cycle_to(LogicalTime(15), &InputBatch::default())
        .unwrap();
    assert_eq!(report.time.fires.len(), 1);
    assert_eq!(report.time.fires[0].timer, target_id);
    assert_eq!(report.time.fires[0].occurrence, 2);
}

#[test]
fn dropped_dynamic_timer_does_not_reappear_after_recovery() {
    let (store, mut journal, mut source, dynamic_id) = durable_dynamic_source(66, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::DropSource { source: dynamic_id }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    journal.release().unwrap();

    let mut recovered_journal = active_journal(66, 2, store);
    let mut recovered = AtomicEventLoop::boot(&target).unwrap();
    recovered
        .recover_runtime_from_audited_journal(&mut recovered_journal)
        .unwrap()
        .unwrap();
    assert!(recovered.timer_snapshot(dynamic_id).is_err());
    assert!(recovered.next_deadline().is_none());
}

#[test]
fn full_dynamic_plan_changes_composite_identity_vs_native_only_plan() {
    let (_store, _journal, source, dynamic_id) = durable_dynamic_source(67, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let native_only = nordoi_kernel::RuntimeTimerAwareUpgradePlan::new(&atom, &native).unwrap();
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: dynamic_id,
        }],
    );
    let all = full_plan(&atom, &native, &dynamic);
    assert_ne!(native_only.composite_plan_hash(), all.composite_plan_hash());
}

#[test]
fn nair_version_remains_0_6_in_k117() {
    assert_eq!(NAIR_FORMAT_MINOR, 6);
}

#[test]
fn runtime_checkpoint_format_remains_compatible_after_dynamic_upgrade() {
    let (_store, mut journal, mut source, dynamic_id) = durable_dynamic_source(68, false);
    let target = atom_program(AtomSlot(7));
    let atom = atom_plan(&source, &target);
    let native = empty_native_timer_plan(&source, &target);
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: dynamic_id,
            target: dynamic_id,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    upgrade_all(&mut source, &target, &migration, &mut journal);
    let checkpoint = source.semantic_checkpoint(&journal).unwrap();
    let bytes = checkpoint.canonical_bytes().unwrap();
    assert_eq!(&bytes[..8], b"NDRTSM01");
}

#[test]
fn native_and_dynamic_timers_migrate_in_one_atomic_upgrade() {
    let store = MemoryRuntimeStore::default();
    let mut journal = active_journal(69, 1, store);
    let source_program = source_with_native_timer(20);
    let mut source = AtomicEventLoop::boot(&source_program).unwrap();
    let source_dynamic = source.schedule_once_at(LogicalTime(25)).unwrap();
    source
        .checkpoint_runtime_with_audited_journal(&mut journal)
        .unwrap();

    let target = target_with_native_timer(30, false);
    let atom = atom_plan(&source, &target);
    let native = native_timer_plan(
        &source,
        &target,
        [TimerUpgradeRule::Carry {
            source: TimerSlot(0),
            target: TimerSlot(5),
        }],
    );
    let dynamic = dynamic_plan(
        &source,
        &target,
        [DynamicTimerUpgradeRule::Carry {
            source: source_dynamic,
            target: source_dynamic,
        }],
    );
    let migration = full_plan(&atom, &native, &dynamic);
    let report = upgrade_all(&mut source, &target, &migration, &mut journal);

    let target_native = source.native_timer_id(TimerSlot(5)).unwrap();
    assert_eq!(
        source.timer_snapshot(target_native).unwrap().next_deadline,
        LogicalTime(20)
    );
    assert_eq!(
        source.timer_snapshot(source_dynamic).unwrap().next_deadline,
        LogicalTime(25)
    );
    assert_eq!(report.upgrade.migrated_timers, 1);
    assert_eq!(report.migrated_dynamic_timers, 1);
}
