# R0.17 Independent Replay — TEST-ONLY Specification

Baseline: `r0.16` `a5d7e9669a9b6d4be6a47bed17756946df24c9bb`. **RESEARCH_ONLY**: 12 native gates **UNMET**, 12 external verification records **ABSENT**. No production/native authority.

## Canonical transcript: 595 bytes

- 9-byte magic `NDR17RPL1`
- 40 ASCII bytes of frozen commit `a5d7e9669a9b6d4be6a47bed17756946df24c9bb`
- 2-byte little-endian record count `12`
- 32 raw bytes of frozen R0.16 bundle SHA-256 (`f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393`)
- 12 records × 40 bytes: `index: u16le`, `payload_offset: u32le`, `payload_length: u16le`, `SHA-256(payload): 32 bytes`
- 32 bytes of `SHA-256(all preceding transcript bytes)` as a terminal checksum

The whole transcript is anchored by SHA-256 `1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab`. A matching checksum alone is NOT provenance, and rehashing an altered payload or metadata MUST NOT bypass the pinned baseline.

## Verification cross-check

Reader A walks R0.16's bounded packet wire format using length prefixes and a moving cursor. Reader B reads independently at the frozen manifest offsets, validates each length prefix and ensures non-overlap/exact coverage. Both must agree exactly on offsets, lengths, and payload SHA-256. A Python `hashlib` reference produces the identical canonical bytes; the Rust test uses the frozen Rust SHA-256. Neither pathway follows external file references or performs network I/O.

## Fail-closed

Corruption, wrong endianness, lengths, count, offset overlap, missing/extra bytes, reordered records, altered manifest or R0.17 registry, modified transcript/checksum, forged trust root or admitted evidence claim return `Invalid`. Missing synthetic CI or synthetic review returns `Blocked`. Matching public fixtures with both synthetic flags return only `ResearchReviewOnly`.

The two decoders and the Python/Rust comparison are **independent local implementations** of the same public bounded fixture, **NOT** independent third-party evidence verification, digital signatures, an external trust-root, or a production security proof. Native execution, installation and deployment remain **NOT** authorized.

44 Rust governance witnesses + two inherited FIPS SHA-256 reference checks = 46 expected tests. GitHub CI, full release gate and true external review remain separate requirements.
