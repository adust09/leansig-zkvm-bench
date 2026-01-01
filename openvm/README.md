# OpenVM - LeanSig Benchmark

## Overview

XMSS signature verification in OpenVM zkVM. Host uses leanSig for key/signature generation; guest re-implements Poseidon2-KoalaBear verification in no_std.

## Quick Start

```bash
# Run benchmark (generate → prove → verify)
cargo run --release --bin xmss-host

# Build guest
cd guest && cargo openvm build --release
```

## Benchmark Results

| Metric | Value |
|--------|-------|
| Signatures | 2 |
| Input Generation | 150.4 ms |
| Proving Time | 294.5 s (~4.9 min) |
| Verification Time | 2.78 s |
| Peak Memory | 6.24 GiB |

## Notes

- Host links leanSig; guest receives pre-serialized data
- Uses accelerated SHA-256 for statement commitment
- Poseidon2-KoalaBear re-implemented in guest (cannot link leanSig)
