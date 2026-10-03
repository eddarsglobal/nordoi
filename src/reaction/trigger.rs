use crate::{
    input::InputSelector,
    time::{TimerFire, TimerId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerSelector {
    pub timer: Option<TimerId>,
    pub occurrence: Option<u64>,
}

impl TimerSelector {
    pub const fn any() -> Self {
        Self {
            timer: None,
            occurrence: None,
        }
    }

    pub const fn timer(timer: TimerId) -> Self {
        Self {
            timer: Some(timer),
            occurrence: None,
        }
    }

    pub const fn exact(timer: TimerId, occurrence: u64) -> Self {
        Self {
            timer: Some(timer),
            occurrence: Some(occurrence),
        }
    }

    pub(crate) fn matches(self, fire: &TimerFire) -> bool {
        !self.timer.is_some_and(|timer| timer != fire.timer)
            && !self
                .occurrence
                .is_some_and(|occurrence| occurrence != fire.occurrence)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReactionTrigger {
    Input(InputSelector),
    Timer(TimerSelector),
}

impl ReactionTrigger {
    pub const fn input(selector: InputSelector) -> Self {
        Self::Input(selector)
    }

    pub const fn timer(selector: TimerSelector) -> Self {
        Self::Timer(selector)
    }
}
