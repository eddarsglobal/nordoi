use crate::{
    effect_dispatch::{
        AtomicEffectOutbox, EffectBackend, EffectBackendError, EffectDeliveryKey,
        EffectDeliveryNamespace, EffectDispatchError, EffectDispatchReceipt, EffectIntentId,
        GovernedEffectDispatcher, QueuedEffectIntent,
    },
    effect_fencing::{EffectJournalLease, EffectJournalWriterId, FencedEffectJournalStore},
    effect_persistence::{EffectJournalCommitReceipt, MAX_EFFECT_JOURNAL_STRING_BYTES},
    effect_retry::{
        DeadLetteredEffect, EffectDeadLetterReason, EffectRetryLedger, EffectRetryPolicy,
        EffectRetryRecord, EffectRetryTick, GovernedRetryEffectJournal,
    },
};

use super::{
    EffectAttemptId, EffectAuditCheckpoint, EffectAuditError, EffectAuditEvent, EffectAuditLedger,
    EffectAuditResult, EffectInDoubtAttempt, MAX_EFFECT_AUDIT_CHECKPOINT_BYTES,
    MAX_EFFECT_AUDIT_EVENTS,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectAuditDispatchOutcome {
    Delivered {
        attempt: EffectAttemptId,
        receipt: EffectDispatchReceipt,
    },
    RetryScheduled {
        attempt: EffectAttemptId,
        request: QueuedEffectIntent,
        failed_attempts: u32,
        next_eligible_tick: EffectRetryTick,
        error: EffectBackendError,
    },
    DeadLettered {
        attempt: EffectAttemptId,
        dead: DeadLetteredEffect,
    },
    Deferred {
        next_eligible_tick: EffectRetryTick,
    },
}

#[derive(Debug)]
pub struct GovernedAuditedEffectJournal<S> {
    retry: GovernedRetryEffectJournal<S>,
    audit: EffectAuditLedger,
}

impl<S> GovernedAuditedEffectJournal<S> {
    pub fn new(
        namespace: EffectDeliveryNamespace,
        writer: EffectJournalWriterId,
        store: S,
        policy: EffectRetryPolicy,
    ) -> Self {
        Self {
            retry: GovernedRetryEffectJournal::new(namespace, writer, store, policy),
            audit: EffectAuditLedger::new(),
        }
    }

    pub fn namespace(&self) -> EffectDeliveryNamespace {
        self.retry.namespace()
    }
    pub fn writer(&self) -> EffectJournalWriterId {
        self.retry.writer()
    }
    pub fn lease(&self) -> Option<EffectJournalLease> {
        self.retry.lease()
    }
    pub fn policy(&self) -> EffectRetryPolicy {
        self.retry.policy()
    }
    pub fn retry_ledger(&self) -> &EffectRetryLedger {
        self.retry.ledger()
    }
    pub fn audit_ledger(&self) -> &EffectAuditLedger {
        &self.audit
    }
    pub fn in_doubt_attempt(&self) -> Option<&EffectInDoubtAttempt> {
        self.audit.in_doubt()
    }
    pub fn store(&self) -> &S {
        self.retry.store()
    }
    pub fn store_mut(&mut self) -> &mut S {
        self.retry.store_mut()
    }
    pub fn into_store(self) -> S {
        self.retry.into_store()
    }
}

impl<S: FencedEffectJournalStore> GovernedAuditedEffectJournal<S> {
    pub fn acquire(&mut self) -> EffectAuditResult<EffectJournalLease> {
        self.retry.acquire().map_err(Into::into)
    }
    pub fn assert_active(&mut self) -> EffectAuditResult<EffectJournalLease> {
        self.retry.assert_active().map_err(Into::into)
    }
    pub fn release(&mut self) -> EffectAuditResult<EffectJournalLease> {
        self.retry.release().map_err(Into::into)
    }

    pub fn checkpoint(
        &mut self,
        outbox: &AtomicEffectOutbox,
    ) -> EffectAuditResult<EffectJournalCommitReceipt> {
        let lease = self.assert_active()?;
        let retry_ledger = self.retry.ledger().clone();
        let audit = self.audit.clone();
        self.commit_state(lease, outbox, &retry_ledger, &audit)
    }

    pub fn recover(&mut self) -> EffectAuditResult<Option<EffectAuditCheckpoint>> {
        let lease = self.assert_active()?;
        let Some(bytes) = self.retry.store_mut().load_fenced(lease)? else {
            return Ok(None);
        };
        let checkpoint = EffectAuditCheckpoint::from_canonical_bytes(&bytes)?;
        if checkpoint.namespace() != self.namespace() {
            return Err(EffectAuditError::AuditNamespaceMismatch);
        }
        if let Some(persisted) = checkpoint.retry().policy() {
            if persisted != self.policy() {
                return Err(EffectAuditError::AuditPolicyMismatch);
            }
        }
        Ok(Some(checkpoint))
    }

    pub(crate) fn adopt_recovered_state(
        &mut self,
        retry_ledger: EffectRetryLedger,
        audit: EffectAuditLedger,
    ) {
        self.retry.replace_ledger(retry_ledger);
        self.audit = audit;
    }

    pub fn dispatch_next<B: EffectBackend>(
        &mut self,
        outbox: &mut AtomicEffectOutbox,
        current_tick: EffectRetryTick,
        dispatcher: &GovernedEffectDispatcher,
        backend: &mut B,
    ) -> EffectAuditResult<Option<EffectAuditDispatchOutcome>> {
        let lease = self.assert_active()?;
        self.retry.ensure_tick_not_backward(current_tick)?;
        if let Some(open) = self.audit.in_doubt() {
            return Err(EffectAuditError::InDoubtAttemptExists(open.attempt));
        }

        let Some((selected, deferred_until)) = self.retry.select_candidate(outbox, current_tick)
        else {
            return Ok(None);
        };
        if let Some(next_eligible_tick) = deferred_until {
            return Ok(Some(EffectAuditDispatchOutcome::Deferred {
                next_eligible_tick,
            }));
        }

        let request = dispatcher.preflight_intent(outbox, selected.id, backend)?;
        validate_string(&request.intent.action_name)?;
        validate_effect_strings(&request.intent.effect)?;
        let delivery_key = EffectDeliveryKey::new(self.namespace(), request.id);
        let failed_attempts_before = self
            .retry
            .ledger()
            .retry(request.id)
            .map_or(0, |record| record.failed_attempts);

        let mut prepared_retry = self.retry.ledger().clone();
        prepared_retry.set_last_tick(current_tick);
        let mut prepared_audit = self.audit.clone();
        let attempt = prepared_audit.allocate_attempt()?;
        prepared_audit.append(EffectAuditEvent::AttemptPrepared(EffectInDoubtAttempt {
            attempt,
            request: request.clone(),
            delivery_key,
            delivery_fence: lease.fence,
            prepared_tick: current_tick,
            failed_attempts_before,
        }))?;
        self.commit_state(lease, outbox, &prepared_retry, &prepared_audit)?;
        self.retry.replace_ledger(prepared_retry);
        self.audit = prepared_audit;

        let mut candidate_outbox = outbox.clone();
        let mut candidate_retry = self.retry.ledger().clone();
        let mut candidate_audit = self.audit.clone();

        match dispatcher.execute_preflighted_intent_with_delivery_context(
            &mut candidate_outbox,
            request.clone(),
            backend,
            Some(delivery_key),
            Some(lease.fence),
        ) {
            Ok(receipt) => {
                if let Some(reference) = receipt.backend.reference.as_deref() {
                    validate_string(reference)?;
                }
                candidate_retry.clear_retry(request.id);
                candidate_retry.set_last_tick(current_tick);
                candidate_audit.append(EffectAuditEvent::AttemptDelivered {
                    attempt,
                    intent: request.id,
                    backend_reference: receipt.backend.reference.clone(),
                })?;
                self.commit_state(lease, &candidate_outbox, &candidate_retry, &candidate_audit)?;
                *outbox = candidate_outbox;
                self.retry.replace_ledger(candidate_retry);
                self.audit = candidate_audit;
                Ok(Some(EffectAuditDispatchOutcome::Delivered {
                    attempt,
                    receipt,
                }))
            }
            Err(EffectDispatchError::BackendFailed { intent, error }) => {
                debug_assert_eq!(intent, request.id);
                validate_string(error.message())?;
                let failed_attempts = failed_attempts_before.checked_add(1).ok_or(
                    crate::effect_retry::EffectRetryError::InvalidFailureCount(u32::MAX),
                )?;

                if !error.is_retryable() || failed_attempts >= self.policy().max_attempts() {
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
                    candidate_retry.insert_dead_letter(dead.clone());
                    candidate_retry.set_last_tick(current_tick);
                    candidate_audit.append(EffectAuditEvent::AttemptDeadLettered {
                        attempt,
                        intent: request.id,
                        failed_attempts,
                        reason,
                        error: error.message().to_owned(),
                    })?;
                    self.commit_state(
                        lease,
                        &candidate_outbox,
                        &candidate_retry,
                        &candidate_audit,
                    )?;
                    *outbox = candidate_outbox;
                    self.retry.replace_ledger(candidate_retry);
                    self.audit = candidate_audit;
                    Ok(Some(EffectAuditDispatchOutcome::DeadLettered {
                        attempt,
                        dead,
                    }))
                } else {
                    let delay = self.policy().delay_ticks(delivery_key, failed_attempts)?;
                    let next_eligible_tick = current_tick
                        .checked_add(delay)
                        .ok_or(crate::effect_retry::EffectRetryError::RetryTickOverflow)?;
                    candidate_retry.set_retry(EffectRetryRecord {
                        intent: request.id,
                        failed_attempts,
                        next_eligible_tick,
                    });
                    candidate_retry.set_last_tick(current_tick);
                    candidate_audit.append(EffectAuditEvent::AttemptRetryScheduled {
                        attempt,
                        intent: request.id,
                        failed_attempts,
                        next_eligible_tick,
                        error: error.message().to_owned(),
                    })?;
                    self.commit_state(
                        lease,
                        &candidate_outbox,
                        &candidate_retry,
                        &candidate_audit,
                    )?;
                    self.retry.replace_ledger(candidate_retry);
                    self.audit = candidate_audit;
                    Ok(Some(EffectAuditDispatchOutcome::RetryScheduled {
                        attempt,
                        request,
                        failed_attempts,
                        next_eligible_tick,
                        error,
                    }))
                }
            }
            Err(error) => Err(EffectAuditError::Dispatch(error)),
        }
    }

    pub fn resolve_in_doubt_as_delivered(
        &mut self,
        outbox: &mut AtomicEffectOutbox,
        resolution_tick: EffectRetryTick,
        backend_reference: Option<String>,
    ) -> EffectAuditResult<QueuedEffectIntent> {
        let lease = self.assert_active()?;
        self.ensure_resolution_tick(resolution_tick)?;
        if let Some(reference) = backend_reference.as_deref() {
            validate_string(reference)?;
        }
        let open = self
            .audit
            .in_doubt()
            .cloned()
            .ok_or(EffectAuditError::NoInDoubtAttempt)?;
        let request = outbox
            .get(open.request.id)
            .cloned()
            .ok_or(EffectAuditError::InDoubtIntentNotPending(open.request.id))?;
        if request != open.request {
            return Err(EffectAuditError::InDoubtIntentMismatch {
                expected: open.request.id,
                actual: request.id,
            });
        }

        let mut candidate_outbox = outbox.clone();
        candidate_outbox.acknowledge(request.id)?;
        let mut candidate_retry = self.retry.ledger().clone();
        candidate_retry.clear_retry(request.id);
        candidate_retry.set_last_tick(resolution_tick);
        let mut candidate_audit = self.audit.clone();
        candidate_audit.append(EffectAuditEvent::InDoubtAssumedDelivered {
            attempt: open.attempt,
            intent: request.id,
            resolution_tick,
            backend_reference,
        })?;
        self.commit_state(lease, &candidate_outbox, &candidate_retry, &candidate_audit)?;
        *outbox = candidate_outbox;
        self.retry.replace_ledger(candidate_retry);
        self.audit = candidate_audit;
        Ok(request)
    }

    pub fn authorize_in_doubt_retry(
        &mut self,
        outbox: &AtomicEffectOutbox,
        resolution_tick: EffectRetryTick,
    ) -> EffectAuditResult<QueuedEffectIntent> {
        let lease = self.assert_active()?;
        self.ensure_resolution_tick(resolution_tick)?;
        let open = self
            .audit
            .in_doubt()
            .cloned()
            .ok_or(EffectAuditError::NoInDoubtAttempt)?;
        let request = outbox
            .get(open.request.id)
            .cloned()
            .ok_or(EffectAuditError::InDoubtIntentNotPending(open.request.id))?;
        let mut candidate_retry = self.retry.ledger().clone();
        candidate_retry.set_last_tick(resolution_tick);
        let mut candidate_audit = self.audit.clone();
        candidate_audit.append(EffectAuditEvent::InDoubtRetryAuthorized {
            attempt: open.attempt,
            intent: request.id,
            resolution_tick,
        })?;
        self.commit_state(lease, outbox, &candidate_retry, &candidate_audit)?;
        self.retry.replace_ledger(candidate_retry);
        self.audit = candidate_audit;
        Ok(request)
    }

    pub fn redrive_dead_letter(
        &mut self,
        outbox: &mut AtomicEffectOutbox,
        id: EffectIntentId,
    ) -> EffectAuditResult<QueuedEffectIntent> {
        let lease = self.assert_active()?;
        self.require_no_in_doubt()?;
        let mut candidate_outbox = outbox.clone();
        let mut candidate_retry = self.retry.ledger().clone();
        let dead = candidate_retry
            .remove_dead_letter(id)
            .ok_or(crate::effect_retry::EffectRetryError::UnknownDeadLetter(id))?;
        candidate_outbox.restore_pending(dead.request.clone())?;
        candidate_retry.clear_retry(id);
        let mut candidate_audit = self.audit.clone();
        candidate_audit.append(EffectAuditEvent::DeadLetterRedriven { intent: id })?;
        self.commit_state(lease, &candidate_outbox, &candidate_retry, &candidate_audit)?;
        *outbox = candidate_outbox;
        self.retry.replace_ledger(candidate_retry);
        self.audit = candidate_audit;
        Ok(dead.request)
    }

    pub fn discard_dead_letter(
        &mut self,
        outbox: &AtomicEffectOutbox,
        id: EffectIntentId,
    ) -> EffectAuditResult<DeadLetteredEffect> {
        let lease = self.assert_active()?;
        self.require_no_in_doubt()?;
        let mut candidate_retry = self.retry.ledger().clone();
        let dead = candidate_retry
            .remove_dead_letter(id)
            .ok_or(crate::effect_retry::EffectRetryError::UnknownDeadLetter(id))?;
        let mut candidate_audit = self.audit.clone();
        candidate_audit.append(EffectAuditEvent::DeadLetterDiscarded { intent: id })?;
        self.commit_state(lease, outbox, &candidate_retry, &candidate_audit)?;
        self.retry.replace_ledger(candidate_retry);
        self.audit = candidate_audit;
        Ok(dead)
    }

    fn require_no_in_doubt(&self) -> EffectAuditResult<()> {
        if let Some(open) = self.audit.in_doubt() {
            Err(EffectAuditError::InDoubtAttemptExists(open.attempt))
        } else {
            Ok(())
        }
    }

    fn ensure_resolution_tick(&self, tick: EffectRetryTick) -> EffectAuditResult<()> {
        let previous = self.retry.ledger().last_tick();
        if tick < previous {
            return Err(EffectAuditError::ResolutionTickMovedBackward {
                previous,
                current: tick,
            });
        }
        Ok(())
    }

    fn commit_state(
        &mut self,
        lease: EffectJournalLease,
        outbox: &AtomicEffectOutbox,
        retry_ledger: &EffectRetryLedger,
        audit: &EffectAuditLedger,
    ) -> EffectAuditResult<EffectJournalCommitReceipt> {
        if audit.len() > MAX_EFFECT_AUDIT_EVENTS {
            return Err(EffectAuditError::AuditEventLimitExceeded {
                events: audit.len() as u64,
                limit: MAX_EFFECT_AUDIT_EVENTS,
            });
        }
        let checkpoint = EffectAuditCheckpoint::capture(
            self.namespace(),
            self.policy(),
            outbox,
            retry_ledger,
            audit,
        );
        let bytes = checkpoint.canonical_bytes();
        if bytes.len() > MAX_EFFECT_AUDIT_CHECKPOINT_BYTES {
            return Err(EffectAuditError::AuditCheckpointTooLarge {
                bytes: bytes.len(),
                limit: MAX_EFFECT_AUDIT_CHECKPOINT_BYTES,
            });
        }
        self.retry
            .store_mut()
            .commit_fenced(lease, &bytes)
            .map_err(Into::into)
    }
}

fn validate_effect_strings(effect: &crate::effect::Effect) -> EffectAuditResult<()> {
    match effect {
        crate::effect::Effect::Network(scope)
        | crate::effect::Effect::FileRead(scope)
        | crate::effect::Effect::FileWrite(scope) => validate_string(scope),
        _ => Ok(()),
    }
}

fn validate_string(value: &str) -> EffectAuditResult<()> {
    if value.len() > MAX_EFFECT_JOURNAL_STRING_BYTES {
        return Err(EffectAuditError::AuditCheckpointStringTooLarge {
            bytes: value.len() as u64,
            limit: MAX_EFFECT_JOURNAL_STRING_BYTES,
        });
    }
    Ok(())
}
