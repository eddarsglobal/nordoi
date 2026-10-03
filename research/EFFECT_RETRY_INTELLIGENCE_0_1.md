# Effect Retry Intelligence 0.1

Research note for NORDOI K1.9.

## External systems studied

### Amazon SQS

AWS documents dead-letter queues as a way to move repeatedly unprocessable messages out of the
active queue after a configured `maxReceiveCount`. AWS also warns that overly small thresholds can
move messages prematurely and that poison messages can distort queue behavior/metrics.

Reference:
https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-dead-letter-queues.html

### Google Cloud Pub/Sub

Pub/Sub exposes retry policy separately from dead-letter policy. It supports immediate retry or
exponential backoff, and dead-letter forwarding after a configured number of delivery attempts.
The retry mechanism is per-message rather than a single global delay.

References:
https://docs.cloud.google.com/pubsub/docs/subscription-retry-policy
https://docs.cloud.google.com/pubsub/docs/handling-failures

### Temporal

Temporal distinguishes retryable execution failures from deterministic workflow meaning and uses
retry policies/backoff around external activity-style work. Its documentation also emphasizes
idempotency because at-least-once execution can invoke external handlers multiple times.

References:
https://docs.temporal.io/tasks
https://docs.temporal.io/nexus/operations

## NORDOI conclusions

NORDOI adopts the strongest common ideas while preserving its own laws:

1. bounded attempts rather than infinite retries;
2. exponential backoff rather than hot looping;
3. dead-letter quarantine rather than silent deletion;
4. per-intent retry state rather than global queue suspension;
5. stable idempotency identity across retry and redrive;
6. explicit distinction between retryable and permanent backend failure;
7. persistence before publication of retry/dead-letter transitions;
8. no ambient clock dependency in the canonical core;
9. deterministic jitter derived from stable identity rather than runtime randomness;
10. retry policy persisted and recovery-checked to prevent silent drift.

NORDOI deliberately does not copy service-specific wall-clock defaults, approximate delivery counts,
or claims that retries alone produce exactly-once external effects.
