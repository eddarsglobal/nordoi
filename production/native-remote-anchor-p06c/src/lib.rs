#![forbid(unsafe_code)]
//! NORDOI Production Track P0.6C: authenticated remote monotonic-anchor client.
//!
//! This crate is deliberately small and blocking. It adds no async runtime, HTTP stack,
//! serialization framework, or dynamic protocol surface. The client speaks a bounded fixed-size
//! binary protocol over TCP, signs each request with Ed25519, binds each signed response to a fresh
//! OS-generated request id, and never automatically retries an ambiguous compare-and-advance.
//!
//! IMPORTANT: this crate certifies a concrete remote-anchor *client backend and protocol surface*.
//! It does not prove that any particular deployment places the remote service in an independent
//! failure/rollback domain, nor that the remote service itself is backed by rollback-resistant
//! hardware/storage. Those are deployment properties and remain outside P0.6C certification.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use nordoi_rollback_anchor_p06::{
    AnchorStoreError, ChainWitness, MonotonicAnchor, ProductionAnchorRecord,
};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const REQUEST_MAGIC: [u8; 8] = *b"NRAREQ01";
const RESPONSE_MAGIC: [u8; 8] = *b"NRARES01";
const PROTOCOL_VERSION: u8 = 1;
const OP_READ: u8 = 1;
const OP_COMPARE_AND_ADVANCE: u8 = 2;

const STATUS_OK: u8 = 0;
const STATUS_CONFLICT: u8 = 1;
const STATUS_CORRUPT: u8 = 2;
const STATUS_UNAVAILABLE: u8 = 3;

const RECORD_BYTES: usize = 8 + 8 + 32 + 8 + 32;
const SIGNATURE_BYTES: usize = 64;
const REQUEST_ID_BYTES: usize = 32;
const REQUEST_SIGNED_BYTES: usize =
    REQUEST_MAGIC.len() + 1 + 1 + REQUEST_ID_BYTES + 1 + RECORD_BYTES + RECORD_BYTES;
const REQUEST_BYTES: usize = REQUEST_SIGNED_BYTES + SIGNATURE_BYTES;
const RESPONSE_SIGNED_BYTES: usize =
    RESPONSE_MAGIC.len() + 1 + 1 + REQUEST_ID_BYTES + 1 + RECORD_BYTES;
const RESPONSE_BYTES: usize = RESPONSE_SIGNED_BYTES + SIGNATURE_BYTES;

/// Concrete P0.6C `MonotonicAnchor` backend speaking the bounded authenticated remote protocol.
///
/// The connection is reused while healthy to keep the critical path small. Any I/O/protocol
/// failure drops the connection and fails closed. In particular, compare-and-advance is never
/// retried automatically because the remote side may already have committed the request.
pub struct RemoteAnchorClient {
    address: SocketAddr,
    timeout: Duration,
    client_key: SigningKey,
    server_key: VerifyingKey,
    stream: Option<TcpStream>,
}

impl RemoteAnchorClient {
    /// Construct a remote anchor client from raw Ed25519 key bytes.
    ///
    /// `client_secret` is the 32-byte Ed25519 signing-key seed. `server_public` is the expected
    /// 32-byte Ed25519 public key of the separately operated anchor service.
    pub fn new(
        address: SocketAddr,
        client_secret: [u8; 32],
        server_public: [u8; 32],
        timeout: Duration,
    ) -> Result<Self, AnchorStoreError> {
        if timeout.is_zero() {
            return Err(AnchorStoreError::Corrupt);
        }
        let server_key =
            VerifyingKey::from_bytes(&server_public).map_err(|_| AnchorStoreError::Corrupt)?;
        Ok(Self {
            address,
            timeout,
            client_key: SigningKey::from_bytes(&client_secret),
            server_key,
            stream: None,
        })
    }

    fn connect(&self) -> Result<TcpStream, AnchorStoreError> {
        let stream = TcpStream::connect_timeout(&self.address, self.timeout)
            .map_err(|_| AnchorStoreError::Unavailable)?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|_| AnchorStoreError::Unavailable)?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|_| AnchorStoreError::Unavailable)?;
        stream
            .set_nodelay(true)
            .map_err(|_| AnchorStoreError::Unavailable)?;
        Ok(stream)
    }

    fn exchange(
        &mut self,
        request: &[u8; REQUEST_BYTES],
    ) -> Result<[u8; RESPONSE_BYTES], AnchorStoreError> {
        if self.stream.is_none() {
            self.stream = Some(self.connect()?);
        }
        let mut response = [0_u8; RESPONSE_BYTES];
        let stream = self.stream.as_mut().ok_or(AnchorStoreError::Unavailable)?;
        let io_result = stream
            .write_all(request)
            .and_then(|_| stream.flush())
            .and_then(|_| stream.read_exact(&mut response));
        if io_result.is_err() {
            self.stream = None;
            return Err(AnchorStoreError::Unavailable);
        }
        Ok(response)
    }

    fn request(
        &mut self,
        operation: u8,
        expected: Option<ProductionAnchorRecord>,
        next: Option<ProductionAnchorRecord>,
    ) -> Result<Option<ProductionAnchorRecord>, AnchorStoreError> {
        let mut request_id = [0_u8; REQUEST_ID_BYTES];
        getrandom::fill(&mut request_id).map_err(|_| AnchorStoreError::Unavailable)?;

        let mut request = [0_u8; REQUEST_BYTES];
        let mut cursor = 0;
        put(&mut request, &mut cursor, &REQUEST_MAGIC);
        request[cursor] = PROTOCOL_VERSION;
        cursor += 1;
        request[cursor] = operation;
        cursor += 1;
        put(&mut request, &mut cursor, &request_id);
        request[cursor] = u8::from(expected.is_some());
        cursor += 1;
        if let Some(record) = expected {
            let encoded = encode_record(record);
            put(&mut request, &mut cursor, &encoded);
        } else {
            cursor += RECORD_BYTES;
        }
        if let Some(record) = next {
            let encoded = encode_record(record);
            put(&mut request, &mut cursor, &encoded);
        } else {
            cursor += RECORD_BYTES;
        }
        if cursor != REQUEST_SIGNED_BYTES {
            return Err(AnchorStoreError::Corrupt);
        }
        let signature = self.client_key.sign(&request[..REQUEST_SIGNED_BYTES]);
        request[REQUEST_SIGNED_BYTES..].copy_from_slice(&signature.to_bytes());

        let response = self.exchange(&request)?;
        self.decode_response(&response, &request_id)
    }

    fn decode_response(
        &self,
        response: &[u8; RESPONSE_BYTES],
        request_id: &[u8; REQUEST_ID_BYTES],
    ) -> Result<Option<ProductionAnchorRecord>, AnchorStoreError> {
        if response[..RESPONSE_MAGIC.len()] != RESPONSE_MAGIC[..] {
            return Err(AnchorStoreError::Corrupt);
        }
        let version_index = RESPONSE_MAGIC.len();
        if response[version_index] != PROTOCOL_VERSION {
            return Err(AnchorStoreError::Corrupt);
        }
        let status_index = version_index + 1;
        let id_start = status_index + 1;
        let id_end = id_start + REQUEST_ID_BYTES;
        if response[id_start..id_end] != request_id[..] {
            return Err(AnchorStoreError::Corrupt);
        }
        let signature_bytes: [u8; SIGNATURE_BYTES] = response[RESPONSE_SIGNED_BYTES..]
            .try_into()
            .map_err(|_| AnchorStoreError::Corrupt)?;
        let signature = Signature::from_bytes(&signature_bytes);
        self.server_key
            .verify_strict(&response[..RESPONSE_SIGNED_BYTES], &signature)
            .map_err(|_| AnchorStoreError::Corrupt)?;

        let present_index = id_end;
        let record_start = present_index + 1;
        let record_end = record_start + RECORD_BYTES;
        let record = match response[present_index] {
            0 => None,
            1 => Some(decode_record(&response[record_start..record_end])?),
            _ => return Err(AnchorStoreError::Corrupt),
        };
        match response[status_index] {
            STATUS_OK => Ok(record),
            STATUS_CONFLICT => Err(AnchorStoreError::Conflict),
            STATUS_CORRUPT => Err(AnchorStoreError::Corrupt),
            STATUS_UNAVAILABLE => Err(AnchorStoreError::Unavailable),
            _ => Err(AnchorStoreError::Corrupt),
        }
    }
}

impl MonotonicAnchor for RemoteAnchorClient {
    fn read(&mut self) -> Result<Option<ProductionAnchorRecord>, AnchorStoreError> {
        self.request(OP_READ, None, None)
    }

    fn compare_and_advance(
        &mut self,
        expected: Option<ProductionAnchorRecord>,
        next: ProductionAnchorRecord,
    ) -> Result<(), AnchorStoreError> {
        match self.request(OP_COMPARE_AND_ADVANCE, expected, Some(next))? {
            Some(committed) if committed == next => Ok(()),
            _ => Err(AnchorStoreError::Corrupt),
        }
    }
}

fn put<const N: usize>(target: &mut [u8], cursor: &mut usize, bytes: &[u8; N]) {
    let end = *cursor + N;
    target[*cursor..end].copy_from_slice(bytes);
    *cursor = end;
}

fn encode_record(record: ProductionAnchorRecord) -> [u8; RECORD_BYTES] {
    let mut out = [0_u8; RECORD_BYTES];
    let mut cursor = 0;
    put(&mut out, &mut cursor, &record.generation().to_be_bytes());
    put(
        &mut out,
        &mut cursor,
        &record.trust().position().to_be_bytes(),
    );
    put(&mut out, &mut cursor, &record.trust().digest());
    put(
        &mut out,
        &mut cursor,
        &record.delivery().position().to_be_bytes(),
    );
    put(&mut out, &mut cursor, &record.delivery().digest());
    out
}

fn decode_record(bytes: &[u8]) -> Result<ProductionAnchorRecord, AnchorStoreError> {
    if bytes.len() != RECORD_BYTES {
        return Err(AnchorStoreError::Corrupt);
    }
    let generation = read_u64(bytes, 0)?;
    let trust_position = read_u64(bytes, 8)?;
    let trust_digest: [u8; 32] = bytes[16..48]
        .try_into()
        .map_err(|_| AnchorStoreError::Corrupt)?;
    let delivery_position = read_u64(bytes, 48)?;
    let delivery_digest: [u8; 32] = bytes[56..88]
        .try_into()
        .map_err(|_| AnchorStoreError::Corrupt)?;
    Ok(ProductionAnchorRecord::new(
        generation,
        ChainWitness::from_parts(trust_position, trust_digest),
        ChainWitness::from_parts(delivery_position, delivery_digest),
    ))
}

fn read_u64(bytes: &[u8], start: usize) -> Result<u64, AnchorStoreError> {
    let end = start + 8;
    let raw: [u8; 8] = bytes
        .get(start..end)
        .ok_or(AnchorStoreError::Corrupt)?
        .try_into()
        .map_err(|_| AnchorStoreError::Corrupt)?;
    Ok(u64::from_be_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nordoi_rollback_anchor_p06::{
        initialize_production_anchor, verify_production_recovery, MonotonicAnchor,
        ProductionWitnessSource, RecoveryError,
    };
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;

    const CLIENT_SECRET: [u8; 32] = [11; 32];
    const OTHER_CLIENT_SECRET: [u8; 32] = [12; 32];
    const SERVER_SECRET: [u8; 32] = [21; 32];

    #[derive(Clone, Copy)]
    enum ReplyMode {
        Normal,
        TamperSignature,
        WrongRequestId,
    }

    fn record(generation: u64, trust_pos: u64, delivery_pos: u64) -> ProductionAnchorRecord {
        ProductionAnchorRecord::new(
            generation,
            ChainWitness::from_parts(trust_pos, [trust_pos as u8; 32]),
            ChainWitness::from_parts(delivery_pos, [delivery_pos as u8; 32]),
        )
    }

    fn client_public(secret: [u8; 32]) -> VerifyingKey {
        SigningKey::from_bytes(&secret).verifying_key()
    }

    struct TestServer {
        address: SocketAddr,
        state: Arc<Mutex<Option<ProductionAnchorRecord>>>,
        handle: thread::JoinHandle<()>,
    }

    fn spawn_server(
        initial: Option<ProductionAnchorRecord>,
        expected_client: VerifyingKey,
        request_count: usize,
        mode: ReplyMode,
    ) -> TestServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("test address");
        let state = Arc::new(Mutex::new(initial));
        let thread_state = Arc::clone(&state);
        let handle = thread::spawn(move || {
            let server_key = SigningKey::from_bytes(&SERVER_SECRET);
            let (mut stream, _) = listener.accept().expect("accept test client");
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .expect("read timeout");
            stream
                .set_write_timeout(Some(Duration::from_secs(3)))
                .expect("write timeout");
            for index in 0..request_count {
                let mut request = [0_u8; REQUEST_BYTES];
                stream.read_exact(&mut request).expect("read request");
                let response = server_reply(
                    &request,
                    &expected_client,
                    &server_key,
                    &thread_state,
                    mode,
                    index,
                );
                stream.write_all(&response).expect("write response");
                stream.flush().expect("flush response");
            }
        });
        TestServer {
            address,
            state,
            handle,
        }
    }

    fn server_reply(
        request: &[u8; REQUEST_BYTES],
        expected_client: &VerifyingKey,
        server_key: &SigningKey,
        state: &Arc<Mutex<Option<ProductionAnchorRecord>>>,
        mode: ReplyMode,
        response_index: usize,
    ) -> [u8; RESPONSE_BYTES] {
        let mut status = STATUS_OK;
        let mut output_record = None;
        let signature_bytes: [u8; SIGNATURE_BYTES] = request[REQUEST_SIGNED_BYTES..]
            .try_into()
            .expect("request signature bytes");
        let request_signature = Signature::from_bytes(&signature_bytes);
        let authenticated = request[..8] == REQUEST_MAGIC[..]
            && request[8] == PROTOCOL_VERSION
            && expected_client
                .verify_strict(&request[..REQUEST_SIGNED_BYTES], &request_signature)
                .is_ok();

        let mut request_id: [u8; REQUEST_ID_BYTES] =
            request[10..42].try_into().expect("request id");
        if !authenticated {
            status = STATUS_UNAVAILABLE;
        } else {
            match request[9] {
                OP_READ => {
                    output_record = *state.lock().expect("state lock");
                }
                OP_COMPARE_AND_ADVANCE => {
                    let expected = match request[42] {
                        0 => None,
                        1 => decode_record(&request[43..43 + RECORD_BYTES]).ok(),
                        _ => {
                            status = STATUS_CORRUPT;
                            None
                        }
                    };
                    if status == STATUS_OK {
                        let next_start = 43 + RECORD_BYTES;
                        let next = decode_record(&request[next_start..next_start + RECORD_BYTES]);
                        match next {
                            Ok(next) => {
                                let mut current = state.lock().expect("state lock");
                                let monotonic = match *current {
                                    None => next.generation() == 1,
                                    Some(previous) => {
                                        next.generation() == previous.generation() + 1
                                            && next.trust().position()
                                                >= previous.trust().position()
                                            && next.delivery().position()
                                                >= previous.delivery().position()
                                            && (next.trust().position()
                                                != previous.trust().position()
                                                || next.trust().digest()
                                                    == previous.trust().digest())
                                            && (next.delivery().position()
                                                != previous.delivery().position()
                                                || next.delivery().digest()
                                                    == previous.delivery().digest())
                                    }
                                };
                                if *current != expected || !monotonic {
                                    status = STATUS_CONFLICT;
                                    output_record = *current;
                                } else {
                                    *current = Some(next);
                                    output_record = Some(next);
                                }
                            }
                            Err(_) => status = STATUS_CORRUPT,
                        }
                    }
                }
                _ => status = STATUS_CORRUPT,
            }
        }

        if matches!(mode, ReplyMode::WrongRequestId) && response_index == 0 {
            request_id[0] ^= 0x80;
        }
        let mut response = [0_u8; RESPONSE_BYTES];
        response[..8].copy_from_slice(&RESPONSE_MAGIC);
        response[8] = PROTOCOL_VERSION;
        response[9] = status;
        response[10..42].copy_from_slice(&request_id);
        response[42] = u8::from(output_record.is_some());
        if let Some(record) = output_record {
            response[43..43 + RECORD_BYTES].copy_from_slice(&encode_record(record));
        }
        let signature = server_key.sign(&response[..RESPONSE_SIGNED_BYTES]);
        response[RESPONSE_SIGNED_BYTES..].copy_from_slice(&signature.to_bytes());
        if matches!(mode, ReplyMode::TamperSignature) && response_index == 0 {
            response[RESPONSE_BYTES - 1] ^= 0x01;
        }
        response
    }

    fn make_client(address: SocketAddr, secret: [u8; 32]) -> RemoteAnchorClient {
        let server_public = SigningKey::from_bytes(&SERVER_SECRET)
            .verifying_key()
            .to_bytes();
        RemoteAnchorClient::new(address, secret, server_public, Duration::from_secs(2))
            .expect("client config")
    }

    #[derive(Clone, Copy)]
    struct Source {
        trust: ChainWitness,
        delivery: ChainWitness,
    }

    impl Source {
        fn at(trust: u64, delivery: u64) -> Self {
            Self {
                trust: ChainWitness::from_parts(trust, [trust as u8; 32]),
                delivery: ChainWitness::from_parts(delivery, [delivery as u8; 32]),
            }
        }
    }

    impl ProductionWitnessSource for Source {
        type Error = ();

        fn trust_witness(&self) -> ChainWitness {
            self.trust
        }

        fn contains_trust_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error> {
            Ok(witness == self.trust)
        }

        fn delivery_witness(&self) -> ChainWitness {
            self.delivery
        }

        fn contains_delivery_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error> {
            Ok(witness == self.delivery)
        }
    }

    #[test]
    fn remote_read_empty_round_trip() {
        let server = spawn_server(None, client_public(CLIENT_SECRET), 1, ReplyMode::Normal);
        let mut client = make_client(server.address, CLIENT_SECRET);
        assert_eq!(client.read(), Ok(None));
        server.handle.join().expect("server join");
    }

    #[test]
    fn remote_compare_and_advance_initializes_and_reads_back() {
        let first = record(1, 7, 9);
        let server = spawn_server(None, client_public(CLIENT_SECRET), 2, ReplyMode::Normal);
        let mut client = make_client(server.address, CLIENT_SECRET);
        assert_eq!(client.compare_and_advance(None, first), Ok(()));
        assert_eq!(client.read(), Ok(Some(first)));
        server.handle.join().expect("server join");
        assert_eq!(*server.state.lock().expect("state lock"), Some(first));
    }

    #[test]
    fn remote_conflicting_expected_record_fails_closed() {
        let first = record(1, 7, 9);
        let second = record(2, 8, 10);
        let server = spawn_server(
            Some(first),
            client_public(CLIENT_SECRET),
            1,
            ReplyMode::Normal,
        );
        let mut client = make_client(server.address, CLIENT_SECRET);
        assert_eq!(
            client.compare_and_advance(None, second),
            Err(AnchorStoreError::Conflict)
        );
        server.handle.join().expect("server join");
        assert_eq!(*server.state.lock().expect("state lock"), Some(first));
    }

    #[test]
    fn remote_non_monotonic_generation_is_rejected() {
        let first = record(1, 7, 9);
        let same_generation = record(1, 8, 10);
        let server = spawn_server(
            Some(first),
            client_public(CLIENT_SECRET),
            1,
            ReplyMode::Normal,
        );
        let mut client = make_client(server.address, CLIENT_SECRET);
        assert_eq!(
            client.compare_and_advance(Some(first), same_generation),
            Err(AnchorStoreError::Conflict)
        );
        server.handle.join().expect("server join");
    }

    #[test]
    fn wrong_client_key_is_rejected_without_state_change() {
        let server = spawn_server(None, client_public(CLIENT_SECRET), 1, ReplyMode::Normal);
        let mut client = make_client(server.address, OTHER_CLIENT_SECRET);
        assert_eq!(client.read(), Err(AnchorStoreError::Unavailable));
        server.handle.join().expect("server join");
        assert_eq!(*server.state.lock().expect("state lock"), None);
    }

    #[test]
    fn tampered_server_signature_is_rejected() {
        let server = spawn_server(
            None,
            client_public(CLIENT_SECRET),
            1,
            ReplyMode::TamperSignature,
        );
        let mut client = make_client(server.address, CLIENT_SECRET);
        assert_eq!(client.read(), Err(AnchorStoreError::Corrupt));
        server.handle.join().expect("server join");
    }

    #[test]
    fn response_for_another_request_id_is_rejected() {
        let server = spawn_server(
            None,
            client_public(CLIENT_SECRET),
            1,
            ReplyMode::WrongRequestId,
        );
        let mut client = make_client(server.address, CLIENT_SECRET);
        assert_eq!(client.read(), Err(AnchorStoreError::Corrupt));
        server.handle.join().expect("server join");
    }

    #[test]
    fn unreachable_remote_service_is_unavailable() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve address");
        let address = listener.local_addr().expect("address");
        drop(listener);
        let mut client = make_client(address, CLIENT_SECRET);
        assert_eq!(client.read(), Err(AnchorStoreError::Unavailable));
    }

    #[test]
    fn p06_recovery_contract_operates_over_remote_backend() {
        let source = Source::at(5, 7);
        let server = spawn_server(None, client_public(CLIENT_SECRET), 4, ReplyMode::Normal);
        let mut client = make_client(server.address, CLIENT_SECRET);
        let initialized = initialize_production_anchor(&mut client, &source).expect("initialize");
        assert_eq!(initialized.generation(), 1);
        let exact = verify_production_recovery(&mut client, &source).expect("exact recovery");
        assert_eq!(exact, initialized);
        server.handle.join().expect("server join");
    }

    #[test]
    fn remote_anchor_detects_older_local_trust_state() {
        let original = Source::at(5, 7);
        let rolled_back = Source::at(4, 7);
        let server = spawn_server(None, client_public(CLIENT_SECRET), 4, ReplyMode::Normal);
        let mut client = make_client(server.address, CLIENT_SECRET);
        initialize_production_anchor(&mut client, &original).expect("initialize");
        assert_eq!(
            verify_production_recovery(&mut client, &rolled_back),
            Err(RecoveryError::TrustRollback)
        );
        server.handle.join().expect("server join");
    }
}
