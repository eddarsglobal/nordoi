use crate::{
    effect_dispatch::{
        AtomicEffectOutbox, EffectBackend, EffectDeliveryKey, EffectDeliveryNamespace,
        EffectDispatchReceipt, GovernedEffectDispatcher,
    },
    effect_persistence::{EffectJournalCommitReceipt, EffectOutboxCheckpoint},
};

use super::{
    EffectFencingError, EffectFencingResult, EffectJournalLease, EffectJournalWriterId,
    FencedEffectJournalStore,
};

#[derive(Debug)]
pub struct GovernedFencedEffectJournal<S> {
    namespace: EffectDeliveryNamespace,
    writer: EffectJournalWriterId,
    store: S,
    lease: Option<EffectJournalLease>,
}

impl<S> GovernedFencedEffectJournal<S> {
    pub fn new(
        namespace: EffectDeliveryNamespace,
        writer: EffectJournalWriterId,
        store: S,
    ) -> Self {
        Self {
            namespace,
            writer,
            store,
            lease: None,
        }
    }

    pub fn namespace(&self) -> EffectDeliveryNamespace {
        self.namespace
    }

    pub fn writer(&self) -> EffectJournalWriterId {
        self.writer
    }

    pub fn lease(&self) -> Option<EffectJournalLease> {
        self.lease
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    pub fn into_store(self) -> S {
        self.store
    }
}

impl<S: FencedEffectJournalStore> GovernedFencedEffectJournal<S> {
    pub fn acquire(&mut self) -> EffectFencingResult<EffectJournalLease> {
        if self.lease.is_some() {
            return Err(EffectFencingError::LeaseAlreadyHeld);
        }
        let lease = self.store.acquire(self.namespace, self.writer)?;
        self.validate_lease(lease)?;
        self.lease = Some(lease);
        Ok(lease)
    }

    pub fn assert_active(&mut self) -> EffectFencingResult<EffectJournalLease> {
        let lease = self.require_lease()?;
        self.store.assert_active(lease)?;
        Ok(lease)
    }

    pub fn release(&mut self) -> EffectFencingResult<EffectJournalLease> {
        let lease = self.require_lease()?;
        self.store.release(lease)?;
        self.lease = None;
        Ok(lease)
    }

    pub fn checkpoint(
        &mut self,
        outbox: &AtomicEffectOutbox,
    ) -> EffectFencingResult<EffectJournalCommitReceipt> {
        let lease = self.require_lease()?;
        let checkpoint = EffectOutboxCheckpoint::capture(self.namespace, outbox);
        let bytes = checkpoint.canonical_bytes();
        self.store.commit_fenced(lease, &bytes).map_err(Into::into)
    }

    pub fn recover(&mut self) -> EffectFencingResult<Option<EffectOutboxCheckpoint>> {
        let lease = self.require_lease()?;
        let Some(bytes) = self.store.load_fenced(lease)? else {
            return Ok(None);
        };
        let checkpoint = EffectOutboxCheckpoint::from_canonical_bytes(&bytes)?;
        if checkpoint.namespace() != self.namespace {
            return Err(EffectFencingError::LeaseNamespaceMismatch {
                expected: self.namespace,
                actual: checkpoint.namespace(),
            });
        }
        Ok(Some(checkpoint))
    }

    pub fn dispatch_next<B: EffectBackend>(
        &mut self,
        outbox: &mut AtomicEffectOutbox,
        dispatcher: &GovernedEffectDispatcher,
        backend: &mut B,
    ) -> EffectFencingResult<Option<EffectDispatchReceipt>> {
        let lease = self.assert_active()?;
        let Some(request) = outbox.peek().cloned() else {
            return Ok(None);
        };
        let delivery_key = EffectDeliveryKey::new(self.namespace, request.id);
        let mut candidate = outbox.clone();
        let receipt = dispatcher.dispatch_next_with_delivery_context(
            &mut candidate,
            backend,
            Some(delivery_key),
            Some(lease.fence),
        )?;
        self.checkpoint(&candidate)?;
        *outbox = candidate;
        Ok(receipt)
    }

    fn require_lease(&self) -> EffectFencingResult<EffectJournalLease> {
        self.lease.ok_or(EffectFencingError::LeaseRequired)
    }

    fn validate_lease(&self, lease: EffectJournalLease) -> EffectFencingResult<()> {
        if lease.namespace != self.namespace {
            return Err(EffectFencingError::LeaseNamespaceMismatch {
                expected: self.namespace,
                actual: lease.namespace,
            });
        }
        if lease.writer != self.writer {
            return Err(EffectFencingError::LeaseWriterMismatch {
                expected: self.writer,
                actual: lease.writer,
            });
        }
        if lease.fence.0 == 0 {
            return Err(EffectFencingError::InvalidFence(lease.fence));
        }
        Ok(())
    }
}
