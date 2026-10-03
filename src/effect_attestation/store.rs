use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{effect_dispatch::EffectDeliveryNamespace, effect_fencing::EffectJournalLease};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAttestationStoreError {
    message: String,
}

impl EffectAttestationStoreError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for EffectAttestationStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EffectAttestationStoreError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectAttestationCommitReceipt {
    pub bytes: u64,
}

impl EffectAttestationCommitReceipt {
    pub const fn new(bytes: u64) -> Self {
        Self { bytes }
    }
}

pub trait EffectAttestationStore {
    fn load_attestation(
        &mut self,
        namespace: EffectDeliveryNamespace,
    ) -> Result<Option<Vec<u8>>, EffectAttestationStoreError>;

    fn commit_fenced_attestation(
        &mut self,
        lease: EffectJournalLease,
        bytes: &[u8],
    ) -> Result<EffectAttestationCommitReceipt, EffectAttestationStoreError>;
}
