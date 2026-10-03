use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectIntentId(pub u64);

impl EffectIntentId {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Host-provided namespace for one persistent effect-delivery journal.
///
/// The core deliberately does not manufacture this value from ambient randomness.
/// Hosts that require globally unique delivery keys must provision a unique namespace
/// and reuse it when recovering the same journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectDeliveryNamespace(pub [u8; 16]);

impl EffectDeliveryNamespace {
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

impl Display for EffectDeliveryNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Monotonic host-issued fencing token for concurrent effect-journal ownership.
///
/// A zero fence is invalid for an acquired lease. The token is not part of deterministic
/// program meaning and may change when journal ownership moves between host instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectDeliveryFence(pub u64);

impl EffectDeliveryFence {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Display for EffectDeliveryFence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Stable client request identity for retry-aware external backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectDeliveryKey {
    pub namespace: EffectDeliveryNamespace,
    pub intent: EffectIntentId,
}

impl EffectDeliveryKey {
    pub const fn new(namespace: EffectDeliveryNamespace, intent: EffectIntentId) -> Self {
        Self { namespace, intent }
    }
}

impl Display for EffectDeliveryKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "nordoi-{}-{:016x}", self.namespace, self.intent.0)
    }
}
