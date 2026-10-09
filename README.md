# NORDOI P2.5 — Capability-Secured File Output

Production Profile 1 and P2.1-P2.4 are immutable certified baselines. P2.5 opens the first bounded filesystem write capability without introducing ambient filesystem authority.

Canonical source:

```noi
module app.main;

effect FileWrite;

entry main writes "report.txt" emits "Hello from NORDOI P2.5!\n";
```

Without authority:

```bash
cargo run --quiet --bin nordoi -- \
  file-write examples/p2_5_file/report.noi
```

The command fails closed with no file creation.

With explicit host authority:

```bash
mkdir -p /tmp/nordoi-p25-output

cargo run --quiet --bin nordoi -- \
  file-write examples/p2_5_file/report.noi \
  --grant-output-dir /tmp/nordoi-p25-output
```

Expected artifact:

```text
/tmp/nordoi-p25-output/report.txt
```

with exact bytes:

```text
Hello from NORDOI P2.5!\n
```

Security boundary:

- exact `effect FileWrite;` declaration;
- exact `Capability::FileWrite("report.txt")` authority;
- one ordinary relative file name only;
- no `/`, `\\`, `.`, `..` or control characters;
- file name <= 128 UTF-8 bytes;
- file content <= 4096 UTF-8 bytes;
- host output directory must already exist;
- create-new only; overwrite is denied;
- canonical receipt excludes host absolute path;
- zero ambient filesystem authority;
- no new kernel, runtime or NAIR semantics.

Public version remains intentionally frozen:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
