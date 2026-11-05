# NORDOI Production P0.6C — Authenticated Remote Anchor Client

P0.6C adds a concrete `MonotonicAnchor` client backend without changing the already certified P0.6B dispatch path.

The client is deliberately small:

- blocking `std::net::TcpStream` only;
- one bounded fixed-size binary protocol;
- Ed25519-authenticated requests and responses;
- fresh OS-generated 256-bit request id on every operation;
- response signature is bound to that request id;
- connection reuse with TCP_NODELAY;
- bounded read/write/connect timeouts;
- no automatic CAS retry after ambiguous transport failure;
- no Tokio, HTTP, TLS framework, Serde, JSON, or async runtime.

P0.6C does **not** certify that a particular operator deploys the service in an independent failure domain, and does not certify the rollback-resistance of the remote service's own durable storage. Those remain deployment/backend-service properties.

Certification labels after a successful P0.6C gate must remain:

- `INDEPENDENT FAILURE-DOMAIN DEPLOYMENT: NOT CERTIFIED`
- `REMOTE SERVICE ROLLBACK-RESISTANT STORAGE: NOT CERTIFIED`
- `KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED`
- `PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED`
