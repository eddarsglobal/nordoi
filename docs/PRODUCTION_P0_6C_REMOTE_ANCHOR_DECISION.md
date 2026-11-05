# P0.6C Decision — Keep the remote anchor backend isolated and minimal

## Decision

Add a new standalone crate, `nordoi-remote-anchor-p06c`, that implements the existing P0.6A `MonotonicAnchor` trait. Do not add networking dependencies to the P0.6A contract crate and do not modify P0.6B's certified dispatch ordering.

## Performance / lightness rule

Remote anchoring is a production-security facility, not a language-runtime dependency. NORDOI's compiler and ordinary runtime must not pay for it when unused. The backend therefore uses only blocking `std::net::TcpStream`, a fixed-size binary protocol, Ed25519, and OS randomness. It intentionally does not pull in an async runtime, HTTP client, serialization framework, or general RPC stack.

A healthy TCP connection is reused. TCP_NODELAY is enabled. Connect/read/write operations have explicit timeouts. A failed compare-and-advance is never automatically retried because an interrupted response can represent an already committed remote transition.

## Authentication and replay binding

Every request is Ed25519-signed by the configured client key. Every response is Ed25519-signed by the configured server key. Every operation carries a fresh 256-bit request id from the operating system CSPRNG, and the signed response must contain the exact id. A recorded response therefore cannot be accepted for a different operation.

Replaying a signed compare-and-advance request against a correct atomic service cannot re-apply it after the first successful transition because the expected record no longer matches.

## Scope boundary

P0.6C can certify the concrete client implementation, wire bounds, authentication, request/response binding, fail-closed error mapping, no hidden CAS retry, and compatibility with P0.6A recovery.

P0.6C cannot certify from CI alone that:

1. the actual server is deployed outside the local rollback/failure domain;
2. the server's own durable store cannot itself be rolled back;
3. all historical kernel effect paths are globally forced through the protected profile;
4. recipient-side exactly-once effect delivery exists.

Those claims remain explicitly forbidden until separately evidenced.
