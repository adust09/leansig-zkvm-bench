# RISC Zero Benchmarks

This project contains benchmarks for RISC Zero zkVM performance measurement.

## Quick Start

### Prerequisites
- Rust (via rustup.rs)
- 16GB+ RAM for local proving

### Installation

```bash
# Install RISC Zero toolchain
curl -L https://risczero.com/install | bash
rzup install

# Verify installation
cargo risczero --version
```

## Claude Code Skills

This project includes custom Claude Code skills for RISC Zero development:

| Command | Description |
|---------|-------------|
| `/risc0-setup` | Set up development environment |
| `/risc0-new-project <name>` | Create new zkVM project |
| `/risc0-benchmark` | Run benchmarks |
| `/risc0-prove` | Generate proofs |
| `/risc0-verify` | Verify receipts |
| `/risc0-optimize` | Optimize performance |

## Benchmark Metrics

| Metric | Description |
|--------|-------------|
| Cycles | RISC-V instruction count |
| Segments | Proof parallelization units |
| Proving Time | Wall-clock proof generation |
| Receipt Size | Proof size in bytes |

## Development Modes

```bash
# Fast development (no real proofs)
RISC0_DEV_MODE=1 cargo run

# Production (real proofs)
RISC0_DEV_MODE=0 cargo run --release

# GPU accelerated
RISC0_CUDA=1 cargo run --release
```

## Resources

- [RISC Zero API Docs](https://dev.risczero.com/api)
- [Official Benchmarks](https://reports.risczero.com)
- [GitHub Repository](https://github.com/risc0/risc0)
