# leanSig XMSS Verification - Zisk zkVM Benchmark

## Overview

Benchmark of XMSS signature verification in Zisk zkVM using the leanSig minimal implementation.

**Date**: 2025-12-27
**Platform**: macOS 14+ (Apple Silicon)
**Zisk Version**: 0.15.0

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Zisk zkVM Guest                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  verifier (RISC-V binary)                           │   │
│  │    └── leansig-minimal (no_std)                     │   │
│  │          ├── Poseidon2 hash (Width-16/24)           │   │
│  │          ├── Merkle tree verification (18 levels)   │   │
│  │          ├── Hash chain verification (DIMENSION)    │   │
│  │          └── Encoding/decoding                      │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Benchmark Results

### Test Configuration

| Parameter | Value |
|-----------|-------|
| Tree Height | 18 (2^18 = 262,144 epochs) |
| Hash Function | Poseidon2 (KoalaBear field) |
| Message Length | 32 bytes |
| Input Size | 5,729 bytes |

### Emulator Performance

| Metric | Value |
|--------|-------|
| **VM Cycles** | 158,022 |
| **Execution Time** | 3.4 ms |
| **Throughput** | 45.97 Msteps/s |
| **Output** | 0 (synthetic data - expected) |

### Proving Performance (macOS)

| Metric | Value |
|--------|-------|
| **Proving Time** | 1,580.3 seconds (~26.3 minutes) |
| **Memory Required** | ~10.45 GB |
| **AIR Instances** | 13 |
| **Proof Type** | FRI (local) |

## Commands

### Build Verifier
```bash
cargo-zisk build --release
```

### Generate Test Data
```bash
cd test-gen && cargo run
```

### Run Emulator
```bash
ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -m -c
```

### Generate ZK Proof
```bash
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -l -v
```

## Notes

- **macOS proving works** but is slower than Linux due to lack of AVX2/AVX-512 SIMD optimizations
- The `-l` flag generates local FRI proofs; use `-f` for final SNARK proof (requires aggregation server)
- Synthetic test data is used; real signatures require integration with full leanSig library

## Cost Analysis

For 158K cycles:
- **Emulator**: ~3.4ms (development/debugging)
- **FRI Proving**: ~26 minutes (privacy-preserving verification)
- **Estimated Linux**: ~5-10x faster with AVX2/AVX-512

## Future Work

1. Benchmark on Linux with AVX2/AVX-512 optimizations
2. Generate final SNARK proof for on-chain verification
3. Test with real (valid) leanSig signatures
4. Optimize Poseidon2 implementation for zkVM constraints
