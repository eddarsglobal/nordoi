#!/usr/bin/env python3
"""NORDOI G0.3 R6F BUREAU-001 R4 — real POSIX SSHSIG allowlist race audit.

Reproduces a genuine second-process atomic os.replace between R3 bind()
and ssh-keygen's re-opening of allowed_signers. R4 holds the validated
original file descriptors across actual ssh-keygen verification and passes
them to the child, rather than trusting a pathname a second time.

All decisions DENY; no external custodian, privileged runtime or production.
A malicious same-administrator controlling process memory or already-open
descriptors is OUT OF SCOPE. FD/path checks are not remote trust evidence.
"""
import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys

import g03_r6e_oob_origin_intake as r6e
import g03_r6f_r3_signer_binding as r3
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R6F-R4-ACTUAL-FD-PATH-RACE-DENY-V1"
PARENT = "2dd46dc07568bf382c49b1c8bca0cba8900b7b1f"
MAX_PUB = 2048
MAX_ALLOW = 4096

class Refused(ValueError):
    pass

def deny(reason, **details):
    return r6e.deny(reason, **details)

def is_posix_ssh():
    return (os.name == "posix" and Path("/usr/bin/ssh-keygen").is_file()
            and Path("/dev/fd").is_dir())

class PinnedReadFile:
    """Hold exact public key bytes + FD while pathname is checked, no grant."""
    def __init__(self, path, budget=MAX_PUB):
        self.path = Path(path)
        self.fd = None
        self.before = None
        self.data = None
        if self.path.is_symlink() or not self.path.is_file():
            raise Refused("PINNED_PUBLIC_PATH_MISSING_OR_SYMLINK")
        flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
        try:
            self.fd = os.open(self.path, flags)
            self.before = fdgate.OpenSnapshot.identity(os.fstat(self.fd))
            size = self.before[3]
            if not stat.S_ISREG(self.before[2]) or not 0 < size <= budget:
                raise Refused("PINNED_PUBLIC_NOT_BOUNDED_REGULAR_FILE")
            self.data = os.read(self.fd, size + 1)
            if len(self.data) != size:
                raise Refused("PINNED_PUBLIC_SHORT_OR_CHANGED_READ")
            self.check()
        except BaseException:
            self.close()
            raise

    def check(self):
        if self.fd is None:
            raise Refused("PINNED_PUBLIC_FD_CLOSED")
        a = fdgate.OpenSnapshot.identity(os.fstat(self.fd))
        b = fdgate.OpenSnapshot.identity(os.lstat(self.path))
        if a != self.before or b != self.before:
            raise Refused("PINNED_PUBLIC_FD_OR_PATH_CHANGED")
        if os.pread(self.fd, len(self.data) + 1, 0) != self.data:
            raise Refused("PINNED_PUBLIC_BYTES_CHANGED")

    def close(self):
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()

def read_allowed(snapshot):
    snapshot.check()
    data = os.pread(snapshot.fd, MAX_ALLOW + 1, 0)
    if hashlib.sha256(data).hexdigest() != snapshot.digest:
        raise Refused("PINNED_ALLOWED_FD_CONTENT_NOT_ORIGINAL_DIGEST")
    if len(data) > MAX_ALLOW:
        raise Refused("PINNED_ALLOWED_FILE_OVERSIZED")
    return data

def freeze_evidence(stack, data, local_allowed, owner_allowed, candidate_allowed):
    """Bind all three allowed files to two held public-key snapshots.

    Shared allowed file path is held once. This is not a lock on a hostile
    same-admin process or a claim of independent administrative custody.
    """
    x = r6e.parse(data["r6e_body"])
    pub_map = {}
    for public, expected in (
        (data["owner_public"], x["owner_key_sha256"]),
        (data["guardian_public"], x["candidate_key_sha256"]),
    ):
        p = str(Path(public))
        snap = stack.enter_context(PinnedReadFile(p))
        canonical = r3.parse_ed25519_public(snap.data)
        actual = hashlib.sha256(canonical + b"\n").hexdigest()
        if actual != expected:
            raise Refused("PINNED_PUBLIC_KEY_NOT_CANDIDATE_OR_OWNER")
        pub_map[p] = snap

    allowed_map = {}
    for public, allowed in (
        (data["owner_public"], owner_allowed),
        (data["guardian_public"], candidate_allowed),
        (data["guardian_public"], local_allowed),
    ):
        p = str(Path(allowed))
        if p not in allowed_map:
            snap = stack.enter_context(fdgate.OpenSnapshot(Path(p)))
            allowed_map[p] = snap
        snap = allowed_map[p]
        pub = pub_map[str(Path(public))]
        if read_allowed(snap) != r3.expected_allowed_bytes(pub.data):
            raise Refused("ALLOWLIST_FD_NOT_BOUND_TO_PINNED_PUBLIC_KEY")
    for snap in pub_map.values():
        snap.check()
    for snap in allowed_map.values():
        snap.check()
    return pub_map, allowed_map

def genuine_fd_verify(command, input_bytes, allowed_map, before_verify=None):
    """Execute installed OpenSSH with an already-open allowlist FD.

    The verified file bytes do not depend on re-opening a mutable pathname.
    An optional exercise hook permits a genuine second process to swap that
    pathname *after* pinning but *before* ssh-keygen is launched.
    """
    if len(command) < 5 or command[:2] != ["-Y", "verify"] or "-f" not in command:
        raise Refused("R4_UNSUPPORTED_SSH_OPERATION")
    idx = command.index("-f") + 1
    if idx >= len(command):
        raise Refused("R4_MISSING_ALLOWLIST_ARGUMENT")
    original = str(Path(command[idx]))
    if original not in allowed_map:
        raise Refused("R4_UNPINNED_ALLOWED_SIGNERS_ARGUMENT")
    snap = allowed_map[original]
    if before_verify is not None:
        before_verify(command)
    # Reading through the held descriptor (not re-opening the mutable name).
    os.lseek(snap.fd, 0, os.SEEK_SET)
    fixed = list(command)
    fixed[idx] = "/dev/fd/" + str(snap.fd)
    p = subprocess.run(
        ["/usr/bin/ssh-keygen"] + fixed, input=input_bytes,
        capture_output=True, timeout=35, check=False, pass_fds=(snap.fd,)
    )
    return p.returncode

def assess_lab(data, *, local_signature=None, local_allowed=None,
               owner_allowed=None, candidate_allowed=None, before_verify=None):
    """R4 denies all outcomes and refuses source replacement in the verified window."""
    if not is_posix_ssh():
        return deny("R4_REAL_POSIX_OPENSSH_FD_VERIFICATION_REQUIRED")
    try:
        if type(data) is not dict or (before_verify is not None and
                                      not callable(before_verify)):
            raise Refused("R4_INVALID_RESEARCH_ARGUMENTS")
        local_allowed = (data["guardian_allowed"] if local_allowed is None
                         else local_allowed)
        owner_allowed = (data["owner_allowed"] if owner_allowed is None
                         else owner_allowed)
        candidate_allowed = (data["guardian_allowed"] if candidate_allowed is None
                             else candidate_allowed)
        local_signature = (data["r6e_local_signature"] if local_signature is None
                           else local_signature)
        with ExitStack() as stack:
            pub_map, allowed_map = freeze_evidence(
                stack, data, local_allowed, owner_allowed, candidate_allowed
            )
            previous = ssh.run_ssh
            def pinned_verify(command, input_bytes=None):
                return genuine_fd_verify(command, input_bytes, allowed_map,
                                         before_verify=before_verify)
            try:
                ssh.run_ssh = pinned_verify
                result = r3.assess_lab(
                    data, local_signature=local_signature,
                    local_allowed=local_allowed, owner_allowed=owner_allowed,
                    candidate_allowed=candidate_allowed
                )
            finally:
                ssh.run_ssh = previous
            for snapshot in pub_map.values():
                snapshot.check()
            for snapshot in allowed_map.values():
                snapshot.check()
        if result["reason"] != (
            "R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY"
        ):
            return deny("R4_INHERITED_R3_SIX_SIGNER_PROOF_NOT_QUALIFIED",
                        inherited_reason=result["reason"])
        return deny(
            "R4_SIX_LOCAL_SSHSIGS_VERIFIED_WITH_HELD_ALLOWED_FDS_NO_EXTERNAL_CUSTODY",
            actual_openssh_used=True,
            allowed_signer_fd_pinned_during_verification=True,
            observed_original_local_file_paths_unchanged=True,
            independent_external_custodian_proven=False,
            hostile_same_admin_full_control_resistance_proven=False,
        )
    except (Refused, fdgate.Refused, r3.Refused, r6e.Refused,
            OSError, ValueError, TypeError, KeyError, subprocess.SubprocessError) as e:
        return deny("R4_SIGNER_FILE_RACE_OR_PROVENANCE_QUARANTINED",
                    detail=(str(e) if isinstance(e, (Refused, fdgate.Refused,
                        r3.Refused, r6e.Refused)) else type(e).__name__))

def atomic_child_replace(source, dest):
    """Real second OS process; no sleep, mock signing, or production files."""
    src, dst = Path(source), Path(dest)
    if src.parent != dst.parent or not src.parent.name.startswith("nordoi-"):
        raise Refused("R4_ATOMIC_SWAP_ONLY_IN_SAME_DISPOSABLE_LAB_ROOT")
    program = (
        "import os,shutil,sys,tempfile; src,dst=sys.argv[1:];"
        "fd,tmp=tempfile.mkstemp(prefix='nordoi-r4-swap-',dir=os.path.dirname(dst));"
        "os.close(fd);shutil.copyfile(src,tmp);os.chmod(tmp,0o600);os.replace(tmp,dst)"
    )
    p = subprocess.run(
        [sys.executable, "-B", "-c", program, str(src), str(dst)],
        capture_output=True, text=True, timeout=25, check=False
    )
    if p.returncode:
        raise Refused("R4_REAL_SECOND_PROCESS_ATOMIC_SWAP_FAILED:"+p.stderr[-200:])

def copy_private(source, dest):
    if Path(source).parent != Path(dest).parent:
        raise Refused("R4_COPY_OUTSIDE_DISPOSABLE_LAB_ROOT")
    shutil.copyfile(source, dest)
    os.chmod(dest, 0o600)

def legacy_r3_race(data, signature, rogue_allowed, victim_allowed):
    """Trigger real path replacement *only after* R3's bind and before OpenSSH."""
    old = ssh.run_ssh
    fired = [False]
    def intercepted(command, input_bytes=None):
        if (not fired[0] and command[:2] == ["-Y", "verify"]
                and "-s" in command
                and command[command.index("-s")+1] == str(signature)):
            atomic_child_replace(rogue_allowed, victim_allowed)
            fired[0] = True
        return old(command, input_bytes)
    try:
        ssh.run_ssh = intercepted
        result = r3.assess_lab(data, local_signature=signature,
                               local_allowed=victim_allowed)
    finally:
        ssh.run_ssh = old
    if not fired[0]:
        raise Refused("R4_LEGACY_R3_INJECTION_WINDOW_NOT_EXERCISED")
    return result

def real_exercise():
    if not is_posix_ssh():
        return deny("R4_REAL_POSIX_OPENSSH_NOT_AVAILABLE"), 2
    home, data = r6e.create_lab()
    try:
        root = data["root"]
        normal = assess_lab(data)
        if normal["reason"] != (
            "R4_SIX_LOCAL_SSHSIGS_VERIFIED_WITH_HELD_ALLOWED_FDS_NO_EXTERNAL_CUSTODY"
        ):
            raise Refused("R4_PINNED_FD_POSITIVE_SIX_LOCAL_DENY_CASE_MISSING:"
                          + str(normal.get("detail", normal["reason"])))
        rogue_sig, rogue_allowed, rogue_pub = r3.create_rogue_signed_receipt(data)
        if r3.keys.public_sha(rogue_pub) == r3.keys.public_sha(data["guardian_public"]):
            raise Refused("R4_ROGUE_KEY_NOT_GENUINELY_DISTINCT")
        victim = root / "nordoi-r4-victim-local.allowed"
        copy_private(data["guardian_allowed"], victim)
        legacy = legacy_r3_race(data, rogue_sig, rogue_allowed, victim)
        if legacy["reason"] != (
            "R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY"
        ):
            raise Refused("R4_R3_PATH_REPLACEMENT_GAP_NOT_REPRODUCED:"
                          + str(legacy.get("detail", legacy["reason"])))
        victim2 = root / "nordoi-r4-victim-fixed.allowed"
        copy_private(data["guardian_allowed"], victim2)
        fired = [False]
        def second_process_between_pin_and_use(command):
            if (not fired[0] and "-s" in command and
                command[command.index("-s")+1] == str(rogue_sig)):
                atomic_child_replace(rogue_allowed, victim2)
                fired[0] = True
        fixed = assess_lab(data, local_signature=rogue_sig,
                           local_allowed=victim2,
                           before_verify=second_process_between_pin_and_use)
        if not fired[0]:
            raise Refused("R4_CORRECTED_REAL_SUBPROCESS_SWAP_NOT_INVOKED")
        if fixed["reason"] != "R4_SIGNER_FILE_RACE_OR_PROVENANCE_QUARANTINED":
            raise Refused("R4_SAME_ROGUE_SIGNER_RACE_NOT_QUARANTINED:"
                          + fixed["reason"])
        return deny(
            "R4_REAL_SECOND_PROCESS_SSHSIG_PATH_RACE_REPRODUCED_AND_QUARANTINED",
            legacy_r3_six_local_signatures_misattributed_during_race=True,
            rogue_real_ed25519_signature=True,
            second_real_os_process_atomically_replaced_allowlist=True,
            r4_uses_already_open_allowed_signers_fds=True,
            r4_detected_real_replacement_and_denied=True,
            previous_r3_native_admission_always_blocked=True,
            external_independent_custody_proven=False,
        ), 2
    finally:
        home.cleanup()

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exercise", action="store_true", required=True)
    parser.parse_args(argv)
    try:
        outcome, code = real_exercise()
    except (Refused, OSError, ValueError, TypeError,
            subprocess.SubprocessError) as exc:
        outcome, code = deny(
            "R4_REAL_SIGNED_FD_RACE_EXERCISE_FAILED",
            detail=str(exc) if isinstance(exc, Refused) else type(exc).__name__
        ), 3
    print(json.dumps(outcome, sort_keys=True))
    return code

if __name__ == "__main__":
    sys.exit(main())
