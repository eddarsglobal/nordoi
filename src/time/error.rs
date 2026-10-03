use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::runtime::RuntimeError;

use super::{LogicalTime, TimerId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeError {
    TimeWentBackward {
        current: LogicalTime,
        requested: LogicalTime,
    },
    DeadlineInPast {
        now: LogicalTime,
        deadline: LogicalTime,
    },
    ZeroInterval,
    TimerIdExhausted,
    TimeOverflow,
    InvalidFireBudget,
    FireBudgetExceeded {
        limit: usize,
    },
    UnknownTimer(TimerId),
}

impl Display for TimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimeWentBackward { current, requested } => write!(
                f,
                "logical time cannot move backward: current={}, requested={}",
                current.0, requested.0
            ),
            Self::DeadlineInPast { now, deadline } => write!(
                f,
                "timer deadline {} is before logical time {}",
                deadline.0, now.0
            ),
            Self::ZeroInterval => write!(f, "repeating timer interval must be greater than zero"),
            Self::TimerIdExhausted => write!(f, "timer identity space is exhausted"),
            Self::TimeOverflow => write!(f, "logical time arithmetic overflowed"),
            Self::InvalidFireBudget => write!(f, "timer fire budget must be greater than zero"),
            Self::FireBudgetExceeded { limit } => write!(
                f,
                "logical time advance would exceed the timer fire budget of {limit}"
            ),
            Self::UnknownTimer(timer) => write!(f, "unknown timer {}", timer.0),
        }
    }
}

impl Error for TimeError {}

pub type TimeResult<T> = Result<T, TimeError>;

#[derive(Debug, PartialEq)]
pub enum EventLoopError {
    Time(TimeError),
    Runtime(RuntimeError),
}

impl Display for EventLoopError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Time(error) => write!(f, "atomic event-loop time failure: {error}"),
            Self::Runtime(error) => write!(f, "atomic event-loop runtime failure: {error}"),
        }
    }
}

impl Error for EventLoopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Time(error) => Some(error),
            Self::Runtime(error) => Some(error),
        }
    }
}

impl From<TimeError> for EventLoopError {
    fn from(value: TimeError) -> Self {
        Self::Time(value)
    }
}

impl From<RuntimeError> for EventLoopError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

pub type EventLoopResult<T> = Result<T, EventLoopError>;
