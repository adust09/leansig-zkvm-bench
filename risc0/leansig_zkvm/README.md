# leanSig XMSS Verification in RISC Zero zkVM

A proof-of-concept implementation proving XMSS signature verification from the [leanSig](https://github.com/leanEthereum/leanSig) library using RISC Zero zkVM.

## Overview

This project demonstrates the feasibility of proving post-quantum XMSS signature verification in a general-purpose zkVM. It ports the essential verification logic from leanSig to a `no_std` compatible format.

## Benchmark Results

| Metric | Value |
|--------|-------|
| Total Cycles | 6,291,456 (~6.3M) |
| User Cycles | 5,728,806 (~5.7M) |
| Proving Time | 1,867.2 s (~31 min) |
| Verification Time | 189 ms |
| Receipt Size | 1.65 MB |

See [FEASIBILITY_REPORT.md](../FEASIBILITY_REPORT.md) for detailed analysis.

## Project Structure

```
leansig_zkvm/
├── core/                     # Shared no_std types and logic
│   ├── src/
│   │   ├── field.rs          # KoalaBear field (p = 2^31 - 2^24 + 1)
│   │   ├── poseidon.rs       # Poseidon2 hash function
│   │   ├── tweak_hash.rs     # Tweakable hash with domain separation
│   │   ├── types.rs          # Signature, PublicKey, etc.
│   │   └── verify.rs         # XMSS verification algorithm
│   └── Cargo.toml
├── methods/
│   └── guest/
│       └── src/main.rs       # RISC Zero guest program
├── host/
│   └── src/main.rs           # Host program with benchmarking
└── Cargo.toml
```

## Quick Start

### Prerequisites

```bash
# Install RISC Zero toolchain
curl -L https://risczero.com/install | bash
rzup install
```

### Run Benchmarks

```bash
# Dev mode (fast, measures cycles)
cd leansig_zkvm
RISC0_DEV_MODE=1 cargo run --release -p host

# Production mode (full proving)
cargo run --release -p host
```

## Key Implementation Details

### Poseidon2 Hash Function

Uses Plonky3's no_std compatible Poseidon2 implementation:
- Field: KoalaBear (prime p = 2^31 - 2^24 + 1)
- Width: 16 field elements
- Sponge capacity: 8 elements
- Output: 8 field elements (256 bits)

### XMSS Verification

The verification process:
1. Parse signature (Merkle path, encoding randomness, hash chains)
2. Derive one-time public key by extending hash chains
3. Compute tweak hash of chain results
4. Verify Merkle path against public key root

### Domain Separation

Uses tweakable hashing with two tweak types:
- `TreeTweak(epoch, position)` - for Merkle tree operations
- `ChainTweak(epoch, chain_idx, step)` - for hash chain iteration

## Dependencies

| Crate | Purpose |
|-------|---------|
| `p3-koala-bear` | Field implementation |
| `p3-poseidon2` | Hash function |
| `p3-symmetric` | Compression function |
| `risc0-zkvm` | zkVM runtime |
| `serde` | Serialization |

All Plonky3 crates pinned to git rev `a33a312` for API stability.

## Limitations

1. **Test Data Only**: Uses synthetic test data; real leanSig signatures require the full library
2. **Performance**: ~6.3M cycles is significant; consider leanMultisig for production
3. **No GPU**: CPU-only benchmarks; GPU proving available via `RISC0_PROVER=cuda`

## License

MIT
