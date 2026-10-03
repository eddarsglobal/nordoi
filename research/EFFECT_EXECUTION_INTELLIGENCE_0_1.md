# NORDOI Language Intelligence — Effect Execution Boundary 0.1

**Research target:** K1.6 Governed Effect Outbox & Dispatch Core

NORDOI's Master Law requires research before freezing a major semantic mechanism. This
note records the design lessons used for K1.6.

## WASI / WebAssembly capability design

Observed strength:

- external resources are made available through explicit capabilities rather than an
  ambient global authority model;
- resource handles are intended to be unforgeable and passed explicitly;
- link-time capabilities remain refusable/interposable.

NORDOI adoption:

- no serialized NAIR instruction may grant itself host authority;
- a backend is reachable only through an explicit host dispatch boundary;
- external authority remains separate from program bytes.

NORDOI difference:

- K1.6 separates deterministic intent publication from environmental delivery and
  re-checks authority at delivery time.

## Deno permission model

Observed strength:

- sensitive system access is denied by default;
- permissions can be scoped to concrete resources such as paths or network hosts;
- permissions can be explicitly denied/revoked.

NORDOI adoption:

- `EffectDispatchAuthority` begins with no privilege;
- exact effect scope must match exact capability scope;
- a queued intent can be blocked by authority revocation before dispatch.

NORDOI difference:

- a queued effect intent is not itself a permission and cannot prompt itself into
  privilege.

## Capsicum

Observed strength:

- capability-oriented authority is represented explicitly rather than inferred from a
  broad ambient process privilege.

NORDOI adoption:

- authority is a separate object passed by the host;
- capability presence is required at the operation boundary.

NORDOI difference:

- K1.6 remains platform-neutral and defines no FreeBSD-specific backend contract.

## Koka effect types and handlers

Observed strength:

- effect information is part of the program's semantic contract;
- the act of declaring/performing an effect can be separated from the handler that
  interprets it.

NORDOI adoption:

- reaction actions declare exact effects;
- K1.5 creates validated intentions;
- K1.6 interprets external intentions only at a separate governed host handler/backend.

NORDOI difference:

- K1.6 is not yet a general algebraic-control-effect system. Its immediate purpose is
  secure external effect delivery from the Atomic Machine.

## Structural conclusion

The common lesson is that external authority should not be ambient, implicit or
self-granted. NORDOI adds another requirement: deterministic program meaning must not
be made dependent on non-deterministic backend completion.

Therefore K1.6 freezes this architecture:

```text
declared effect
  -> exact reaction authority
  -> deterministic effect intent
  -> atomic semantic outbox
  -> exact current dispatch authority
  -> host backend
  -> environmental result
```

No host result may silently mutate NAM. Completion re-entry requires a later governed
semantic cause contract.
