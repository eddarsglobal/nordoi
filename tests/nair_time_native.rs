use nordoi_kernel::{
    execute_nair, AtomSlot, AtomicEventLoop, AtomicKernel, DomainRef, Instruction, LogicalDuration,
    LogicalTime, NairError, NairProgram, PersistentAtomicRuntime, RegisterId, RuntimeError,
    TimerSlot, Value, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR,
};

fn one_shot_program(deadline: u64) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(deadline),
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_0_4_is_current_and_0_3_programs_still_decode() {
    assert_eq!(NAIR_FORMAT_MAJOR, 0);
    assert_eq!(NAIR_FORMAT_MINOR, 4);

    let program = NairProgram::from_instructions(vec![Instruction::Halt]);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 3;
    bytes[7] = 0;

    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, program);
    let reencoded = decoded.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([reencoded[6], reencoded[7]]), 4);
}

#[test]
fn time_opcodes_are_rejected_under_declared_nair_0_3() {
    let program = one_shot_program(5);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 3;
    bytes[7] = 0;

    assert_eq!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidOpcode(0x50))
    );
}

#[test]
fn timer_slots_are_single_assignment() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(1),
        },
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(2),
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::DuplicateTimerSlot(TimerSlot(0)))
    );
}

#[test]
fn timer_slot_must_exist_before_cancel() {
    let program = NairProgram::from_instructions(vec![
        Instruction::CancelTimer {
            timer: TimerSlot(9),
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownTimerSlot(TimerSlot(9)))
    );
}

#[test]
fn repeating_timer_interval_must_be_non_zero() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerRepeatingAt {
            dst: TimerSlot(2),
            first_deadline: LogicalTime(1),
            interval: LogicalDuration::ZERO,
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::ZeroTimerInterval(TimerSlot(2)))
    );
}

#[test]
fn native_time_program_canonical_round_trip_is_byte_stable() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(7),
        },
        Instruction::ScheduleTimerRepeatingAt {
            dst: TimerSlot(1),
            first_deadline: LogicalTime(9),
            interval: LogicalDuration(3),
        },
        Instruction::CancelTimer {
            timer: TimerSlot(0),
        },
        Instruction::Halt,
    ]);

    let bytes = program.canonical_bytes().unwrap();
    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, program);
    assert_eq!(decoded.canonical_bytes().unwrap(), bytes);
}

#[test]
fn state_executor_rejects_native_time_before_execution() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(5),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(5),
        },
        Instruction::Halt,
    ]);
    let mut kernel = AtomicKernel::new();

    assert_eq!(
        execute_nair(&mut kernel, &program),
        Err(NairError::TimeContextRequired)
    );
    assert!(kernel.get(nordoi_kernel::AtomId(1)).is_err());
}

#[test]
fn persistent_runtime_requires_event_loop_for_native_time() {
    assert!(matches!(
        PersistentAtomicRuntime::boot(&one_shot_program(3)),
        Err(RuntimeError::Nair(NairError::TimeContextRequired))
    ));
}

#[test]
fn event_loop_boot_schedules_native_one_shot() {
    let loop_ = AtomicEventLoop::boot(&one_shot_program(5)).unwrap();
    let id = loop_.native_timer_id(TimerSlot(0)).unwrap();
    let snapshot = loop_.timer_snapshot(id).unwrap();

    assert_eq!(loop_.native_timer_count(), 1);
    assert_eq!(loop_.pending_timers(), 1);
    assert_eq!(loop_.next_deadline(), Some(LogicalTime(5)));
    assert_eq!(snapshot.next_deadline, LogicalTime(5));
    assert_eq!(snapshot.interval, None);
}

#[test]
fn event_loop_boot_schedules_native_repeating_timer() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerRepeatingAt {
            dst: TimerSlot(0),
            first_deadline: LogicalTime(4),
            interval: LogicalDuration(6),
        },
        Instruction::Halt,
    ]);
    let loop_ = AtomicEventLoop::boot(&program).unwrap();
    let id = loop_.native_timer_id(TimerSlot(0)).unwrap();
    let snapshot = loop_.timer_snapshot(id).unwrap();

    assert_eq!(snapshot.next_deadline, LogicalTime(4));
    assert_eq!(snapshot.interval, Some(LogicalDuration(6)));
}

#[test]
fn equal_deadline_native_timers_follow_program_timer_id_order() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(5),
            deadline: LogicalTime(8),
        },
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(1),
            deadline: LogicalTime(8),
        },
        Instruction::Halt,
    ]);
    let mut loop_ = AtomicEventLoop::boot(&program).unwrap();
    let first = loop_.native_timer_id(TimerSlot(5)).unwrap();
    let second = loop_.native_timer_id(TimerSlot(1)).unwrap();
    let report = loop_.cycle_to(LogicalTime(8), &Default::default()).unwrap();
    let order: Vec<_> = report.time.fires.iter().map(|fire| fire.timer).collect();

    assert_eq!(order, vec![first, second]);
}

#[test]
fn native_cancel_removes_timer_before_first_cycle() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(2),
        },
        Instruction::CancelTimer {
            timer: TimerSlot(0),
        },
        Instruction::Halt,
    ]);
    let mut loop_ = AtomicEventLoop::boot(&program).unwrap();

    assert_eq!(loop_.native_timer_count(), 1);
    assert_eq!(loop_.pending_timers(), 0);
    assert!(loop_
        .cycle_to_next_deadline(&Default::default())
        .unwrap()
        .is_none());
}

#[test]
fn host_timer_identity_follows_native_timer_identity() {
    let mut loop_ = AtomicEventLoop::boot(&one_shot_program(10)).unwrap();
    let native = loop_.native_timer_id(TimerSlot(0)).unwrap();
    let host = loop_.schedule_once_at(LogicalTime(20)).unwrap();

    assert_eq!(native.value(), 1);
    assert_eq!(host.value(), 2);
}

#[test]
fn next_deadline_drives_native_timer_without_wall_clock() {
    let mut loop_ = AtomicEventLoop::boot(&one_shot_program(12)).unwrap();
    let report = loop_
        .cycle_to_next_deadline(&Default::default())
        .unwrap()
        .unwrap();

    assert_eq!(report.logical_time, LogicalTime(12));
    assert_eq!(report.time.fires.len(), 1);
    assert!(loop_.next_deadline().is_none());
}

#[test]
fn native_time_coexists_with_nam_bootstrap_state() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(42),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(3),
        },
        Instruction::Halt,
    ]);
    let loop_ = AtomicEventLoop::boot(&program).unwrap();

    assert_eq!(
        loop_.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Int(42)
    );
    assert_eq!(loop_.next_deadline(), Some(LogicalTime(3)));
    assert!(loop_.is_runtime_quiescent());
}

#[test]
fn equal_native_time_programs_produce_equal_boot_replay_keys() {
    let program = one_shot_program(5);
    let a = AtomicEventLoop::boot(&program).unwrap();
    let b = AtomicEventLoop::boot(&program).unwrap();

    assert_eq!(a.replay_key(), b.replay_key());
    assert_eq!(
        a.native_timer_id(TimerSlot(0)),
        b.native_timer_id(TimerSlot(0))
    );
}

#[test]
fn native_timer_declaration_changes_event_loop_replay_identity() {
    let a = AtomicEventLoop::boot(&one_shot_program(5)).unwrap();
    let b = AtomicEventLoop::boot(&one_shot_program(6)).unwrap();

    assert_ne!(a.replay_key(), b.replay_key());
}

#[test]
fn native_time_only_cycle_creates_no_nam_or_render_work() {
    let mut loop_ = AtomicEventLoop::boot(&one_shot_program(1)).unwrap();
    let report = loop_.cycle_to(LogicalTime(1), &Default::default()).unwrap();

    assert_eq!(report.time.fires.len(), 1);
    assert!(report.runtime.frame.is_none());
    assert!(report.runtime.final_atoms.is_empty());
    assert!(loop_.is_runtime_quiescent());
}
