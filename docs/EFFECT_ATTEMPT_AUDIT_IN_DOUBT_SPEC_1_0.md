# NORDOI Effect Attempt Audit & In-Doubt Recovery Specification 1.0

Status: Certified by NORDOI K1.10; inherited unchanged by K1.11.

## 1. Scope

This specification defines durable audit semantics for external-effect delivery attempts layered on
top of certified K1.8 fencing and K1.9 retry/dead-letter behavior. It does not change NAIR program
meaning and does not claim distributed exactly-once execution.

## 2. Identities

`EffectIntentId` identifies the semantic queued effect. `EffectDeliveryKey` is stable across retry
and takeover. `EffectDeliveryFence` identifies the current writer epoch. `EffectAttemptId`
identifies one prepared external execution attempt and is monotonically allocated without reuse.

## 3. Prepare-before-I/O rule

A governed audited dispatcher MUST durably commit `AttemptPrepared` before invoking the external
backend. Failure to persist preparation MUST create zero backend work.

## 4. Terminal rule

After backend execution, a private candidate MUST append exactly one terminal event and update
outbox/retry/dead-letter state. Local publication occurs only after a fenced durable commit.

## 5. In-doubt rule

If a prepared attempt lacks a terminal record, it is in-doubt. Automatic redispatch MUST stop. The
host MUST explicitly reconcile and either mark it delivered or authorize retry.

## 6. Audit hash chain

Records are canonically ordered by `EffectAuditSequence`. Each record SHA-256 hashes a domain
separator, sequence, previous record hash and canonical event bytes. The first previous hash is
all-zero. Checkpoint bytes are independently SHA-256 protected.

Hash chaining is tamper-evidence, not writer authentication. A future signing/attestation layer may
bind roots to identities or hardware keys.

## 7. Canonical checkpoint

K1.10 uses magic `NDEFXA01`, format 1.0. It embeds the complete K1.9 retry checkpoint and the audit
record stream. K1.9 `NDEFXR01` and certified K1.7/K1.8 `NDEFXJ01` checkpoints are accepted as legacy
inputs with an empty audit ledger; no historical attempts are invented.

Maximum checkpoint size is 256 MiB and maximum audit record count is 262,144. Existing 1 MiB journal
string bounds remain enforced for audit error/receipt text.

## 8. Recovery

Recovery validates outer digest, embedded retry checkpoint, audit sequence continuity, previous-hash
links, per-record hashes and event-state transitions. An unresolved prepared event MUST reference a
currently pending outbox intent.

## 9. Resolution

`assume delivered` removes the pending intent without invoking the backend and appends a resolution
record. `retry authorized` preserves the pending intent and stable delivery key, closes the in-doubt
attempt and permits a future explicitly scheduled attempt.

## 10. Replay and NAIR

Audit state is delivery metadata and SHALL NOT change event-loop replay identity. K1.10 introduces
no new NAIR instruction; NAIR remains 0.5.
