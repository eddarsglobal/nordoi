use super::{
    hash::sha256, EffectAttemptId, EffectAuditError, EffectAuditEvent, EffectAuditHash,
    EffectAuditRecord, EffectAuditResult, EffectAuditSequence, EffectInDoubtAttempt,
};

const EVENT_HASH_DOMAIN: &[u8] = b"NORDOI-EFFECT-AUDIT-EVENT-1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAuditLedger {
    records: Vec<EffectAuditRecord>,
    next_attempt_id: u64,
    in_doubt: Option<EffectInDoubtAttempt>,
}

impl Default for EffectAuditLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectAuditLedger {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_attempt_id: 1,
            in_doubt: None,
        }
    }

    pub fn records(&self) -> &[EffectAuditRecord] {
        &self.records
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    pub fn next_attempt_id(&self) -> u64 {
        self.next_attempt_id
    }
    pub fn in_doubt(&self) -> Option<&EffectInDoubtAttempt> {
        self.in_doubt.as_ref()
    }
    pub fn root_hash(&self) -> EffectAuditHash {
        self.records
            .last()
            .map_or(EffectAuditHash::ZERO, |r| r.hash)
    }

    pub(crate) fn allocate_attempt(&mut self) -> EffectAuditResult<EffectAttemptId> {
        let id = self.next_attempt_id;
        if id == 0 {
            return Err(EffectAuditError::AttemptIdExhausted);
        }
        self.next_attempt_id = id
            .checked_add(1)
            .ok_or(EffectAuditError::AttemptIdExhausted)?;
        Ok(EffectAttemptId(id))
    }

    pub(crate) fn append(
        &mut self,
        event: EffectAuditEvent,
    ) -> EffectAuditResult<EffectAuditRecord> {
        self.validate_transition(&event)?;
        let sequence_value = u64::try_from(self.records.len())
            .map_err(|_| EffectAuditError::AuditSequenceExhausted)?
            .checked_add(1)
            .ok_or(EffectAuditError::AuditSequenceExhausted)?;
        let sequence = EffectAuditSequence(sequence_value);
        let previous_hash = self.root_hash();
        let hash = hash_event(sequence, previous_hash, &event);
        let record = EffectAuditRecord {
            sequence,
            previous_hash,
            event: event.clone(),
            hash,
        };
        self.apply_transition(&event);
        self.records.push(record.clone());
        Ok(record)
    }

    pub(crate) fn from_records(
        next_attempt_id: u64,
        records: Vec<EffectAuditRecord>,
    ) -> EffectAuditResult<Self> {
        if next_attempt_id == 0 {
            return Err(EffectAuditError::AttemptIdExhausted);
        }
        let mut ledger = Self::new();
        ledger.next_attempt_id = next_attempt_id;
        let mut previous = EffectAuditHash::ZERO;
        for (index, record) in records.iter().enumerate() {
            let expected_sequence = EffectAuditSequence((index as u64) + 1);
            if record.sequence != expected_sequence {
                return Err(EffectAuditError::AuditSequenceMismatch {
                    expected: expected_sequence,
                    actual: record.sequence,
                });
            }
            if record.previous_hash != previous {
                return Err(EffectAuditError::AuditPreviousHashMismatch {
                    sequence: record.sequence,
                });
            }
            let expected_hash = hash_event(record.sequence, record.previous_hash, &record.event);
            if record.hash != expected_hash {
                return Err(EffectAuditError::AuditHashChainMismatch {
                    sequence: record.sequence,
                });
            }
            ledger.validate_transition(&record.event)?;
            ledger.apply_transition(&record.event);
            previous = record.hash;
        }
        if let Some(max_attempt) = records
            .iter()
            .filter_map(|record| attempt_id(&record.event))
            .map(|id| id.0)
            .max()
        {
            if next_attempt_id <= max_attempt {
                return Err(EffectAuditError::InvalidAuditEventState(
                    "next attempt id does not exceed recorded attempts".into(),
                ));
            }
        }
        ledger.records = records;
        Ok(ledger)
    }

    fn validate_transition(&self, event: &EffectAuditEvent) -> EffectAuditResult<()> {
        match event {
            EffectAuditEvent::AttemptPrepared(attempt) => {
                if let Some(open) = &self.in_doubt {
                    return Err(EffectAuditError::InDoubtAttemptExists(open.attempt));
                }
                if attempt.attempt.0 == 0
                    || attempt.request.id.0 == 0
                    || attempt.delivery_fence.0 == 0
                {
                    return Err(EffectAuditError::InvalidAuditEventState(
                        "prepared attempt contains zero identity".into(),
                    ));
                }
            }
            EffectAuditEvent::AttemptDelivered {
                attempt, intent, ..
            }
            | EffectAuditEvent::AttemptRetryScheduled {
                attempt, intent, ..
            }
            | EffectAuditEvent::AttemptDeadLettered {
                attempt, intent, ..
            }
            | EffectAuditEvent::InDoubtAssumedDelivered {
                attempt, intent, ..
            }
            | EffectAuditEvent::InDoubtRetryAuthorized {
                attempt, intent, ..
            } => {
                let open = self
                    .in_doubt
                    .as_ref()
                    .ok_or(EffectAuditError::NoInDoubtAttempt)?;
                if open.attempt != *attempt {
                    return Err(EffectAuditError::InvalidAuditEventState(
                        "terminal event attempt id does not match open attempt".into(),
                    ));
                }
                if open.request.id != *intent {
                    return Err(EffectAuditError::InDoubtIntentMismatch {
                        expected: open.request.id,
                        actual: *intent,
                    });
                }
            }
            EffectAuditEvent::DeadLetterRedriven { .. }
            | EffectAuditEvent::DeadLetterDiscarded { .. } => {
                if let Some(open) = &self.in_doubt {
                    return Err(EffectAuditError::InDoubtAttemptExists(open.attempt));
                }
            }
        }
        Ok(())
    }

    fn apply_transition(&mut self, event: &EffectAuditEvent) {
        match event {
            EffectAuditEvent::AttemptPrepared(attempt) => self.in_doubt = Some(attempt.clone()),
            EffectAuditEvent::AttemptDelivered { .. }
            | EffectAuditEvent::AttemptRetryScheduled { .. }
            | EffectAuditEvent::AttemptDeadLettered { .. }
            | EffectAuditEvent::InDoubtAssumedDelivered { .. }
            | EffectAuditEvent::InDoubtRetryAuthorized { .. } => self.in_doubt = None,
            EffectAuditEvent::DeadLetterRedriven { .. }
            | EffectAuditEvent::DeadLetterDiscarded { .. } => {}
        }
    }
}

fn attempt_id(event: &EffectAuditEvent) -> Option<EffectAttemptId> {
    match event {
        EffectAuditEvent::AttemptPrepared(value) => Some(value.attempt),
        EffectAuditEvent::AttemptDelivered { attempt, .. }
        | EffectAuditEvent::AttemptRetryScheduled { attempt, .. }
        | EffectAuditEvent::AttemptDeadLettered { attempt, .. }
        | EffectAuditEvent::InDoubtAssumedDelivered { attempt, .. }
        | EffectAuditEvent::InDoubtRetryAuthorized { attempt, .. } => Some(*attempt),
        EffectAuditEvent::DeadLetterRedriven { .. }
        | EffectAuditEvent::DeadLetterDiscarded { .. } => None,
    }
}

pub(crate) fn hash_event(
    sequence: EffectAuditSequence,
    previous: EffectAuditHash,
    event: &EffectAuditEvent,
) -> EffectAuditHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(EVENT_HASH_DOMAIN);
    bytes.extend_from_slice(&sequence.0.to_le_bytes());
    bytes.extend_from_slice(&previous.0);
    super::format::push_event(&mut bytes, event);
    EffectAuditHash(sha256(&bytes))
}
