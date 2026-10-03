use std::collections::BTreeMap;

use super::{LogicalDuration, LogicalTime, TimeError, TimeResult, TimerId};

pub const DEFAULT_TIMER_FIRE_BUDGET: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerSnapshot {
    pub id: TimerId,
    pub next_deadline: LogicalTime,
    pub interval: Option<LogicalDuration>,
    pub occurrences: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerFire {
    pub timer: TimerId,
    pub deadline: LogicalTime,
    pub occurrence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeAdvanceReport {
    pub from: LogicalTime,
    pub to: LogicalTime,
    pub fires: Vec<TimerFire>,
}

impl TimeAdvanceReport {
    pub fn is_empty(&self) -> bool {
        self.fires.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
struct TimerState {
    deadline: LogicalTime,
    interval: Option<LogicalDuration>,
    occurrences: u64,
}

#[derive(Debug, Clone)]
pub struct AtomicTimeCore {
    now: LogicalTime,
    next_timer_id: u64,
    timers: BTreeMap<TimerId, TimerState>,
    schedule: BTreeMap<(LogicalTime, TimerId), ()>,
    max_fires_per_advance: usize,
}

impl Default for AtomicTimeCore {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicTimeCore {
    pub fn new() -> Self {
        Self {
            now: LogicalTime::ZERO,
            next_timer_id: 1,
            timers: BTreeMap::new(),
            schedule: BTreeMap::new(),
            max_fires_per_advance: DEFAULT_TIMER_FIRE_BUDGET,
        }
    }

    pub fn with_fire_budget(max_fires_per_advance: usize) -> TimeResult<Self> {
        if max_fires_per_advance == 0 {
            return Err(TimeError::InvalidFireBudget);
        }

        Ok(Self {
            max_fires_per_advance,
            ..Self::new()
        })
    }

    pub const fn now(&self) -> LogicalTime {
        self.now
    }

    pub const fn fire_budget(&self) -> usize {
        self.max_fires_per_advance
    }

    pub fn pending_timers(&self) -> usize {
        self.timers.len()
    }

    pub fn next_deadline(&self) -> Option<LogicalTime> {
        self.schedule.keys().next().map(|(deadline, _)| *deadline)
    }

    pub fn timer(&self, id: TimerId) -> TimeResult<TimerSnapshot> {
        let timer = self.timers.get(&id).ok_or(TimeError::UnknownTimer(id))?;
        Ok(TimerSnapshot {
            id,
            next_deadline: timer.deadline,
            interval: timer.interval,
            occurrences: timer.occurrences,
        })
    }

    pub fn schedule_once_at(&mut self, deadline: LogicalTime) -> TimeResult<TimerId> {
        self.require_future_or_now(deadline)?;
        let id = self.allocate_timer_id()?;
        self.insert_timer(id, deadline, None);
        Ok(id)
    }

    pub fn schedule_once_after(&mut self, delay: LogicalDuration) -> TimeResult<TimerId> {
        let deadline = self.now.checked_add(delay).ok_or(TimeError::TimeOverflow)?;
        self.schedule_once_at(deadline)
    }

    pub fn schedule_repeating_at(
        &mut self,
        first_deadline: LogicalTime,
        interval: LogicalDuration,
    ) -> TimeResult<TimerId> {
        if interval.is_zero() {
            return Err(TimeError::ZeroInterval);
        }
        self.require_future_or_now(first_deadline)?;
        let id = self.allocate_timer_id()?;
        self.insert_timer(id, first_deadline, Some(interval));
        Ok(id)
    }

    pub fn schedule_repeating_after(
        &mut self,
        initial_delay: LogicalDuration,
        interval: LogicalDuration,
    ) -> TimeResult<TimerId> {
        let first_deadline = self
            .now
            .checked_add(initial_delay)
            .ok_or(TimeError::TimeOverflow)?;
        self.schedule_repeating_at(first_deadline, interval)
    }

    pub fn cancel(&mut self, id: TimerId) -> bool {
        let Some(timer) = self.timers.remove(&id) else {
            return false;
        };
        self.schedule.remove(&(timer.deadline, id));
        true
    }

    pub fn advance_by(&mut self, duration: LogicalDuration) -> TimeResult<TimeAdvanceReport> {
        let target = self
            .now
            .checked_add(duration)
            .ok_or(TimeError::TimeOverflow)?;
        self.advance_to(target)
    }

    /// Advances logical time transactionally.
    ///
    /// Timer firings are ordered by `(deadline, TimerId)`. Repeating timers are
    /// lossless: every elapsed deadline is emitted. If doing so would exceed the
    /// configured fire budget, the whole advance is rejected and `self` is unchanged.
    pub fn advance_to(&mut self, target: LogicalTime) -> TimeResult<TimeAdvanceReport> {
        if target < self.now {
            return Err(TimeError::TimeWentBackward {
                current: self.now,
                requested: target,
            });
        }

        let mut candidate = self.clone();
        let report = candidate.advance_to_in_place(target)?;
        *self = candidate;
        Ok(report)
    }

    fn advance_to_in_place(&mut self, target: LogicalTime) -> TimeResult<TimeAdvanceReport> {
        let from = self.now;
        let mut fires = Vec::new();

        while let Some((deadline, id)) = self.schedule.keys().next().copied() {
            if deadline > target {
                break;
            }
            if fires.len() >= self.max_fires_per_advance {
                return Err(TimeError::FireBudgetExceeded {
                    limit: self.max_fires_per_advance,
                });
            }

            self.schedule.remove(&(deadline, id));
            let mut timer = self
                .timers
                .remove(&id)
                .expect("scheduled timer must have timer state");
            timer.occurrences = timer
                .occurrences
                .checked_add(1)
                .ok_or(TimeError::TimeOverflow)?;

            fires.push(TimerFire {
                timer: id,
                deadline,
                occurrence: timer.occurrences,
            });

            if let Some(interval) = timer.interval {
                let next_deadline = deadline
                    .checked_add(interval)
                    .ok_or(TimeError::TimeOverflow)?;
                timer.deadline = next_deadline;
                self.timers.insert(id, timer);
                self.schedule.insert((next_deadline, id), ());
            }
        }

        self.now = target;
        Ok(TimeAdvanceReport {
            from,
            to: target,
            fires,
        })
    }

    fn require_future_or_now(&self, deadline: LogicalTime) -> TimeResult<()> {
        if deadline < self.now {
            return Err(TimeError::DeadlineInPast {
                now: self.now,
                deadline,
            });
        }
        Ok(())
    }

    fn allocate_timer_id(&mut self) -> TimeResult<TimerId> {
        let id = TimerId(self.next_timer_id);
        self.next_timer_id = self
            .next_timer_id
            .checked_add(1)
            .ok_or(TimeError::TimerIdExhausted)?;
        Ok(id)
    }

    fn insert_timer(
        &mut self,
        id: TimerId,
        deadline: LogicalTime,
        interval: Option<LogicalDuration>,
    ) {
        self.timers.insert(
            id,
            TimerState {
                deadline,
                interval,
                occurrences: 0,
            },
        );
        self.schedule.insert((deadline, id), ());
    }
}
