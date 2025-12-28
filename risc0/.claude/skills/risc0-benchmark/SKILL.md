---
name: RISC Zero Benchmark
description: This skill should be used when the user asks to "benchmark risc zero", "measure zkvm performance", "count cycles", "measure proving time", "profile risc0 app", "optimize zkvm", or mentions performance measurement for RISC Zero applications.
version: 0.1.0
---

# RISC Zero Benchmark

Measure and analyze RISC Zero zkVM performance metrics.

## Purpose

Provide comprehensive guidance for benchmarking RISC Zero applications, including cycle counting, proving time measurement, memory profiling, and performance optimization strategies.

## Key Metrics

| Metric | Description | Impact |
|--------|-------------|--------|
| Cycles | RISC-V instruction count | Lower = faster proving |
| Segments | Proof parallelization units | More = more memory |
| Proving Time | Wall-clock proof generation | Hardware dependent |
| Receipt Size | Proof size in bytes | ~200KB typical |

## Cycle Counting

Measure RISC-V cycles in guest code:

```rust
use risc0_zkvm::guest::env;

let start = env::cycle_count();
// ... computation to measure ...
let end = env::cycle_count();
let cycles_used = end - start;
env::log(&format!("Cycles: {}", cycles_used));
```

## Proving Time Measurement

Measure end-to-end proving time in host:

```rust
use std::time::Instant;
use risc0_zkvm::{default_prover, ExecutorEnv};

let env = ExecutorEnv::builder()
    .write_slice(&input)
    .build()?;

let prover = default_prover();

let start = Instant::now();
let receipt = prover.prove(env, ELF)?;
let proving_time = start.elapsed();

println!("Proving time: {:?}", proving_time);
println!("Receipt size: {} bytes", bincode::serialize(&receipt)?.len());
```

## Benchmark Modes

### Development Mode (No Real Proofs)

```bash
RISC0_DEV_MODE=1 cargo bench
```

Fast iteration without actual proof generation.

### Production Mode

```bash
RISC0_DEV_MODE=0 cargo bench --release
```

Real proofs with actual cryptographic operations.

### GPU-Accelerated

```bash
RISC0_CUDA=1 cargo bench --release
```

NVIDIA GPU acceleration for faster proving.

## Hardware Recommendations

| Configuration | RAM | Use Case |
|--------------|-----|----------|
| Minimum | 16GB | Small proofs |
| Recommended | 32GB+ | Production workloads |
| GPU | CUDA GPU | Maximum performance |
| Cloud | Boundless | Heavy computations |

## Profiling

Enable cycle profiling:

```bash
RISC0_PPROF=1 cargo run --release
```

Analyze hotspots and optimize critical paths.

## Additional Resources

### Reference Files

- **`references/optimization-guide.md`** - Detailed optimization strategies
- **`references/hardware-comparison.md`** - Hardware performance data

### Scripts

- **`scripts/benchmark-harness.rs`** - Complete benchmark template

### Examples

- **`examples/basic-benchmark.rs`** - Simple benchmark example
