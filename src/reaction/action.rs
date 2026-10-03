use crate::{atom::AtomId, effect::Effect, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub enum ReactionValue {
    Literal(Value),
    InputValue,
    TimerOccurrence,
    TimerDeadlineTicks,
}

impl ReactionValue {
    pub fn literal(value: impl Into<Value>) -> Self {
        Self::Literal(value.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReactionStep {
    Set { atom: AtomId, value: ReactionValue },
    EmitEffect { effect: Effect },
}

impl ReactionStep {
    pub fn set(atom: AtomId, value: ReactionValue) -> Self {
        Self::Set { atom, value }
    }

    pub fn emit(effect: Effect) -> Self {
        Self::EmitEffect { effect }
    }
}
