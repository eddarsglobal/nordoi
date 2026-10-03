use nordoi_kernel::{
    AtomSlot, AtomicEventLoop, AtomicTimeCore, DirtyMask, DomainRef, EventLoopError, InputBatch,
    InputBridgeSlot, InputDeviceId, InputError, InputEvent, InputPayload, InputSequence,
    InputSignal, InputSource, InputTarget, InputTargetRef, Instruction, LogicalDuration,
    LogicalTime, NairProgram, RegisterId, RenderNodeSlot, RenderPrimitive, RenderSpace,
    RuntimeError, TimeError, TimerId, Value,
};

fn key_batch(sequence: u64, pressed: bool) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(sequence),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: 13,
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

#[test]
fn logical_time_starts_at_zero_without_ambient_clock_access() {
    let time = AtomicTimeCore::new();
    assert_eq!(time.now(), LogicalTime::ZERO);
    assert_eq!(time.pending_timers(), 0);
    assert!(time.next_deadline().is_none());
}

#[test]
fn logical_time_never_moves_backward() {
    let mut time = AtomicTimeCore::new();
    time.advance_to(LogicalTime(10)).unwrap();

    assert_eq!(
        time.advance_to(LogicalTime(9)),
        Err(TimeError::TimeWentBackward {
            current: LogicalTime(10),
            requested: LogicalTime(9),
        })
    );
    assert_eq!(time.now(), LogicalTime(10));
}

#[test]
fn one_shot_timer_fires_once_at_its_deadline() {
    let mut time = AtomicTimeCore::new();
    let timer = time.schedule_once_at(LogicalTime(5)).unwrap();

    assert!(time.advance_to(LogicalTime(4)).unwrap().fires.is_empty());
    let report = time.advance_to(LogicalTime(5)).unwrap();

    assert_eq!(report.fires.len(), 1);
    assert_eq!(report.fires[0].timer, timer);
    assert_eq!(report.fires[0].deadline, LogicalTime(5));
    assert_eq!(report.fires[0].occurrence, 1);
    assert_eq!(time.pending_timers(), 0);
}

#[test]
fn equal_deadline_timers_fire_in_timer_id_order() {
    let mut time = AtomicTimeCore::new();
    let first = time.schedule_once_at(LogicalTime(7)).unwrap();
    let second = time.schedule_once_at(LogicalTime(7)).unwrap();

    let report = time.advance_to(LogicalTime(7)).unwrap();
    let order: Vec<TimerId> = report.fires.iter().map(|fire| fire.timer).collect();

    assert_eq!(order, vec![first, second]);
}

#[test]
fn repeating_timer_catches_up_losslessly_across_large_advance() {
    let mut time = AtomicTimeCore::new();
    let timer = time
        .schedule_repeating_at(LogicalTime(5), LogicalDuration(5))
        .unwrap();

    let report = time.advance_to(LogicalTime(16)).unwrap();
    let deadlines: Vec<LogicalTime> = report.fires.iter().map(|fire| fire.deadline).collect();
    let occurrences: Vec<u64> = report.fires.iter().map(|fire| fire.occurrence).collect();

    assert_eq!(
        deadlines,
        vec![LogicalTime(5), LogicalTime(10), LogicalTime(15)]
    );
    assert_eq!(occurrences, vec![1, 2, 3]);
    assert_eq!(time.timer(timer).unwrap().next_deadline, LogicalTime(20));
}

#[test]
fn repeating_timer_rejects_zero_interval() {
    let mut time = AtomicTimeCore::new();
    assert_eq!(
        time.schedule_repeating_at(LogicalTime(1), LogicalDuration::ZERO),
        Err(TimeError::ZeroInterval)
    );
    assert_eq!(time.pending_timers(), 0);
}

#[test]
fn cancellation_removes_future_work_and_unknown_cancel_is_zero_effect() {
    let mut time = AtomicTimeCore::new();
    let timer = time.schedule_once_at(LogicalTime(3)).unwrap();

    assert!(time.cancel(timer));
    assert!(!time.cancel(timer));
    assert!(time.advance_to(LogicalTime(3)).unwrap().fires.is_empty());
}

#[test]
fn scheduling_in_the_past_is_rejected_without_mutation() {
    let mut time = AtomicTimeCore::new();
    time.advance_to(LogicalTime(10)).unwrap();

    assert_eq!(
        time.schedule_once_at(LogicalTime(9)),
        Err(TimeError::DeadlineInPast {
            now: LogicalTime(10),
            deadline: LogicalTime(9),
        })
    );
    assert_eq!(time.pending_timers(), 0);
    assert_eq!(time.now(), LogicalTime(10));
}

#[test]
fn fire_budget_failure_is_atomic() {
    let mut time = AtomicTimeCore::with_fire_budget(2).unwrap();
    let timer = time
        .schedule_repeating_at(LogicalTime(1), LogicalDuration(1))
        .unwrap();

    assert_eq!(
        time.advance_to(LogicalTime(3)),
        Err(TimeError::FireBudgetExceeded { limit: 2 })
    );
    assert_eq!(time.now(), LogicalTime::ZERO);
    assert_eq!(time.timer(timer).unwrap().occurrences, 0);
    assert_eq!(time.next_deadline(), Some(LogicalTime(1)));
}

#[test]
fn zero_duration_advance_can_fire_timer_due_now() {
    let mut time = AtomicTimeCore::new();
    let timer = time.schedule_once_after(LogicalDuration::ZERO).unwrap();
    let report = time.advance_by(LogicalDuration::ZERO).unwrap();

    assert_eq!(report.fires.len(), 1);
    assert_eq!(report.fires[0].timer, timer);
    assert_eq!(report.from, LogicalTime::ZERO);
    assert_eq!(report.to, LogicalTime::ZERO);
}

#[test]
fn event_loop_boots_at_zero_with_quiescent_persistent_runtime() {
    let loop_ = AtomicEventLoop::boot(&interactive_program()).unwrap();

    assert_eq!(loop_.logical_time(), LogicalTime::ZERO);
    assert_eq!(loop_.cycle_index(), 0);
    assert!(loop_.is_runtime_quiescent());
    assert_eq!(
        loop_.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(false)
    );
}

#[test]
fn one_cycle_advances_logical_time_and_persistent_runtime_together() {
    let mut loop_ = AtomicEventLoop::boot(&interactive_program()).unwrap();
    let report = loop_.cycle_to(LogicalTime(5), &key_batch(1, true)).unwrap();

    assert_eq!(report.cycle, 1);
    assert_eq!(report.logical_time, LogicalTime(5));
    assert_eq!(
        report.runtime.final_atoms[&AtomSlot(0)].value,
        Value::Bool(true)
    );
    assert!(report.runtime.frame.is_some());
    assert_eq!(loop_.logical_time(), LogicalTime(5));
    assert!(loop_.is_runtime_quiescent());
}

#[test]
fn timer_can_fire_on_empty_input_cycle_without_creating_runtime_work() {
    let mut loop_ = AtomicEventLoop::boot(&interactive_program()).unwrap();
    let timer = loop_.schedule_once_at(LogicalTime(4)).unwrap();
    let report = loop_
        .cycle_to(LogicalTime(4), &InputBatch::default())
        .unwrap();

    assert_eq!(report.time.fires.len(), 1);
    assert_eq!(report.time.fires[0].timer, timer);
    assert!(report.runtime.frame.is_none());
    assert_eq!(report.runtime.final_atoms[&AtomSlot(0)].version, 0);
}

#[test]
fn failed_runtime_cycle_does_not_publish_time_timer_or_cycle_progress() {
    let mut loop_ = AtomicEventLoop::boot(&interactive_program()).unwrap();
    loop_.cycle_to(LogicalTime(5), &key_batch(5, true)).unwrap();
    let timer = loop_.schedule_once_at(LogicalTime(8)).unwrap();
    let replay_before = loop_.replay_key();

    let error = loop_.cycle_to(LogicalTime(10), &key_batch(5, false));
    assert!(matches!(
        error,
        Err(EventLoopError::Runtime(RuntimeError::Input(
            InputError::NonMonotonicSequence { .. }
        )))
    ));
    assert_eq!(loop_.logical_time(), LogicalTime(5));
    assert_eq!(loop_.cycle_index(), 1);
    assert_eq!(loop_.next_deadline(), Some(LogicalTime(8)));
    assert_eq!(loop_.replay_key(), replay_before);
    assert_eq!(
        loop_.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(true)
    );
    assert_ne!(timer, TimerId(0));
}

#[test]
fn equal_event_loop_traces_produce_equal_replay_keys() {
    let program = interactive_program();
    let mut a = AtomicEventLoop::boot(&program).unwrap();
    let mut b = AtomicEventLoop::boot(&program).unwrap();

    a.schedule_repeating_at(LogicalTime(2), LogicalDuration(3))
        .unwrap();
    b.schedule_repeating_at(LogicalTime(2), LogicalDuration(3))
        .unwrap();
    a.cycle_to(LogicalTime(2), &key_batch(1, true)).unwrap();
    b.cycle_to(LogicalTime(2), &key_batch(1, true)).unwrap();
    a.cycle_to(LogicalTime(5), &key_batch(2, false)).unwrap();
    b.cycle_to(LogicalTime(5), &key_batch(2, false)).unwrap();

    assert_eq!(a.replay_key(), b.replay_key());
    assert_eq!(a.snapshot().unwrap(), b.snapshot().unwrap());
}

#[test]
fn time_boundaries_are_part_of_event_loop_replay_identity() {
    let program = interactive_program();
    let mut a = AtomicEventLoop::boot(&program).unwrap();
    let mut b = AtomicEventLoop::boot(&program).unwrap();

    a.cycle_to(LogicalTime(1), &key_batch(1, true)).unwrap();
    b.cycle_to(LogicalTime(2), &key_batch(1, true)).unwrap();

    assert_ne!(a.replay_key(), b.replay_key());
    assert_eq!(a.snapshot().unwrap(), b.snapshot().unwrap());
}

#[test]
fn schedule_and_effective_cancel_are_part_of_replay_identity() {
    let program = interactive_program();
    let mut a = AtomicEventLoop::boot(&program).unwrap();
    let mut b = AtomicEventLoop::boot(&program).unwrap();
    let initial = a.replay_key();

    let timer_a = a.schedule_once_at(LogicalTime(9)).unwrap();
    let timer_b = b.schedule_once_at(LogicalTime(9)).unwrap();
    assert_eq!(a.replay_key(), b.replay_key());
    assert_ne!(a.replay_key(), initial);

    assert!(a.cancel_timer(timer_a));
    assert!(b.cancel_timer(timer_b));
    let after_cancel = a.replay_key();
    assert!(!a.cancel_timer(timer_a));
    assert_eq!(a.replay_key(), after_cancel);
    assert_eq!(a.replay_key(), b.replay_key());
}

#[test]
fn cycle_to_next_deadline_drives_timer_without_wall_clock() {
    let mut loop_ = AtomicEventLoop::boot(&interactive_program()).unwrap();
    loop_.schedule_once_at(LogicalTime(12)).unwrap();

    let report = loop_
        .cycle_to_next_deadline(&InputBatch::default())
        .unwrap()
        .unwrap();

    assert_eq!(report.logical_time, LogicalTime(12));
    assert_eq!(report.time.fires.len(), 1);
    assert!(loop_.next_deadline().is_none());
    assert!(loop_
        .cycle_to_next_deadline(&InputBatch::default())
        .unwrap()
        .is_none());
}
