# NORDOI P2.6 — Dynamic Capability-Secured File Output

Production Profile 1 and P2.1-P2.5 are immutable certified baselines. P2.6 composes the certified P2.4 multi-segment runtime renderer with the certified P2.5 create-new file boundary without adding ambient filesystem or console authority.

Canonical source:

```noi
module app.main;

effect FileWrite;

input key_code;
const bias = 1;

entry main writes "report.txt" emits "input=" + key_code + ", next=" + (key_code + bias) + ", accepted=" + (key_code >= 40);
```

Without authority:

```bash
cargo run --quiet --bin nordoi -- \
  dynamic-file-write examples/p2_6_dynamic_file/report.noi 40
```

The command fails closed with no file creation.

With explicit host authority:

```bash
mkdir -p /tmp/nordoi-p26-output

cargo run --quiet --bin nordoi -- \
  dynamic-file-write examples/p2_6_dynamic_file/report.noi 40 \
  --grant-output-dir /tmp/nordoi-p26-output
```

Expected artifact:

```text
/tmp/nordoi-p26-output/report.txt
```

with exact bytes:

```text
input=40, next=41, accepted=true
```

Security boundary:

- exact `effect FileWrite;` declaration;
- exact `Capability::FileWrite("report.txt")` authority;
- no `ConsoleWrite` grant or observable console side effect;
- P2.4-style 2..8 ordered runtime Int/Bool segments only;
- one ordinary relative file name under the P2.5 target policy;
- total dynamic output <= 4096 UTF-8 bytes, proved by the P2.4 plan before authority;
- host output directory must already exist;
- create-new only; overwrite is denied;
- canonical P2.6 receipt excludes host absolute path;
- deterministic per-segment runtime proofs and global receipt;
- no new kernel, runtime or NAIR semantics.

Public version remains intentionally frozen:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
