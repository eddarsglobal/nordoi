use super::EffectJournalStoreError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectJournalCommitReceipt {
    pub reference: Option<String>,
}

impl EffectJournalCommitReceipt {
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: Some(reference.into()),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }
}

/// Host-injected persistence boundary for effect-journal checkpoints.
///
/// Contract: when `commit` returns `Ok`, the supplied checkpoint must atomically replace
/// the previously committed checkpoint for this journal and be recoverable according to
/// the durability guarantees advertised by the host implementation. The NORDOI core can
/// validate bytes and protocol ordering but cannot prove that a host store actually
/// flushed data to non-volatile media.
pub trait EffectJournalStore {
    fn load(&mut self) -> Result<Option<Vec<u8>>, EffectJournalStoreError>;

    fn commit(
        &mut self,
        bytes: &[u8],
    ) -> Result<EffectJournalCommitReceipt, EffectJournalStoreError>;
}
