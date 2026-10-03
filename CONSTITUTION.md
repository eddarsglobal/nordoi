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
