# SP1 - LeanSig Benchmark

## Overview

XMSS signature verification in SP1 zkVM. Uses `leansig-minimal` library (no_std) shared with Zisk implementation.

## Quick Start

```bash
# Generate test data
cargo run -p test-gen

# Build guest program
cd program && cargo prove build --release && cd ..

# Execute (measure cycles)
cd script && cargo run --release -- --execute

# Generate proof
cd script && cargo run --release -- --prove
```

## Benchmark Results

| Metric | Value |
|--------|-------|
| VM Cycles | 135,801 |
| Execution Time | ~18 ms |
| Proving Time | 71.4 s (CPU, M3 Max) |
| Verification Time | 160 ms |
| Proof Size | 1.28 MB (compressed) |

## Notes

- Uses 32-bit RISC-V architecture
- No Poseidon2-KoalaBear precompile; software implementation
- Most efficient cycle count among tested zkVMs
