# NORDOI Constitutional Principles — K0.7

NORDOI is designed as a universal programming architecture for humans, AI/SI and machines.

## C1 — Universal Execution
NORDOI must never be confined to the Web. The Web is one target among others.

## C2 — Atomic Speed
The system should perform only the minimum necessary work for each observable effect.

## C3 — Atomic Security
Authority is denied by default and granted explicitly through narrow capabilities.

## C4 — Zero Legacy Debt
NORDOI must not preserve a weakness merely because older languages historically contain it.

## C5 — Impossible States First
When a dangerous program state can reasonably be made unrepresentable, NORDOI should make it unrepresentable rather than merely document it.

## C6 — No Work Without Effect
A computation with no useful state change or observable effect should create no downstream work.

## C7 — What You Do Not Use Must Cost Nothing
Unused subsystems should contribute no meaningful runtime cost.

## C8 — AI/SI Native, Not AI Dependent
AI/SI may generate, analyze, optimize and verify NORDOI programs, but ordinary compiled execution must not require an AI model.

## C9 — Simple Surface, Deep Engine
Complexity that can safely be absorbed by compiler/runtime/tooling should not be imposed on the developer.

## C10 — Canonical Semantics
Human source, AI generation, visual tooling and future frontends should converge on the same semantic model and NAIR.

## C11 — Declared Effects
A unit of code may not perform an effect it did not declare.

## C12 — Authority Is Not Intent
Possessing a capability does not authorize undeclared behavior. Both declaration and authority are required.

## C13 — Least Authority
Capabilities must be narrow, explicit and scoped as precisely as practical.

## C14 — Evidence Before Evolution
Changes to core semantics require tests, counterexamples, performance evidence and security analysis.

## C15 — Community Evolvable, Constitutionally Governed
The community may propose and build freely; core invariants may only evolve through evidence-backed governance.

## C16 — Master Law Supremacy
`laws/LAW_0001_NORDOI_MASTER_LAW.md` is a foundational design law. Kernel, NAIR, NAM, compiler, runtime, tooling and future language proposals must be evaluated against it.

## C17 — Verified Performance Supremacy
The founder's declaration — **"NORDOI est le plus leger langage au monde et le plus vite dans l'univer"** — is adopted as a permanent engineering mission. Claims of superiority must be demonstrated with reproducible evidence; correctness and security may not be sacrificed for benchmark results.

## C18 — Atomic Transactions
A multi-state operation must not expose partially committed state. Transaction failure before commit must leave unrelated staged state unchanged.

## C19 — Explicit Ownership Boundaries
Mutable state belongs to an explicit ownership domain. Mutation from another domain is forbidden unless ownership or authority is explicitly transferred/delegated by a future verified mechanism.

## C20 — Testing & Release Law
No NORDOI version is complete until its automated regression and invariant tests pass in the repository's GitHub CI test gate. A subsequent version must not be treated as started before the preceding version is green.

## C21 — IR Before Surface Lock-In
NORDOI must stabilize semantic invariants and NAIR contracts before freezing convenient surface syntax. Syntax serves semantics; semantics do not serve syntax.

## C22 — Validate Before Execute
Invalid NAIR must be rejected before it can mutate NAM state.

## C23 — Safe By Omission
A privileged operation that has not yet received a complete effect, authority, validation and execution contract must not be representable in canonical NAIR.

## C24 — Canonical Machine Meaning
The same valid NAIR program and initial NAM state must have one defined machine meaning. Canonical encoding must not depend on incidental map order, platform syntax or frontend origin.

## C25 — Unified Spatial Semantics
2D, 3D and future XR presentation must share one semantic render model wherever their underlying concepts are equivalent. Platform-specific APIs may not fragment NORDOI core semantics.

## C26 — Minimum Render Frontier
A render change must invalidate only the smallest correct set of render nodes. Duplicate invalidations must collapse before reaching a backend.

## C27 — Backend Independence
NORDOI render semantics must not depend on DOM, WebGPU, Metal, Vulkan, DirectX or another specific backend. Backends implement NORDOI semantics; they do not define them.

## C28 — Invalid Visual State Rejection
Render values known to be invalid, including non-finite transforms and out-of-range normalized opacity, must be rejected before entering canonical render state.

## C29 — Single Causal Frontier

State-to-render propagation SHALL consume NAM's canonical deduplicated atomic work frontier. NORDOI SHALL NOT introduce a second competing dependency engine for rendering.

## C30 — Zero Render Without State Effect

If a state operation produces no semantic change, the NAIR-to-render bridge SHALL produce no new render work. Transaction rollback SHALL likewise produce no render work.

## C31 — Canonical Slot Binding

A NAIR `AtomSlot` may reach rendering only through the runtime `AtomId` binding emitted by the exact validated NAIR execution that created it. Unknown or foreign slots SHALL be rejected before rendering.



## C32 — Native Render Meaning in NAIR

Backend-independent render creation, hierarchy, bindings and core visual mutation MAY be canonical NAIR semantics. Platform APIs SHALL remain backend implementation details.

## C33 — Render Slots Are Semantic Identities

A NAIR `RenderNodeSlot` SHALL be single assignment, SHALL exist before use, and SHALL resolve only to runtime render identities created by the exact validated execution.

## C34 — Explicit Frame Boundaries

`RENDER_FLUSH` SHALL define an explicit deterministic render-frame boundary. HALT SHALL emit a final implicit frame only when pending state or render work exists.

## C35 — Backward-Decodable NAIR Evolution

A newer NAIR runtime SHOULD retain decoding of older valid canonical formats when doing so does not weaken semantics. New opcodes SHALL NOT be accepted under an older declared format version.

## C36 — Render Context Must Be Explicit

A program containing render operations SHALL NOT execute through a state-only execution path. Missing render authority/context must be rejected before runtime mutation.

## C37 — Unified Input Semantics

Keyboard, pointer, touch, pen, gamepad and XR interaction SHALL converge on one
backend-independent semantic event model where their underlying concepts are
equivalent. Platform APIs translate into NORDOI semantics; they do not define them.

## C38 — Deterministic Interaction Order

Accepted input SHALL receive a deterministic monotonic sequence. Sequence identity
SHALL NOT silently wrap or depend on platform event-loop implementation details.

## C39 — Lossless Transition Law

Observable transitions such as key/button down and up SHALL NOT be removed by input
coalescing. An optimization may collapse replaceable state samples but SHALL NOT
erase interaction history required for correct program meaning.

## C40 — Safe Input Coalescing

Input coalescing SHALL be limited to semantically replaceable consecutive samples
on the same route and control. Coalescing MUST preserve the newest accepted state
and MUST stop across transition/additive-event boundaries.

## C41 — Atomic Input-to-State Commit

A canonical input batch mapped into NAM state SHALL commit through atomic transaction
semantics. Invalid ownership or binding state MUST be rejected before partial input
state becomes visible.

## C42 — Authorized Input Adapter Boundary

The Atomic Input Core SHALL NOT imply permission to poll raw devices. Privileged
hardware acquisition, operating-system permission access and global capture SHALL
remain outside canonical core input until governed by explicit effect and capability
contracts.

## C43 — Zero Interaction Work Without Effect

An input batch with no matching semantic binding, or whose resulting values equal
current NAM state, SHALL create no new downstream NAM work.

## C44 — Native Interaction Meaning in NAIR

Backend-independent interaction bindings MAY be canonical NAIR semantics. Raw
hardware and operating-system acquisition APIs SHALL remain outside canonical NAIR
until governed by explicit effect and capability contracts.

## C45 — Explicit Input Context

A NAIR program containing input operations SHALL NOT execute through a path that
was not given an explicit normalized input context. Missing input context MUST be
rejected before program execution begins.

## C46 — Input Bridge Slots Are Semantic Identities

A NAIR `InputBridgeSlot` SHALL be single assignment, SHALL exist before use and
SHALL resolve only to a bridge created by the exact validated execution.

## C47 — Explicit Atomic Input Boundary

`APPLY_INPUT` SHALL define an explicit deterministic input-to-state boundary. All
matching state projections for one bridge application SHALL use one atomic NAM
transaction in that bridge's ownership domain.

## C48 — Render Targets Use Canonical Slot Identity

A native input selector targeting rendered content SHALL reference a NAIR
`RenderNodeSlot`, never a serialized backend/runtime render identifier. Resolution
SHALL occur only through the current execution's validated render binding map.

## C49 — Interaction Policy Is Not Device Authority

Declaring an input binding expresses semantic routing intent only. It SHALL NOT
confer permission to poll devices, install global hooks, capture privileged input
or bypass host/platform authorization.

## C50 — Closed Runtime Activation

A certified closed runtime activation SHALL derive its meaning only from its
validated NAIR program, its canonical input batch and constitutionally governed
runtime semantics. Hidden ambient application state SHALL NOT be required.

## C51 — Canonical Input Before Runtime Execution

Any externally supplied `InputBatch` SHALL pass canonical normalization and strict
sequence-order validation before entering a closed NORDOI runtime activation.
Public construction of an input structure SHALL NOT bypass canonical input laws.

## C52 — Quiescent Success

A successful closed runtime activation SHALL end with zero pending NAM work and zero
pending Atomic Render Core work. Residual work SHALL be treated as an incomplete
activation, not silently published as success.

## C53 — Error Isolation at the Closed Runtime Boundary

If a closed activation fails, its private intermediate NAM/render state SHALL NOT be
published as a successful runtime report. Failure does not imply that every internal
instruction was rolled back; it means partial activation state is not promoted to a
valid closed result.

## C54 — Deterministic Replay Identity

NORDOI SHALL provide a deterministic identity derived from canonical program and
canonical input representations for replay, regression and evidence workflows.
Equal canonical inputs SHALL produce equal replay identities.

## C55 — Replay Identity Is Not Security Authority

A deterministic replay fingerprint SHALL NOT be treated as a cryptographic signature,
authentication credential, authorization token or proof of integrity unless a future
cryptographic specification explicitly establishes those properties.

## C56 — K1.0 Is a Semantic Kernel Milestone, Not Syntax Freeze

K1.0 certifies the first closed atomic execution architecture. It SHALL NOT be used
as justification to prematurely freeze NORDOI surface syntax, final compiler design,
backend APIs or future persistent-runtime semantics.

## C57 — Persistent Runtime Identity

A K1.1 persistent runtime SHALL bootstrap semantic NAM, render and input identities
once and preserve those identities across accepted ticks. Persistent execution SHALL
NOT recreate the application world for every input batch.

## C58 — Bootstrap Once, Tick Many

The validated NAIR program SHALL execute once for persistent-runtime bootstrap.
Subsequent ticks SHALL apply only the persistent interaction boundaries established
by that bootstrap unless a future specification explicitly defines additional
persistent instruction semantics.

## C59 — Cross-Tick Input Monotonicity

For one persistent runtime session, the first input sequence of every non-empty tick
SHALL be strictly greater than the last sequence accepted by any earlier tick.
Rejected or empty ticks SHALL NOT advance the stored sequence frontier.

## C60 — Atomic Tick Publication

A persistent tick SHALL be evaluated in isolated candidate NAM/render state and SHALL
replace published session state only after the complete tick succeeds and reaches
quiescence. A failed tick SHALL NOT publish partial persistent mutation.

## C61 — Quiescent Tick Boundary

Every successful persistent tick SHALL finish with zero pending NAM work and zero
pending Atomic Render Core work. Persistent execution SHALL NOT accumulate hidden
unfinished work between externally observable ticks.

## C62 — No Frame Without Visual Effect

A persistent tick that creates no NAM/render effect SHALL NOT emit a render frame.
Tick acceptance, replay accounting and visual work are distinct responsibilities.

## C63 — Persistent Replay Preserves Tick Boundaries

The persistent replay identity SHALL incorporate canonical input batches in accepted
tick order with unambiguous component boundaries. Equal program plus equal canonical
tick trace SHALL produce equal session replay identity.

## C64 — Persistent Runtime Does Not Imply Ambient Effects

Long-lived execution SHALL NOT weaken NORDOI effect or capability laws. Persistence
alone SHALL NOT confer access to clocks, files, network, devices, threads or other
privileged ambient resources.

## C65 — Logical Time Is Explicit Semantic Input

The NORDOI semantic core SHALL NOT obtain current time from an ambient operating-system
clock. Logical time SHALL advance only through an explicit governed input to the time
runtime or through future semantics that are equivalent and auditable.

## C66 — Logical Time Is Monotonic

Accepted logical time SHALL never move backward within one time-runtime identity.
Rejected advances SHALL NOT alter the published logical-time frontier.

## C67 — Deterministic Timer Order

Timers due in one logical-time advance SHALL fire in canonical `(deadline, TimerId)`
order. Platform timer queue order SHALL NOT define NORDOI timer meaning.

## C68 — Repeating Deadlines Are Lossless

A repeating timer SHALL represent every elapsed accepted logical deadline unless a
future explicit semantic operation defines another behavior. Timer optimization SHALL
NOT silently erase occurrences.

## C69 — Timer Bursts Must Be Bounded

Lossless timer semantics SHALL NOT require unbounded work from one externally supplied
time advance. A certified runtime SHALL enforce an explicit fire budget or an equally
strong bounded-work mechanism. Budget failure SHALL be atomic.

## C70 — Atomic Logical-Time Advance

A logical-time advance that fails validation, arithmetic or bounded-work requirements
SHALL NOT publish partial clock movement, timer rescheduling or occurrence counts.

## C71 — Atomic Event-Loop Publication

A K1.2 event-loop cycle SHALL publish logical-time state and persistent-runtime state
together only after both candidate computations succeed. Failure of either side SHALL
leave the previously published cycle intact.

## C72 — Future Timers Are Dormant, Not Residual Work

A timer scheduled for a future logical deadline SHALL NOT by itself make NAM/render
execution non-quiescent. Quiescence concerns work that is currently executable, not
valid future schedule state.

## C73 — Timer Scheduling Is Not Clock Authority

The ability to create logical timers SHALL NOT imply permission to read wall clocks,
install operating-system timers, sleep threads or access privileged platform event
loops. Those capabilities require explicit future effects and authority.

## C74 — Temporal Replay Preserves Accepted History

The event-loop replay identity SHALL incorporate successful timer schedule/cancel
operations and successful logical cycle boundaries in deterministic order. Equal
accepted temporal traces SHALL produce equal replay identities. Such identity remains
non-cryptographic unless a future cryptographic law explicitly states otherwise.

## C75 — Native Logical Time Meaning in NAIR

NAIR MAY encode logical timer declarations only when their meaning is defined by the
canonical NORDOI logical-time model. Native time instructions SHALL NOT inherit
platform timer semantics implicitly.

## C76 — Timer Declaration Is Not Clock Observation

Declaring, cancelling or referencing a logical timer SHALL NOT grant a program the
ability to read wall-clock time, monotonic OS clocks, time zones or other ambient
clock sources.

## C77 — Timer Slots Are Single-Assignment Semantic Identities

Each NAIR `TimerSlot` SHALL be defined at most once and SHALL be resolved
deterministically within the execution identity that owns it. Runtime timer handles
SHALL NOT be serialized as substitutes for semantic slots.

## C78 — K1.3 Native Timers Bootstrap Once

In K1.3, native timer declarations SHALL be applied exactly once during governed
`AtomicEventLoop` bootstrap. Persistent runtime ticks SHALL NOT recreate those timers
unless a future certified semantic version explicitly introduces dynamic scheduling.

## C79 — AtomicEventLoop Owns Logical-Time Progression

Native NAIR timer declarations SHALL NOT advance logical time. Only an explicit
governed event-loop/time-runtime operation MAY publish a new logical-time frontier.

## C80 — Missing Time Context Fails Before Mutation

An execution surface that does not provide native logical-time semantics SHALL reject
NAIR time instructions before NAM, render or input mutation becomes observable.

## C81 — NAIR Time Version Gating Is Strict

A NAIR binary declaring a format version older than the introduction of native time
opcodes SHALL reject those opcodes. Compatibility SHALL NOT permit new semantics to
be smuggled under an older declared binary contract.

## C82 — Native Timer Declarations Affect Replay Identity

Canonical native timer declarations and cancellations SHALL contribute to the event
loop's deterministic replay identity. Equal canonical programs and equal accepted
logical traces SHALL remain replay-equivalent.

## C83 — Semantic Causes Before Reactions

The canonical Reaction Core SHALL consume only governed semantic causes. Raw device
polling, wall-clock observation and backend callbacks SHALL NOT define reaction
meaning directly.

## C84 — Deterministic Reaction Order

For one canonical cause trace, matching reactions SHALL execute in a deterministic
stable order. Hash-map iteration, platform callback order and host thread scheduling
SHALL NOT determine NORDOI reaction meaning.

## C85 — Atomic Reaction-Batch Publication

A governed reaction activation SHALL publish its resulting NAM candidate only after
the complete activation succeeds. Failure of a later reaction SHALL NOT expose state
mutated only by an earlier candidate reaction from that activation.

## C86 — Reaction Writes Respect Ownership

A reaction SHALL NOT mutate an atom outside its declared ownership domain. Ownership
MUST be validated before the reaction's state is published.

## C87 — Reaction Effects Require Declaration And Authority

A reaction MAY request an effect only when its action declared that exact effect and
possesses any capability required for that exact effect scope. Authority without
declaration and declaration without required authority are both insufficient.

## C88 — Effect Intent Is Not Effect Execution

A core reaction effect intent SHALL NOT itself perform network, filesystem, process,
device or other privileged platform activity. External execution requires a separate
governed executor contract.

## C89 — Zero Reaction Work Without Semantic Effect

An unmatched cause SHALL create no NAM transaction. A matched state projection equal
to current NAM state SHALL create no downstream NAM work.

## C90 — Reaction Semantics Before NAIR Encoding

K1.4 SHALL certify reaction/action semantics independently before canonical NAIR
reaction opcodes are introduced. Future NAIR encoding MUST implement these certified
laws rather than redefine them implicitly.

## C91 — Native Reactions Implement Certified Reaction Meaning

NAIR native reaction declarations SHALL compile to the certified Atomic Reaction &
Action Core semantics. Canonical encoding SHALL NOT silently redefine reaction
ordering, ownership, effect validation, value projection or publication laws.

## C92 — Reaction Slots Are Single-Assignment Semantic Identities

Each NAIR `ReactionSlot` SHALL be defined at most once. Runtime `ReactionId` handles
SHALL be resolved deterministically from canonical program-definition order and SHALL
NOT be serialized as substitutes for semantic reaction slots.

## C93 — Canonical Code Declares Effects But Cannot Grant Itself Authority

A canonical NAIR program MAY declare effects required by a reaction action. It SHALL
NOT encode a capability grant whose presence alone authorizes privileged execution.
Authority grants MUST originate outside the serialized program under a governed host
boundary.

## C94 — Native Reaction Authority Is Exact And External

When native reactions require privileged capability, the host-provided authority SHALL
be matched to the exact semantic effect scope using the existing capability law. An
absent authority entry SHALL mean no privileged authority.

## C95 — Native Reactions Bootstrap Once

Native NAIR reaction declarations SHALL be resolved and registered once during
governed event-loop bootstrap. Persistent cycles SHALL reuse the resulting reaction
identities and SHALL NOT recreate reaction registrations on every tick.

## C96 — Native Reaction Phase Order Is Deterministic

Within one governed K1.5 event-loop cycle, certified input bridge application SHALL
precede native Input reactions, and native Input reactions SHALL precede native Timer
reactions. Within each reaction phase the certified K1.4 canonical cause and
`ReactionId` ordering SHALL remain authoritative.

## C97 — Reaction Failure Aborts The Whole Event-Loop Candidate

A failure during native Input or Timer reaction activation SHALL abort publication of
the entire candidate cycle. Candidate logical time, timer state, NAM state, render
state, replay state and effect-intent reports SHALL NOT become observable.

## C98 — Missing Native Reaction Context Fails Before Mutation

Execution surfaces that do not implement native reaction semantics SHALL reject NAIR
reaction declarations before NAM, render, input or time mutation becomes observable.
They SHALL NOT ignore reaction declarations as no-ops.

## C99 — NAIR Reaction Version Gating Is Strict

A NAIR binary declaring a format version older than the introduction of native
reaction opcodes SHALL reject those opcodes. New reaction semantics SHALL NOT be
smuggled under an older declared canonical contract.

## C100 — Native Reaction Declarations Participate In Replay Identity

Canonical native reaction declarations SHALL participate in the governed event-loop
replay identity through canonical NAIR program bytes. Reaction-driving timer causes
SHALL be represented in candidate replay progression before publication. Equal
canonical programs and equal accepted cause traces SHALL remain replay-equivalent.

## C101 — Effect Intent Precedes External Execution

A governed external effect SHALL first exist as a validated semantic `EffectIntent`.
Reaction evaluation SHALL NOT invoke external network, filesystem, process, device or
other privileged backends directly.

## C102 — Effect Outbox Publishes With The Successful Cycle

New effect intents produced by one event-loop cycle SHALL be staged on private candidate
state and published into the effect outbox only if the complete candidate cycle
succeeds. A failed candidate SHALL expose no newly staged effect envelope.

## C103 — Effect Intent Identity And Order Are Deterministic

Published effect envelopes SHALL receive stable monotonic semantic identities and a
canonical within-cycle ordinal. K1.5 Input-before-Timer phase order and certified
reaction order SHALL determine K1.6 outbox order; host thread scheduling and backend
callback timing SHALL NOT.

## C104 — A Queued Intent Is Not Authority

Possession of a queued `EffectIntent`, `QueuedEffectIntent` or `EffectIntentId` SHALL NOT
grant permission to execute the corresponding external operation.

## C105 — Dispatch Revalidates Current Exact Authority

Immediately before a queued external effect is handed to a backend, the dispatcher
SHALL validate the exact currently granted capability required by that exact effect
scope. Authority valid at program bootstrap SHALL NOT bypass later revocation.

## C106 — External Dispatch Denies By Default

The governed effect dispatcher SHALL begin without external authority unless the host
explicitly supplies it. Missing authority SHALL fail closed and SHALL preserve the
pending intent.

## C107 — Internal Effects Shall Not Escape Through External Backends

`Pure`, `StateRead` and `StateWrite` are internal semantic effects. They SHALL NOT be
executed through network/filesystem/process/device effect backends merely because an
intent object exists.

## C108 — Core Effect Backends Are Host-Injected

The NORDOI core SHALL NOT acquire ambient network, filesystem, process, camera,
microphone, location, GPU, XR or equivalent authority by linking a default privileged
backend. External backends SHALL be explicit host-provided trust boundaries.

## C109 — Failed Dispatch Preserves Pending Intent

Capability denial, capability revocation, backend incompatibility or backend-reported
failure SHALL NOT acknowledge or discard a pending effect intent.

## C110 — Successful Backend Completion Precedes Local Acknowledgement

A pending effect envelope MAY be removed from the live outbox only after the selected
backend reports successful completion for that dispatch attempt.

## C111 — Backend Results Do Not Rewrite Deterministic Program Meaning

Backend completion status and opaque backend receipt metadata SHALL NOT alter the
already-published deterministic replay identity of the cycle that created the intent.
Environmental results MAY affect future program state only through a separately
governed semantic re-entry mechanism.

## C112 — NORDOI Shall Not Claim Universal Exactly-Once Side Effects

External systems may observe an operation across failure boundaries that cannot be
universally rolled back or proven exactly once without cooperation from the external
system. NORDOI SHALL expose deterministic intent identity suitable for future
idempotency/deduplication protocols, but SHALL NOT claim universal exactly-once delivery
without a certified protocol that provides it.

## C113 — Semantic Outbox Does Not Imply Crash Durability

K1.6 certifies the semantic in-memory effect outbox and its publication law. Crash,
process, machine or distributed durability SHALL NOT be inferred until a future
certified persistence law defines and proves such guarantees.

## C114 — K1.6 Does Not Increment NAIR Without New Program Semantics

Because K1.6 introduces a host delivery boundary rather than a new serialized program
instruction, NAIR SHALL remain at format 0.5. Host backend bindings, dispatch authority
and delivery receipts SHALL NOT be serialized into canonical NAIR 0.5 bytes.

## C115 — Persistent Effect State Uses An Explicit Host Store

K1.7 SHALL NOT obtain ambient filesystem, database or cloud-storage authority. Effect
journal persistence SHALL cross an explicit host-provided `EffectJournalStore` trust
boundary.

## C116 — Successful Store Commit Is The Persistence Contract Boundary

For the K1.7 protocol, a host store returning success from checkpoint commit SHALL mean
that the supplied checkpoint atomically replaced the prior checkpoint and is recoverable
according to that store's declared durability guarantees. NORDOI SHALL NOT claim to
prove physical-media durability of an arbitrary host implementation.

## C117 — Journal Checkpoints Are Canonical And Versioned

Persistent effect-outbox state SHALL have one canonical, versioned binary
representation containing the next semantic intent identity and all pending effect
envelopes required for recovery. Equal semantic checkpoint state SHALL produce equal
checkpoint bytes.

## C118 — Persisted Input Fails Closed Under Corruption Or Resource Abuse

Checkpoint recovery SHALL validate magic, version, bounds, canonical intent ordering,
identities, UTF-8 and checksum before installation. Truncated, corrupt, oversized or
otherwise invalid checkpoint input SHALL NOT partially mutate the live event loop.

## C119 — Delivery Namespace Is Host Policy, Not Ambient Randomness

The NORDOI core SHALL NOT secretly generate a delivery namespace from ambient clocks,
randomness, machine identifiers or network state. A persistent journal namespace SHALL
be explicitly supplied by the host and reused when recovering that same journal.

## C120 — Retry Identity Is Stable Across Journal Recovery

A persistent effect retry SHALL expose a stable `EffectDeliveryKey` derived from the
explicit journal namespace and the semantic `EffectIntentId`. If acknowledgement
persistence fails after external backend success, a later retry of that pending intent
SHALL receive the same delivery key.

## C121 — Stable Retry Identity Does Not Imply Universal Exactly-Once Delivery

A stable delivery key enables cooperating destinations to deduplicate retries. It SHALL
NOT be described as universal exactly-once execution when the external destination does
not honor an idempotency/deduplication protocol.

## C122 — Journaled Cycle Publication Is Fail-Closed

When the journal-aware event-loop surface is used, the complete candidate cycle SHALL
be evaluated privately and its candidate effect-outbox checkpoint SHALL be accepted by
the journal store before the candidate replaces live event-loop state. Journal commit
failure SHALL leave the live event loop unchanged.

## C123 — Durable Acknowledgement Is Published After Persistence

Journal-aware effect dispatch SHALL evaluate acknowledgement on a private outbox
candidate. After backend success, the updated candidate checkpoint SHALL be committed
to the journal before the live pending intent is removed. Failed acknowledgement
persistence SHALL preserve the live intent for retry.

## C124 — Recovery Is A Bootstrap Operation

A recovered effect checkpoint MAY be installed into an `AtomicEventLoop` only before
its first cycle. Recovery SHALL NOT silently overwrite an already-running delivery
history.

## C125 — Recovery State Participates In Replay; Delivery Namespace Does Not

Recovered pending intent state and the next semantic intent identity SHALL explicitly
participate in subsequent replay identity because they affect future deterministic
intent allocation. Host-only delivery namespace and backend receipt metadata SHALL NOT
change deterministic program meaning.

## C126 — K1.7 Does Not Claim Whole-Runtime Durability

K1.7 certifies effect-journal persistence and recovery protocol semantics. It SHALL NOT
be interpreted as durable checkpoint/recovery of NAM state, render state, logical time,
timers or every other runtime subsystem. Whole-runtime durability requires separate
certification.

## C127 — Journal Backends Remain Replaceable And Non-Canonical

SQLite, native files, databases, remote stores or future platform persistence systems
MAY implement the K1.7 store contract. No such backend SHALL become mandatory canonical
NORDOI program semantics or a privileged dependency of the minimal core.

## C128 — K1.7 Does Not Increment NAIR Without New Program Semantics

K1.7 adds persistence, recovery and retry-delivery protocol surfaces rather than a new
serialized program instruction. NAIR SHALL remain format 0.5. Journal namespace,
checkpoint storage policy and backend idempotency routing SHALL NOT be serialized as
canonical NAIR authority.

## C129 — Recovered Effect Journals Require Explicit Writer Identity

Concurrent journal ownership SHALL NOT be inferred from process identity, machine identity,
thread identity or ambient runtime state. A K1.8 writer identity SHALL be explicitly supplied by
the host.

## C130 — Fencing Epochs Are Monotonic And Non-Zero

Every successful ownership acquisition for one delivery namespace SHALL produce a strictly newer,
non-zero fencing epoch according to the host fencing authority.

## C131 — Stale Writers Shall Not Mutate The Durable Journal

A fenced journal store SHALL reject checkpoint load, commit, active-assertion and release operations
performed with a stale lease. A stale writer SHALL NOT overwrite a checkpoint accepted under a
newer fence.

## C132 — Fenced Commit Is One Atomic Host Boundary

Fence validation and checkpoint replacement SHALL be one atomic operation from the protocol's
point of view. A host implementation that validates a fence and later writes without protecting
against takeover does not satisfy the K1.8 contract.

## C133 — Core Correctness Shall Not Depend On Ambient Lease Time

K1.8 SHALL NOT require a wall clock, synchronized clock, implicit TTL or hidden heartbeat in the
canonical core. Hosts MAY implement expiry policies externally, but NORDOI consumes explicit lease
results rather than ambient time authority.

## C134 — Stale Writers Are Checked Before External Dispatch

The fenced dispatch surface SHALL validate current journal ownership before invoking an external
backend. A writer already known to be stale SHALL create zero new backend work through that
surface.

## C135 — Current Fence Is Exposed To Cooperative Backends

The active fencing epoch SHALL be available in the external dispatch request context so a
cooperating destination or gateway can reject stale writer epochs beyond the local journal
boundary.

## C136 — Stable Delivery Identity Survives Ownership Transfer

Ownership takeover SHALL NOT change the semantic `EffectDeliveryKey` of an already-pending intent.
The delivery key identifies the semantic request; the fencing epoch identifies the current writer
authority. These identities SHALL remain distinct.

## C137 — Fencing Does Not Create Universal Exactly-Once Semantics

A local pre-dispatch fence check cannot atomically control an arbitrary remote system. If the
external destination ignores fencing and idempotency keys, duplicate observation across failure or
takeover boundaries remains possible. NORDOI SHALL state this limit explicitly.

## C138 — Fencing State Is Host Policy, Not Replay Meaning

Writer identity, lease ownership and fencing epoch SHALL NOT alter deterministic program replay
identity. Semantic recovered outbox state continues to affect replay as defined by K1.7.

## C139 — Legacy Backends Remain Valid But Weaker At The Boundary

Existing effect backends MAY ignore the K1.8 fence through the compatibility adapter. Such a
backend remains usable but SHALL NOT be described as destination-fenced unless it actually honors
the supplied epoch.

## C140 — K1.8 Does Not Increment NAIR Without New Program Semantics

K1.8 adds host concurrency-control and delivery-boundary context rather than a new serialized
program instruction. NAIR SHALL remain format 0.5. Writer identities, leases and fencing epochs
SHALL NOT be serialized as canonical program authority.

## C141 — Retry State Is Explicit And Durable

A failed external effect SHALL NOT be retried through hidden process-local counters or implicit
loops. Retry attempt count and next eligibility SHALL be explicit delivery state and SHALL be
persisted before becoming published runtime state.

## C142 — Retry Time Is Explicit Host Input, Not Ambient Time

K1.9 retry scheduling SHALL consume an explicit `EffectRetryTick`. The canonical core SHALL NOT
read wall clocks, synchronized clocks, sleep state or implicit timer services to decide when an
external effect becomes retry-eligible.

## C143 — Retry Budgets Are Finite

Every K1.9 retry policy SHALL define a finite, non-zero maximum attempt count. Infinite automatic
retry is not a certified default semantic.

## C144 — Backoff Is Deterministic And Bounded

Retry delay SHALL be computed from explicit policy, stable delivery identity and failure ordinal.
Exponential growth SHALL be capped. Jitter, when configured, SHALL be deterministic rather than
ambient-random so that crash recovery and ownership takeover preserve scheduling decisions.

## C145 — Permanent Backend Failure Shall Not Be Retried Automatically

A backend MAY classify an execution failure as permanent. Such a failure SHALL bypass automatic
retry scheduling and move the intent to durable dead-letter state after the failed attempt is
persisted.

## C146 — Dead-Lettering Is Quarantine, Not Silent Loss

When retry budget is exhausted or a permanent backend failure occurs, the complete queued effect
intent, stable delivery key, attempt count, terminal reason and last error SHALL remain durably
inspectable until explicit redrive or discard.

## C147 — Dead-Letter Redrive Preserves Semantic Delivery Identity

Redriving a dead-lettered effect SHALL restore the original `EffectIntentId` and therefore the same
`EffectDeliveryKey`. Redrive SHALL NOT manufacture a new semantic request identity for the same
quarantined effect.

## C148 — Retry Policy Is Part Of Durable Delivery Policy

Once persisted by K1.9, retry policy SHALL be recovered and compared against the configured policy.
A process restart SHALL NOT silently alter maximum attempts, backoff or jitter. Policy migration
requires an explicit future protocol.

## C149 — Stale Writers Shall Not Retry Or Redrive

All K1.9 retry execution, dead-letter transition, redrive and discard operations SHALL remain under
K1.8 fencing authority. A stale writer SHALL create zero new backend retry work through the governed
surface.

## C150 — Retry Publication Follows Durable Commit

Retry scheduling, dead-letter transition, acknowledgement, redrive and discard SHALL be evaluated
on private candidates. Local publication SHALL occur only after the fenced durable checkpoint
commit succeeds.

## C151 — Delayed Intents Shall Not Block Later Eligible Intents

Among currently eligible intents, dispatch order remains ascending by `EffectIntentId`. An older
intent whose retry eligibility lies in the future SHALL NOT prevent a later already-eligible intent
from being dispatched.

## C152 — Policy Failures Do Not Consume Backend Retry Budget

Capability denial, unsupported backend selection and other failures that occur before backend
execution SHALL NOT increment backend attempt counts.

## C153 — Retry Control Is Not Program Replay Meaning

Retry ticks, attempt counts, dead-letter metadata, backend error text and retry policy are host
delivery policy. They SHALL NOT alter deterministic program replay identity. Recovered semantic
pending outbox state continues to participate in replay identity under the K1.7 recovery law.

## C154 — Retry Does Not Create Universal Exactly-Once Semantics

A retry protocol cannot atomically control an arbitrary remote system. If a remote destination does
not honor stable delivery keys and/or fencing, duplicate external observation remains possible
across timeout, crash, failed acknowledgement persistence or ownership takeover.

## C155 — Legacy Effect Checkpoints Migrate Fail-Closed

Certified K1.7/K1.8 outbox checkpoints MAY be opened by K1.9 as an empty retry/dead-letter ledger.
K1.9 SHALL preserve the prior namespace, pending intents and next semantic intent identity while
adding no invented historical attempts.

## C156 — K1.9 Does Not Increment NAIR Without New Program Semantics

K1.9 adds external delivery-control policy rather than a new serialized program instruction. NAIR
SHALL remain format 0.5. Retry ticks, retry policy, backend error classification, attempt counters
and dead-letter state SHALL NOT be serialized as canonical program authority.

## C157 — External Attempts Require Durable Preparation Before I/O

The K1.10 audited delivery surface SHALL durably publish an `AttemptPrepared` record under the
current fence before invoking an external backend. Failed preparation persistence SHALL create zero
new backend work.

## C158 — Semantic Intent, Delivery Key, Fence And Attempt Identity Are Distinct

`EffectIntentId`, `EffectDeliveryKey`, `EffectDeliveryFence` and `EffectAttemptId` SHALL retain
separate meanings. Retry or takeover SHALL NOT conflate semantic request identity with writer epoch
or individual execution-attempt identity.

## C159 — Every Persisted Prepared Attempt Has At Most One Terminal Resolution

A prepared attempt MAY terminate as delivered, retry-scheduled, dead-lettered, assumed-delivered or
retry-authorized after recovery. The canonical audit state machine SHALL reject a second terminal
resolution for the same open attempt.

## C160 — Missing Terminal Persistence Creates In-Doubt State

If external backend work may have occurred but no terminal outcome is durably committed, NORDOI
SHALL represent that attempt as in-doubt rather than silently classifying it as success or failure.

## C161 — In-Doubt State Blocks Automatic Redispatch

While an unresolved in-doubt attempt exists, the governed K1.10 surface SHALL create zero new
external dispatch work. Retry requires an explicit host resolution.

## C162 — In-Doubt Retry Authorization Is An Explicit Duplicate-Risk Decision

Authorizing retry of an in-doubt attempt SHALL preserve the original `EffectDeliveryKey` and SHALL
be an explicit host action. NORDOI SHALL NOT describe that action as duplicate-free unless the
external destination provides the required idempotency guarantees.

## C163 — Assumed Delivery Requires Explicit Reconciliation Authority

An in-doubt intent MAY be resolved as delivered without another backend call only through an
explicit host operation. This operation SHALL be durably audited and SHALL remove the pending intent
only after the fenced checkpoint commit succeeds.

## C164 — Audit Records Are Append-Only And Canonically Ordered

Effect audit records SHALL use monotonically increasing sequence identities and SHALL NOT be
reordered or rewritten by the canonical K1.10 journal surface.

## C165 — Audit Records Are Hash-Chained

Every K1.10 audit record SHALL cryptographically bind its canonical event bytes, sequence and prior
record hash using SHA-256. Recovery SHALL reject broken sequence, previous-hash or record-hash links.

## C166 — Checkpoint Integrity Is Independently Protected

The complete K1.10 audit checkpoint SHALL carry an independent SHA-256 digest. Corrupted checkpoint
bytes SHALL be rejected before state publication.

## C167 — Hash Chaining Is Not Writer Authentication

K1.10 hash chaining provides tamper evidence relative to a trusted checkpoint/root. It SHALL NOT be
described as a digital signature, proof of author identity or universal non-repudiation. Such claims
require a separately certified signing/attestation layer.

## C168 — Audit Recovery Shall Not Invent Historical Attempts

Migrating a certified K1.7, K1.8 or K1.9 checkpoint into K1.10 SHALL preserve existing delivery
state while creating an empty audit history. NORDOI SHALL NOT fabricate attempts that were never
recorded by the older protocol.

## C169 — Audit Publication Remains Fenced And Persistence-First

Preparation, terminal outcomes, in-doubt resolution, redrive and discard SHALL remain protected by
the active K1.8 fence. Candidate audit/delivery state SHALL become local state only after the
corresponding durable commit succeeds.

## C170 — Audit Metadata Is Not Program Replay Meaning

Attempt IDs, audit sequence numbers, hashes, backend receipt references and in-doubt resolution
metadata SHALL NOT alter deterministic program replay identity. Semantic recovered pending outbox
state remains governed by the certified K1.7 recovery law.

## C171 — Audit Storage Remains Host-Replaceable

The canonical core defines bytes, ordering, validation and fencing requirements but SHALL NOT bind
K1.10 to SQLite, files, a cloud database, a particular consensus system or another mandatory storage
backend.

## C172 — K1.10 Does Not Increment NAIR Without New Program Semantics

K1.10 adds delivery audit and crash-window recovery semantics rather than a new serialized program
instruction. NAIR SHALL remain format 0.5. Attempt IDs, audit hashes, backend receipts and in-doubt
resolution authority SHALL NOT be serialized as canonical NAIR program authority.

## C173 — Audit Hashing And Writer Attestation Are Distinct Layers

K1.10 hash chaining SHALL remain an integrity mechanism relative to a trusted root. K1.11
attestation SHALL be a separate authentication layer and SHALL NOT redefine hash-chain semantics.

## C174 — Signing Authority Is Host-Injected

The canonical core SHALL NOT contain a default private signing key, ambient key lookup, mandatory
cloud KMS, operating-system keychain dependency or serialized program signing authority. Signers and
verifiers SHALL be explicit host-provided boundaries.

## C175 — Attestation Binds Exact Durable Audit State

A K1.11 signed statement SHALL bind namespace, writer identity, delivery fence, trust epoch, key ID,
algorithm ID, K1.10 audit root, audit record count and a cryptographic hash of the complete K1.10
canonical checkpoint.

## C176 — Only Durable State May Be Attested By The Governed Live Surface

Before signing current runtime state, K1.11 SHALL confirm that the live K1.10 checkpoint exactly
matches the checkpoint recoverable from the governed durable journal. Missing or divergent durable
state SHALL fail before signing.

## C177 — Trust Epoch Is Explicit And Non-Zero

Cryptographic trust generation SHALL be represented by an explicit non-zero `EffectTrustEpoch`.
The core SHALL NOT derive trust generation from wall time, process start time, machine identity or
ambient configuration.

## C178 — Trust Epoch Shall Not Move Backward

For a retained namespace anchor, a newly committed attestation SHALL NOT use a trust epoch lower
than the previously retained attestation.

## C179 — Key Or Algorithm Rotation Requires Epoch Advance

Within one trust epoch, attestation key identity and algorithm identity SHALL remain stable. A key or
algorithm change SHALL require an explicit greater trust epoch.

## C180 — Writer Takeover Does Not Imply Cryptographic Rotation

`EffectJournalWriterId`/`EffectDeliveryFence` ownership and attestation trust epoch/key identity are
independent. A valid writer takeover MAY produce a newer fence while retaining the same trust epoch
and signing key.

## C181 — Attestation Anchors Are Fenced

Committing an attestation anchor SHALL carry the active K1.8 lease. A conforming attestation store
SHALL reject stale writers rather than allowing an obsolete process to publish a new trusted anchor.

## C182 — Signed Audit Progression Is Monotonic

A new retained anchor SHALL NOT attest fewer audit records than the prior anchor. At an equal audit
record count, the audit root SHALL remain identical. When height increases, the previously attested
root SHALL appear at the prior height in the current K1.10 chain. Divergence or non-descendant
history SHALL fail closed.

## C183 — Signature Verification Fails Closed

A rejected signature, verifier backend error, malformed attestation, checkpoint mismatch or broken
envelope integrity SHALL NOT be treated as authenticated state.

## C184 — Signature Bytes Are Opaque And Bounded

The core SHALL treat signature bytes as opaque host output subject to explicit size bounds. K1.11
SHALL NOT infer cryptographic strength from signature length or algorithm naming alone.

## C185 — Attestation Storage Is Separate From The K1.10 Journal

K1.11 anchors SHALL NOT replace or mutate the canonical `NDEFXA01` journal format. Effect delivery
recovery remains governed by K1.10; authentication anchors remain a separate host-replaceable
storage concern.

## C186 — Unattested State Is Not Silently Authenticated

The absence of a K1.11 anchor SHALL be represented as absence, not as successful verification.
Legacy K1.10 state MAY remain usable under explicit host policy but SHALL NOT be described as signed
or authenticated by K1.11.

## C187 — Attestation Does Not Prove Universal Non-Repudiation

K1.11 proves only what the configured signer, verifier, trust policy and anchor store actually
guarantee. It SHALL NOT claim hardware-backed identity, revocation freshness, transparency inclusion,
rollback-proof storage or legal non-repudiation without those mechanisms being separately present
and certified.

## C188 — Trust Metadata Is Not Program Replay Meaning

Key IDs, algorithm IDs, trust epochs, signatures and attestation-store state SHALL NOT alter
deterministic program replay identity.

## C189 — Attestation Backends Remain Replaceable

The canonical K1.11 interfaces SHALL permit software keys, hardware security modules, platform secure
enclaves, remote KMS systems, threshold signers and future cryptographic schemes without changing
canonical program semantics.

## C190 — K1.11 Does Not Increment NAIR Without New Program Semantics

K1.11 adds host trust and audit authentication semantics rather than a serialized program
instruction. NAIR SHALL remain format 0.5. Signing keys, trust epochs, signatures and verifier policy
SHALL NOT be serialized as canonical NAIR program authority.

## C191 — External Results Re-enter Only As Governed Causes

An external effect result SHALL NOT mutate NAM directly. It may influence program state only through
an explicit completion cause validated by the governed completion re-entry core.

## C192 — Completion Sources Have No Ambient Authority

A completion source SHALL have zero authority by default. Authority MUST be granted explicitly for
an exact `(EffectCompletionSourceId, EffectDeliveryNamespace)` pair.

## C193 — Completion Stream Identity And Delivery Identity Are Distinct

`EffectCompletionSourceId`, `EffectCompletionSequence`, `EffectDeliveryKey` and `EffectAttemptId`
SHALL retain separate meanings. Source ordering SHALL NOT replace stable delivery identity or audited
attempt correlation.

## C194 — Completion Requires Exact Audited Attempt Correlation

A completion SHALL identify a K1.10 `AttemptPrepared` record whose `EffectDeliveryKey` exactly
matches the completion key and whose request intent matches that key.

## C195 — Only Delivered Or Explicitly Assumed-Delivered Attempts May Complete Semantics

A completion MAY re-enter semantics only when its correlated attempt terminated as
`AttemptDelivered` or `InDoubtAssumedDelivered`. Retry-scheduled, dead-lettered, unknown and open
in-doubt attempts SHALL fail closed.

## C196 — One Delivery Key Produces At Most One Live Semantic Completion

Within one live completion-core state, an `EffectDeliveryKey` SHALL be accepted as a semantic
completion at most once, regardless of newer source sequence or alternate physical delivery attempt.

## C197 — Completion Source Sequences Are Strictly Monotonic

Every accepted completion source sequence SHALL be non-zero and strictly greater than that source's
previous accepted sequence. Duplicate or backward source sequence SHALL fail the whole batch.

## C198 — Completion Batches Are Canonically Ordered

Completion batches SHALL be canonicalized before application. Equal valid causes SHALL be processed
in the same source/sequence/delivery/attempt order independent of host insertion order.

## C199 — Completion Projection Is Pre-Registered Policy

Completion payloads SHALL NOT choose arbitrary NAM destinations. Semantic projections SHALL be
registered explicitly for an exact source/namespace route before completion application.

## C200 — Completion Projection Respects Explicit Ownership

Projection registration and projection application SHALL both validate the target atom's expected
ownership domain. Ownership transfer SHALL invalidate obsolete completion write authority rather
than being silently followed.

## C201 — Completion Writes Use Normal Atomic Transactions

Completion-induced NAM writes SHALL pass through normal `AtomicTransaction` validation and commit
rules. Completion processing SHALL NOT gain a privileged mutation shortcut.

## C202 — A Completion Batch Publishes Atomically

Completion-core state, source-sequence advancement, delivery-key deduplication and NAM writes SHALL
be evaluated on private candidates. Failure of any cause or projection SHALL publish none of the
batch.

## C203 — Completion Re-entry Precedes Render Flush

In the persistent runtime phase, governed completions SHALL be applied after input/timer reactions
and before NAM/render flush so their state changes participate in the same invalidation frontier.

## C204 — Accepted Completion Is Deterministic Replay Meaning

Unlike delivery fences, retry metadata or signing metadata, an accepted completion is program-visible
semantic input. Its canonical cause and exact projected writes SHALL participate in deterministic
runtime replay identity.

## C205 — Completion Payloads Are Canonical And Bounded

Completion batches and payload values SHALL be explicitly bounded. Floating-point values SHALL be
finite, text SHALL obey the certified size bound, and canonical encoding SHALL not depend on host
map order or ambient serialization behavior.

## C206 — Stale Effect-Journal Ownership Cannot Re-enter Semantics

The high-level governed completion surface SHALL require an active K1.8/K1.10 journal lease. A stale
writer that has lost its fence SHALL fail before completion publication.

## C207 — K1.12 Does Not Claim Whole-Runtime Crash Durability

K1.12 completion deduplication, source sequence and completion-induced NAM state are live runtime
state. Until a whole-runtime persistence protocol atomically persists and recovers those semantics,
K1.12 SHALL NOT claim crash-persistent exactly-once completion application.

## C208 — K1.12 Does Not Increment NAIR Before Native Completion Semantics

K1.12 certifies the completion re-entry core beneath NAIR. NAIR SHALL remain format 0.5. Native
completion triggers, completion projection declarations or `.noi` syntax SHALL require a later
separately certified semantic milestone.

## C209 — Native Completion Declaration Is Program Meaning

A native effect-completion declaration encoded in NAIR 0.6 SHALL be canonical program semantics.
Its slot, name, declared domain, target atom slots and projection selectors SHALL participate in
canonical NAIR bytes.

## C210 — Native Completion Authority Is Never Serialized By The Program

`EffectCompletionSourceId`, `EffectDeliveryNamespace` and the authority to accept that route SHALL
remain host-supplied policy. A NAIR program SHALL NOT mint or serialize its own completion-source
authority.

## C211 — Completion Slots Are Single-Assignment

Every `CompletionSlot` SHALL be defined at most once in a valid NAIR program. Duplicate native
completion declarations for the same slot SHALL fail validation before execution.

## C212 — Native Completion Projections Are Predeclared And Bounded

Every native completion declaration SHALL contain at least one projection to an atom already defined
in program order. Duplicate atom projections within one declaration SHALL fail validation.

## C213 — Native Completion Bootstrap Reuses The Certified K1.12 Core

K1.13 SHALL NOT introduce a privileged alternative completion executor. Native declarations SHALL
resolve into the K1.12 `AtomicEffectCompletionCore`, including its audit correlation, deduplication,
ownership and atomic transaction laws.

## C214 — Every Native Completion Slot Requires Explicit Host Binding

An event-loop boot containing a native completion declaration SHALL fail closed unless the host
provides an exact binding from that `CompletionSlot` to a non-zero `EffectCompletionSourceId` and an
`EffectDeliveryNamespace`.

## C215 — Host Binding Is Exact To The Declared Slot

A binding supplied for one `CompletionSlot` SHALL NOT authorize another slot. Extra host bindings MAY
exist, but undeclared bindings SHALL NOT create program projections by themselves.

## C216 — Direct Executors Reject Native Completion Declarations

NAIR execution surfaces that do not own the governed completion context SHALL reject native
completion declarations explicitly. They SHALL NOT ignore, partially execute or reinterpret them as
ordinary bootstrap instructions.

## C217 — Native Completion Ownership Is Revalidated At Bootstrap

Resolving a native projection SHALL validate that the declared domain currently owns the resolved
atom. Host source authority SHALL NOT bypass NAM ownership.

## C218 — Native Binding Does Not Rewrite Canonical Program Bytes

Changing host source/namespace binding SHALL NOT rewrite canonical NAIR 0.6 program bytes. Binding is
runtime authority; the native declaration is program meaning.

## C219 — Accepted Native Completion Remains Replay Meaning

Once a completion is accepted through a native route, its semantic cause and projected writes SHALL
participate in replay identity exactly as required by C204. Native declaration SHALL NOT weaken
completion replay semantics.

## C220 — NAIR 0.6 Is Backward-Decoding Compatible

A K1.13 implementation SHALL continue to decode valid supported NAIR 0.1 through 0.5 programs.
Canonical re-encoding SHALL emit the current NAIR 0.6 format.

## C221 — Completion Opcode Is Minor-Version Gated

`DEFINE_EFFECT_COMPLETION` opcode `0x70` SHALL be valid only for declared NAIR minor version 6 or
newer. The opcode under an earlier declared minor SHALL fail as invalid rather than being guessed or
silently upgraded.

## C222 — K1.13 Does Not Freeze `.noi` Surface Syntax

K1.13 certifies native completion semantics in NAIR. It SHALL NOT be interpreted as freezing the
human-facing `.noi` syntax, grammar or keywords used by future language frontends.

## C223 — Whole-Runtime Semantic Durability Is Explicit

Crash-consistent recovery of mutable runtime semantics SHALL occur only through an explicit,
host-injected persistence protocol. K1.14 SHALL NOT acquire ambient filesystem, database, cloud or
other storage authority.

## C224 — Runtime Checkpoint Binds The Exact Canonical Program

Every K1.14 runtime checkpoint SHALL bind the SHA-256 identity of the exact canonical NAIR program
whose booted runtime produced it. Recovery under different canonical program bytes SHALL fail
closed.

## C225 — Mutable Semantic Continuation State Is Checkpointed

A K1.14 checkpoint SHALL retain the mutable state required for deterministic continuation, including
NAM values/versions, runtime and event-loop replay state, logical time/timers, input sequencing,
render revisions, runtime identity frontiers and effect-completion deduplication state.

## C226 — Static Program Structure And Host Authority Are Reconstructed, Not Minted

Static program structure SHALL be rebuilt by booting the same canonical program. Host capabilities,
completion-source authority and other privileges SHALL be supplied again by the host and SHALL NOT
be restored from checkpoint bytes as authority.

## C227 — Effect And Runtime Checkpoints Share One Fenced Publication Boundary

A K1.14 durable publication SHALL commit the governed effect/audit checkpoint and runtime semantic
checkpoint atomically under the same active effect-journal lease and fencing epoch.

## C228 — Durable Commit Precedes Live Runtime Publication

A candidate cycle using the K1.14 durable API SHALL become visible in the live runtime only after the
combined host store commit succeeds.

## C229 — Failed Bundle Commit Publishes Zero Runtime Progress

If the combined runtime/effect checkpoint commit fails, the live event-loop cycle, NAM state, logical
time, timers, replay state, render revisions, completion consumption state and effect outbox SHALL
remain unchanged.

## C230 — Whole-Runtime Recovery Is Bootstrap-Only

K1.14 whole-runtime recovery SHALL be allowed only before the event loop has published a runtime
cycle. Recovery SHALL fail closed if invoked after runtime execution has begun.

## C231 — Recovery Is Continuation, Not A New Replay Cause

Successful recovery SHALL restore the saved replay accumulators exactly. The act of recovery SHALL
NOT add a synthetic semantic event or otherwise change replay identity.

## C232 — Later Effect Audit Progress Must Descend From The Saved Prefix

Effect dispatch, retry, dead-letter and in-doubt resolution MAY advance the effect audit after the
last runtime checkpoint. Recovery SHALL accept such progress only when the current audit history
contains the exact audit root recorded by K1.14 at the exact recorded prefix height.

## C233 — Effect Intent Frontier Detects Non-Durable Semantic Progress

Every runtime checkpoint SHALL bind the next `EffectIntentId` allocation frontier. A current effect
checkpoint with a different frontier SHALL be rejected during K1.14 recovery, even if its audit
history otherwise descends from the saved prefix.

## C234 — Partial Runtime/Effect Bundles Fail Closed

If exactly one half of the K1.14 effect/runtime bundle is available, recovery SHALL fail. NORDOI SHALL
NOT guess, synthesize or silently pair checkpoint halves from unrelated durable points.

## C235 — Stale Writers Cannot Publish A Runtime Bundle

The host runtime-checkpoint store SHALL atomically validate the supplied K1.8 lease/fence. A stale
writer SHALL NOT update either the runtime half or the effect/audit half of a K1.14 combined commit.

## C236 — Recovered Semantic Identities And Versions Do Not Regress Behind Bootstrap

Recovery SHALL reject state that would move transaction identity, timer identity, NAM atom versions
or render revisions behind the state established by booting the same canonical program.

## C237 — Completion Consumption Survives Recovery

The last accepted completion sequence per source and every consumed `EffectDeliveryKey` SHALL be
part of K1.14 semantic state. A delivery already applied as a semantic completion before the durable
checkpoint SHALL remain deduplicated after recovery.

## C238 — Runtime Checkpoint Format Is Canonical, Bounded And Integrity-Protected

K1.14 checkpoint bytes SHALL use a versioned canonical format with deterministic collection order,
explicit resource bounds, validated UTF-8/finite values and a SHA-256 integrity digest with domain
separation.

## C239 — Host Persistence Contract Defines Physical Durability

A successful host combined-commit return is the K1.14 persistence contract boundary. NORDOI may
validate protocol, fencing and canonical bytes but SHALL NOT claim to prove fsync, replication,
power-loss or physical media guarantees of an arbitrary host implementation.

## C240 — Legacy Runtime APIs Do Not Gain Hidden Durability

Existing non-K1.14 cycle and scheduling APIs MAY remain available. Their use SHALL NOT imply crash
durability. After a crash, semantic work not included in a successful K1.14 checkpoint MAY roll back
to the last durable runtime point.

## C241 — K1.14 Does Not Increment NAIR

K1.14 adds runtime persistence and recovery protocol, not new canonical program semantics. NAIR SHALL
remain format 0.6 and no new opcode is required by this milestone.

## C242 — K1.14 Does Not Freeze `.noi` Surface Syntax

K1.14 SHALL NOT be interpreted as freezing human-facing `.noi` syntax, grammar or persistence
keywords. Future frontends may expose runtime durability without changing the certified semantic
protocol.

## C243 — Existing Runtime Bundle Requires Recovery Before Replacement

A freshly booted K1.14 runtime SHALL NOT overwrite an already existing complete runtime/effect bundle
merely because it acquired a newer fence. It SHALL first recover that bundle and establish its
semantic lineage. An empty store MAY establish a new lineage. A legacy partial checkpoint SHALL fail
closed unless a separately certified explicit migration protocol is used.

## C244 — Program Change Is A Governed Semantic Event

Changing the canonical NAIR program of a durable runtime SHALL occur only through an explicit
program-upgrade protocol. A program replacement is semantic work and SHALL NOT be disguised as
ordinary recovery.

## C245 — K1.14 Exact-Program Recovery Remains Fail-Closed

K1.15 SHALL NOT weaken K1.14 `ProgramMismatch`. Ordinary recovery under different canonical program
bytes SHALL continue to fail. Cross-program continuation requires the governed K1.15 upgrade path.

## C246 — Upgrade Authority Is Exact To Source And Target Program Identity

No program transition has ambient authority. `RuntimeUpgradeAuthority` SHALL grant an exact pair of
source and target canonical program hashes. The target program SHALL NOT authorize its own
installation.

## C247 — Every Source Atom Has An Explicit Disposition

Every source atom participating in K1.15 SHALL be explicitly copied to one target atom or explicitly
dropped. Omission SHALL fail closed.

## C248 — Every Target Atom Has An Explicit Origin

Every target atom SHALL either receive one explicit source copy or retain its explicitly declared
target bootstrap default. Unstated target initialization SHALL fail closed.

## C249 — K1.15 Atom Migration Is One-To-One And Deterministic

K1.15 SHALL NOT merge, split, fan out or invoke arbitrary migration code. A `Copy` transfers the
semantic `Value` into the target atom while target identity, ownership and static dependency
structure remain defined by the target program.

## C250 — Upgrade Plans Have Canonical Identity

A K1.15 upgrade plan SHALL have a deterministic canonical representation and domain-separated
cryptographic hash independent of host iteration order.

## C251 — Program Epochs Advance Monotonically

Fresh boot begins at `ProgramEpoch(0)`. Every successfully published K1.15 program upgrade SHALL
advance the epoch by exactly one. Epoch exhaustion SHALL fail closed.

## C252 — Upgrade Lineage Is Cryptographically Chained

Every successful K1.15 upgrade SHALL extend a SHA-256 lineage root over the prior lineage, target
epoch, exact source/target program hashes, migration-plan hash and exact source semantic-checkpoint
hash.

## C253 — Upgrade Changes Replay Identity

A program upgrade SHALL change runtime and event-loop replay identity. Equal source state, equal
target program and equal migration plan SHALL be deterministic; different migration semantics SHALL
not share replay identity accidentally.

## C254 — Recovery And Upgrade Remain Distinct

Recovery SHALL remain a continuation and SHALL NOT add a replay cause. Upgrade SHALL remain an
explicit semantic transition and SHALL add replay meaning.

## C255 — Durable Upgrade Commit Precedes Target Publication

The target runtime SHALL remain private until the combined effect/runtime checkpoint for the target
program succeeds under the active fenced lease.

## C256 — Failed Upgrade Commit Preserves The Source Runtime

If K1.15 target publication fails at the persistence boundary, the live source program, source NAM,
time, replay, completion state and effect outbox SHALL remain unchanged.

## C257 — External Effect Identity Does Not Reset Across Program Upgrade

`EffectIntentId`, `EffectDeliveryKey`, pending outbox state and the governed audit/retry lineage SHALL
survive K1.15 program replacement. Program upgrade SHALL NOT create a new external delivery identity
universe.

## C258 — Input, Time And Completion Deduplication Frontiers Survive Upgrade

K1.15 SHALL preserve logical time, event-loop cycle/tick, last accepted public input sequence,
completion-source sequences and consumed delivery-key state unless a later separately certified
protocol explicitly changes those laws.

## C259 — Pending Source Timers Block K1.15 Upgrade

K1.15 SHALL fail if source timers remain pending. Timer topology SHALL NOT be silently guessed or
reinterpreted across program versions.

## C260 — Target Timers Cannot Begin In The Preserved Past

A target native timer whose bootstrap deadline is before the preserved logical time SHALL cause the
upgrade to fail closed.

## C261 — Upgrade Authority Is Never Serialized As Program Privilege

Host upgrade grants SHALL NOT be encoded in NAIR, checkpoint state or migration-plan bytes as
program-owned authority. Authority must be supplied again by the host.

## C262 — Certified K1.14 Runtime Checkpoints Remain Readable

K1.15 runtime checkpoint format 1.1 SHALL decode certified K1.14 format 1.0 bytes using the certified
K1.14 integrity domain. Legacy checkpoints SHALL enter K1.15 as epoch zero with empty upgrade
lineage; no prior upgrade history may be invented.

## C263 — K1.15 Migration Does Not Execute Arbitrary Host Code In The Core

The certified K1.15 migration kernel SHALL use only bounded deterministic migration rules. Arbitrary
host closures, scripts or model-generated code SHALL NOT execute as implicit migration authority.

## C264 — K1.15 Does Not Increment NAIR

K1.15 governs runtime program evolution and state migration, not new canonical program semantics.
NAIR SHALL remain format 0.6.

## C265 — K1.15 Does Not Freeze `.noi` Surface Syntax

K1.15 SHALL NOT freeze human-facing program-upgrade or migration syntax. Future `.noi` frontends may
express certified upgrade semantics without changing these underlying laws.

## C266 — Delivery Audit Metadata Does Not Become Upgrade Replay Meaning

The durable upgrade lineage MAY bind the exact source runtime checkpoint and therefore its audit
prefix. Runtime/event replay identity SHALL NOT hash the source checkpoint digest, audit root,
backend receipt or upgrade-lineage root. Upgrade replay meaning is derived from semantic source
replay state, exact source/target program identity, migration-plan identity and program epoch.

## C267 — Timer Migration Requires An Explicit Upgrade Protocol

K1.16 SHALL NOT weaken the K1.15 zero-pending-timer rule on the legacy upgrade API. Pending timer
continuity is permitted only through the explicit timer-aware upgrade path with a validated
`RuntimeTimerUpgradePlan`.

## C268 — Every Source Native Timer Slot Has An Explicit Disposition

Every native source `TimerSlot` SHALL be explicitly carried to one target timer slot or explicitly
dropped. Missing source timer disposition SHALL fail closed, including for source slots whose timer
was already canceled.

## C269 — Every Target Native Timer Slot Has An Explicit Origin

Every native target `TimerSlot` SHALL either receive exactly one explicit carried source timer or
explicitly retain its target bootstrap default. Duplicate or unstated target timer disposition SHALL
fail closed.

## C270 — Timer Continuity Preserves State But Adopts Target Identity

A carried active timer SHALL preserve its remaining logical deadline, interval and occurrence count,
but SHALL use the `TimerId` bound to the target `TimerSlot`. Source runtime timer identity SHALL NOT
override target reaction topology.

## C271 — Carried Cancellation Remains Cancellation

If a carried source timer slot is no longer pending, K1.16 SHALL treat that state as cancellation and
remove the corresponding target timer rather than resurrecting target bootstrap work.

## C272 — Timer Kind And Cadence Are Not Implicitly Converted

K1.16 SHALL carry one-shot timers only to one-shot timers and repeating timers only to repeating
timers with the exact same logical interval. Kind or interval conversion requires a later separately
certified protocol.

## C273 — Pending Dynamic Timers Are Not Inferred Across Upgrade

A pending timer that is not bound to a native source `TimerSlot` SHALL fail the K1.16 timer-aware
upgrade. Raw `TimerId` values SHALL NOT be treated as sufficient program-level migration identity.

## C274 — Target Default Timers Cannot Begin In The Preserved Past

A target timer explicitly kept at its bootstrap default SHALL continue to obey the K1.15 rule that
its next deadline may not precede the preserved logical time.

## C275 — Explicit Carry May Replace An Obsolete Target Bootstrap Deadline

A target timer selected by `Carry` MAY have a bootstrap deadline that precedes the preserved logical
time, because the bootstrap schedule is replaced before target time-state publication. The carried
source deadline itself MUST remain valid at or after preserved logical time.

## C276 — Timer Allocation Identity Never Moves Backward Across Upgrade

K1.16 SHALL preserve the source timer-allocation frontier. The target `next TimerId` frontier SHALL be
at least the maximum of source and target frontiers, including identities of dynamic timers that were
allocated and later canceled.

## C277 — Timer Upgrade Plans Have Canonical Identity

`RuntimeTimerUpgradePlan` SHALL have deterministic canonical bytes and a domain-separated SHA-256
identity independent of host iteration or rule input order.

## C278 — Atom And Timer Migration Semantics Share One Upgrade Commitment

For a timer-aware upgrade, K1.16 SHALL derive a domain-separated composite migration hash from the
certified atom-plan hash and timer-plan hash. The durable upgrade-lineage record SHALL commit this
composite identity.

## C279 — Timer Migration Changes Replay Meaning

Different timer migration semantics SHALL produce different runtime/event replay identity even when
source state, target program and atom migration are otherwise identical. Backend receipts, audit
roots and other delivery metadata remain excluded from semantic replay as required by C266.

## C280 — Timer-Aware Upgrade Remains Persistence-First

The migrated target time state SHALL remain private until the same fenced combined effect/runtime
checkpoint commit required by K1.15 succeeds. A validation or persistence failure SHALL leave the
source program and source timers live and unchanged.

## C281 — K1.16 Preserves Runtime Checkpoint Format 1.1

K1.16 SHALL NOT require a new runtime checkpoint field merely to represent timer-aware upgrade
identity. `NDRTSM01` format 1.1 remains canonical; the existing latest-upgrade `plan_hash` field MAY
contain the K1.16 composite atom+timer migration hash.

## C282 — Timer Migration Does Not Mint Upgrade Authority

A timer plan SHALL NOT authorize a program transition. Exact source→target authorization continues
to come only from host-supplied `RuntimeUpgradeAuthority`, and timer-plan bytes SHALL NOT serialize
that authority as program privilege.

## C283 — K1.16 Does Not Increment NAIR

K1.16 governs continuity of already-certified native timer state across program replacement. NAIR
SHALL remain format 0.6 and no new opcode is required by this milestone.

## C284 — K1.16 Does Not Freeze `.noi` Surface Syntax

K1.16 SHALL NOT freeze human-facing timer-migration or upgrade syntax. Future `.noi` frontends may
express these certified laws without changing their runtime semantics.

## C285 — Dynamic Timer Migration Is Explicit And Additive

K1.17 SHALL NOT weaken the certified K1.16 timer-aware upgrade path. Active dynamic timers remain
rejected by that API. Dynamic timer continuity is permitted only through the separate K1.17
full-timer upgrade protocol with a validated `RuntimeDynamicTimerUpgradePlan`.

## C286 — Every Active Dynamic Source Timer Has An Explicit Disposition

Every active source timer not bound to a native source `TimerSlot` SHALL be explicitly carried to one
target `TimerId` or explicitly dropped. Omission SHALL fail closed.

## C287 — Canceled Dynamic Timers Need No Synthetic State

A dynamic timer that has already been canceled has no active timer snapshot and SHALL NOT require a
synthetic migration rule. Its allocation history SHALL remain protected by the preserved timer
identity frontier.

## C288 — Raw TimerId Gains Meaning Only Inside A Governed Dynamic Plan

A raw `TimerId` SHALL NOT acquire program-level migration meaning by numeric coincidence. It may be
used as dynamic resource identity only inside a canonical K1.17 plan bound to exact source/target
program identity, source epoch, durable source state and separately supplied upgrade authority.

## C289 — Dynamic Carry May Preserve Or Explicitly Remap Identity

A carried dynamic timer MAY preserve its exact identity or MAY map to one explicit target identity.
No implicit target identity selection is permitted.

## C290 — Remapped Dynamic Timer Identity Must Be Fresh

If a dynamic timer is remapped to a different target ID, that target SHALL be at or beyond the
maximum source/target allocation frontier. Exact `source == target` preservation is the only
exception because it continues an existing identity rather than recycling one.

## C291 — Target Native Timer Identities Remain Reserved

A dynamic timer SHALL NOT migrate onto any `TimerId` bound to a target native `TimerSlot`, including
a slot whose bootstrap timer is already canceled. Native reaction topology owns that identity.

## C292 — Dynamic Carry Preserves Timer Progress Exactly

A carried dynamic timer SHALL preserve next logical deadline, interval and occurrence count exactly.
K1.17 SHALL NOT synthesize elapsed firings or implicitly change one-shot/repeating cadence.

## C293 — Dynamic Target Identity Zero And Exhaustion Fail Closed

`TimerId(0)` SHALL NOT be a valid carried dynamic target. A target identity that cannot advance the
allocation frontier without overflow SHALL also fail closed.

## C294 — Dynamic Timer Plans Have Canonical Identity

`RuntimeDynamicTimerUpgradePlan` SHALL have deterministic canonical bytes and a domain-separated
SHA-256 identity independent of host iteration or rule insertion order.

## C295 — Full Timer Upgrade Commits Atom, Native Timer And Dynamic Timer Semantics

A K1.17 full-timer upgrade SHALL derive one domain-separated composite plan hash from the certified
atom-plan hash, native-timer-plan hash and dynamic-timer-plan hash. Durable upgrade lineage SHALL
commit this full identity.

## C296 — Dynamic Timer Mapping Changes Replay Meaning

Different dynamic timer carry/drop/remap semantics SHALL produce different runtime/event replay
identity. Backend receipts, audit roots, writer/fence metadata and source-checkpoint digest remain
excluded from semantic replay as required by C266.

## C297 — Dynamic Timer Mapping Is Observable Upgrade Output

A successful K1.17 upgrade SHALL report the exact source→target dynamic timer mappings that were
published so hosts retaining dynamic timer handles can update them deterministically.

## C298 — Dynamic Timer Upgrade Remains Persistence-First

The target runtime and all migrated dynamic timer state SHALL remain private until the existing
fenced combined effect/runtime checkpoint commit succeeds.

## C299 — Failed Dynamic Timer Upgrade Preserves Source State

Validation or persistence failure during K1.17 SHALL preserve the live source program, source timer
state, timer identities, replay state, program epoch, upgrade lineage and effect state unchanged.

## C300 — Dynamic Timer Recovery Restores Target Identity And Progress

After successful publication, ordinary exact-program recovery SHALL restore each carried dynamic
timer under its target identity with the persisted deadline, interval and occurrence count.

## C301 — Timer Allocation Frontier Never Moves Backward Or Behind A Remap

The target timer allocation frontier SHALL be at least the source frontier, target bootstrap frontier
and one greater than every carried dynamic target identity. Dropped, canceled and remapped identities
SHALL NOT become silently reusable.

## C302 — K1.17 Preserves Runtime Checkpoint Format 1.1

K1.17 SHALL NOT add checkpoint fields merely for dynamic timer migration. `NDRTSM01` format 1.1
remains canonical; the existing latest-upgrade `plan_hash` MAY contain the K1.17 full composite
migration identity.

## C303 — K1.17 Does Not Increment NAIR

K1.17 governs runtime-resource continuity for timers already represented by the certified time core.
NAIR SHALL remain format 0.6 and no new opcode is required by this milestone.

## C304 — K1.17 Does Not Freeze `.noi` Surface Syntax

K1.17 SHALL NOT freeze human-facing dynamic timer migration or upgrade syntax. Future `.noi`
frontends may express these certified runtime laws without changing their semantic protocol.
