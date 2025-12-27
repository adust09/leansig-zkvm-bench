# RISC Zero Optimization Guide

Detailed strategies for optimizing RISC Zero zkVM applications.

## Optimization Hierarchy

Impact levels from highest to lowest:

1. **Algorithm Choice** (10-1000x impact)
2. **Data Structure Selection** (2-10x impact)
3. **Code-level Optimization** (1.1-2x impact)
4. **Compiler Flags** (1.1-1.5x impact)

## Algorithm Optimization

### Use zkVM-Accelerated Crypto

RISC Zero provides accelerated implementations:

```rust
// BAD: Standard hashing (expensive)
use sha2::{Sha256, Digest};
let hash = Sha256::digest(data);

// GOOD: zkVM-accelerated hashing
use risc0_zkvm::sha::Impl as Sha256;
let hash = Sha256::hash_bytes(data);
```

Accelerated primitives:
- SHA-256
- Keccak
- RSA
- ECDSA (secp256k1, secp256r1)
- Ed25519

### Precomputation Strategy

Move expensive operations outside zkVM:

```rust
// Host: Precompute expensive values
let precomputed = expensive_calculation();

// Guest: Receive precomputed values
let env = ExecutorEnv::builder()
    .write(&precomputed)
    .build()?;
```

## Memory Optimization

### Minimize Heap Allocations

```rust
// BAD: Repeated allocations in loop
for item in items {
    let vec = Vec::new();
    // ...
}

// GOOD: Reuse allocations
let mut buffer = Vec::with_capacity(expected_size);
for item in items {
    buffer.clear();
    // reuse buffer
}
```

### Prefer Stack Allocation

```rust
// BAD: Heap allocation for small data
let data = Box::new([0u8; 32]);

// GOOD: Stack allocation
let data = [0u8; 32];
```

## Loop Optimization

### Use Iterators

```rust
// BAD: Bounds checking each iteration
for i in 0..vec.len() {
    process(vec[i]);
}

// GOOD: No bounds checks
for item in vec.iter() {
    process(item);
}
```

### Avoid Expensive Operations

| Operation | Cost | Alternative |
|-----------|------|-------------|
| Division | High | Bit shifts for powers of 2 |
| Modulo | High | Bitwise AND for powers of 2 |
| Floating point | Very High | Fixed-point arithmetic |
| Dynamic dispatch | Medium | Static dispatch |

## Compiler Optimization

### Cargo.toml Settings

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.release.build-override]
opt-level = 3
```

### Guest-Specific Optimization

```toml
# In guest/Cargo.toml
[profile.release]
opt-level = "z"  # Optimize for size (often faster in zkVM)
```

## Segment Management

### Understanding Segments

- Each segment ≈ 1M cycles
- More segments = more parallel proving potential
- More segments = more memory usage

### Monitoring Segments

```rust
let session = executor.run()?;
println!("Segments: {}", session.segments.len());
println!("Total cycles: {}", session.total_cycles());
```

## Optimization Checklist

- [ ] Use accelerated crypto primitives
- [ ] Precompute expensive operations outside zkVM
- [ ] Minimize heap allocations
- [ ] Use iterators over index access
- [ ] Enable LTO and optimize-for-size
- [ ] Profile before and after changes
- [ ] Consider segment count vs memory tradeoff
