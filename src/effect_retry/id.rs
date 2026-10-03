use std::fmt::{Display, Formatter};

/// Host-supplied monotonic scheduling coordinate for external-effect retry policy.
///
/// This is deliberately distinct from NORDOI program logical time. Hosts may map it
/// to wall time, scheduler epochs, durable ticks, or another monotonic domain, but
/// the core never reads an ambient clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct EffectRetryTick(pub u64);

impl EffectRetryTick {
    pub const ZERO: Self = Self(0);

    pub const fn value(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, delta: u64) -> Option<Self> {
        self.0.checked_add(delta).map(Self)
    }
}

impl Display for EffectRetryTick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
