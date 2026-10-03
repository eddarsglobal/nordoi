# Effect Persistence Intelligence 0.1

## Research question

How can NORDOI survive process restart with pending external effect intents while
preserving deterministic program meaning, least authority, retry safety and the rule
against false exactly-once claims?

## Findings applied to K1.7

### 1. Durable commit must be an explicit contract

Write-ahead and transactional storage systems distinguish logical commit from the
physical guarantees of the storage configuration. K1.7 therefore makes durability a
host store contract instead of silently assuming that writing bytes means they reached
non-volatile media.

### 2. Idempotency requires a stable caller identity

Retry-safe APIs commonly rely on a caller-provided request token whose identity is
reused across retries. K1.7 exposes a stable `EffectDeliveryKey` so a cooperating
backend/destination can deduplicate the same semantic request after uncertainty or
restart.

### 3. Idempotency and exactly-once are different claims

A stable client token is useful only when the receiving system participates in the
protocol. K1.7 therefore improves retry semantics without claiming universal
exactly-once execution.

### 4. Persistence authority must remain outside canonical program bytes

A NORDOI program should not be able to serialize its own filesystem/database authority.
Journal stores remain host-injected. NAIR 0.5 is unchanged.

### 5. Recovery input is untrusted input

Persistent bytes can be truncated, corrupted or intentionally hostile. K1.7 validates
version, bounds, identities, UTF-8, ordering and checksum before installation, and uses
hard decoder budgets before allocation.

### 6. Acknowledgement has its own failure window

External execution can complete while local acknowledgement persistence fails. NORDOI
cannot erase that ambiguity. It keeps the intent pending and retries with the same
stable delivery key, enabling destination-level deduplication where supported.

## Reference systems studied

- SQLite write-ahead logging and synchronous durability semantics;
- AWS Builders' Library guidance on retry-safe idempotent APIs and client request IDs;
- Stripe idempotency-key retry semantics;
- transactional-outbox principles separating durable intent from external delivery.

These references inform the architecture; NORDOI does not copy their APIs or make any
one storage/database system mandatory.
