use crate::{
    effect_dispatch::EffectDeliveryNamespace, effect_persistence::EffectJournalCommitReceipt,
};

use super::{EffectFenceStoreError, EffectJournalLease, EffectJournalWriterId};

/// Host-injected fencing + persistence authority for one effect journal.
///
/// Contract for a conforming implementation:
/// - every successful `acquire` for the same namespace returns a strictly greater non-zero fence;
/// - `assert_active`, `load_fenced`, `commit_fenced`, and `release` reject stale leases;
/// - `commit_fenced` atomically validates the fence and replaces the committed checkpoint;
/// - a stale writer can never overwrite a checkpoint committed by a newer fence.
///
/// The core validates protocol shape and ordering but cannot prove that an arbitrary host store
/// honestly implements distributed consensus, durable media, compare-and-swap, or linearizability.
pub trait FencedEffectJournalStore {
    fn acquire(
        &mut self,
        namespace: EffectDeliveryNamespace,
        writer: EffectJournalWriterId,
    ) -> Result<EffectJournalLease, EffectFenceStoreError>;

    fn assert_active(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError>;

    fn load_fenced(
        &mut self,
        lease: EffectJournalLease,
    ) -> Result<Option<Vec<u8>>, EffectFenceStoreError>;

    fn commit_fenced(
        &mut self,
        lease: EffectJournalLease,
        bytes: &[u8],
    ) -> Result<EffectJournalCommitReceipt, EffectFenceStoreError>;

    fn release(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError>;
}
