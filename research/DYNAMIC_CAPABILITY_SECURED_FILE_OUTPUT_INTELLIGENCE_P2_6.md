# Dynamic Capability-Secured File Output Intelligence — P2.6

P2.6 deliberately composes proof-bearing layers rather than creating a second dynamic text engine or a second filesystem authority model.

The central architectural decision is that P2.4 is reused as the compile-time structured rendering contract, but P2.6 does not fabricate a `ConsoleWrite` host grant. Runtime expressions are evaluated directly from the certified V0.7 dynamic plans embedded by the P2.4 plan. Their results are rendered with the same canonical Int/Bool textual representation used by P2.4.

The completed byte string is then passed through a synthetic P2.5 static file plan. P2.5 therefore remains the sole create-new materialization boundary, including exact target capability checks, path restrictions, output limits and no-overwrite behavior.

This separation avoids hidden authority: the only host capability is `FileWrite(target)`. Console output is not a side effect of P2.6, and the P2.6 receipt never includes the host absolute output root.
