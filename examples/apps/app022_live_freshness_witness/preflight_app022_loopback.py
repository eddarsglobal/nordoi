#!/usr/bin/env python3
"""APP-022 native loopback bind preflight; does NOT simulate cryptography.

The witness HTTP server must bind literal 127.0.0.1 without DNS/FQDN
resolution, even on macOS runner hosts with unreliable reverse DNS.
This only verifies real OS socket binding/connection, not process isolation
or SSHSIG. The full APP022 tests prove separate-process cryptography.
"""
import pathlib
import socket
import tempfile
from unittest.mock import patch
import witness_app022 as witness

with tempfile.TemporaryDirectory(prefix="nordoi-app022-loopback-") as temp:
    with patch("socket.getfqdn",
               side_effect=RuntimeError("DNS_LOOKUP_MUST_NOT_HAPPEN")):
        server=witness.Witness(pathlib.Path(temp)/"unused-state",
                               pathlib.Path(temp)/"unused-key",0)
        try:
            assert server.server_address[0]=="127.0.0.1"
            assert server.server_name=="127.0.0.1"
            port=server.server_port
            assert 1<=port<65536
            with socket.create_connection(("127.0.0.1",port),timeout=4) as client:
                assert client.getpeername()[0]=="127.0.0.1"
        finally:
            server.server_close()
print("APP022_REAL_OS_IPV4_LOOPBACK_BIND_NO_REVERSE_DNS=PASS")
