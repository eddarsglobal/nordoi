use crate::effect_fencing::{EffectJournalLease, FencedEffectJournalStore};

use super::{RuntimeCheckpointCommitReceipt, RuntimeCheckpointStoreError};

/// Host-injected persistence authority for the K1.14 whole-runtime semantic checkpoint.
///
/// A conforming implementation is also the effect journal store. The combined commit MUST:
/// - validate the supplied lease/fence atomically;
/// - atomically replace both the effect-audit checkpoint and runtime checkpoint;
/// - expose the effect bytes through `FencedEffectJournalStore::load_fenced`;
/// - expose the runtime bytes through `load_runtime_fenced`;
/// - never allow a stale fence to update either half of the bundle.
///
/// Later effect dispatch/retry/audit commits are allowed to advance the effect half by using the
/// inherited K1.8/K1.10 store API. Runtime recovery validates that such audit history is a
/// descendant of the prefix bound into the runtime checkpoint and that no newer effect-intent
/// allocation frontier exists.
pub trait FencedRuntimeCheckpointStore: FencedEffectJournalStore {
    fn load_runtime_fenced(
        &mut self,
        lease: EffectJournalLease,
    ) -> Result<Option<Vec<u8>>, RuntimeCheckpointStoreError>;

    fn commit_effect_and_runtime_fenced(
        &mut self,
        lease: EffectJournalLease,
        effect_checkpoint: &[u8],
        runtime_checkpoint: &[u8],
    ) -> Result<RuntimeCheckpointCommitReceipt, RuntimeCheckpointStoreError>;
}
