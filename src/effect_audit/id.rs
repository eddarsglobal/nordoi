use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectAttemptId(pub u64);

impl EffectAttemptId {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Display for EffectAttemptId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectAuditSequence(pub u64);

impl EffectAuditSequence {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Display for EffectAuditSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct EffectAuditHash(pub [u8; 32]);

impl EffectAuditHash {
    pub const ZERO: Self = Self([0; 32]);

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl Display for EffectAuditHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
