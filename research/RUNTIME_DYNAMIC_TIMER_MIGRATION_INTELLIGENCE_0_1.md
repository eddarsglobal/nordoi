# Runtime Dynamic Timer Migration Intelligence 0.1

K1.17 research note.

## Why this milestone exists

K1.16 completed native timer continuity but correctly rejected dynamic timers. The distinction is
important: native timers have program-level `TimerSlot` identity, while dynamic timers have only
runtime allocation identity.

The safest next step is not to pretend that raw IDs suddenly have declarative meaning. Instead,
K1.17 treats dynamic timer migration as an explicit runtime-resource mapping bound to a durable
source state and an authorized program transition.

## Identity preservation vs remapping

Dynamic timers differ from native timers because there is no target `TimerSlot` whose identity must
replace the source ID. Preserving the same runtime ID is therefore meaningful when possible.

However, target native bootstrap may reserve that same numeric identity. A deterministic explicit
remap is needed for those cases.

A remap must mint a fresh identity rather than recycle a canceled or unrelated timer. This motivates
the allocation-frontier freshness rule.

## Why target native bindings stay reserved after cancellation

A canceled target native timer may no longer be pending, but its `TimerId` is still the identity
resolved by native reactions for its `TimerSlot`. Allowing a dynamic timer to occupy that ID would
silently change target reaction meaning.

Reservation therefore follows binding identity, not current pending state.

## Replay boundary

The migration mapping itself is semantic because future timer fires contain the target `TimerId` and
therefore affect reaction matching and replay traces. The mapping must be committed by the upgrade
plan hash.

Transport/audit metadata remains outside replay meaning.

## Future work

K1.17 is still not a general resource migration system. Similar principles may later be applied to
other governed runtime resources, but only after each resource class has explicit identity,
authority, durability and replay laws.
