use std::fmt::{Display, Formatter};

use crate::{
    input::InputBatch,
    nair::{Instruction, NairError, NairProgram, TimerSlot},
    runtime::{
        hash_bytes, hash_component, PersistentAtomicRuntime, PersistentRuntimeTickReport,
        RuntimeAtomSnapshot, RuntimeError, FNV_OFFSET_BASIS,
    },
    AtomSlot,
};

use std::collections::BTreeMap;

use super::{
    AtomicTimeCore, EventLoopResult, LogicalDuration, LogicalTime, TimeAdvanceReport, TimeError,
    TimerId, TimerSnapshot, DEFAULT_TIMER_FIRE_BUDGET,
};

const EVENT_LOOP_REPLAY_DOMAIN: &[u8] = b"NORDOI-ATOMIC-EVENT-LOOP-1.3";
const OP_SCHEDULE_ONCE: u8 = 0x01;
const OP_SCHEDULE_REPEATING: u8 = 0x02;
const OP_CANCEL: u8 = 0x03;
const OP_CYCLE: u8 = 0x04;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventLoopReplayKey(pub u64);

impl EventLoopReplayKey {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Display for EventLoopReplayKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventLoopCycleReport {
    pub cycle: u64,
    pub logical_time: LogicalTime,
    pub time: TimeAdvanceReport,
    pub runtime: PersistentRuntimeTickReport,
    pub replay_key: EventLoopReplayKey,
}

#[derive(Debug, Clone)]
pub struct AtomicEventLoop {
    runtime: PersistentAtomicRuntime,
    time: AtomicTimeCore,
    cycle: u64,
    replay_state: u64,
    replay_key: EventLoopReplayKey,
    native_timer_bindings: BTreeMap<TimerSlot, TimerId>,
}

impl AtomicEventLoop {
    pub fn boot(program: &NairProgram) -> EventLoopResult<Self> {
        Self::boot_with_fire_budget(program, DEFAULT_TIMER_FIRE_BUDGET)
    }

    pub fn boot_with_fire_budget(
        program: &NairProgram,
        fire_budget: usize,
    ) -> EventLoopResult<Self> {
        program.validate().map_err(RuntimeError::from)?;
        let program_bytes = program.canonical_bytes().map_err(RuntimeError::from)?;
        let mut time = AtomicTimeCore::with_fire_budget(fire_budget)?;
        let native_timer_bindings = apply_native_time_bootstrap(program, &mut time)?;
        let runtime_program = NairProgram::from_instructions(
            program
                .instructions()
                .iter()
                .filter(|instruction| !instruction.requires_time_context())
                .cloned()
                .collect(),
        );
        let runtime = PersistentAtomicRuntime::boot(&runtime_program)?;

        let mut replay_state = FNV_OFFSET_BASIS;
        hash_bytes(&mut replay_state, EVENT_LOOP_REPLAY_DOMAIN);
        hash_component(&mut replay_state, &program_bytes);
        hash_component(
            &mut replay_state,
            &runtime.replay_key().value().to_le_bytes(),
        );
        hash_component(&mut replay_state, &(fire_budget as u64).to_le_bytes());
        let replay_key = EventLoopReplayKey(replay_state);

        Ok(Self {
            runtime,
            time,
            cycle: 0,
            replay_state,
            replay_key,
            native_timer_bindings,
        })
    }

    pub fn logical_time(&self) -> LogicalTime {
        self.time.now()
    }

    pub fn cycle_index(&self) -> u64 {
        self.cycle
    }

    pub fn replay_key(&self) -> EventLoopReplayKey {
        self.replay_key
    }

    pub fn pending_timers(&self) -> usize {
        self.time.pending_timers()
    }

    pub fn native_timer_count(&self) -> usize {
        self.native_timer_bindings.len()
    }

    pub fn native_timer_id(&self, slot: TimerSlot) -> Option<TimerId> {
        self.native_timer_bindings.get(&slot).copied()
    }

    pub fn timer_snapshot(&self, id: TimerId) -> EventLoopResult<TimerSnapshot> {
        self.time.timer(id).map_err(Into::into)
    }

    pub fn next_deadline(&self) -> Option<LogicalTime> {
        self.time.next_deadline()
    }

    pub fn is_runtime_quiescent(&self) -> bool {
        self.runtime.is_quiescent()
    }

    pub fn snapshot(&self) -> EventLoopResult<BTreeMap<AtomSlot, RuntimeAtomSnapshot>> {
        self.runtime.snapshot().map_err(Into::into)
    }

    pub fn schedule_once_at(&mut self, deadline: LogicalTime) -> EventLoopResult<TimerId> {
        let id = self.time.schedule_once_at(deadline)?;
        self.hash_schedule_once(id, deadline);
        Ok(id)
    }

    pub fn schedule_once_after(&mut self, delay: LogicalDuration) -> EventLoopResult<TimerId> {
        let deadline = self
            .time
            .now()
            .checked_add(delay)
            .ok_or(TimeError::TimeOverflow)?;
        self.schedule_once_at(deadline)
    }

    pub fn schedule_repeating_at(
        &mut self,
        first_deadline: LogicalTime,
        interval: LogicalDuration,
    ) -> EventLoopResult<TimerId> {
        let id = self.time.schedule_repeating_at(first_deadline, interval)?;
        self.hash_schedule_repeating(id, first_deadline, interval);
        Ok(id)
    }

    pub fn schedule_repeating_after(
        &mut self,
        initial_delay: LogicalDuration,
        interval: LogicalDuration,
    ) -> EventLoopResult<TimerId> {
        let deadline = self
            .time
            .now()
            .checked_add(initial_delay)
            .ok_or(TimeError::TimeOverflow)?;
        self.schedule_repeating_at(deadline, interval)
    }

    pub fn cancel_timer(&mut self, id: TimerId) -> bool {
        if !self.time.cancel(id) {
            return false;
        }
        hash_bytes(&mut self.replay_state, &[OP_CANCEL]);
        hash_component(&mut self.replay_state, &id.0.to_le_bytes());
        self.replay_key = EventLoopReplayKey(self.replay_state);
        true
    }

    /// Advances time and the persistent runtime as one publication boundary.
    ///
    /// Both subsystems are evaluated on private clones. If either logical-time
    /// advancement or the runtime tick fails, the published event-loop state is unchanged.
    pub fn cycle_to(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
    ) -> EventLoopResult<EventLoopCycleReport> {
        let mut time = self.time.clone();
        let mut runtime = self.runtime.clone();

        let time_report = time.advance_to(target)?;
        let runtime_report = runtime.tick(input)?;
        let cycle = self.cycle.checked_add(1).ok_or(TimeError::TimeOverflow)?;

        let mut replay_state = self.replay_state;
        hash_bytes(&mut replay_state, &[OP_CYCLE]);
        hash_component(&mut replay_state, &cycle.to_le_bytes());
        hash_component(&mut replay_state, &target.0.to_le_bytes());
        hash_component(
            &mut replay_state,
            &runtime_report.replay_key.value().to_le_bytes(),
        );
        hash_component(
            &mut replay_state,
            &(time_report.fires.len() as u64).to_le_bytes(),
        );
        for fire in &time_report.fires {
            hash_component(&mut replay_state, &fire.timer.0.to_le_bytes());
            hash_component(&mut replay_state, &fire.deadline.0.to_le_bytes());
            hash_component(&mut replay_state, &fire.occurrence.to_le_bytes());
        }
        let replay_key = EventLoopReplayKey(replay_state);

        self.time = time;
        self.runtime = runtime;
        self.cycle = cycle;
        self.replay_state = replay_state;
        self.replay_key = replay_key;

        Ok(EventLoopCycleReport {
            cycle,
            logical_time: target,
            time: time_report,
            runtime: runtime_report,
            replay_key,
        })
    }

    pub fn cycle_by(
        &mut self,
        duration: LogicalDuration,
        input: &InputBatch,
    ) -> EventLoopResult<EventLoopCycleReport> {
        let target = self
            .time
            .now()
            .checked_add(duration)
            .ok_or(TimeError::TimeOverflow)?;
        self.cycle_to(target, input)
    }

    pub fn cycle_to_next_deadline(
        &mut self,
        input: &InputBatch,
    ) -> EventLoopResult<Option<EventLoopCycleReport>> {
        let Some(deadline) = self.next_deadline() else {
            return Ok(None);
        };
        self.cycle_to(deadline, input).map(Some)
    }

    fn hash_schedule_once(&mut self, id: TimerId, deadline: LogicalTime) {
        hash_bytes(&mut self.replay_state, &[OP_SCHEDULE_ONCE]);
        hash_component(&mut self.replay_state, &id.0.to_le_bytes());
        hash_component(&mut self.replay_state, &deadline.0.to_le_bytes());
        self.replay_key = EventLoopReplayKey(self.replay_state);
    }

    fn hash_schedule_repeating(
        &mut self,
        id: TimerId,
        first_deadline: LogicalTime,
        interval: LogicalDuration,
    ) {
        hash_bytes(&mut self.replay_state, &[OP_SCHEDULE_REPEATING]);
        hash_component(&mut self.replay_state, &id.0.to_le_bytes());
        hash_component(&mut self.replay_state, &first_deadline.0.to_le_bytes());
        hash_component(&mut self.replay_state, &interval.0.to_le_bytes());
        self.replay_key = EventLoopReplayKey(self.replay_state);
    }
}

fn apply_native_time_bootstrap(
    program: &NairProgram,
    time: &mut AtomicTimeCore,
) -> EventLoopResult<BTreeMap<TimerSlot, TimerId>> {
    let mut bindings = BTreeMap::new();

    for instruction in program.instructions() {
        match instruction {
            Instruction::ScheduleTimerOnceAt { dst, deadline } => {
                let id = time.schedule_once_at(*deadline)?;
                bindings.insert(*dst, id);
            }
            Instruction::ScheduleTimerRepeatingAt {
                dst,
                first_deadline,
                interval,
            } => {
                let id = time.schedule_repeating_at(*first_deadline, *interval)?;
                bindings.insert(*dst, id);
            }
            Instruction::CancelTimer { timer } => {
                let id = bindings
                    .get(timer)
                    .copied()
                    .ok_or(RuntimeError::Nair(NairError::UnknownTimerSlot(*timer)))?;
                time.cancel(id);
            }
            _ => {}
        }
    }

    Ok(bindings)
}
