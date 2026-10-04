# LAW 0001 — The NORDOI Master Law

**Status:** Foundational / Constitutional  
**Applies to:** Language, NAM, NAIR, compiler, runtime, standard library, tooling, agents, backends, packages, governance and community evolution.

## Founder's performance declaration

> **"NORDOI est le plus leger langage au monde et le plus vite dans l'univer"**

Canonical English mission:

> **NORDOI shall pursue becoming the lightest programming language in the world and the fastest in the universe.**

This sentence is a constitutional engineering target, not permission to make an unmeasured marketing claim. NORDOI agents must continuously attempt to prove it with reproducible benchmarks, transparent methodology and comparisons across relevant workloads and platforms.

---

## 1. Why NORDOI exists

NORDOI exists because the computing world is entering an era in which humans, AI/SI agents, heterogeneous processors, autonomous machines, spatial computing and distributed systems must cooperate through programming foundations that were mostly designed for earlier eras.

NORDOI does not exist merely to introduce another syntax.

NORDOI exists to rebuild the path from **intention to verified execution** while refusing unnecessary historical complexity, unsafe defaults, avoidable runtime work and platform confinement.

Its fundamental mission is:

> **Minimize the distance between intelligence and verified machine execution.**

NORDOI is designed for humans, AI/SI and machines at the same time.

---

## 2. NORDOI is larger than a language surface

To a developer, NORDOI should feel small, coherent and easy to learn.

Internally, NORDOI is a complete programming architecture composed progressively of:

- the NORDOI language surface;
- canonical semantic structures;
- a type and proof system;
- an effect system;
- a capability-security system;
- ownership and transaction semantics;
- NAIR — NORDOI Atomic Intermediate Representation;
- NAM — NORDOI Atomic Machine;
- an atomic scheduler;
- an atomic memory model;
- an atomic renderer;
- CPU/GPU/NPU execution planning;
- 2D/3D/VR/AR/XR execution;
- networking and distributed execution;
- compiler and optimizer infrastructure;
- packages and reproducible builds;
- verification and adversarial-testing infrastructure;
- agent-assisted research and evolution;
- multiple platform backends.

The visible syntax is the simple surface of a deep engine.

---

## 3. Universal Execution is constitutional

NORDOI must never be architecturally confined to the Web.

The architecture must progressively be capable of targeting:

- Web;
- desktop;
- mobile;
- servers and command-line software;
- 2D and 3D;
- games;
- VR, AR and XR;
- GPU and general compute;
- AI workloads;
- networked and distributed systems;
- embedded devices;
- robotics and edge computing;
- selected system-level layers when justified.

The Web is one backend among others, never NORDOI's prison.

---

## 4. NORDOI does not begin where historical languages began

NORDOI agents must not assume that the historical development sequence of existing languages is the correct sequence for NORDOI.

Before freezing syntax, they must first ask fundamental questions:

- What is state?
- What is identity?
- What is ownership?
- What is authority?
- What is an effect?
- What is a resource?
- What is time?
- What is failure?
- What is concurrency?
- What is observation?
- What is dependency?
- What is a proof?
- What is execution cost?
- What is security at the semantic level?

The required development order is therefore biased toward:

```text
knowledge
  -> laws
  -> invariants
  -> semantics
  -> machine model
  -> NAIR
  -> compiler architecture
  -> language surface
```

Syntax must express a coherent machine, not define one accidentally.

---

## 5. Language Intelligence before invention

Before introducing a major mechanism, NORDOI agents must study relevant existing programming-language families and record:

1. the original problem being solved;
2. the mechanism chosen;
3. the mechanism's strengths;
4. its structural weaknesses;
5. its historical compatibility debt;
6. security consequences;
7. performance consequences;
8. cognitive cost for humans;
9. generation and verification cost for AI/SI;
10. what NORDOI can eliminate by construction.

NORDOI must learn from C, C++, Rust, Zig, Ada, Java, C#, Go, Swift, Kotlin, Python, JavaScript, TypeScript, Dart, Ruby, PHP, Lua, functional languages, actor languages, data languages, shader/GPU languages, WebAssembly, formal-verification languages and future paradigms.

The purpose is not imitation.

The purpose is to avoid rediscovering known failures and to identify unresolved problems before they become NORDOI debt.

---

## 6. Zero Legacy Debt

NORDOI must not preserve a weakness simply because previous languages historically contained it.

Compatibility with old systems may be provided through explicit boundaries and backends, but legacy behavior must not silently become core NORDOI semantics.

No historical convention has authority merely because it is old or popular.

---

## 7. Impossible States First

When a dangerous or invalid state can reasonably be made unrepresentable, NORDOI should make it unrepresentable instead of merely documenting it or warning after the fact.

Examples of target classes include:

- accidental nullability;
- uninitialized values;
- invalid memory access;
- use-after-free;
- double-free;
- dangling references;
- silent lossy conversions;
- accidental integer overflow modes;
- undeclared effects;
- implicit privilege escalation;
- uncontrolled shared mutation;
- secret-data leakage;
- invalid units and dimensions;
- partially committed state.

Warnings are weaker than invariants. Runtime crashes are weaker than prevention.

---

## 8. Atomic Speed

Atomic means that NORDOI seeks the smallest necessary amount of work for every observable result.

NORDOI must optimize toward:

- minimum computation;
- minimum state mutation;
- minimum allocation;
- minimum copying;
- minimum serialization;
- minimum synchronization;
- minimum CPU/GPU transfer;
- minimum invalidation;
- minimum redraw;
- minimum startup work;
- minimum latency.

The governing rules are:

> **No Work Without Effect.**

and:

> **What You Do Not Use Must Cost Nothing.**

Unused capabilities and subsystems should contribute no meaningful runtime cost whenever architecture permits it.

Atomic Speed must be measured rather than assumed.

---

## 9. Atomic Security

Security is not an optional library layer.

It belongs to NORDOI semantics and the Atomic Machine.

The default authority model is:

```text
network     denied
filesystem  denied
camera      denied
microphone  denied
location    denied
process     denied
```

Sensitive behavior requires at least:

```text
declared intent/effect
        +
explicit narrow authority
        +
policy validation
```

Possessing a capability does not authorize undeclared behavior.

NORDOI must pursue memory safety, least privilege, capability isolation, sandboxing, effect checking, information-flow protection, signed/reproducible builds, supply-chain integrity and formal verification where practical.

No system can scientifically promise invulnerability against every possible future attack, hardware failure, stolen credential or external compromise. Therefore NORDOI's security mission is stronger and testable: eliminate entire vulnerability classes by construction wherever possible and minimize the authority available to anything that fails.

---

## 10. Simple Surface, Deep Engine

NORDOI's internal sophistication must not become developer burden.

Any complexity that can safely and predictably be absorbed by the compiler, NAM, tooling or verifier should not be forced onto the programmer.

A simple action should remain simple even if the system internally performs:

- type validation;
- effect validation;
- ownership validation;
- capability checking;
- dependency analysis;
- memory planning;
- incremental updates;
- rendering optimization;
- accessibility generation;
- platform adaptation;
- security checks;
- performance planning.

NORDOI should be easy to begin, deep to master and pleasant to read.

---

## 11. Human + AI/SI + Machine native

NORDOI must not merely be easy for an LLM to autocomplete.

Human source, AI-generated programs, visual tools and future intelligent systems should converge on canonical semantics and NAIR.

AI/SI may generate, analyze, optimize and verify NORDOI, but ordinary compiled applications must not require an AI model merely to execute ordinary application logic.

> **AI-native does not mean AI-dependent.**

---

## 12. NAM — NORDOI Atomic Machine

NAM is the execution model that gives NORDOI its fundamental guarantees.

It progressively defines execution in terms such as:

- atoms;
- state;
- ownership domains;
- effects;
- capabilities;
- dependencies;
- resources;
- transactions;
- tasks;
- spaces;
- data flows;
- proofs and validated contracts.

NAM must remain independent enough that NORDOI can evolve without being permanently trapped by one operating system, browser, runtime, processor architecture or vendor.

---

## 13. NAIR — canonical execution meaning

NAIR is the canonical intermediate representation between NORDOI semantics and execution backends.

Its purpose is to let multiple frontends and multiple machines converge on the same meaning:

```text
Human Source ----\
AI/SI ------------+--> Semantic NORDOI --> NAIR --> NAM/backends
Visual Tools -----/
```

NAIR must be deterministic, inspectable, optimizable, security-aware and suitable for verification.

---

## 14. Memory without historical traps

NORDOI must search for a memory model that combines:

```text
safety
+ predictability
+ speed
+ simplicity
```

It must not blindly copy garbage collection, ownership/borrowing, reference counting or manual memory management merely because they already exist.

Each mechanism must be judged against NORDOI's laws.

The default safe language must not expose arbitrary memory corruption as ordinary behavior.

---

## 15. Concurrency without accidental races

Threads, mutexes and locks must not become unavoidable surface complexity for everyday code.

NORDOI should favor higher-level semantics such as tasks, transactions, structured concurrency, actors/isolated ownership or other mechanisms selected from evidence.

Shared mutable state should require an explicit and verifiable coherence model.

---

## 16. 2D, 3D and XR belong to one spatial model

NORDOI rejects the assumption that interface, 2D, 3D and XR must be completely separate programming worlds.

They should progressively converge on a coherent spatial model capable of expressing screen space, world space, camera space, hand/controller space and other coordinate domains.

The developer should not need a different programming philosophy merely because an object moved from a screen to an XR environment.

---

## 17. Heterogeneous compute is native

Modern machines contain CPU, GPU, NPU and specialized accelerators.

NORDOI should understand execution cost and placement sufficiently to map work to appropriate hardware where it can do so safely and profitably.

Boilerplate for buffers, pipelines, synchronization and transfers should be absorbed when the system can infer and verify it.

---

## 18. Security, correctness and performance are evidence systems

Core claims require evidence.

NORDOI evolution should progressively use:

- unit and integration tests;
- property testing;
- fuzzing;
- adversarial generation;
- model checking;
- static analysis;
- formal proof where useful;
- reproducible benchmarks;
- differential testing;
- deterministic builds;
- red-team review.

The system must distinguish aspiration from proven guarantee.

---

## 19. Self-improving under governance

NORDOI must continuously learn from:

- new processors;
- new attack classes;
- new compiler techniques;
- research papers;
- new programming paradigms;
- new AI/SI architectures;
- failures found in NORDOI itself;
- community experience.

Agents may propose evolution automatically, but they must not silently rewrite constitutional semantics.

Evolution follows evidence, tests, adversarial review and governance.

---

## 20. Community Evolvable, Constitutionally Governed

NORDOI must be open to a community capable of contributing to:

- compiler development;
- libraries;
- backends;
- documentation;
- examples;
- tooling;
- IDE integration;
- GPU/XR support;
- benchmarks;
- security research;
- proposals.

The architecture should allow contributors to work on one subsystem without understanding every detail of NAM.

Community participation should be easy. Core guarantees should be difficult to weaken accidentally.

---

## 21. Joy of Programming

Power is not enough.

NORDOI should be readable, teachable, debuggable, discoverable and enjoyable.

The desired reaction to ordinary NORDOI code is:

> **"I can almost understand this before learning NORDOI."**

Novelty must never be confused with syntactic obscurity.

---

## 22. NORDOI must discover problems, not only inherit them

Agents must work across three categories:

```text
KNOWN PROBLEMS
  -> eliminate them

KNOWN UNSOLVED PROBLEMS
  -> search for better solutions

UNKNOWN PROBLEMS
  -> actively try to discover them
```

NORDOI's red-team and research processes must attempt to find weaknesses before they become ecosystem-wide debt.

---

## 23. The NORDOI complexity inversion

NORDOI aims for:

```text
internally extremely sophisticated
externally extremely simple
```

The engine should become smarter so that application code can become smaller and clearer.

The community should not be forced to pay for internal complexity with everyday cognitive complexity.

---

## 24. The NORDOI performance law

The founder's declaration — **"NORDOI est le plus leger langage au monde et le plus vite dans l'univer"** — creates a permanent engineering obligation.

Every architecture decision must ask:

- Can this representation be smaller?
- Can this operation be eliminated?
- Can this state be known at compile time?
- Can this allocation disappear?
- Can this dependency be removed?
- Can this transfer be avoided?
- Can this effect be proven earlier?
- Can this feature cost zero when unused?
- Can startup be shorter?
- Can latency be lower?

If a heavier mechanism is required for correctness or security, agents must measure the tradeoff and investigate whether a better mechanism can achieve all three.

NORDOI does not sacrifice correctness for benchmark theater. Its target is **verified speed**.

---

## 25. Final master law

All NORDOI subsystems must serve the following equation:

```text
INTELLIGENCE / INTENT
        ↓
      NORDOI
        ↓
CANONICAL SEMANTICS
        ↓
PROOF + SAFETY + EFFECTS + OWNERSHIP
        ↓
       NAIR
        ↓
       NAM
        ↓
ATOMIC OPTIMIZATION
        ↓
CPU · GPU · NPU · WEB · NATIVE · XR · EMBEDDED · SYSTEM
```

The permanent objective is:

> **Understand more. Prove more. Expose less complexity. Perform less unnecessary work. Execute faster. Remain secure. Evolve continuously.**

NORDOI is not designed for the limitations of yesterday's computing.

**NORDOI is designed for the intelligence and machines of tomorrow.**

---

## 26. Governed distributed ownership law

Durability alone does not establish exclusive authority. Whenever recoverable NORDOI state may be
opened by multiple host writers, ownership must be explicit, revocable and stale-writer safe.

For the K1.8 effect journal this means:

```text
semantic delivery identity != writer ownership epoch
```

The semantic intent keeps a stable delivery key across recovery. The current host writer carries a
monotonic fencing epoch. New ownership must supersede old ownership, and protected persistence
mutations must reject stale epochs.

NORDOI shall not hide this problem behind local mutexes, process IDs, wall-clock assumptions or
claims of universal exactly-once delivery. Cross-system guarantees require cooperation from every
boundary that participates in the guarantee.

---

## 27. Governed retry and poison-effect law

Durable delivery must distinguish transient failure from permanently or repeatedly failing work.
NORDOI shall not solve external failure with hidden infinite loops.

For K1.9:

```text
external failure
      ↓
explicit retry classification
      ↓
bounded deterministic backoff
      ↓
durable retry state
      ↓
retry success OR durable dead-letter quarantine
```

Retry scheduling must remain explicit, fenced and persistence-first. A poison effect may not hold
unrelated later eligible work hostage forever. Dead-lettering is quarantine, not deletion, and
redrive must preserve the original semantic delivery identity.

The core must continue to avoid ambient time and ambient randomness. Hosts may map explicit retry
ticks to real scheduling infrastructure, but policy and state transitions remain inspectable and
recoverable.

---

## 28. Crash-consistent semantic runtime law

Durable execution must not be simulated by serializing arbitrary process memory or by allowing
storage to become ambient authority. A durable NORDOI runtime shall reconstruct static program
structure from canonical NAIR and persist only the mutable semantic state required for exact
continuation.

For K1.14:

```text
candidate semantic cycle
      ↓
effect/audit checkpoint + runtime checkpoint
      ↓
one fenced atomic host commit
      ↓
live publication
```

Recovery must bind the exact canonical program, preserve replay identity, retain input/timer/NAM and
completion-deduplication progress, and reject partial bundles, stale fences, audit forks, identity
frontier mismatches and incompatible boot configuration. Recovery is continuation, not a new
semantic event.

Host authority remains host authority. A checkpoint may restore facts and identities, but it may not
mint capabilities, credentials or completion-source permissions.

A newer fence is not permission to erase a previously durable semantic lineage. If a complete
runtime bundle already exists, a fresh process must recover it before replacement. Partial legacy
state must fail closed unless a separately governed migration protocol explicitly handles it.

---

## 29. Governed program evolution law

Durable state must not make a NORDOI program impossible to evolve, but evolution must never become a
backdoor around deterministic recovery.

For K1.15:

```text
exact source program + durable state
        ↓
explicit source→target authority
        ↓
total deterministic migration plan
        ↓
private target boot + migration
        ↓
new program epoch + cryptographic lineage
        ↓
one fenced durable bundle commit
        ↓
target publication
```

Ordinary recovery still requires the exact program that produced the checkpoint. Upgrade is a
separate semantic act and therefore changes replay identity.

The migration core must prefer explicit loss over silent loss: every source atom is copied or
dropped explicitly, and every target atom is copied or explicitly retains its target default. The
core may not execute arbitrary host scripts as hidden migration authority.

External effect identity, completion deduplication, logical time and input sequence frontiers survive
program replacement. Pending source timers remain a fail-closed boundary until NORDOI certifies a
stable timer migration protocol.
