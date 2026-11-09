# G0.3 R4U — Public research-only SSHSIG observer ledger

These are **actual detached Ed25519 SSHSIG signatures**, produced with cryptographically random ephemeral keys using the open-source `cryptography` Ed25519 implementation and the published OpenSSH SSHSIG wire protocol. The ephemeral private key bytes were **never stored or committed**. This fixture cannot sign future operational checkpoints.

`evidence_pack.txt` contains a guardian's public key, a witness-observer's different public key, a signed epoch-3 checkpoint, a signed epoch-4 checkpoint linked by the exact predecessor SHA-256, both guardian and observer signatures on the accepted epochs, and a deliberately conflicting epoch-4 fork signed by the guardian **only**. The observer *did not* sign the fork.

- Exact evidence pack SHA-256: `692809e55aed8ad188208e04653d3bf4fdb9945aa7fa661db6ee22fe8100400d`
- Epoch-2 anchor digest: `f69ab4ba094c555b0064e2128e1a90458abb82a2ea652e1a149fa217069e7aa8`
- Signed epoch-3 digest: `11e000dc42cf66554c46d68a385959d0098bc516925b6bf33afa5bbb18f5b3c8`
- Signed epoch-4 digest: `65b71123804659cea8b334c783f51d7e63ad3c8283057c22e432a8a54f6c9d1c`
- Conflicting epoch-4 fork digest: `989377b80da386e7344aefa514614853da8e832b1bec826681dce96ffa218e0a`

**Crucial:** This GitHub evidence branch is administered by the SAME `eddarsglobal` account as the main project. Both fixture private keys were owned by the SAME lab process. The names 'guardian' and 'observer' do not make them independent organizations or operators. This is NOT a transparent append-only log, hardware counter, secure external principal, trusted remote signing service, production custody, or real out-of-band owner enrollment.

The verification research test must pin both public keys and the immutable Git commit independently of the bytes it downloads. A newly authenticated owner-selected externally administered witness is required before any independent trust certification. A malicious administrator capable of changing the verifier, git evidence branch and pins can bypass this research guard. Any remote outage must fail closed.

The public artifact contains no secrets and grants no runtime host authority.
