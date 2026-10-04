# Runtime Timer Migration Intelligence 0.1

K1.16 research note.

Timer migration is more dangerous than copying a scalar because a timer simultaneously represents
future work, a logical-time position, an occurrence sequence and a reaction-routing identity.
Preserving only its deadline can duplicate or erase semantic work; preserving its runtime ID can
bind the new program's reactions to the wrong resource.

NORDOI therefore separates **semantic timer continuity** from **runtime timer identity**:

```text
semantic continuity = deadline + interval + occurrences + canceled/pending state
runtime identity     = target TimerId bound to target TimerSlot
```

A migration plan maps source slots to target slots explicitly. The target ID is retained while the
source schedule state is carried. This lets the target reaction graph remain authoritative while the
source temporal progress continues.

The protocol refuses dynamic pending timers because they have no program-level slot identity to map.
A later resource-migration layer may govern those timers, but K1.16 does not infer intent from raw
`TimerId` values.

The same principle generalizes beyond timers: future upgrade protocols for subscriptions, streams,
GPU resources, sockets or distributed leases should migrate semantic continuity through explicit
resource identity mappings rather than copying opaque host handles.
