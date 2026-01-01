# SP1 leanSig XMSS Verification

XMSS signature verification in SP1 zkVM for benchmarking against other zkVMs.

## Overview

This project proves XMSS signature verification using SP1 zkVM. It uses the same leanSig minimal library as the Zisk and RISC Zero implementations for fair comparison.

## Project Structure

```
sp1/
├── leansig-minimal/    # Core verification library (no_std)
├── program/            # SP1 guest program
├── script/             # Host script for benchmarking
├── test-gen/           # Test data generator
└── data/               # Generated test vectors
```

## Prerequisites

- Rust (stable)
- SP1 toolchain: `curl -L https://sp1up.succinct.xyz | bash && sp1up`

## Quick Start

```bash
# 1. Generate test data
cargo run -p test-gen

# 2. Build the guest program
cd program && cargo prove build --release && cd ..

# 3. Run execution (measure cycles)
cd script
cargo run --release -- --execute

# 4. Generate proof (warning: may take >10 minutes on CPU)
cargo run --release -- --prove
```

## Benchmark Results

| Metric | Value |
|--------|-------|
| **VM Cycles** | 135,801 |
| **Execution Time** | ~18ms |
| **Setup Time** | 1.5s |
| **Proving Time** | **71.4s** (CPU, M3 Max) |
| **Verification Time** | **160ms** |
| **Proof Size** | **1.28 MB** (Compressed) |

## Configuration

| Parameter | Value |
|-----------|-------|
| Tree Height | 18 (2^18 = 262,144 epochs) |
| Hash Function | Poseidon2 (KoalaBear field) |
| Message Length | 32 bytes |
| Hash Chains | 256 (DIMENSION) |

## Performance Analysis

Actual results show significantly better performance than initially expected:

- **Cycles**: 135,801 (vs predicted 10M+) - 74x better than expected!
- **Efficiency**: 14% faster than Zisk's 158K cycles
- **Proving**: 71s on CPU (M3 Max), GPU recommended for production
- **Verification**: 160ms - fast enough for real-time applications

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SP1_PROVER` | `local`, `mock`, `network` |
| `RUST_LOG` | Log level (info, debug, trace) |

## Comparison with Other zkVMs

| zkVM | Cycles | Architecture |
|------|--------|--------------|
| SP1 | **136K** | RISC-V 32bit |
| Zisk | 158K | RISC-V 64bit |
| RISC Zero | 6.3M | RISC-V 32bit |

Note: SP1's efficient execution is achieved with custom KoalaBear field implementation

## Notes

- SP1 uses BabyBear field internally, but leanSig requires KoalaBear
- No Poseidon2-KoalaBear precompile available
- All Poseidon2 operations run in software
