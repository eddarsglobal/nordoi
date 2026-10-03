#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReactionId(pub u64);

impl ReactionId {
    pub const fn value(self) -> u64 {
        self.0
    }
}
