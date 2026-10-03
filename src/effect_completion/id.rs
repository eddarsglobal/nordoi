use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectCompletionSourceId(pub u64);

impl EffectCompletionSourceId {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Display for EffectCompletionSourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectCompletionSequence(pub u64);

impl EffectCompletionSequence {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Display for EffectCompletionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
