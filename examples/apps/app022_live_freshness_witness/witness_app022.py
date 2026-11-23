#!/usr/bin/env python3
"""APP-022: live, loopback-only, separate-process freshness signing research.

The server reads a canonical locally stored witness state and signs the
receiver's unpredictable nonce. Witness key is held by this process, not
passed to the receiver. An actual isolated administrative/host custody or
external monotonic storage is NOT proven; all fixture keys are ephemeral.

No HTTP GET, network listener outside 127.0.0.1, Python dependencies,
production service or native admission is provided by this experiment.
"""
from __future__ import annotations
import argparse
import base64
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import threading

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app021_authority_rotation"))
import receiver_app021 as authority

anchor = authority.anchor
SCHEMA = "NORDOI_APP022_WITNESS_STATE_V1"
CHALLENGE_SCHEMA = "NORDOI_APP022_CHALLENGE_V1"
ATTEST_SCHEMA = "NORDOI_APP022_LIVE_WITNESS_ATTESTATION_V1"
RESPONSE_SCHEMA = "NORDOI_APP022_LIVE_WITNESS_RESPONSE_V1"
NAMESPACE = "nordoi-freshness@eddarsglobal"
FIELDS = frozenset(("schema","authority_id","receiver_id","chain_sha256",
   "head_sha256","epoch","minimum_sequence","minimum_generation","counter",
   "release","production"))
CHALLENGE_FIELDS = frozenset(("schema","nonce","authority_id","receiver_id"))
SHA256 = re.compile(r"[0-9a-f]{64}\Z",re.ASCII)
NAME = re.compile(r"[A-Za-z0-9._-]{1,96}\Z",re.ASCII)


def check(test, code):
    if not test:
        raise ValueError(code)


def state_from_disk(path: Path) -> tuple[dict,bytes]:
    check(path.is_file() and not path.is_symlink(),"WITNESS_STATE_NOT_REGULAR")
    raw = path.read_bytes()
    check(0<len(raw)<=8192,"WITNESS_STATE_SIZE_INVALID")
    s = anchor.decode_strict(raw,"WITNESS_STATE_NOT_CANONICAL")
    check(type(s) is dict and set(s)==FIELDS and s["schema"]==SCHEMA
          and s["release"]=="RESEARCH_ONLY"
          and s["production"]=="NOT_AUTHORIZED","WITNESS_STATE_SCHEMA_INVALID")
    check(all(type(s[k]) is str and NAME.fullmatch(s[k]) is not None
              for k in ("authority_id","receiver_id")),
          "WITNESS_IDENTITY_INVALID")
    check(all(type(s[k]) is str and SHA256.fullmatch(s[k]) is not None
              for k in ("chain_sha256","head_sha256")),"WITNESS_HASH_INVALID")
    check(all(type(s[k]) is int and 1<=s[k]<=2**31-1
              for k in ("epoch","minimum_sequence","minimum_generation","counter")),
          "WITNESS_COUNTER_OR_FLOOR_INVALID")
    return s,raw


class Witness(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = False

    def __init__(self, state_file:Path, private_key:Path, port:int):
        self.state_file=state_file
        self.private_key=private_key
        self.gate=threading.Lock()
        self.last=None
        self.last_hash=None
        super().__init__(("127.0.0.1",port), Handler)

    def attest(self, request:dict):
        check(type(request) is dict and set(request)==CHALLENGE_FIELDS and
              request.get("schema")==CHALLENGE_SCHEMA,
              "WITNESS_CHALLENGE_SCHEMA_INVALID")
        check(type(request.get("nonce")) is str and
              SHA256.fullmatch(request["nonce"]) is not None,
              "WITNESS_NONCE_INVALID")
        check(all(type(request.get(k)) is str and NAME.fullmatch(request[k]) is not None
                  for k in ("authority_id","receiver_id")),
              "WITNESS_CHALLENGE_IDENTITIES_INVALID")
        with self.gate:
            state,raw=state_from_disk(self.state_file)
            check(state["authority_id"]==request["authority_id"] and
                  state["receiver_id"]==request["receiver_id"],
                  "WITNESS_RECEIVER_SCOPE_MISMATCH")
            fingerprint=hashlib.sha256(raw).hexdigest()
            if self.last is not None:
                prior=self.last
                check(state["counter"]>=prior["counter"] and
                      state["epoch"]>=prior["epoch"] and
                      state["minimum_sequence"]>=prior["minimum_sequence"] and
                      state["minimum_generation"]>=prior["minimum_generation"],
                      "WITNESS_IN_PROCESS_ROLLBACK_DENIED")
                check(state["counter"]!=prior["counter"] or
                      fingerprint==self.last_hash,
                      "WITNESS_SAME_COUNTER_STATE_MUTATION_DENIED")
            self.last=state
            self.last_hash=fingerprint
            certificate={
                "schema":ATTEST_SCHEMA,
                "nonce":request["nonce"],
                "authority_id":state["authority_id"],
                "receiver_id":state["receiver_id"],
                "chain_sha256":state["chain_sha256"],
                "head_sha256":state["head_sha256"],
                "epoch":state["epoch"],
                "minimum_sequence":state["minimum_sequence"],
                "minimum_generation":state["minimum_generation"],
                "counter":state["counter"],
                "release":"RESEARCH_ONLY",
                "production":"NOT_AUTHORIZED"
            }
            key=self.private_key
            check(key.is_file() and not key.is_symlink() and
                  shutil.which("ssh-keygen") is not None,
                  "WITNESS_SSHSIG_SIGNER_UNAVAILABLE")
            try:
                done=subprocess.run(["ssh-keygen","-Y","sign","-f",str(key),
                                     "-n",NAMESPACE],
                    input=anchor.canonical(certificate),capture_output=True,
                    timeout=15)
            except (OSError,subprocess.TimeoutExpired):
                raise ValueError("WITNESS_SSHSIG_SIGNER_UNAVAILABLE") from None
            check(done.returncode==0 and
                  done.stdout.startswith(b"-----BEGIN SSH SIGNATURE-----\n"),
                  "WITNESS_SSHSIG_SIGNING_FAILED")
            return {"schema":RESPONSE_SCHEMA,"statement":certificate,
                    "signature_b64":base64.b64encode(done.stdout).decode("ascii"),
                    "production":"NOT_AUTHORIZED"}


class Handler(BaseHTTPRequestHandler):
    def log_message(self,*args):
        return

    def do_GET(self):
        self.send_error(405)

    def do_POST(self):
        try:
            check(self.path=="/attest","WITNESS_UNKNOWN_ENDPOINT")
            declared=self.headers.get("Content-Length")
            check(declared is not None and declared.isdecimal() and
                  0<int(declared)<=4096,"WITNESS_REQUEST_SIZE_INVALID")
            raw=self.rfile.read(int(declared))
            challenge=anchor.decode_strict(raw,"WITNESS_CHALLENGE_NOT_CANONICAL")
            response=self.server.attest(challenge)
            code=200
        except (ValueError,authority.Denied,OSError) as exc:
            response={"schema":RESPONSE_SCHEMA,"status":"REJECTED",
                      "error_code":getattr(exc,"code",str(exc)),
                      "production":"NOT_AUTHORIZED"}
            code=400
        result=anchor.canonical(response)
        self.send_response(code)
        self.send_header("Content-Type","application/json")
        self.send_header("Content-Length",str(len(result)))
        self.send_header("Cache-Control","no-store")
        self.end_headers()
        self.wfile.write(result)


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--state-file",required=True)
    p.add_argument("--private-key",required=True)
    p.add_argument("--port",type=int,default=0)
    args=p.parse_args(argv)
    check(0<=args.port<65536,"WITNESS_PORT_INVALID")
    server=Witness(Path(args.state_file),Path(args.private_key),args.port)
    # Distinct OS process and real loopback endpoint; not distinct admin.
    print("APP022_WITNESS_PORT="+str(server.server_port),flush=True)
    try:
        server.serve_forever(poll_interval=.1)
    finally:
        server.server_close()


if __name__=="__main__":
    raise SystemExit(main())
