# Zisk - LeanSig Benchmark

## Overview

XMSS signature verification in Zisk zkVM. Uses `leansig-minimal` library (no_std) shared with SP1 implementation.

## Quick Start

```bash
# Build verifier
cargo-zisk build --release

# Generate test data
cd test-gen && cargo run

# Run emulator
ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -m -c

# Generate proof
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/verifier -i data/input.bin -l -v
```

## Benchmark Results

| Metric | Value |
|--------|-------|
| VM Cycles | 158,022 |
| Execution Time | 3.4 ms |
| Proving Time | 1,253.9 s (~21 min, macOS) |
| Peak Memory | ~10.45 GB |
| Proof Type | FRI (local) |

## Notes

- Uses 64-bit RISC-V architecture
- macOS proving is slow; Linux with AVX2/AVX-512 expected 5-10x faster
- Use `-l` for local FRI proof, `-f` for final SNARK (requires aggregation server)
