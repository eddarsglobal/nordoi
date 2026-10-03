use crate::{
    effect_dispatch::{
        AtomicEffectOutbox, EffectBackend, EffectBackendError, EffectDeliveryKey,
        EffectDeliveryNamespace, EffectDispatchError, EffectDispatchReceipt, EffectIntentId,
        GovernedEffectDispatcher, QueuedEffectIntent,
    },
    effect_fencing::{
        EffectJournalLease, EffectJournalWriterId, FencedEffectJournalStore,
        GovernedFencedEffectJournal,
    },
    effect_persistence::{
        EffectJournalCommitReceipt, EffectOutboxCheckpoint, MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES,
        MAX_EFFECT_JOURNAL_STRING_BYTES,
    },
};

use super::{
    DeadLetteredEffect, EffectDeadLetterReason, EffectRetryCheckpoint, EffectRetryError,
    EffectRetryLedger, EffectRetryPolicy, EffectRetryRecord, EffectRetryResult, EffectRetryTick,
    MAX_EFFECT_DEAD_LETTERS, MAX_EFFECT_RETRY_CHECKPOINT_BYTES, MAX_EFFECT_RETRY_ENTRIES,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectRetryDispatchOutcome {
    Delivered(EffectDispatchReceipt),
    RetryScheduled {
        request: QueuedEffectIntent,
        failed_attempts: u32,
        next_eligible_tick: EffectRetryTick,
        error: EffectBackendError,
    },
    DeadLettered(DeadLetteredEffect),
    Deferred {
        next_eligible_tick: EffectRetryTick,
    },
}

#[derive(Debug)]
pub struct GovernedRetryEffectJournal<S> {
    fenced: GovernedFencedEffectJournal<S>,
    policy: EffectRetryPolicy,
    ledger: EffectRetryLedger,
}

impl<S> GovernedRetryEffectJournal<S> {
    pub fn new(
        namespace: EffectDeliveryNamespace,
        writer: EffectJournalWriterId,
        store: S,
        policy: EffectRetryPolicy,
    ) -> Self {
        Self {
            fenced: GovernedFencedEffectJournal::new(namespace, writer, store),
            policy,
            ledger: EffectRetryLedger::new(),
        }
    }

    pub fn namespace(&self) -> EffectDeliveryNamespace {
        self.fenced.namespace()
    }

    pub fn writer(&self) -> EffectJournalWriterId {
        self.fenced.writer()
    }

    pub fn lease(&self) -> Option<EffectJournalLease> {
        self.fenced.lease()
    }

    pub fn policy(&self) -> EffectRetryPolicy {
        self.policy
    }

    pub fn ledger(&self) -> &EffectRetryLedger {
        &self.ledger
    }

    pub fn store(&self) -> &S {
        self.fenced.store()
    }

    pub fn store_mut(&mut self) -> &mut S {
        self.fenced.store_mut()
    }

    pub fn into_store(self) -> S {
        self.fenced.into_store()
    }
}

impl<S: FencedEffectJournalStore> GovernedRetryEffectJournal<S> {
    pub fn acquire(&mut self) -> EffectRetryResult<EffectJournalLease> {
        self.fenced.acquire().map_err(Into::into)
    }

    pub fn assert_active(&mut self) -> EffectRetryResult<EffectJournalLease> {
        self.fenced.assert_active().map_err(Into::into)
    }

    pub fn release(&mut self) -> EffectRetryResult<EffectJournalLease> {
        self.fenced.release().map_err(Into::into)
    }

    pub fn checkpoint(
        &mut self,
        outbox: &AtomicEffectOutbox,
    ) -> EffectRetryResult<EffectJournalCommitReceipt> {
        let lease = self.assert_active()?;
        let bytes = self.encode_candidate(outbox, &self.ledger)?;
        self.fenced
            .store_mut()
            .commit_fenced(lease, &bytes)
            .map_err(Into::into)
    }

    pub fn recover(&mut self) -> EffectRetryResult<Option<EffectRetryCheckpoint>> {
        let lease = self.assert_active()?;
        let Some(bytes) = self.fenced.store_mut().load_fenced(lease)? else {
            return Ok(None);
        };
        let checkpoint = EffectRetryCheckpoint::from_canonical_bytes(&bytes)?;
        if checkpoint.namespace() != self.namespace() {
            return Err(EffectRetryError::RetryNamespaceMismatch);
        }
        if let Some(persisted) = checkpoint.policy() {
            if persisted != self.policy {
                return Err(EffectRetryError::RetryPolicyMismatch);
            }
        }
        Ok(Some(checkpoint))
    }

    pub(crate) fn adopt_recovered_ledger(&mut self, ledger: EffectRetryLedger) {
        self.ledger = ledger;
    }

    pub(crate) fn replace_ledger(&mut self, ledger: EffectRetryLedger) {
        self.ledger = ledger;
    }

    pub fn dispatch_next<B: EffectBackend>(
        &mut self,
        outbox: &mut AtomicEffectOutbox,
        current_tick: EffectRetryTick,
        dispatcher: &GovernedEffectDispatcher,
        backend: &mut B,
    ) -> EffectRetryResult<Option<EffectRetryDispatchOutcome>> {
        let lease = self.assert_active()?;
        self.ensure_tick_not_backward(current_tick)?;

        let Some((request, deferred_until)) = self.select_candidate(outbox, current_tick) else {
            return Ok(None);
        };
        if let Some(next_eligible_tick) = deferred_until {
            return Ok(Some(EffectRetryDispatchOutcome::Deferred {
                next_eligible_tick,
            }));
        }

        let delivery_key = EffectDeliveryKey::new(self.namespace(), request.id);
        let mut candidate_outbox = outbox.clone();
        let mut candidate_ledger = self.ledger.clone();

        match dispatcher.dispatch_intent_with_delivery_context(
            &mut candidate_outbox,
            request.id,
            backend,
            Some(delivery_key),
            Some(lease.fence),
        ) {
            Ok(receipt) => {
                candidate_ledger.clear_retry(request.id);
                candidate_ledger.set_last_tick(current_tick);
                self.commit_candidate(lease, &candidate_outbox, &candidate_ledger)?;
                *outbox = candidate_outbox;
                self.ledger = candidate_ledger;
                Ok(Some(EffectRetryDispatchOutcome::Delivered(receipt)))
            }
            Err(EffectDispatchError::BackendFailed { intent, error }) => {
                debug_assert_eq!(intent, request.id);
                let previous_failures = candidate_ledger
                    .retry(request.id)
                    .map_or(0, |record| record.failed_attempts);
                let failed_attempts = previous_failures
                    .checked_add(1)
                    .ok_or(EffectRetryError::InvalidFailureCount(u32::MAX))?;

                if !error.is_retryable() || failed_attempts >= self.policy.max_attempts() {
                    let removed = candidate_outbox.abandon(request.id)?;
                    let reason = if error.is_retryable() {
                        EffectDeadLetterReason::AttemptsExhausted
                    } else {
                        EffectDeadLetterReason::PermanentBackendFailure
                    };
                    let dead = DeadLetteredEffect {
                        request: removed,
                        delivery_key,
                        failed_attempts,
                        reason,
                        last_error: error.message().to_owned(),
                    };
                    candidate_ledger.insert_dead_letter(dead.clone());
                    candidate_ledger.set_last_tick(current_tick);
                    self.commit_candidate(lease, &candidate_outbox, &candidate_ledger)?;
                    *outbox = candidate_outbox;
                    self.ledger = candidate_ledger;
                    Ok(Some(EffectRetryDispatchOutcome::DeadLettered(dead)))
                } else {
                    let delay = self.policy.delay_ticks(delivery_key, failed_attempts)?;
                    let next_eligible_tick = current_tick
                        .checked_add(delay)
                        .ok_or(EffectRetryError::RetryTickOverflow)?;
                    candidate_ledger.set_retry(EffectRetryRecord {
                        intent: request.id,
                        failed_attempts,
                        next_eligible_tick,
                    });
                    candidate_ledger.set_last_tick(current_tick);
                    self.commit_candidate(lease, &candidate_outbox, &candidate_ledger)?;
                    self.ledger = candidate_ledger;
                    Ok(Some(EffectRetryDispatchOutcome::RetryScheduled {
                        request,
                        failed_attempts,
                        next_eligible_tick,
                        error,
                    }))
                }
            }
            Err(error) => Err(EffectRetryError::Dispatch(error)),
        }
    }

    pub fn redrive_dead_letter(
        &mut self,
        outbox: &mut AtomicEffectOutbox,
        id: EffectIntentId,
    ) -> EffectRetryResult<QueuedEffectIntent> {
        let lease = self.assert_active()?;
        let mut candidate_outbox = outbox.clone();
        let mut candidate_ledger = self.ledger.clone();
        let dead = candidate_ledger
            .remove_dead_letter(id)
            .ok_or(EffectRetryError::UnknownDeadLetter(id))?;
        candidate_outbox.restore_pending(dead.request.clone())?;
        candidate_ledger.clear_retry(id);
        self.commit_candidate(lease, &candidate_outbox, &candidate_ledger)?;
        *outbox = candidate_outbox;
        self.ledger = candidate_ledger;
        Ok(dead.request)
    }

    pub fn discard_dead_letter(
        &mut self,
        outbox: &AtomicEffectOutbox,
        id: EffectIntentId,
    ) -> EffectRetryResult<DeadLetteredEffect> {
        let lease = self.assert_active()?;
        let mut candidate_ledger = self.ledger.clone();
        let dead = candidate_ledger
            .remove_dead_letter(id)
            .ok_or(EffectRetryError::UnknownDeadLetter(id))?;
        self.commit_candidate(lease, outbox, &candidate_ledger)?;
        self.ledger = candidate_ledger;
        Ok(dead)
    }

    pub(crate) fn ensure_tick_not_backward(
        &self,
        current_tick: EffectRetryTick,
    ) -> EffectRetryResult<()> {
        let previous = self.ledger.last_tick();
        if current_tick < previous {
            return Err(EffectRetryError::RetryTickMovedBackward {
                previous,
                current: current_tick,
            });
        }
        Ok(())
    }

    pub(crate) fn select_candidate(
        &self,
        outbox: &AtomicEffectOutbox,
        current_tick: EffectRetryTick,
    ) -> Option<(QueuedEffectIntent, Option<EffectRetryTick>)> {
        let mut earliest_deferred: Option<EffectRetryTick> = None;
        for request in outbox.iter() {
            match self.ledger.retry(request.id) {
                Some(record) if record.next_eligible_tick > current_tick => {
                    earliest_deferred = Some(match earliest_deferred {
                        Some(existing) => existing.min(record.next_eligible_tick),
                        None => record.next_eligible_tick,
                    });
                }
                _ => return Some((request.clone(), None)),
            }
        }

        // There are pending intents but all are delayed. Return the first pending request
        // only as a carrier for the Deferred outcome; no backend work will occur.
        outbox
            .peek()
            .cloned()
            .map(|request| (request, earliest_deferred))
    }

    pub(crate) fn commit_candidate(
        &mut self,
        lease: EffectJournalLease,
        outbox: &AtomicEffectOutbox,
        ledger: &EffectRetryLedger,
    ) -> EffectRetryResult<EffectJournalCommitReceipt> {
        let bytes = self.encode_candidate(outbox, ledger)?;
        self.fenced
            .store_mut()
            .commit_fenced(lease, &bytes)
            .map_err(Into::into)
    }

    pub(crate) fn encode_candidate(
        &self,
        outbox: &AtomicEffectOutbox,
        ledger: &EffectRetryLedger,
    ) -> EffectRetryResult<Vec<u8>> {
        if ledger.retry_count() > MAX_EFFECT_RETRY_ENTRIES {
            return Err(EffectRetryError::RetryEntryLimitExceeded {
                entries: ledger.retry_count() as u64,
                limit: MAX_EFFECT_RETRY_ENTRIES,
            });
        }
        if ledger.dead_letter_count() > MAX_EFFECT_DEAD_LETTERS {
            return Err(EffectRetryError::DeadLetterLimitExceeded {
                entries: ledger.dead_letter_count() as u64,
                limit: MAX_EFFECT_DEAD_LETTERS,
            });
        }
        for dead in ledger.dead_letters() {
            if dead.last_error.len() > MAX_EFFECT_JOURNAL_STRING_BYTES {
                return Err(EffectRetryError::RetryCheckpointStringTooLarge {
                    bytes: dead.last_error.len() as u64,
                    limit: MAX_EFFECT_JOURNAL_STRING_BYTES,
                });
            }
        }

        let outbox_bytes =
            EffectOutboxCheckpoint::capture(self.namespace(), outbox).canonical_bytes();
        if outbox_bytes.len() > MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES {
            return Err(EffectRetryError::Persistence(
                crate::effect_persistence::EffectPersistenceError::CheckpointTooLarge {
                    bytes: outbox_bytes.len(),
                    limit: MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES,
                },
            ));
        }

        let bytes = EffectRetryCheckpoint::capture(self.namespace(), self.policy, outbox, ledger)
            .canonical_bytes();
        if bytes.len() > MAX_EFFECT_RETRY_CHECKPOINT_BYTES {
            return Err(EffectRetryError::RetryCheckpointTooLarge {
                bytes: bytes.len(),
                limit: MAX_EFFECT_RETRY_CHECKPOINT_BYTES,
            });
        }
        Ok(bytes)
    }
}
