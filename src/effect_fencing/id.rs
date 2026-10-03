use std::fmt::{Display, Formatter};

use crate::effect_dispatch::{EffectDeliveryFence, EffectDeliveryNamespace};

/// Host-provided identity of one effect-journal writer/dispatcher instance.
///
/// NORDOI never derives this value from ambient process, machine, time or random state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectJournalWriterId(pub [u8; 16]);

impl EffectJournalWriterId {
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

impl Display for EffectJournalWriterId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Proof returned by a host fencing authority for one journal writer epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectJournalLease {
    pub namespace: EffectDeliveryNamespace,
    pub writer: EffectJournalWriterId,
    pub fence: EffectDeliveryFence,
}

impl EffectJournalLease {
    pub const fn new(
        namespace: EffectDeliveryNamespace,
        writer: EffectJournalWriterId,
        fence: EffectDeliveryFence,
    ) -> Self {
        Self {
            namespace,
            writer,
            fence,
        }
    }
}
