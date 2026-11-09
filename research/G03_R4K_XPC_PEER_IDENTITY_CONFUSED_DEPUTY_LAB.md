# NORDOI G0.3 R4K — XPC peer identity and confused-deputy boundary (research)

**DRAFT RESEARCH · NOT PRODUCTION · FAIL CLOSED · NO `.noi` LANGUAGE CHANGE**

## Provenance
Exact parent R4J: `5bb3eaeea558e25de82422e9566933493260c923`. R4J owner macOS 14.5 Intel receipt: **98 regular R4A–R4J tests PASS, 3 helper entries ignored, Rustfmt PASS, Clippy PASS**. The stacked R4A–R4J draft PRs #56/#58/#60/#62/#64/#66/#68/#70/#72/#74 remain unmerged. R4K is additive only and targets a new draft PR.

## Hypothesis / test design
R4J tested genuine independently signed, launchd-activated XPC process and kernel sandbox denial, but **not peer signing identity**. R4K adds a server-side message-bound check using `SecCodeCreateWithXPCMessage(msg, …)`, a designated-client signing identifier requirement, and `SecCodeCheckValidity`. It never trusts a user-supplied PID or identity to establish OS process provenance. It also rejects a `probe` message carrying extra purported `grant`, `authority` or `identity` fields, and keeps unknown operations (including `writefile`) denied. The client and service still have different sandbox privileges; the service must remain unable to read/write/create the disposable owner file. Four exact, unique client receipt lines are expected and a separate service PID is necessary but not sufficient. Any unavailable XPC registration or broken attestation MUST fail the positive test.

The test app and XPC service are signed ad hoc; this is **not** developer-team identity, trusted release signing, notarization, remote party authorization, or an immutable production trust anchor. In particular an adversary who can replace and re-sign the entire disposable test bundle could mimic the signing identifier. Service identity from the *client* side and an actual independent wrong-signed-client rejection are **not yet covered** in the first candidate and must remain **REMEDIATE** rather than be claimed PASS.

## Controls and limitations
- No external host IO grant and no execution of user-supplied `.noi` code.
- No arbitrary path authority. Probe accepts paths solely to test OS sandbox denial; it is *not* a safe path-grant protocol. Missing canonicalization, symlink, file-descriptor and TOCTOU defenses are not silently accepted.
- No networking, sudo, global launchd service, Mac system configuration, unapproved filesystem access or source modifications outside this draft branch.
- macOS Intel and ARM runners must actually compile and execute the native XPC exchange; Linux/Windows can only assert unsupported behavior and default-denied `ResourceEffect::ExternalIo`.
- Keep all 12 native admission gates UNMET; no production activation or merge.

## Admission checklist
1. CI macOS Intel & ARM: genuine XPC reply and signature check **PENDING**.
2. Legitimate code-signed client: receipt 4 frames, distinct PID, known sandbox-denied OS calls **PENDING**.
3. Unknown `writefile` and forged grant/identity fields: denied **PENDING**.
4. Wrong-signed independent client and client-side service authentication **NOT IMPLEMENTED**.
5. Strict Clippy and Rustfmt; cumulative R4A–R4K full regression; owner macOS 14.5 replay **PENDING**.

## Primary references
- https://developer.apple.com/documentation/security/seccodecreatewithxpcmessage
- https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_code_signing_requirement
- https://developer.apple.com/forums/thread/681053

```ini
G03_R4K=RESEARCH_CANDIDATE
PARENT_R4J=5bb3eaeea558e25de82422e9566933493260c923
PEER_MESSAGE_BOUND_SIGNING_CHECK=CI_PENDING
WRONG_SIGNED_CLIENT_NEGATIVE_TEST=NOT_IMPLEMENTED
TRUSTED_DEVELOPER_ID=NONE
CONFUSED_DEPUTY_FULL_HARDENING=NOT_IMPLEMENTED
OWNER_MAC_INTEL=PENDING
NATIVE_ADMISSION_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```
