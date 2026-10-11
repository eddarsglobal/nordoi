# R0.20 research boundary decision

Research-only synthetic trust-root lifecycle study. Base `r0.19` `834c20d68e86f0ed6ab14141e0f9f9e1df241901`; R0.19 evidence fixture `ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33`.

**Decision:** all mock registration, rotation and revocation paths are `RESEARCH_REVIEW_ONLY`; stale, replayed, forged, cross-scope and native paths are `BLOCKED` or `INVALID`. A root marked revoked is never resurrected.

No real cryptographic signature verification, independent third party, production trust root, production admission, or native authorization is implemented. External evidence 12/12 `ABSENT`, native gates 12/12 `UNMET`; this is strictly `TEST-ONLY / RESEARCH_ONLY`.

The binary transcript root is `2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817`.
