#!/usr/bin/env python3
"""NORDOI G0.3 R6F / BUREAU-001 R3 — authentic SSHSIG key/allowlist attribution audit.

A valid SSHSIG under an allowed_signers file proves a signature by *a key
admitted in that file*, not necessarily the separately supplied public key
whose hash was recorded in a candidate dossier. Earlier R6E code verified
these two facts separately. This additive deny-only research guard requires
all three local allowlists to be bound to exactly their declared Ed25519
public key before relying on the inherited real six-signature exercise.
No runtime or privileged admission code is changed. No external custodian.
"""
import argparse
import base64
import binascii
import json
import os
from pathlib import Path
import sys

import g03_r6e_oob_origin_intake as r6e
import g03_r6d_admin_domain_provenance as r6d
import g03_r6c_two_key_quorum_negative as r6c
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5x_signed_external_provenance as signed
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R6F-R3-SIGNER-ALLOWLIST-BINDING-NEGATIVE-V1"
PARENT = "80010bdddd5e4743f023c0eed888a1c3360339da"
MAX_KEY_BYTES = 2048
MAX_ALLOWED_BYTES = 4096

class Refused(ValueError):
    pass

def deny(reason, **details):
    return r6e.deny(reason, **details)

def parse_ed25519_public(raw):
    """Require a single well-formed ed25519 SSH key line, not a key label."""
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_KEY_BYTES:
        raise Refused("PUBLIC_KEY_MISSING_OR_OVERSIZED")
    if b"\x00" in raw or b"\r" in raw or b"\n" in raw.rstrip(b"\n"):
        raise Refused("PUBLIC_KEY_MALFORMED_LINES")
    if not raw.endswith(b"\n") or raw.endswith(b"\n\n"):
        raise Refused("PUBLIC_KEY_NOT_ONE_LINE")
    fields = raw[:-1].split()
    if len(fields) < 2 or fields[0] != b"ssh-ed25519":
        raise Refused("PUBLIC_KEY_NOT_ED25519")
    try:
        blob = base64.b64decode(fields[1], validate=True)
    except (ValueError, binascii.Error) as exc:
        raise Refused("PUBLIC_KEY_INVALID_BASE64") from exc
    if base64.b64encode(blob) != fields[1] or len(blob) != 51:
        raise Refused("PUBLIC_KEY_NOT_CANONICAL_ED25519_BLOB")
    # Binary SSH string: "ssh-ed25519" + exactly 32 raw key octets.
    if blob[:4] != b"\x00\x00\x00\x0b" or blob[4:15] != b"ssh-ed25519" or (
        blob[15:19] != b"\x00\x00\x00\x20"
    ):
        raise Refused("PUBLIC_KEY_WRONG_SSH_KEY_WIRE_TYPE")
    if len(fields) > 2 and any(b"\x00" in f or b"\n" in f for f in fields[2:]):
        raise Refused("PUBLIC_KEY_INVALID_COMMENT")
    return b" ".join(fields[:2])

def expected_allowed_bytes(pub_raw):
    key_pair = parse_ed25519_public(pub_raw)
    return ssh.IDENTITY.encode("ascii") + b" " + key_pair + b"\n"

def bind(public_path, allowed_path, expected_pin):
    """Research fixture policy: exactly one authorized signer, no extra keys."""
    if not isinstance(expected_pin, str) or len(expected_pin) != 64:
        raise Refused("MISSING_EXPECTED_SIGNER_PIN")
    pub = Path(public_path)
    allowed = Path(allowed_path)
    if pub.is_symlink() or not pub.is_file() or allowed.is_symlink() or not allowed.is_file():
        raise Refused("BINDING_PROOF_MISSING_OR_SYMLINK")
    with fdgate.OpenSnapshot(allowed) as frozen:
        pub_raw = pub.read_bytes()
        allow_raw = allowed.read_bytes()
        if len(allow_raw) > MAX_ALLOWED_BYTES or allow_raw != expected_allowed_bytes(pub_raw):
            raise Refused("ALLOWLIST_NOT_EXACT_ONE_DECLARED_PINNED_ED25519_KEY")
        if keys.public_sha(pub) != expected_pin:
            raise Refused("DECLARED_ED25519_PUBLIC_KEY_PIN_CHANGED")
        frozen.check()
    return True

def assess_lab(data, *, local_signature=None, local_allowed=None,
               owner_allowed=None, candidate_allowed=None, **kwargs):
    """Corrected additive R6E research assessment, same BLOCK-only semantics.

    Only the exact verified local signature chain earns the *six bound local
    signatures* description. Caller-injected allowed_signer keys cannot be
    substituted for the local public keys mentioned by the record.
    """
    try:
        if type(data) is not dict:
            raise Refused("INVALID_LAB_DATA")
        record = r6e.parse(data["r6e_body"])
        owner_pin = record["owner_key_sha256"]
        candidate_pin = record["candidate_key_sha256"]
        local_allowed = data["guardian_allowed"] if local_allowed is None else local_allowed
        owner_allowed = data["owner_allowed"] if owner_allowed is None else owner_allowed
        candidate_allowed = (data["guardian_allowed"] if candidate_allowed is None
                             else candidate_allowed)
        for pub, allowed, digest in (
            (data["owner_public"], owner_allowed, owner_pin),
            (data["guardian_public"], candidate_allowed, candidate_pin),
            (data["guardian_public"], local_allowed, candidate_pin),
        ):
            bind(pub, allowed, digest)
        local_signature = (data["r6e_local_signature"] if local_signature is None
                           else local_signature)
        result = r6e.assess_lab(
            data, local_signature=local_signature, local_allowed=local_allowed,
            owner_allowed=owner_allowed, candidate_allowed=candidate_allowed,
            **kwargs
        )
        if result["reason"] != "SIX_VALID_LOCAL_SIGNATURES_NOT_OWNER_OOB_ENROLLMENT":
            return result  # Keep inherited refusal and exact reason.
        return deny(
            "R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY",
            all_six_signatures_real_local_only=True,
            allowlists_bound_to_owner_and_candidate_public_keys=True,
            owner_oob_channel_independently_authenticated=False,
            signer_identity_outside_same_admin_domain=False,
            independent_quorum_proven=False,
            external_history_monotonicity_verified=False,
        )
    except (Refused, KeyError, TypeError, OSError, fdgate.Refused,
            r6e.Refused, keys.Refused) as exc:
        return deny("R3_SIGNATURE_ATTRIBUTION_QUARANTINED",
                    detail=(str(exc) if isinstance(exc, (
                        Refused, fdgate.Refused, r6e.Refused, keys.Refused
                    )) else type(exc).__name__))

def create_rogue_signed_receipt(data):
    """Real distinct Ed25519 signature on exact R6E bytes, disposable root only."""
    root = data["root"]
    rogue, rogue_allowed = ssh.create_same_admin_key(root, "r6f_r3_extra_local")
    path = root / "r6f_r3_rogue_original_receipt.json"
    path.write_bytes(data["r6e_body"])
    if os.name != "nt":
        os.chmod(path, 0o600)
    sig = signed.sign_local(path, rogue, r6e.NAMESPACE)
    return sig, rogue_allowed, root / "r6f_r3_extra_local.pub"

def exercise():
    if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
        return deny("R3_REAL_NATIVE_OPENSSH_REQUIRED"), 2
    home, data = r6e.create_lab()
    try:
        normal = assess_lab(data)
        if normal["reason"] != (
            "R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY"
        ):
            raise Refused("EXPECTED_GENUINE_SIX_SIGNER_BINDING_MISSING")
        rogue_sig, rogue_allowed, rogue_public = create_rogue_signed_receipt(data)
        if keys.public_sha(rogue_public) == keys.public_sha(data["guardian_public"]):
            raise Refused("ROGUE_KEY_NOT_DISTINCT")
        # Reproducer: inherited R6E accepts a true signature from another
        # signer in caller-supplied allowed_signers, even when local_public
        # remains pinned to the original candidate. All outcomes still BLOCK.
        original = r6e.assess_lab(
            data, local_signature=rogue_sig, local_allowed=rogue_allowed
        )
        if original["reason"] != "SIX_VALID_LOCAL_SIGNATURES_NOT_OWNER_OOB_ENROLLMENT":
            raise Refused("ORIGINAL_R6E_KEY_ATTRIBUTION_GAP_NOT_REPRODUCED")
        repaired = assess_lab(
            data, local_signature=rogue_sig, local_allowed=rogue_allowed
        )
        if repaired["reason"] != "R3_SIGNATURE_ATTRIBUTION_QUARANTINED":
            raise Refused("ROGUE_ALLOWED_SIGNER_WAS_NOT_REFUSED")
        if "ALLOWLIST_NOT_EXACT_ONE_DECLARED_PINNED_ED25519_KEY" not in repaired.get("detail", ""):
            raise Refused("ROGUE_SIGNER_NOT_REJECTED_FOR_EXACT_PIN_MISMATCH")
        return deny("R3_GENUINE_ROGUE_SSHSIG_ATTRIBUTION_GAP_REPRODUCED_AND_BLOCKED",
                    native_rogue_key_and_signature_real=True,
                    old_r6e_attribution_gap_reproduced=True,
                    corrected_r3_allowlist_binding=True,
                    old_r6e_native_admission_also_blocked=True,
                    native_tests_research_only=True), 2
    finally:
        home.cleanup()

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exercise", action="store_true", required=True)
    parser.parse_args(argv)
    try:
        result, status = exercise()
    except (Refused, OSError, ValueError, RuntimeError) as exc:
        result, status = deny(
            "R3_REAL_NATIVE_CROSS_BOUNDARY_EXERCISE_FAILED",
            detail=str(exc) if isinstance(exc, Refused) else type(exc).__name__
        ), 3
    print(json.dumps(result, sort_keys=True))
    return status

if __name__ == "__main__":
    sys.exit(main())
