# NORDOI Effect Completion Intelligence 0.1

## Problem

Outbound side effects are only half of an application semantics model. Real systems eventually
receive remote outcomes: payment confirmation, file-upload completion, AI inference result, network
response, job completion, device acknowledgement, or provider failure.

A naive runtime often handles these results in host callbacks that mutate application state directly.
That breaks NORDOI's authority, ownership, atomicity and replay model.

## K1.12 decision

Treat external results as governed causes rather than privileged callbacks.

The cause must be:

1. source-authorized;
2. correlated to a certified delivery attempt;
3. canonically ordered;
4. deduplicated at semantic delivery-key level;
5. projected through pre-authorized bindings;
6. committed through normal NAM transactions;
7. included in replay identity.

## Why delivery key and attempt are both required

`EffectDeliveryKey` answers: *which semantic external request is this?*

`EffectAttemptId` answers: *which physical delivery attempt produced or was reconciled with this
result?*

Using only the attempt loses stable request identity across retries. Using only the delivery key
loses the audited physical evidence that a valid delivery occurred. K1.12 requires both.

## Why only delivered/assumed-delivered attempts qualify

A retryable transport failure or dead-letter proves that NORDOI did not establish a successful
completion-producing delivery. Accepting a remote semantic result against such a record would let a
host bypass the certified delivery state machine.

`InDoubtAssumedDelivered` is accepted because K1.10 makes that ambiguity resolution an explicit,
durable host decision.

## Why source sequence and delivery-key dedup are separate

A monotonic source sequence prevents replay/reordering within one external source stream, but two
source messages could still refer to the same logical effect. Stable-key dedup prevents that second
semantic application.

Together they protect stream order and semantic uniqueness without claiming network exactly-once.

## Why projection precedes native completion reactions

K1.12 first certifies the invariant layer: validation, authority, audit correlation, transaction
projection and replay. Native NAIR completion triggers can then be designed on top of proven laws
instead of freezing surface syntax around an untested callback model.

## Future work

Possible later milestones include:

- native NAIR completion trigger semantics;
- canonical structured completion payloads beyond the current `Value` capsule;
- whole-runtime persistence of completion-consumption state + NAM;
- authenticated transport/source provenance;
- completion-driven effect chaining under explicit effect declarations;
- distributed/consensus-backed semantic cause ingestion.
