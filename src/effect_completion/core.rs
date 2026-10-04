use std::collections::{BTreeMap, BTreeSet};

use crate::{
    atom::AtomId,
    effect_audit::{EffectAuditEvent, EffectAuditLedger, EffectInDoubtAttempt},
    effect_dispatch::{EffectDeliveryKey, EffectDeliveryNamespace},
    kernel::AtomicKernel,
    ownership::DomainId,
    runtime_checkpoint::CompletionCheckpointState,
    value::Value,
};

use super::{
    EffectCompletion, EffectCompletionApplicationReport, EffectCompletionAuthority,
    EffectCompletionBatch, EffectCompletionBatchReport, EffectCompletionError,
    EffectCompletionOutcome, EffectCompletionProjection, EffectCompletionProjectionValue,
    EffectCompletionResult, EffectCompletionSequence, EffectCompletionSourceId,
    EffectCompletionWriteReport, MAX_EFFECT_COMPLETION_BATCH, MAX_EFFECT_COMPLETION_TEXT_BYTES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CompletionRoute {
    source: EffectCompletionSourceId,
    namespace: EffectDeliveryNamespace,
}

impl CompletionRoute {
    const fn new(source: EffectCompletionSourceId, namespace: EffectDeliveryNamespace) -> Self {
        Self { source, namespace }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AtomicEffectCompletionCore {
    authority: EffectCompletionAuthority,
    projections: BTreeMap<CompletionRoute, BTreeMap<AtomId, EffectCompletionProjection>>,
    last_sequences: BTreeMap<EffectCompletionSourceId, EffectCompletionSequence>,
    completed_deliveries: BTreeSet<EffectDeliveryKey>,
}

impl AtomicEffectCompletionCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn authority(&self) -> &EffectCompletionAuthority {
        &self.authority
    }

    pub fn authority_mut(&mut self) -> &mut EffectCompletionAuthority {
        &mut self.authority
    }

    pub fn last_sequence(
        &self,
        source: EffectCompletionSourceId,
    ) -> Option<EffectCompletionSequence> {
        self.last_sequences.get(&source).copied()
    }

    pub fn has_completed(&self, key: EffectDeliveryKey) -> bool {
        self.completed_deliveries.contains(&key)
    }

    pub(crate) fn capture_checkpoint_state(&self) -> CompletionCheckpointState {
        CompletionCheckpointState {
            last_sequences: self.last_sequences.clone(),
            completed_deliveries: self.completed_deliveries.clone(),
        }
    }

    pub(crate) fn restore_checkpoint_state(&mut self, state: &CompletionCheckpointState) {
        self.last_sequences = state.last_sequences.clone();
        self.completed_deliveries = state.completed_deliveries.clone();
    }

    pub fn register_projection(
        &mut self,
        kernel: &AtomicKernel,
        projection: EffectCompletionProjection,
    ) -> EffectCompletionResult<()> {
        if projection.source.0 == 0 {
            return Err(EffectCompletionError::InvalidSourceId);
        }
        kernel.require_owner(projection.atom, projection.domain)?;
        let route = CompletionRoute::new(projection.source, projection.namespace);
        let entries = self.projections.entry(route).or_default();
        if entries.contains_key(&projection.atom) {
            return Err(EffectCompletionError::DuplicateProjection {
                source: projection.source,
                namespace: projection.namespace,
                atom: projection.atom,
            });
        }
        entries.insert(projection.atom, projection);
        Ok(())
    }

    pub fn unregister_projection(
        &mut self,
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
        atom: AtomId,
    ) -> bool {
        let route = CompletionRoute::new(source, namespace);
        let (removed, empty) = {
            let Some(entries) = self.projections.get_mut(&route) else {
                return false;
            };
            let removed = entries.remove(&atom).is_some();
            (removed, entries.is_empty())
        };
        if empty {
            self.projections.remove(&route);
        }
        removed
    }

    pub fn apply_batch(
        &mut self,
        kernel: &mut AtomicKernel,
        audit: &EffectAuditLedger,
        batch: &EffectCompletionBatch,
    ) -> EffectCompletionResult<EffectCompletionBatchReport> {
        let canonical = canonicalize_batch(batch)?;
        let mut report = EffectCompletionBatchReport {
            causes: canonical.len(),
            ..EffectCompletionBatchReport::default()
        };
        if canonical.is_empty() {
            return Ok(report);
        }

        let mut candidate_core = self.clone();
        let mut candidate_kernel = kernel.clone();

        for completion in &canonical {
            let application =
                candidate_core.apply_one(&mut candidate_kernel, audit, completion, &mut report)?;
            report.applications.push(application);
            report.accepted += 1;
        }

        *self = candidate_core;
        *kernel = candidate_kernel;
        Ok(report)
    }

    fn apply_one(
        &mut self,
        kernel: &mut AtomicKernel,
        audit: &EffectAuditLedger,
        completion: &EffectCompletion,
        report: &mut EffectCompletionBatchReport,
    ) -> EffectCompletionResult<EffectCompletionApplicationReport> {
        validate_completion_value(&completion.outcome)?;
        if !self
            .authority
            .allows(completion.source, completion.delivery_key.namespace)
        {
            return Err(EffectCompletionError::UnauthorizedSource {
                source: completion.source,
                namespace: completion.delivery_key.namespace,
            });
        }
        if let Some(previous) = self.last_sequences.get(&completion.source).copied() {
            if completion.sequence <= previous {
                return Err(EffectCompletionError::NonMonotonicSequence {
                    source: completion.source,
                    previous,
                    current: completion.sequence,
                });
            }
        }
        if self.completed_deliveries.contains(&completion.delivery_key) {
            return Err(EffectCompletionError::DeliveryAlreadyCompleted(
                completion.delivery_key,
            ));
        }

        validate_audit_correlation(audit, completion)?;

        let route = CompletionRoute::new(completion.source, completion.delivery_key.namespace);
        let projections =
            self.projections
                .get(&route)
                .ok_or(EffectCompletionError::NoProjection {
                    source: completion.source,
                    namespace: completion.delivery_key.namespace,
                })?;

        let mut by_domain: BTreeMap<DomainId, Vec<(AtomId, Value)>> = BTreeMap::new();
        let mut writes = Vec::with_capacity(projections.len());
        for projection in projections.values() {
            kernel.require_owner(projection.atom, projection.domain)?;
            let value = project_value(projection.value, completion)?;
            by_domain
                .entry(projection.domain)
                .or_default()
                .push((projection.atom, value.clone()));
            writes.push(EffectCompletionWriteReport {
                domain: projection.domain,
                atom: projection.atom,
                value,
            });
        }

        for (domain, domain_writes) in by_domain {
            let mut transaction = kernel.begin_transaction(domain)?;
            for (atom, value) in domain_writes {
                transaction.set(kernel, atom, value)?;
            }
            let tx_report = kernel.commit(transaction)?;
            report.transactions += 1;
            report.staged_writes += tx_report.staged_writes;
            report.changed_atoms += tx_report.changed_atoms;
        }

        self.last_sequences
            .insert(completion.source, completion.sequence);
        self.completed_deliveries.insert(completion.delivery_key);

        Ok(EffectCompletionApplicationReport {
            source: completion.source,
            sequence: completion.sequence,
            delivery_key: completion.delivery_key,
            attempt: completion.attempt,
            outcome: completion.outcome.clone(),
            writes,
        })
    }
}

fn canonicalize_batch(
    batch: &EffectCompletionBatch,
) -> EffectCompletionResult<Vec<EffectCompletion>> {
    if batch.len() > MAX_EFFECT_COMPLETION_BATCH {
        return Err(EffectCompletionError::BatchTooLarge {
            actual: batch.len(),
            limit: MAX_EFFECT_COMPLETION_BATCH,
        });
    }

    let mut canonical = batch.completions.clone();
    for completion in &canonical {
        if completion.source.0 == 0 {
            return Err(EffectCompletionError::InvalidSourceId);
        }
        if completion.sequence.0 == 0 {
            return Err(EffectCompletionError::InvalidSequence);
        }
        validate_completion_value(&completion.outcome)?;
    }
    canonical.sort_by_key(|completion| {
        (
            completion.source,
            completion.sequence,
            completion.delivery_key,
            completion.attempt,
        )
    });

    for pair in canonical.windows(2) {
        if pair[0].source == pair[1].source && pair[0].sequence == pair[1].sequence {
            return Err(EffectCompletionError::DuplicateSourceSequence {
                source: pair[0].source,
                sequence: pair[0].sequence,
            });
        }
    }
    Ok(canonical)
}

fn validate_completion_value(outcome: &EffectCompletionOutcome) -> EffectCompletionResult<()> {
    match outcome.value() {
        Value::Float(value) if !value.is_finite() => Err(EffectCompletionError::NonFiniteFloat),
        Value::Text(value) if value.len() > MAX_EFFECT_COMPLETION_TEXT_BYTES => {
            Err(EffectCompletionError::TextTooLarge {
                actual: value.len(),
                limit: MAX_EFFECT_COMPLETION_TEXT_BYTES,
            })
        }
        _ => Ok(()),
    }
}

fn validate_audit_correlation(
    audit: &EffectAuditLedger,
    completion: &EffectCompletion,
) -> EffectCompletionResult<()> {
    let mut prepared: Option<&EffectInDoubtAttempt> = None;
    let mut delivered = false;

    for record in audit.records() {
        match &record.event {
            EffectAuditEvent::AttemptPrepared(value) if value.attempt == completion.attempt => {
                prepared = Some(value);
            }
            EffectAuditEvent::AttemptDelivered {
                attempt, intent, ..
            } if *attempt == completion.attempt => {
                if *intent != completion.delivery_key.intent {
                    return Err(EffectCompletionError::AttemptIntentMismatch {
                        attempt: completion.attempt,
                    });
                }
                delivered = true;
            }
            EffectAuditEvent::InDoubtAssumedDelivered {
                attempt, intent, ..
            } if *attempt == completion.attempt => {
                if *intent != completion.delivery_key.intent {
                    return Err(EffectCompletionError::AttemptIntentMismatch {
                        attempt: completion.attempt,
                    });
                }
                delivered = true;
            }
            _ => {}
        }
    }

    let prepared = prepared.ok_or(EffectCompletionError::UnknownAttempt(completion.attempt))?;
    if prepared.delivery_key != completion.delivery_key {
        return Err(EffectCompletionError::DeliveryKeyMismatch {
            attempt: completion.attempt,
            expected: prepared.delivery_key,
            actual: completion.delivery_key,
        });
    }
    if prepared.request.id != completion.delivery_key.intent {
        return Err(EffectCompletionError::AttemptIntentMismatch {
            attempt: completion.attempt,
        });
    }
    if !delivered {
        return Err(EffectCompletionError::AttemptNotDelivered(
            completion.attempt,
        ));
    }
    Ok(())
}

fn project_value(
    projection: EffectCompletionProjectionValue,
    completion: &EffectCompletion,
) -> EffectCompletionResult<Value> {
    match projection {
        EffectCompletionProjectionValue::OutcomeValue => Ok(completion.outcome.value().clone()),
        EffectCompletionProjectionValue::Succeeded => {
            Ok(Value::Bool(completion.outcome.succeeded()))
        }
        EffectCompletionProjectionValue::IntentId => project_u64(completion.delivery_key.intent.0),
        EffectCompletionProjectionValue::AttemptId => project_u64(completion.attempt.0),
        EffectCompletionProjectionValue::SourceSequence => project_u64(completion.sequence.0),
    }
}

fn project_u64(value: u64) -> EffectCompletionResult<Value> {
    let value = i64::try_from(value)
        .map_err(|_| EffectCompletionError::IntegerProjectionOverflow(value))?;
    Ok(Value::Int(value))
}

pub(crate) fn canonical_report_bytes(
    report: &EffectCompletionBatchReport,
) -> EffectCompletionResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"NORDOI-EFFECT-COMPLETION-CAUSES-1.0");
    push_len(&mut bytes, report.applications.len())?;
    for application in &report.applications {
        bytes.extend_from_slice(&application.source.0.to_le_bytes());
        bytes.extend_from_slice(&application.sequence.0.to_le_bytes());
        bytes.extend_from_slice(&application.delivery_key.namespace.0);
        bytes.extend_from_slice(&application.delivery_key.intent.0.to_le_bytes());
        bytes.extend_from_slice(&application.attempt.0.to_le_bytes());
        match &application.outcome {
            EffectCompletionOutcome::Success(value) => {
                bytes.push(0x01);
                push_value(&mut bytes, value)?;
            }
            EffectCompletionOutcome::Failure(value) => {
                bytes.push(0x02);
                push_value(&mut bytes, value)?;
            }
        }
        push_len(&mut bytes, application.writes.len())?;
        for write in &application.writes {
            bytes.extend_from_slice(&write.domain.0.to_le_bytes());
            bytes.extend_from_slice(&write.atom.0.to_le_bytes());
            push_value(&mut bytes, &write.value)?;
        }
    }
    Ok(bytes)
}

fn push_len(bytes: &mut Vec<u8>, len: usize) -> EffectCompletionResult<()> {
    let len = u64::try_from(len).map_err(|_| EffectCompletionError::CanonicalLengthOverflow)?;
    bytes.extend_from_slice(&len.to_le_bytes());
    Ok(())
}

fn push_value(bytes: &mut Vec<u8>, value: &Value) -> EffectCompletionResult<()> {
    match value {
        Value::Null => bytes.push(0x00),
        Value::Bool(false) => bytes.push(0x01),
        Value::Bool(true) => bytes.push(0x02),
        Value::Int(value) => {
            bytes.push(0x03);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Value::Float(value) => {
            if !value.is_finite() {
                return Err(EffectCompletionError::NonFiniteFloat);
            }
            bytes.push(0x04);
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        Value::Text(value) => {
            if value.len() > MAX_EFFECT_COMPLETION_TEXT_BYTES {
                return Err(EffectCompletionError::TextTooLarge {
                    actual: value.len(),
                    limit: MAX_EFFECT_COMPLETION_TEXT_BYTES,
                });
            }
            bytes.push(0x05);
            push_len(bytes, value.len())?;
            bytes.extend_from_slice(value.as_bytes());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        effect::Effect,
        effect_audit::{
            EffectAttemptId, EffectAuditEvent, EffectAuditLedger, EffectInDoubtAttempt,
        },
        effect_dispatch::{
            EffectDeliveryFence, EffectDeliveryKey, EffectDeliveryNamespace, EffectIntentId,
            QueuedEffectIntent,
        },
        effect_retry::EffectRetryTick,
        kernel::AtomicKernel,
        reaction::{EffectIntent, ReactionId},
        value::Value,
    };

    use super::*;

    const SOURCE: EffectCompletionSourceId = EffectCompletionSourceId(9);

    fn namespace(seed: u8) -> EffectDeliveryNamespace {
        EffectDeliveryNamespace::new([seed; 16])
    }

    fn request(intent: u64) -> QueuedEffectIntent {
        QueuedEffectIntent {
            id: EffectIntentId(intent),
            cycle: 1,
            ordinal: 0,
            intent: EffectIntent {
                reaction: ReactionId(1),
                action_name: "completion-test".into(),
                effect: Effect::Network("api.example.test".into()),
            },
        }
    }

    fn delivered_ledger(
        intent: u64,
        attempt: u64,
        ns: EffectDeliveryNamespace,
    ) -> EffectAuditLedger {
        let mut audit = EffectAuditLedger::new();
        let key = EffectDeliveryKey::new(ns, EffectIntentId(intent));
        audit
            .append(EffectAuditEvent::AttemptPrepared(EffectInDoubtAttempt {
                attempt: EffectAttemptId(attempt),
                request: request(intent),
                delivery_key: key,
                delivery_fence: EffectDeliveryFence(1),
                prepared_tick: EffectRetryTick(1),
                failed_attempts_before: 0,
            }))
            .unwrap();
        audit
            .append(EffectAuditEvent::AttemptDelivered {
                attempt: EffectAttemptId(attempt),
                intent: EffectIntentId(intent),
                backend_reference: Some("remote-ok".into()),
            })
            .unwrap();
        audit
    }

    fn assumed_delivered_ledger(
        intent: u64,
        attempt: u64,
        ns: EffectDeliveryNamespace,
    ) -> EffectAuditLedger {
        let mut audit = EffectAuditLedger::new();
        let key = EffectDeliveryKey::new(ns, EffectIntentId(intent));
        audit
            .append(EffectAuditEvent::AttemptPrepared(EffectInDoubtAttempt {
                attempt: EffectAttemptId(attempt),
                request: request(intent),
                delivery_key: key,
                delivery_fence: EffectDeliveryFence(1),
                prepared_tick: EffectRetryTick(1),
                failed_attempts_before: 0,
            }))
            .unwrap();
        audit
            .append(EffectAuditEvent::InDoubtAssumedDelivered {
                attempt: EffectAttemptId(attempt),
                intent: EffectIntentId(intent),
                resolution_tick: EffectRetryTick(2),
                backend_reference: None,
            })
            .unwrap();
        audit
    }

    fn retry_ledger(intent: u64, attempt: u64, ns: EffectDeliveryNamespace) -> EffectAuditLedger {
        let mut audit = EffectAuditLedger::new();
        let key = EffectDeliveryKey::new(ns, EffectIntentId(intent));
        audit
            .append(EffectAuditEvent::AttemptPrepared(EffectInDoubtAttempt {
                attempt: EffectAttemptId(attempt),
                request: request(intent),
                delivery_key: key,
                delivery_fence: EffectDeliveryFence(1),
                prepared_tick: EffectRetryTick(1),
                failed_attempts_before: 0,
            }))
            .unwrap();
        audit
            .append(EffectAuditEvent::AttemptRetryScheduled {
                attempt: EffectAttemptId(attempt),
                intent: EffectIntentId(intent),
                failed_attempts: 1,
                next_eligible_tick: EffectRetryTick(5),
                error: "temporary".into(),
            })
            .unwrap();
        audit
    }

    fn completion(
        sequence: u64,
        intent: u64,
        attempt: u64,
        ns: EffectDeliveryNamespace,
        outcome: EffectCompletionOutcome,
    ) -> EffectCompletion {
        EffectCompletion::new(
            SOURCE,
            EffectCompletionSequence(sequence),
            EffectDeliveryKey::new(ns, EffectIntentId(intent)),
            EffectAttemptId(attempt),
            outcome,
        )
    }

    fn ready_core(
        kernel: &AtomicKernel,
        atom: AtomId,
        ns: EffectDeliveryNamespace,
        projection: EffectCompletionProjectionValue,
    ) -> AtomicEffectCompletionCore {
        let mut core = AtomicEffectCompletionCore::new();
        core.authority_mut().grant(SOURCE, ns).unwrap();
        core.register_projection(
            kernel,
            EffectCompletionProjection::new(SOURCE, ns, kernel.root_domain(), atom, projection),
        )
        .unwrap();
        core
    }

    #[test]
    fn source_requires_explicit_authority() {
        let ns = namespace(1);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = AtomicEffectCompletionCore::new();
        core.register_projection(
            &kernel,
            EffectCompletionProjection::new(
                SOURCE,
                ns,
                kernel.root_domain(),
                atom,
                EffectCompletionProjectionValue::OutcomeValue,
            ),
        )
        .unwrap();
        let batch = EffectCompletionBatch::new(vec![completion(
            1,
            1,
            1,
            ns,
            EffectCompletionOutcome::success("ok"),
        )]);
        assert!(matches!(
            core.apply_batch(&mut kernel, &audit, &batch),
            Err(EffectCompletionError::UnauthorizedSource { .. })
        ));
    }

    #[test]
    fn grant_and_revoke_are_exact_to_namespace() {
        let mut core = AtomicEffectCompletionCore::new();
        let first = namespace(2);
        let second = namespace(3);
        assert!(core.authority_mut().grant(SOURCE, first).unwrap());
        assert!(core.authority().allows(SOURCE, first));
        assert!(!core.authority().allows(SOURCE, second));
        assert!(core.authority_mut().revoke(SOURCE, first));
        assert!(!core.authority().allows(SOURCE, first));
    }

    #[test]
    fn zero_source_is_rejected() {
        let mut core = AtomicEffectCompletionCore::new();
        assert_eq!(
            core.authority_mut()
                .grant(EffectCompletionSourceId(0), namespace(1)),
            Err(EffectCompletionError::InvalidSourceId)
        );
    }

    #[test]
    fn zero_sequence_is_rejected() {
        let ns = namespace(4);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let batch = EffectCompletionBatch::new(vec![completion(
            0,
            1,
            1,
            ns,
            EffectCompletionOutcome::success("ok"),
        )]);
        assert_eq!(
            core.apply_batch(&mut kernel, &audit, &batch),
            Err(EffectCompletionError::InvalidSequence)
        );
    }

    #[test]
    fn duplicate_source_sequence_inside_batch_is_rejected() {
        let ns = namespace(5);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let batch = EffectCompletionBatch::new(vec![
            completion(1, 1, 1, ns, EffectCompletionOutcome::success("a")),
            completion(1, 1, 1, ns, EffectCompletionOutcome::success("b")),
        ]);
        assert!(matches!(
            core.apply_batch(&mut kernel, &audit, &batch),
            Err(EffectCompletionError::DuplicateSourceSequence { .. })
        ));
    }

    #[test]
    fn source_sequence_must_move_forward_across_batches() {
        let ns = namespace(6);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                2,
                1,
                1,
                ns,
                EffectCompletionOutcome::success("first"),
            )]),
        )
        .unwrap();
        let result = core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                1,
                ns,
                EffectCompletionOutcome::success("older"),
            )]),
        );
        assert!(matches!(
            result,
            Err(EffectCompletionError::NonMonotonicSequence { .. })
                | Err(EffectCompletionError::DeliveryAlreadyCompleted(_))
        ));
    }

    #[test]
    fn batch_order_is_canonical_by_source_then_sequence() {
        let ns = namespace(7);
        let mut audit = delivered_ledger(1, 1, ns);
        audit
            .append(EffectAuditEvent::AttemptPrepared(EffectInDoubtAttempt {
                attempt: EffectAttemptId(2),
                request: request(2),
                delivery_key: EffectDeliveryKey::new(ns, EffectIntentId(2)),
                delivery_fence: EffectDeliveryFence(1),
                prepared_tick: EffectRetryTick(2),
                failed_attempts_before: 0,
            }))
            .unwrap();
        audit
            .append(EffectAuditEvent::AttemptDelivered {
                attempt: EffectAttemptId(2),
                intent: EffectIntentId(2),
                backend_reference: None,
            })
            .unwrap();
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let report = core
            .apply_batch(
                &mut kernel,
                &audit,
                &EffectCompletionBatch::new(vec![
                    completion(2, 2, 2, ns, EffectCompletionOutcome::success("second")),
                    completion(1, 1, 1, ns, EffectCompletionOutcome::success("first")),
                ]),
            )
            .unwrap();
        assert_eq!(report.applications[0].sequence, EffectCompletionSequence(1));
        assert_eq!(report.applications[1].sequence, EffectCompletionSequence(2));
        assert_eq!(kernel.get(atom).unwrap(), &Value::Text("second".into()));
    }

    #[test]
    fn delivery_key_can_complete_only_once() {
        let ns = namespace(8);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let first = completion(1, 1, 1, ns, EffectCompletionOutcome::success("ok"));
        core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![first]),
        )
        .unwrap();
        let second = completion(2, 1, 1, ns, EffectCompletionOutcome::success("again"));
        assert!(matches!(
            core.apply_batch(
                &mut kernel,
                &audit,
                &EffectCompletionBatch::new(vec![second])
            ),
            Err(EffectCompletionError::DeliveryAlreadyCompleted(_))
        ));
    }

    #[test]
    fn unknown_attempt_is_rejected() {
        let ns = namespace(9);
        let audit = EffectAuditLedger::new();
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let result = core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                99,
                ns,
                EffectCompletionOutcome::success("x"),
            )]),
        );
        assert_eq!(
            result,
            Err(EffectCompletionError::UnknownAttempt(EffectAttemptId(99)))
        );
    }

    #[test]
    fn mismatched_delivery_key_is_rejected() {
        let ns = namespace(10);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        core.authority_mut().grant(SOURCE, namespace(11)).unwrap();
        core.register_projection(
            &kernel,
            EffectCompletionProjection::new(
                SOURCE,
                namespace(11),
                kernel.root_domain(),
                atom,
                EffectCompletionProjectionValue::OutcomeValue,
            ),
        )
        .unwrap();
        let result = core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                1,
                namespace(11),
                EffectCompletionOutcome::success("x"),
            )]),
        );
        assert!(matches!(
            result,
            Err(EffectCompletionError::DeliveryKeyMismatch { .. })
        ));
    }

    #[test]
    fn retry_scheduled_attempt_cannot_reenter_semantics() {
        let ns = namespace(12);
        let audit = retry_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        assert_eq!(
            core.apply_batch(
                &mut kernel,
                &audit,
                &EffectCompletionBatch::new(vec![completion(
                    1,
                    1,
                    1,
                    ns,
                    EffectCompletionOutcome::success("x"),
                )]),
            ),
            Err(EffectCompletionError::AttemptNotDelivered(EffectAttemptId(
                1
            )))
        );
    }

    #[test]
    fn assumed_delivered_attempt_can_reenter_semantics() {
        let ns = namespace(13);
        let audit = assumed_delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                1,
                ns,
                EffectCompletionOutcome::success("reconciled"),
            )]),
        )
        .unwrap();
        assert_eq!(kernel.get(atom).unwrap(), &Value::Text("reconciled".into()));
    }

    #[test]
    fn route_without_projection_is_rejected() {
        let ns = namespace(14);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let mut core = AtomicEffectCompletionCore::new();
        core.authority_mut().grant(SOURCE, ns).unwrap();
        let result = core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                1,
                ns,
                EffectCompletionOutcome::success("x"),
            )]),
        );
        assert!(matches!(
            result,
            Err(EffectCompletionError::NoProjection { .. })
        ));
    }

    #[test]
    fn duplicate_projection_for_same_route_and_atom_is_rejected() {
        let ns = namespace(15);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let projection = EffectCompletionProjection::new(
            SOURCE,
            ns,
            kernel.root_domain(),
            atom,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let mut core = AtomicEffectCompletionCore::new();
        core.register_projection(&kernel, projection.clone())
            .unwrap();
        assert!(matches!(
            core.register_projection(&kernel, projection),
            Err(EffectCompletionError::DuplicateProjection { .. })
        ));
    }

    #[test]
    fn projection_registration_respects_current_atom_owner() {
        let ns = namespace(16);
        let mut kernel = AtomicKernel::new();
        let owner = kernel.create_domain("owner").unwrap();
        let atom = kernel.create_atom_owned(owner, Value::Null).unwrap();
        let mut core = AtomicEffectCompletionCore::new();
        assert!(matches!(
            core.register_projection(
                &kernel,
                EffectCompletionProjection::new(
                    SOURCE,
                    ns,
                    kernel.root_domain(),
                    atom,
                    EffectCompletionProjectionValue::OutcomeValue,
                ),
            ),
            Err(EffectCompletionError::Atomic(_))
        ));
    }

    #[test]
    fn outcome_value_projection_writes_success_payload() {
        let ns = namespace(17);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let report = core
            .apply_batch(
                &mut kernel,
                &audit,
                &EffectCompletionBatch::new(vec![completion(
                    1,
                    1,
                    1,
                    ns,
                    EffectCompletionOutcome::success("done"),
                )]),
            )
            .unwrap();
        assert_eq!(report.changed_atoms, 1);
        assert_eq!(kernel.get(atom).unwrap(), &Value::Text("done".into()));
    }

    #[test]
    fn outcome_value_projection_writes_failure_payload() {
        let ns = namespace(18);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                1,
                ns,
                EffectCompletionOutcome::failure("remote-failed"),
            )]),
        )
        .unwrap();
        assert_eq!(
            kernel.get(atom).unwrap(),
            &Value::Text("remote-failed".into())
        );
    }

    #[test]
    fn succeeded_projection_distinguishes_outcome_kind() {
        let ns = namespace(19);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::Succeeded,
        );
        core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                1,
                ns,
                EffectCompletionOutcome::failure("x"),
            )]),
        )
        .unwrap();
        assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(false));
    }

    #[test]
    fn intent_id_projection_is_exact() {
        let ns = namespace(20);
        let audit = delivered_ledger(44, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(&kernel, atom, ns, EffectCompletionProjectionValue::IntentId);
        core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                44,
                1,
                ns,
                EffectCompletionOutcome::success(Value::Null),
            )]),
        )
        .unwrap();
        assert_eq!(kernel.get(atom).unwrap(), &Value::Int(44));
    }

    #[test]
    fn attempt_id_projection_is_exact() {
        let ns = namespace(21);
        let audit = delivered_ledger(1, 77, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::AttemptId,
        );
        core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                77,
                ns,
                EffectCompletionOutcome::success(Value::Null),
            )]),
        )
        .unwrap();
        assert_eq!(kernel.get(atom).unwrap(), &Value::Int(77));
    }

    #[test]
    fn source_sequence_projection_is_exact() {
        let ns = namespace(22);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::SourceSequence,
        );
        core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                55,
                1,
                1,
                ns,
                EffectCompletionOutcome::success(Value::Null),
            )]),
        )
        .unwrap();
        assert_eq!(kernel.get(atom).unwrap(), &Value::Int(55));
    }

    #[test]
    fn batch_failure_rolls_back_earlier_completion_and_nam_write() {
        let ns = namespace(23);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Text("before".into()));
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let batch = EffectCompletionBatch::new(vec![
            completion(1, 1, 1, ns, EffectCompletionOutcome::success("after")),
            completion(2, 99, 99, ns, EffectCompletionOutcome::success("bad")),
        ]);
        assert!(core.apply_batch(&mut kernel, &audit, &batch).is_err());
        assert_eq!(kernel.get(atom).unwrap(), &Value::Text("before".into()));
        assert_eq!(core.last_sequence(SOURCE), None);
        assert!(!core.has_completed(EffectDeliveryKey::new(ns, EffectIntentId(1))));
    }

    #[test]
    fn multiple_domains_publish_atomically_on_candidate_kernel() {
        let ns = namespace(24);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let root_atom = kernel.create_atom(Value::Null);
        let domain = kernel.create_domain("secondary").unwrap();
        let domain_atom = kernel.create_atom_owned(domain, Value::Null).unwrap();
        let mut core = AtomicEffectCompletionCore::new();
        core.authority_mut().grant(SOURCE, ns).unwrap();
        core.register_projection(
            &kernel,
            EffectCompletionProjection::new(
                SOURCE,
                ns,
                kernel.root_domain(),
                root_atom,
                EffectCompletionProjectionValue::OutcomeValue,
            ),
        )
        .unwrap();
        core.register_projection(
            &kernel,
            EffectCompletionProjection::new(
                SOURCE,
                ns,
                domain,
                domain_atom,
                EffectCompletionProjectionValue::Succeeded,
            ),
        )
        .unwrap();
        let report = core
            .apply_batch(
                &mut kernel,
                &audit,
                &EffectCompletionBatch::new(vec![completion(
                    1,
                    1,
                    1,
                    ns,
                    EffectCompletionOutcome::success("ok"),
                )]),
            )
            .unwrap();
        assert_eq!(report.transactions, 2);
        assert_eq!(kernel.get(root_atom).unwrap(), &Value::Text("ok".into()));
        assert_eq!(kernel.get(domain_atom).unwrap(), &Value::Bool(true));
    }

    #[test]
    fn identical_projection_is_zero_nam_change_but_completion_is_consumed() {
        let ns = namespace(25);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Text("same".into()));
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let key = EffectDeliveryKey::new(ns, EffectIntentId(1));
        let report = core
            .apply_batch(
                &mut kernel,
                &audit,
                &EffectCompletionBatch::new(vec![completion(
                    1,
                    1,
                    1,
                    ns,
                    EffectCompletionOutcome::success("same"),
                )]),
            )
            .unwrap();
        assert_eq!(report.changed_atoms, 0);
        assert!(core.has_completed(key));
    }

    #[test]
    fn non_finite_float_is_rejected() {
        let ns = namespace(26);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let result = core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                1,
                ns,
                EffectCompletionOutcome::success(Value::Float(f64::NAN)),
            )]),
        );
        assert_eq!(result, Err(EffectCompletionError::NonFiniteFloat));
    }

    #[test]
    fn oversized_text_is_rejected() {
        let ns = namespace(27);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let result = core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                1,
                1,
                ns,
                EffectCompletionOutcome::success("x".repeat(MAX_EFFECT_COMPLETION_TEXT_BYTES + 1)),
            )]),
        );
        assert!(matches!(
            result,
            Err(EffectCompletionError::TextTooLarge { .. })
        ));
    }

    #[test]
    fn integer_projection_overflow_is_rejected() {
        let ns = namespace(28);
        let intent = (i64::MAX as u64) + 1;
        let audit = delivered_ledger(intent, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(&kernel, atom, ns, EffectCompletionProjectionValue::IntentId);
        let result = core.apply_batch(
            &mut kernel,
            &audit,
            &EffectCompletionBatch::new(vec![completion(
                1,
                intent,
                1,
                ns,
                EffectCompletionOutcome::success(Value::Null),
            )]),
        );
        assert_eq!(
            result,
            Err(EffectCompletionError::IntegerProjectionOverflow(intent))
        );
    }

    #[test]
    fn oversized_batch_is_rejected_before_any_work() {
        let ns = namespace(29);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let item = completion(1, 1, 1, ns, EffectCompletionOutcome::success("x"));
        let batch = EffectCompletionBatch::new(vec![item; MAX_EFFECT_COMPLETION_BATCH + 1]);
        assert!(matches!(
            core.apply_batch(&mut kernel, &audit, &batch),
            Err(EffectCompletionError::BatchTooLarge { .. })
        ));
        assert_eq!(core.last_sequence(SOURCE), None);
    }

    #[test]
    fn unregister_projection_removes_route_when_last_target_is_removed() {
        let ns = namespace(30);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(Value::Null);
        let mut core = ready_core(
            &kernel,
            atom,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        assert!(core.unregister_projection(SOURCE, ns, atom));
        assert!(!core.unregister_projection(SOURCE, ns, atom));
    }

    #[test]
    fn canonical_report_bytes_are_stable_for_equal_semantic_application() {
        let ns = namespace(31);
        let audit = delivered_ledger(1, 1, ns);
        let mut kernel_a = AtomicKernel::new();
        let atom_a = kernel_a.create_atom(Value::Null);
        let mut core_a = ready_core(
            &kernel_a,
            atom_a,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let report_a = core_a
            .apply_batch(
                &mut kernel_a,
                &audit,
                &EffectCompletionBatch::new(vec![completion(
                    1,
                    1,
                    1,
                    ns,
                    EffectCompletionOutcome::success("ok"),
                )]),
            )
            .unwrap();

        let mut kernel_b = AtomicKernel::new();
        let atom_b = kernel_b.create_atom(Value::Null);
        let mut core_b = ready_core(
            &kernel_b,
            atom_b,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let report_b = core_b
            .apply_batch(
                &mut kernel_b,
                &audit,
                &EffectCompletionBatch::new(vec![completion(
                    1,
                    1,
                    1,
                    ns,
                    EffectCompletionOutcome::success("ok"),
                )]),
            )
            .unwrap();

        assert_eq!(
            canonical_report_bytes(&report_a).unwrap(),
            canonical_report_bytes(&report_b).unwrap()
        );
    }

    #[test]
    fn different_outcome_changes_canonical_replay_bytes() {
        let ns = namespace(32);
        let audit = delivered_ledger(1, 1, ns);

        let mut kernel_a = AtomicKernel::new();
        let atom_a = kernel_a.create_atom(Value::Null);
        let mut core_a = ready_core(
            &kernel_a,
            atom_a,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let report_a = core_a
            .apply_batch(
                &mut kernel_a,
                &audit,
                &EffectCompletionBatch::new(vec![completion(
                    1,
                    1,
                    1,
                    ns,
                    EffectCompletionOutcome::success("ok"),
                )]),
            )
            .unwrap();

        let mut kernel_b = AtomicKernel::new();
        let atom_b = kernel_b.create_atom(Value::Null);
        let mut core_b = ready_core(
            &kernel_b,
            atom_b,
            ns,
            EffectCompletionProjectionValue::OutcomeValue,
        );
        let report_b = core_b
            .apply_batch(
                &mut kernel_b,
                &audit,
                &EffectCompletionBatch::new(vec![completion(
                    1,
                    1,
                    1,
                    ns,
                    EffectCompletionOutcome::failure("ok"),
                )]),
            )
            .unwrap();

        assert_ne!(
            canonical_report_bytes(&report_a).unwrap(),
            canonical_report_bytes(&report_b).unwrap()
        );
    }
}
