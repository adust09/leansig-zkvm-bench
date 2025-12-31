# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository benchmarks XMSS (eXtended Merkle Signature Scheme) signature verification from [leanSig](https://github.com/geometryxyz/leanSig) across five zkVM platforms: Miden VM, OpenVM, RISC Zero, SP1, and Zisk.

## Architecture

Each zkVM has a different integration approach due to `no_std` constraints:

| zkVM | Approach | Language |
|------|----------|----------|
| **Miden** | Full re-implementation in Miden Assembly (MASM) | MASM |
| **OpenVM** | Host uses leanSig; guest re-implements verification | Rust |
| **RISC Zero** | Shared `core` crate extracted for no_std | Rust |
| **SP1** | Uses `leansig-minimal` library | Rust |
| **Zisk** | Uses `leansig-minimal` library | Rust |

Key constraint: leanSig uses `std` features (rayon, dashmap). Verification logic must be extracted or re-implemented for zkVM guests.

## Build Commands

### Zisk
```bash
cd zisk
cargo-zisk build --release
ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -m -c  # Emulator
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -l -v  # Prove
```

### RISC Zero
```bash
cd risc0/leansig_zkvm
RISC0_DEV_MODE=1 cargo run --release -p host  # Dev mode (no real proofs)
cargo run --release -p host  # Production
```

### SP1
```bash
cd sp1
cargo run -p test-gen  # Generate test data
cd program && cargo prove build --release && cd ..  # Build guest
cd script && cargo run --release -- --execute  # Measure cycles
cd script && cargo run --release -- --prove  # Generate proof
```

### OpenVM
```bash
cd openvm
cargo run --release --bin xmss-host  # Run benchmark
cd guest && cargo openvm build --release  # Build guest
```

### Miden VM
```bash
cd miden
miden-run tests/poseidon2_full_test.masm  # Run MASM tests
```

## Directory Structure

```
├── miden/           # Miden Assembly implementation
│   ├── masm/        # Core MASM modules (field ops, Poseidon2)
│   └── tests/       # MASM test files
├── openvm/          # OpenVM implementation
│   ├── guest/       # no_std guest program
│   ├── host/        # std host with leanSig
│   ├── lib/         # Shared library (xmss-lib)
│   └── xmss-types/  # Serialization types
├── risc0/           # RISC Zero implementation
│   └── leansig_zkvm/
│       ├── core/    # Shared no_std verification logic
│       ├── host/    # Host benchmarking
│       └── methods/ # Guest program
├── sp1/             # SP1 implementation
│   ├── leansig-minimal/  # Core verification library
│   ├── program/     # SP1 guest program
│   ├── script/      # Host script
│   └── test-gen/    # Test data generator
└── zisk/            # Zisk implementation
    ├── leansig-minimal/  # Core verification library
    ├── verifier/    # Guest program
    └── test-gen/    # Test data generator
```

## Cryptographic Parameters

| Parameter | Value |
|-----------|-------|
| Hash Function | Poseidon2 (KoalaBear field, p = 2^31 - 2^24 + 1) |
| Tree Height | 18 (2^18 = 262,144 signatures per key) |
| Encoding | TargetSum (W=1, 155 chains) |
| Chain Length | 2 steps per chain |

## Key Dependencies

- **Plonky3**: `p3-koala-bear`, `p3-poseidon2` for field and hash implementations
- All guest programs must be `#![no_std]`
- Uses `alloc` crate for heap allocation in no_std environments

## Performance Reference

| zkVM | VM Cycles | Proving Time | Status |
|------|-----------|--------------|--------|
| SP1 | ~136K | 71.4 s | Done |
| Zisk | ~158K | ~26 min (macOS) | Done |
| RISC Zero | ~11M | >10 min | Done |
| OpenVM | - | ~4.9 min | Done |
| Miden | ~15.5M | OOM | WIP (proof blocked) |

Note: Proving on macOS is slow due to lack of AVX2/AVX-512. Linux recommended for benchmarking.
