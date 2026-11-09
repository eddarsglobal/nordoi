# R4P external checkpoint evidence — research ONLY

SSH SIG Ed25519 V1 epoch=1/seq=1, V2 epoch=2/seq=2 are public, separately signed checkpoints. Public signing key is included solely for transparency. The key was generated ephemerally offline and its private material was **never committed** or retained. Thus this research signature **cannot be used to operate ongoing production releases**.

This evidence branch is remotely hosted by GitHub, **outside the owner's local state files**. The R4P verifier pins a GitHub branch tip commit SHA and checks both the pinned public signer bytes and signature. A same-device adversary able to restore local files, but without GitHub control, cannot use those restored copies to alter this remote snapshot. GitHub repository administrators or someone able to rewrite the witness branch are **NOT** independent security principals; full multi-authority witness and platform rollback protections remain absent.

The intentionally *old* V1 SSHSIG is authentic but **must be rejected** against the pinned remote V2 head, even after BOTH local cursors are restored to V1.

These data are not an owner-enrolled trust root, not a live attestation endpoint, not a monotonic secure counter and not production authorization. Do not merge this branch or remove research controls without explicit governance.
