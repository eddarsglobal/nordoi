use crate::effect_dispatch::{
    AtomicEffectOutbox, EffectBackend, EffectDeliveryKey, EffectDeliveryNamespace,
    EffectDispatchReceipt, GovernedEffectDispatcher,
};

use super::{
    EffectJournalCommitReceipt, EffectJournalStore, EffectOutboxCheckpoint, EffectPersistenceError,
    EffectPersistenceResult,
};

#[derive(Debug)]
pub struct GovernedEffectJournal<S> {
    namespace: EffectDeliveryNamespace,
    store: S,
}

impl<S> GovernedEffectJournal<S> {
    pub fn new(namespace: EffectDeliveryNamespace, store: S) -> Self {
        Self { namespace, store }
    }

    pub fn namespace(&self) -> EffectDeliveryNamespace {
        self.namespace
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

impl<S: EffectJournalStore> GovernedEffectJournal<S> {
    pub fn checkpoint(
        &mut self,
        outbox: &AtomicEffectOutbox,
    ) -> EffectPersistenceResult<EffectJournalCommitReceipt> {
        let checkpoint = EffectOutboxCheckpoint::capture(self.namespace, outbox);
        let bytes = checkpoint.canonical_bytes();
        self.store.commit(&bytes).map_err(Into::into)
    }

    pub fn recover(&mut self) -> EffectPersistenceResult<Option<EffectOutboxCheckpoint>> {
        let Some(bytes) = self.store.load()? else {
            return Ok(None);
        };
        let checkpoint = EffectOutboxCheckpoint::from_canonical_bytes(&bytes)?;
        if checkpoint.namespace() != self.namespace {
            return Err(EffectPersistenceError::NamespaceMismatch {
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
    ) -> EffectPersistenceResult<Option<EffectDispatchReceipt>> {
        let Some(request) = outbox.peek().cloned() else {
            return Ok(None);
        };
        let delivery_key = EffectDeliveryKey::new(self.namespace, request.id);
        let mut candidate = outbox.clone();
        let receipt = dispatcher.dispatch_next_with_delivery_key(
            &mut candidate,
            backend,
            Some(delivery_key),
        )?;
        self.checkpoint(&candidate)?;
        *outbox = candidate;
        Ok(receipt)
    }
}
