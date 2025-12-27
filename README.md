# LeanSig zkVM Benchmarks

Benchmarking XMSS signature verification across multiple zero-knowledge virtual machines.

## Overview

This repository compares the performance of [LeanSig](https://github.com/geometryxyz/leanSig) XMSS signature verification across three zkVM platforms:

| zkVM | Status | Description |
|------|--------|-------------|
| **Miden VM** | WIP | Polygon's STARK-based VM with custom Miden Assembly |
| **RISC Zero** | WIP | RISC-V zkVM with STARK proofs |
| **Zisk** | Done | Polygon's high-performance zkVM |

## Project Structure

```
.
├── miden-leanSig/     # Miden VM implementation (MASM)
│   ├── masm/          # Miden Assembly source
│   └── tests/         # Test files
├── risc0-leansig/     # RISC Zero implementation
│   └── leansig_zkvm/  # Guest/Host code
└── zisk-leanSig/      # Zisk implementation
    ├── leansig-minimal/  # no_std core library
    └── verifier/         # zkVM guest program
```

## Benchmark Configuration

| Parameter | Value |
|-----------|-------|
| Signature Scheme | XMSS (eXtended Merkle Signature Scheme) |
| Tree Height | 18 (2^18 = 262,144 epochs) |
| Hash Function | Poseidon2 (KoalaBear field) |
| Message Length | 32 bytes |

## Benchmark Results

### Comparison Table

| Metric | Zisk | RISC Zero | Miden VM |
|--------|------|-----------|----------|
| **VM Cycles** | 158,022 | 11,010,048 | WIP |
| **Execution Time** | 3.4 ms | 233 ms | - |
| **Proving Time** | ~26 min | >10 min* | - |
| **Memory** | ~10.5 GB | - | - |
| **Platform** | macOS (Apple Silicon) | macOS (M3) | - |

*RISC Zero production proof did not complete within timeout on CPU.

### Zisk

| Metric | Value |
|--------|-------|
| VM Cycles | 158,022 |
| Emulator Time | 3.4 ms |
| Throughput | 45.97 Msteps/s |
| Proving Time | ~26 minutes |
| Memory | ~10.5 GB |
| AIR Instances | 13 |

See [zisk-leanSig/BENCHMARK.md](zisk-leanSig/BENCHMARK.md) for details.

### RISC Zero

| Metric | Value |
|--------|-------|
| Total Cycles | 11,010,048 (~11M) |
| User Cycles | 10,246,516 (~10.2M) |
| Execution Time (dev) | 233.39 ms |
| Receipt Size (dev) | 473 bytes |
| Proving Time (CPU) | >10 minutes (timeout) |

See [risc0-leansig/FEASIBILITY_REPORT.md](risc0-leansig/FEASIBILITY_REPORT.md) for details.

### Miden VM

Work in progress. Poseidon2 implementation in Miden Assembly is under development.

### Analysis

- **Zisk** achieves ~70x fewer cycles than RISC Zero for the same verification
- **RISC Zero** overhead is primarily due to software Poseidon2 (no precompile)
- Both zkVMs use Poseidon2 over KoalaBear field for hash operations

## Quick Start

### Zisk

```bash
cd zisk-leanSig

# Build
cargo-zisk build --release

# Run emulator
ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -m -c

# Generate proof
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -l -v
```

### RISC Zero

```bash
cd risc0-leansig/leansig_zkvm

# Development mode (no real proofs)
RISC0_DEV_MODE=1 cargo run

# Production
cargo run --release
```

### Miden VM

```bash
cd miden-leanSig

# Run tests
miden-run tests/poseidon2_full_test.masm
```

## References

- [LeanSig Paper](https://eprint.iacr.org/2024/1205)
- [Zisk Documentation](https://docs.zisk.io)
- [RISC Zero Docs](https://dev.risczero.com)
- [Miden VM Docs](https://docs.polygon.technology/miden)

## License

MIT
