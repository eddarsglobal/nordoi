use std::collections::BTreeSet;

use crate::effect_dispatch::EffectDeliveryNamespace;

use super::{EffectCompletionError, EffectCompletionResult, EffectCompletionSourceId};

#[derive(Debug, Clone, Default)]
pub struct EffectCompletionAuthority {
    grants: BTreeSet<(EffectCompletionSourceId, EffectDeliveryNamespace)>,
}

impl EffectCompletionAuthority {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(
        &mut self,
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
    ) -> EffectCompletionResult<bool> {
        if source.0 == 0 {
            return Err(EffectCompletionError::InvalidSourceId);
        }
        Ok(self.grants.insert((source, namespace)))
    }

    pub fn revoke(
        &mut self,
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
    ) -> bool {
        self.grants.remove(&(source, namespace))
    }

    pub fn allows(
        &self,
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
    ) -> bool {
        self.grants.contains(&(source, namespace))
    }
}
