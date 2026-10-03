#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectIntentId(pub u64);

impl EffectIntentId {
    pub const fn value(self) -> u64 {
        self.0
    }
}
