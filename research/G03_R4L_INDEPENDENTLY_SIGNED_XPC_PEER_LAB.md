# NORDOI G0.3 R4L — Independently signed inbound XPC peer admission lab

**RESEARCH · DRAFT · DENY BY DEFAULT · NO PRODUCT AUTHORIZATION**

## Parent provenance
Exact accepted-with-controls owner macOS 14.5 Intel R4K FIX3: `5c2fa93cd3dbc729e0d09195ef1a0084a18f92f3`. That R4K parent produced 106/106 R4A–R4K tests PASS, 0 fail, 3 intentionally ignored, rustfmt PASS and Clippy strict PASS on owner macOS 14.5 Intel. Its native dual macOS 15 CI + portable CI PASS. R4A–R4K PRs (#56, #58, #60, #62, #64, #66, #68, #70, #72, #74, #75) are draft/unmerged; this research PR stacks on #75 without modifying it or `main`.

## Scientific hypothesis
R4K tested signature requirements on the service, and independently verified the received *service*'s signed identity on the client. It **did not** establish that a separately signed *wrong client* can reach the same private XPC service and is rejected before the event handler. In earlier R4K exploration, a wrong-signed helper timed out, which is **not** a sufficient security witness.

R4L builds a real launchd-managed embedded XPC service, one actual app/client, and **two separate independently ad-hoc-signed sibling executables** from exactly the same source, under the same `.app/Contents/MacOS` directory and UID:

- **Allowed peer**: `r4l-good`, signed identifier `org.eddarsglobal.nordoi.r4l.client`.
- **Unauthorized peer**: `r4l-bad`, signed identifier `org.eddarsglobal.nordoi.r4l.impostor`.
- **Service requirement**: `xpc_connection_set_peer_code_signing_requirement` before resuming each inbound connection; allows only the expected identifier. Ad-hoc identifier is not a production identity trust anchor.

An unsandboxed signed coordinator maintains a live XPC connection to the service, reads its **accepted-ping counter**, starts and awaits the **good** signed sibling to demonstrate live accessibility and increases count by one, then starts the **bad** signed sibling. The bad peer must observe an actual XPC error; no dictionary `pong` and no silent timeout are accepted. The coordinator queries **the same live service PID** again: its accepted ping counter must be **unchanged** after the bad peer. Both sibling signatures must verify and identify as different before launch. Service remains independently sandboxed without extra file privileges.

This combines **positive reachability** of a separately signed sibling and **negative non-delivery** to the service handler, avoiding the earlier misleading `timeout=denial` conclusion.

## Required fail-closed verdicts
1. If the good sibling cannot access the private service (e.g. private launchd app association), FAIL — the bad peer's nonresponse alone is inconclusive.
2. If the bad peer returns `pong`, FAIL — signature admission bypassed.
3. If the bad peer times out or crashes, FAIL — no explicit XPC error witness.
4. If the service's counter increments after the bad peer, FAIL — unauthorized handler access.
5. If service PID changes, either signature invalid, sandbox entitlement missing, or sideband receipts malformed, FAIL.
6. If no Apple native environment, only portable model external-IO default denial is tested; never claim native XPC proof.

## Intentional limitations
An `identifier` constraint without a trusted Developer ID / Team ID, full hardened runtime, entitlements and update security is NOT proof of trusted production identity. Client signature creation is ad-hoc and deliberately disposable. The test does not implement complete audit-token authorization for inbound messages, secure file-descriptor transfer, replay resistance, real host resource mediation, symlink/TOCTOU protection, or execution of untrusted `.noi`. The OS may return `XPC_ERROR_CONNECTION_INVALID` rather than an explicit signature requirement error on an inbound denied peer: **this test only claims denial of handler delivery**, and requires the signed-positive reachability and before/after counter evidence.

R4L does not grant external IO authority in the NORDOI reference model and changes no public syntax. Native admission gates = 12 UNMET. No merge, release, install, or production activation.

```ini
G03_R4L=RESEARCH_CANDIDATE
SOURCE_PARENT_R4K=5c2fa93cd3dbc729e0d09195ef1a0084a18f92f3
INDEPENDENT_WRONG_SIGNED_CLIENT=CI_PENDING
POSITIVE_SIBLING_REACHABILITY=CI_PENDING
SERVER_HANDLER_COUNT_UNCHANGED=CI_PENDING
OWNER_MAC_INTEL_14_5=PENDING
PRODUCTION_SIGNING_TRUST_ANCHOR=NONE
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

### Apple API references
- https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_code_signing_requirement
- https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_team_identity_requirement
- https://developer.apple.com/documentation/security/seccodecheckvalidity


## R4L early CI finding — per-client instances, not a shared counter

Initial native Apple Silicon CI: signature fixture **PASS**, portable default-denial **PASS**, but actual peer exercise **REMEDIATE**. The separately signed positive sibling received a successful XPC reply, while the original coordinator still observed `COUNT=0` for its *own* service instance. This is consistent with Apple's documented **one embedded XPC service process per client** architecture; a coordinator's counter cannot establish whether another client's per-client service instance ran its handler. Hence the originally proposed shared service counter is **NOT VALID EVIDENCE** for a separate client's admission; no PASS is awarded from unchanged counter values.

The revised strict R4L scenario demands (a) two code-signing identities verified independently for binaries built from the same C source and directory; (b) the legitimate sibling's real `pong`; (c) the intentionally wrong-signed sibling receiving exactly `XPC_ERROR_PEER_CODE_SIGNING_REQUIREMENT`, not a generic XPC error and never a timeout, crash or dictionary reply; (d) no unexpected change of the coordinator's own service instance. The latter is only an isolation sanity check, **not a denial witness**. The *actual identity mismatch witness* is strictly the specific XPC signing error for the wrong-signed sibling.

**Pending evidence**: we have not yet observed whether the native XPC service returns that exact error to an initiating client on either macOS 15 or owner macOS 14.5. Apple's documentation describes a nonconforming initiating peer's message being silently withheld; if it returns a generic XPC error instead, this strict test **must remain REMEDIATE** until a conclusive denial mechanism can be demonstrated. Do not broaden the acceptance gate to count a generic connection failure.

Reference: https://developer.apple.com/documentation/xpc
