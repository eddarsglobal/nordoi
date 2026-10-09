# NORDOI P2.7 — Bounded Atomic Multi-File Bundle Output

Production Profile 1 and P2.1-P2.6 are immutable certified baselines. P2.7 composes multiple P2.6-style dynamic file outputs into one bounded create-new bundle publication while preserving exact capabilities and zero ambient filesystem authority.

## Candidate surface

```noi
module app.main;

effect FileWrite;

input key_code;
const bias = 1;

entry main writes "report-bundle" {
    "report.txt" emits "input=" + key_code + ", next=" + (key_code + bias);
    "summary.txt" emits "accepted=" + (key_code >= 40) + ", key=" + key_code;
};
```

Run:

```bash
nordoi bundle-write examples/p2_7_bundle/report.noi 40 \
  --grant-output-dir /tmp/nordoi-p27-output
```

The grant is compiled into exact target capabilities only:

```text
FileWrite("report-bundle/report.txt")
FileWrite("report-bundle/summary.txt")
```

P2.7 validates every target and quota, authorizes all exact capabilities, evaluates all dynamic outputs in memory, prepares them in private same-root staging, then publishes the final bundle with one directory rename. The final namespace is create-new only; overwrite is denied.

P2.7 claims no crash durability, cross-filesystem transaction, distributed transaction, hostile concurrent-host mutation resistance, read, append, arbitrary path, network, time, randomness or process authority.

Bounds: 2..8 files, each P2.6-bounded to 4096 UTF-8 bytes, global worst-case <= 16384 UTF-8 bytes, bundle/file names <= 128 UTF-8 bytes and no traversal.

Normative candidate design: `docs/NOI_ATOMIC_MULTI_FILE_BUNDLE_OUTPUT_SPEC_P2_7.md`.
Architecture/research record: `research/ATOMIC_MULTI_FILE_BUNDLE_OUTPUT_INTELLIGENCE_P2_7.md`.

Public certified version output remains intentionally frozen:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
