# Effect Audit Intelligence 0.1

K1.10 research question: how should NORDOI represent the irreducible ambiguity between an external
side effect and local durable acknowledgement?

The selected model uses a write-ahead attempt record rather than claiming atomicity across unrelated
systems. This is analogous to transaction/outbox reasoning but deliberately avoids describing the
remote side as participating in a local transaction.

Key conclusions:

- persist intention-to-attempt before irreversible I/O;
- distinguish semantic request identity from attempt identity and writer epoch;
- retain stable idempotency identity across retries;
- after crash in the external-I/O window, expose uncertainty rather than infer success/failure;
- require explicit operator/host resolution before redispatch;
- hash-chain canonical audit records so mutation is detectable;
- keep audit outside deterministic program replay meaning;
- do not call hash chaining a digital signature or universal non-repudiation;
- do not claim universal exactly-once delivery without destination cooperation.
