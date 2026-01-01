# RISC Zero - LeanSig Benchmark

## Overview

XMSS signature verification in RISC Zero zkVM. Uses `core` crate with no_std-compatible Poseidon2 from Plonky3.

## Quick Start

```bash
cd leansig_zkvm

# Development mode (no real proofs)
RISC0_DEV_MODE=1 cargo run --release -p host

# Production (real proofs)
cargo run --release -p host
```

## Benchmark Results

| Metric | Value |
|--------|-------|
| Total Cycles | 6,291,456 (~6.3M) |
| User Cycles | 5,728,806 (~5.7M) |
| Proving Time | 1,867.2 s (~31 min, CPU) |
| Verification Time | 189 ms |
| Receipt Size | 1.65 MB |

## Notes

- No Poseidon2 precompile; software implementation accounts for high cycle count
- Uses 32-bit RISC-V architecture
- GPU acceleration (`RISC0_CUDA=1`) or Bonsai service recommended for production
