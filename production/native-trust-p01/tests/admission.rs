use ed25519_dalek::{Signer, SigningKey};
use nordoi_native_trust_p01::{sha256_payload, GateError, Intent, SignedEnvelope, TrustGate};

fn signer(byte: u8) -> SigningKey {
    SigningKey::from_bytes(&[byte; 32])
}
fn signed(
    sk: &SigningKey,
    root_id: u64,
    epoch: u64,
    sequence: u64,
    intent: Intent,
) -> SignedEnvelope {
    let mut req = SignedEnvelope {
        root_id,
        epoch,
        sequence,
        intent,
        signature: [0; 64],
    };
    req.signature = sk
        .sign(&req.signing_bytes().expect("valid test envelope"))
        .to_bytes();
    req
}
fn exec(sk: &SigningKey, root_id: u64, epoch: u64, sequence: u64, body: &[u8]) -> SignedEnvelope {
    signed(
        sk,
        root_id,
        epoch,
        sequence,
        Intent::Execute {
            capability: "file.write".to_owned(),
            resource: "workspace/report".to_owned(),
            payload_sha256: sha256_payload(body),
        },
    )
}
fn two_roots() -> (TrustGate, SigningKey, SigningKey) {
    let a = signer(7);
    let b = signer(8);
    let gate = TrustGate::from_trusted_bootstrap(&[
        (10, a.verifying_key().to_bytes(), 1),
        (20, b.verifying_key().to_bytes(), 1),
    ])
    .expect("trusted test bootstrap");
    (gate, a, b)
}

#[test]
fn valid_signature_uses_actual_ed25519_verification() {
    let (mut gate, a, _) = two_roots();
    let req = exec(&a, 10, 1, 1, b"payload");
    let receipt = gate
        .verify_execution(&req, "file.write", "workspace/report", b"payload")
        .unwrap();
    assert_eq!(receipt.root_id(), 10);
    assert_eq!(gate.root_status(10), Some((1, 1, false)));
}
#[test]
fn corrupted_signature_fails_without_counter_mutation() {
    let (mut gate, a, _) = two_roots();
    let mut req = exec(&a, 10, 1, 1, b"payload");
    req.signature[12] ^= 1;
    assert_eq!(
        gate.verify_execution(&req, "file.write", "workspace/report", b"payload"),
        Err(GateError::BadSignature)
    );
    assert_eq!(gate.root_status(10), Some((1, 0, false)));
}
#[test]
fn unrelated_signer_cannot_claim_trusted_identity() {
    let (mut gate, _, b) = two_roots();
    let req = exec(&b, 10, 1, 1, b"payload");
    assert_eq!(
        gate.verify_execution(&req, "file.write", "workspace/report", b"payload"),
        Err(GateError::BadSignature)
    );
}
#[test]
fn cross_root_signature_is_not_transferable() {
    let (mut gate, a, _) = two_roots();
    let mut req = exec(&a, 10, 1, 1, b"payload");
    req.root_id = 20;
    assert_eq!(
        gate.verify_execution(&req, "file.write", "workspace/report", b"payload"),
        Err(GateError::BadSignature)
    );
}
#[test]
fn tampered_payload_is_rejected_without_mutation() {
    let (mut gate, a, _) = two_roots();
    let req = exec(&a, 10, 1, 1, b"payload");
    assert_eq!(
        gate.verify_execution(&req, "file.write", "workspace/report", b"changed"),
        Err(GateError::WrongPayload)
    );
    assert_eq!(gate.root_status(10), Some((1, 0, false)));
}
#[test]
fn wrong_capability_and_resource_are_denied() {
    let (mut gate, a, _) = two_roots();
    let req = exec(&a, 10, 1, 1, b"payload");
    assert_eq!(
        gate.verify_execution(&req, "network.send", "workspace/report", b"payload"),
        Err(GateError::WrongScope)
    );
    assert_eq!(
        gate.verify_execution(&req, "file.write", "workspace/other", b"payload"),
        Err(GateError::WrongScope)
    );
    assert_eq!(gate.root_status(10), Some((1, 0, false)));
}
#[test]
fn exact_sequence_blocks_replay_and_skips() {
    let (mut gate, a, _) = two_roots();
    let first = exec(&a, 10, 1, 1, b"payload");
    gate.verify_execution(&first, "file.write", "workspace/report", b"payload")
        .unwrap();
    assert_eq!(
        gate.verify_execution(&first, "file.write", "workspace/report", b"payload"),
        Err(GateError::WrongSequence)
    );
    let skip = exec(&a, 10, 1, 3, b"payload");
    assert_eq!(
        gate.verify_execution(&skip, "file.write", "workspace/report", b"payload"),
        Err(GateError::WrongSequence)
    );
}
#[test]
fn rotation_requires_signature_of_prior_key_and_advances_epoch() {
    let (mut gate, a, _) = two_roots();
    let new = signer(9);
    let rotate = signed(
        &a,
        10,
        1,
        1,
        Intent::Rotate {
            next_epoch: 2,
            next_public_key: new.verifying_key().to_bytes(),
        },
    );
    gate.rotate(&rotate).unwrap();
    assert_eq!(gate.root_status(10), Some((2, 1, false)));
    let stale = exec(&a, 10, 1, 2, b"payload");
    assert_eq!(
        gate.verify_execution(&stale, "file.write", "workspace/report", b"payload"),
        Err(GateError::WrongEpoch)
    );
    let fresh = exec(&new, 10, 2, 2, b"payload");
    gate.verify_execution(&fresh, "file.write", "workspace/report", b"payload")
        .unwrap();
}
#[test]
fn attacker_cannot_rotate_root_with_new_key_signature() {
    let (mut gate, _, _) = two_roots();
    let attacker = signer(9);
    let req = signed(
        &attacker,
        10,
        1,
        1,
        Intent::Rotate {
            next_epoch: 2,
            next_public_key: attacker.verifying_key().to_bytes(),
        },
    );
    assert_eq!(gate.rotate(&req), Err(GateError::BadSignature));
    assert_eq!(gate.root_status(10), Some((1, 0, false)));
}
#[test]
fn invalid_rotation_does_not_consume_sequence() {
    let (mut gate, a, _) = two_roots();
    let next = signer(9);
    let req = signed(
        &a,
        10,
        1,
        1,
        Intent::Rotate {
            next_epoch: 5,
            next_public_key: next.verifying_key().to_bytes(),
        },
    );
    assert_eq!(gate.rotate(&req), Err(GateError::InvalidRotation));
    assert_eq!(gate.root_status(10), Some((1, 0, false)));
}
#[test]
fn signed_revocation_is_irreversible_and_does_not_affect_other_root() {
    let (mut gate, a, b) = two_roots();
    gate.revoke(&signed(&a, 10, 1, 1, Intent::Revoke)).unwrap();
    let stale = exec(&a, 10, 1, 2, b"payload");
    assert_eq!(
        gate.verify_execution(&stale, "file.write", "workspace/report", b"payload"),
        Err(GateError::Revoked)
    );
    let other = exec(&b, 20, 1, 1, b"payload");
    gate.verify_execution(&other, "file.write", "workspace/report", b"payload")
        .unwrap();
}
#[test]
fn untrusted_bootstrap_and_duplicate_keys_are_rejected() {
    assert!(matches!(
        TrustGate::from_trusted_bootstrap(&[]),
        Err(GateError::MissingRoots)
    ));
    let a = signer(7).verifying_key().to_bytes();
    assert!(matches!(
        TrustGate::from_trusted_bootstrap(&[(1, a, 1), (1, a, 1)]),
        Err(GateError::DuplicateRoot)
    ));
    assert!(matches!(
        TrustGate::from_trusted_bootstrap(&[(1, a, 1), (2, a, 1)]),
        Err(GateError::DuplicateKey)
    ));
}
#[test]
fn operation_tag_is_bound_by_ed25519_signature() {
    let (mut gate, a, _) = two_roots();
    let mut req = exec(&a, 10, 1, 1, b"payload");
    req.intent = Intent::Revoke;
    assert_eq!(gate.revoke(&req), Err(GateError::BadSignature));
}
#[test]
fn invalid_scope_is_rejected_before_signature_verification() {
    let (mut gate, a, _) = two_roots();
    let req = exec(&a, 10, 1, 1, b"payload");
    assert_eq!(
        gate.verify_execution(&req, "file.write", "../secrets", b"payload"),
        Err(GateError::InvalidScope)
    );
    assert_eq!(
        gate.verify_execution(&req, "file.write", "\n", b"payload"),
        Err(GateError::InvalidScope)
    );
}
#[test]
fn canonical_bytes_are_domain_separated_and_stable() {
    let a = signer(7);
    let req = exec(&a, 10, 1, 1, b"payload");
    let bytes = req.signing_bytes().unwrap();
    assert!(bytes.starts_with(b"NORDOI:P0.1:ADMISSION:V1\0"));
    assert_eq!(bytes, req.signing_bytes().unwrap());
}

// Independent reference vector signed with Python cryptography (OpenSSL backend), not dalek.
#[test]
fn independent_openssl_ed25519_vector_verifies_exact_canonical_bytes() {
    let a = signer(7);
    let req = exec(&a, 10, 1, 1, b"payload");
    assert_eq!(
        sha256_payload(&req.signing_bytes().unwrap()),
        [
            0x87, 0x20, 0x72, 0x3f, 0xad, 0xee, 0x0e, 0x93, 0x59, 0x4b, 0x16, 0x8f, 0xc9, 0xf6,
            0x8e, 0xe0, 0x4e, 0x5a, 0x1d, 0x24, 0x24, 0xc7, 0x08, 0x6c, 0x9d, 0x33, 0x33, 0x51,
            0xf5, 0x80, 0x7a, 0xca
        ]
    );
    let external_signature: [u8; 64] = [
        0xc8, 0x63, 0x7f, 0x69, 0xbb, 0x5c, 0x6d, 0x92, 0xef, 0x35, 0xae, 0xa6, 0x05, 0x74, 0xfe,
        0x01, 0xbc, 0xde, 0xc5, 0xbb, 0x00, 0x8d, 0xc8, 0x02, 0xfa, 0x48, 0x9c, 0xdc, 0xfb, 0x4f,
        0xb6, 0x8e, 0xec, 0x73, 0xbd, 0xf9, 0xa6, 0x62, 0xe8, 0x0a, 0x30, 0x9c, 0x88, 0x1e, 0xb3,
        0x5f, 0xed, 0x1b, 0x87, 0xc9, 0x87, 0x02, 0x3a, 0x58, 0xdb, 0x80, 0x92, 0x02, 0xad, 0xd3,
        0xca, 0x54, 0x0c, 0x08,
    ];
    assert_eq!(req.signature, external_signature);
    let (mut gate, _, _) = two_roots();
    let mut external_request = req;
    external_request.signature = external_signature;
    assert!(gate
        .verify_execution(
            &external_request,
            "file.write",
            "workspace/report",
            b"payload"
        )
        .is_ok());
}
