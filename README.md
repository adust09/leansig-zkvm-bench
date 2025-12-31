# LeanSig zkVM Benchmarks

Benchmarking XMSS signature verification across multiple zero-knowledge virtual machines.

## Overview

This repository compares the performance of [LeanSig](https://github.com/geometryxyz/leanSig) XMSS signature verification across five zkVM platforms:

| zkVM | Status | Description |
|------|--------|-------------|
| **Miden VM** | WIP | Polygon's STARK-based VM with custom Miden Assembly |
| **OpenVM** | Done | Succinct's modular zkVM with accelerated SHA-256 |
| **RISC Zero** | Done | RISC-V zkVM with STARK proofs |
| **SP1** | Done | Succinct's RISC-V zkVM with STARK/Groth16 proofs |
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

| Metric | SP1 | Zisk | OpenVM | RISC Zero | Miden VM |
|--------|-----|------|--------|-----------|----------|
| **VM Cycles** | 135,801 | 158,022 | - | 11,010,048 | 15,552,770 |
| **Execution Time** | 18 ms | 3.4 ms | - | 233 ms | 16 s |
| **Proving Time** | 71.4 s | ~26 min | ~4.9 min | >10 min* | OOM** |
| **Verification Time** | 160 ms | - | 2.78 s | - | - |
| **Memory** | - | ~10.5 GB | 6.24 GB | - | - |
| **Platform** | macOS (M3 Max) | macOS (Apple Silicon) | macOS (Apple Silicon) | macOS (M3) | macOS (M2) |

*RISC Zero production proof did not complete within timeout on CPU.
**Miden VM proof generation runs out of memory on MacBook Air M2 (~15.5M cycles exceeds hardware limits).

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

| Metric | Value |
|--------|-------|
| VM Cycles | 135,801 |
| Execution Time | ~18 ms |
| Setup Time | 1.5 s |
| Proving Time | 71.4 s (CPU, M3 Max) |
| Verification Time | 160 ms |
| Proof Size | 1.28 MB (Compressed) |

See [sp1/README.md](sp1/README.md) for details.

### OpenVM

| Metric | Value |
|--------|-------|
| Signatures | 2 |
| Input Generation | 150.4 ms |
| Proving Time | 294.5 s (~4.9 min) |
| Verification Time | 2.78 s |
| Peak Memory | 6.24 GiB |
| Platform | macOS (Apple Silicon) |

See [openvm/README.md](openvm/README.md) for details.

### Miden VM

| Metric | Value |
|--------|-------|
| VM Cycles | 15,552,770 (~15.5M) |
| Execution Time | 16 s |
| Proving Time | OOM (killed after 11+ min) |
| Status | Implementation complete, proof generation blocked |

Miden VM implementation is functionally complete but cannot generate STARK proofs at scale due to hardware memory limits (~15.5M cycles exceeds MacBook Air M2 capacity). Smaller tests (41 cycles) successfully generate proofs in 31ms.

See [miden/PROGRESS.md](miden/PROGRESS.md) for details.

### Analysis

- **SP1** achieves the lowest cycle count (136K) and fastest proving time (71s)
- **Zisk** has comparable cycles (158K) but slower proving (~26 min on macOS)
- **OpenVM** has competitive proving time (~5 min) with efficient memory usage (6.24 GB)
- **RISC Zero** overhead (~11M cycles) is primarily due to software Poseidon2 (no precompile)
- **Miden VM** (~15.5M cycles) requires field arithmetic emulation (KoalaBear on Goldilocks)
- All zkVMs use Poseidon2 over KoalaBear field for hash operations (software implementation)

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
- Poseidon2-KoalaBear implemented in Miden Assembly from scratch (complete)
- Miden's native field is Goldilocks (p = 2^64 - 2^32 + 1), not KoalaBear
- ~15.5M cycles for full verification exceeds MacBook Air M2 memory limits
- Proof generation requires more powerful hardware or algorithm optimization

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

# Generate test data
cargo run -p test-gen

# Build guest program
cd program && cargo prove build --release && cd ..

# Execute (measure cycles)
cd script && cargo run --release -- --execute

# Generate proof
cd script && cargo run --release -- --prove
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
