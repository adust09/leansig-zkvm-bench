# LeanSig zkVM Benchmarks

Benchmarking XMSS signature verification across multiple zero-knowledge virtual machines.

## Overview

This repository compares the performance of [LeanSig](https://github.com/geometryxyz/leanSig) XMSS signature verification across five zkVM platforms:

| zkVM | Status | Description |
|------|--------|-------------|
| **Miden VM** | WIP | Polygon's STARK-based VM with custom Miden Assembly |
| **OpenVM** | WIP | Succinct's modular zkVM with accelerated SHA-256 |
| **RISC Zero** | Done | RISC-V zkVM with STARK proofs |
| **SP1** | WIP | Succinct's RISC-V zkVM with STARK/Groth16 proofs |
| **Zisk** | Done | Polygon's high-performance zkVM |

## Benchmark Configuration

| Parameter | Value |
|-----------|-------|
| Signature Scheme | XMSS (eXtended Merkle Signature Scheme) |
| Tree Height | 18 (2^18 = 262,144 epochs) |
| Hash Function | Poseidon2 (KoalaBear field) |
| Message Length | 32 bytes |

## Benchmark Results

### Comparison Table

| Metric | Zisk | RISC Zero | SP1 | OpenVM | Miden VM |
|--------|------|-----------|-----|--------|----------|
| **VM Cycles** | 158,022 | 11,010,048 | WIP | WIP | WIP |
| **Execution Time** | 3.4 ms | 233 ms | - | - | - |
| **Proving Time** | ~26 min | >10 min* | - | - | - |
| **Memory** | ~10.5 GB | - | - | - | - |
| **Platform** | macOS (Apple Silicon) | macOS (M3) | - | - | - |

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

See [zisk/BENCHMARK.md](zisk/BENCHMARK.md) for details.

### RISC Zero

| Metric | Value |
|--------|-------|
| Total Cycles | 11,010,048 (~11M) |
| User Cycles | 10,246,516 (~10.2M) |
| Execution Time (dev) | 233.39 ms |
| Receipt Size (dev) | 473 bytes |
| Proving Time (CPU) | >10 minutes (timeout) |

See [risc0/FEASIBILITY_REPORT.md](risc0/FEASIBILITY_REPORT.md) for details.

### SP1

Work in progress. SP1 implementation using Succinct's RISC-V zkVM.

### OpenVM

Work in progress. OpenVM implementation with accelerated SHA-256 and Poseidon2-KoalaBear verification.

See [openvm/README.md](openvm/README.md) for details.

### Miden VM

Work in progress. Poseidon2 implementation in Miden Assembly is under development.

### Analysis

- **Zisk** achieves ~70x fewer cycles than RISC Zero for the same verification
- **RISC Zero** overhead is primarily due to software Poseidon2 (no precompile)
- Both zkVMs use Poseidon2 over KoalaBear field for hash operations

### Challenges

**Zisk**
- Currently tested with synthetic data; real signature integration pending
- macOS proving is slow (~26 min) due to lack of AVX2/AVX-512; Linux expected 5-10x faster
- Final SNARK proof requires aggregation server (`-f` flag)

**RISC Zero**
- No Poseidon2 precompile: ~11M cycles vs ~15K cycles for SHA-256
- `no_std` port required: `OnceLock` → direct init, `ethereum_ssz` → custom serialization
- CPU proving impractical (>10 min timeout); GPU/Bonsai recommended

**SP1**
- Similar architecture to RISC Zero (RISC-V based)
- No native Poseidon2-KoalaBear precompile; software implementation required
- Supports Groth16/PLONK proof wrapping for on-chain verification

**OpenVM**
- Guest must be `#![no_std]` — leanSig is host-only, guest receives pre-serialized data
- Poseidon2-KoalaBear re-implemented in guest (cannot link leanSig)
- Statement message must be 32-byte SHA-256 digest (not raw message)

**Miden VM**
- Poseidon2-KoalaBear must be implemented in Miden Assembly from scratch
- Miden's native field is Goldilocks (p = 2^64 - 2^32 + 1), not KoalaBear
- No existing XMSS/Merkle tree library for MASM

## Quick Start

### Zisk

```bash
cd zisk

# Build
cargo-zisk build --release

# Run emulator
ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -m -c

# Generate proof
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -l -v
```

### RISC Zero

```bash
cd risc0/leansig_zkvm

# Development mode (no real proofs)
RISC0_DEV_MODE=1 cargo run

# Production
cargo run --release
```

### SP1

```bash
cd sp1

# Build and run (WIP)
cargo prove build
cargo run --release
```

### OpenVM

```bash
cd openvm

# Run default benchmark (generate → prove → verify)
cargo run --release --bin xmss-host

# Build guest for OpenVM
cd guest && cargo openvm build --release
```

### Miden VM

```bash
cd miden

# Run tests
miden-run tests/poseidon2_full_test.masm
```

## References

- [LeanSig Paper](https://eprint.iacr.org/2024/1205)
- [Zisk Documentation](https://docs.zisk.io)
- [RISC Zero Docs](https://dev.risczero.com)
- [SP1 Docs](https://docs.succinct.xyz)
- [OpenVM Docs](https://docs.openvm.dev)
- [Miden VM Docs](https://docs.polygon.technology/miden)
