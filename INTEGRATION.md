# leanSig Integration Architecture Across zkVMs

This document describes how each zkVM integrates the leanSig XMSS signature verification library, along with the theoretical background and technical challenges encountered.

## 1. Background

### 1.1 What is leanSig?

leanSig is a post-quantum signature library implementing XMSS (eXtended Merkle Signature Scheme) optimized for zkVM environments. Key characteristics:

| Property | Value |
|----------|-------|
| Hash Function | Poseidon2 (KoalaBear field) |
| Field Prime | p = 0x7F000001 = 2^31 - 2^24 + 1 |
| Tree Height | 18 (2^18 = 262,144 signatures per key) |
| Encoding | TargetSum (W=1, 155 chains) |
| Chain Length | 2 steps per chain |

### 1.2 XMSS Verification Components

XMSS verification consists of three main phases:

```
┌──────────────────────────────────────────────────────────────┐
│  1. TargetSum Decoding                                        │
│     message → codeword (155 step counts)                      │
├──────────────────────────────────────────────────────────────┤
│  2. WOTS Chain Verification (155 chains)                      │
│     for each chain i:                                         │
│       walk from signature[i] by step_count[i] → leaf[i]       │
│       each step = Poseidon2 permutation                       │
├──────────────────────────────────────────────────────────────┤
│  3. Merkle Tree Verification (18 levels)                      │
│     hash leaves → root, verify against public key             │
│     each level = Poseidon2 permutation                        │
└──────────────────────────────────────────────────────────────┘
```

### 1.3 The no_std Challenge

All zkVMs execute in constrained environments:

| Constraint | Impact |
|------------|--------|
| `#![no_std]` | No standard library (no files, threads, networking) |
| No dynamic linking | All code must be statically compiled |
| Limited memory | Typically no virtual memory, fixed heap |
| Deterministic execution | Required for proof reproducibility |

The leanSig library uses `std` features (e.g., `rayon`, `dashmap`) for key generation and signing. Verification-only code must be extracted or re-implemented.

---

## 2. Integration Approaches

### 2.1 Miden VM: Pure Miden Assembly

**Approach**: Complete re-implementation in Miden Assembly (MASM)

```
┌─────────────────────────────────────────────────────────────┐
│                      Miden VM                                │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  MASM Implementation (from scratch)                    │  │
│  │  ├── kb_add, kb_sub, kb_mul, kb_pow3 (field ops)      │  │
│  │  ├── Poseidon2 permutation (28 rounds)                │  │
│  │  ├── Chain walk verification                           │  │
│  │  └── Merkle tree verification                          │  │
│  └───────────────────────────────────────────────────────┘  │
│                           ↓                                  │
│  Input: Signature data loaded to memory addresses            │
│  Output: Boolean (valid/invalid)                             │
└─────────────────────────────────────────────────────────────┘
```

**Why this approach?**

- Miden VM uses Goldilocks field (p = 2^64 - 2^32 + 1), not KoalaBear
- No Rust compilation to MASM exists
- Miden has no Poseidon2-KoalaBear precompile
- Direct MASM gives maximum control over cycle count

**Technical Challenges**:

| Challenge | Solution |
|-----------|----------|
| Field mismatch | Implement KoalaBear arithmetic in Goldilocks-native VM |
| 148 round constants | Load to memory at startup (~300 mem_store operations) |
| Stack limitations | Use memory for state, return to stack for computation |
| No loops with dynamic bounds | Unroll or use repeat blocks |

### 2.2 OpenVM: Host/Guest Separation

**Approach**: Host uses leanSig directly; Guest re-implements verification

```
┌─────────────────────────────────────────────────────────────┐
│                         HOST (std)                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  xmss-lib: Links leanSig crate directly               │  │
│  │  ├── key_gen() → (PublicKey, SecretKey)               │  │
│  │  ├── sign() → Signature                               │  │
│  │  └── export to VerificationBatch (JSON)               │  │
│  └───────────────────────────────────────────────────────┘  │
│                           │                                  │
│                           ▼ JSON serialization               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                     GUEST (no_std)                     │  │
│  │  ├── Poseidon2 re-implementation (p3-koala-bear)      │  │
│  │  ├── TargetSum decoding                               │  │
│  │  ├── Chain verification                               │  │
│  │  ├── Merkle verification                              │  │
│  │  └── SHA-256 commitment (OpenVM accelerated)          │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Why this approach?**

- OpenVM supports RISC-V guest programs in Rust
- Host can use full `std` environment with leanSig
- Guest receives pre-computed data, only verifies
- SHA-256 is accelerated (useful for commitment hashing)

**Technical Challenges**:

| Challenge | Solution |
|-----------|----------|
| leanSig not no_std | Host-only linking; guest re-implements crypto |
| Type marshalling | Custom `xmss-types` crate with serde |
| Poseidon2 not accelerated | Software implementation via Plonky3 |
| Message format | Must be 32-byte SHA-256 digest |

### 2.3 RISC Zero: Core Library Extraction

**Approach**: Extract verification logic into no_std `core` crate

```
┌─────────────────────────────────────────────────────────────┐
│                      RISC Zero                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  core/ (no_std, shared)                               │  │
│  │  ├── field.rs: KoalaBear via p3-koala-bear            │  │
│  │  ├── poseidon.rs: Poseidon2 via p3-poseidon2          │  │
│  │  ├── types.rs: PublicKey, Signature, etc.             │  │
│  │  └── verify.rs: verify_signature()                    │  │
│  └───────────────────────────────────────────────────────┘  │
│                           │                                  │
│           ┌───────────────┴───────────────┐                  │
│           ▼                               ▼                  │
│  ┌─────────────────┐             ┌─────────────────┐        │
│  │  Host (std)     │             │  Guest (no_std) │        │
│  │  - Test data    │             │  - Verify only  │        │
│  │  - Benchmarking │             │  - RISC-V exec  │        │
│  └─────────────────┘             └─────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

**Why this approach?**

- RISC Zero supports Rust with `#![no_std]`
- Plonky3 crates are already no_std compatible
- Shared `core` crate ensures consistency between host tests and guest execution

**Technical Challenges**:

| Challenge | Solution |
|-----------|----------|
| No Poseidon2 precompile | Software implementation (~11M cycles) |
| 32-bit RISC-V for 31-bit field | Modular reduction on every operation |
| CPU proving slow | GPU/Bonsai recommended for production |
| No OnceLock in no_std | Direct initialization |

### 2.4 Zisk: Minimal Library Extraction

**Approach**: Create `leansig-minimal` crate with verification-only code

```
┌─────────────────────────────────────────────────────────────┐
│                         Zisk                                 │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  leansig-minimal/ (no_std)                            │  │
│  │  ├── types.rs: PublicKey, Signature, VerifyInput      │  │
│  │  ├── poseidon.rs: Poseidon2 (p3-poseidon2)            │  │
│  │  ├── encoding.rs: TargetSum decode                    │  │
│  │  └── verify.rs: verify_signature()                    │  │
│  └───────────────────────────────────────────────────────┘  │
│                           │                                  │
│                           ▼                                  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  verifier/ (RISC-V guest)                             │  │
│  │  └── Calls leansig-minimal::verify_signature()        │  │
│  └───────────────────────────────────────────────────────┘  │
│                           │                                  │
│                           ▼                                  │
│  Input: Binary serialized (input.bin)                        │
│  Output: u32 (1 = valid, 0 = invalid)                        │
└─────────────────────────────────────────────────────────────┘
```

**Why this approach?**

- Zisk uses RISC-V (similar to RISC Zero)
- Minimal dependency footprint reduces cycle count
- Binary input format is more efficient than JSON

**Technical Challenges**:

| Challenge | Solution |
|-----------|----------|
| leanSig std dependencies | Manual extraction to leansig-minimal |
| No precompiles | All crypto in software |
| macOS proving slow | Recommend Linux with AVX2/AVX-512 |

---

## 3. Comparison

### 3.1 Integration Complexity

| zkVM | Integration Effort | Code Reuse |
|------|-------------------|------------|
| Miden | **High** - Full MASM rewrite | None (new implementation) |
| OpenVM | **Medium** - Host uses leanSig, guest re-implements | Partial (host reuses leanSig) |
| RISC Zero | **Medium** - Extract to no_std crate | High (Plonky3 crates) |
| Zisk | **Medium** - Create minimal library | High (Plonky3 crates) |

### 3.2 Performance Characteristics

| zkVM | VM Cycles | Dominant Cost |
|------|-----------|---------------|
| Miden | ~15.5M | Round constant loading + Poseidon2 |
| OpenVM | WIP | Poseidon2 (software) |
| RISC Zero | ~11M | Poseidon2 (software, no precompile) |
| Zisk | ~158K | Poseidon2 (more efficient execution) |

### 3.3 Architectural Trade-offs

| Aspect | Miden | OpenVM | RISC Zero | Zisk |
|--------|-------|--------|-----------|------|
| **Language** | MASM | Rust | Rust | Rust |
| **Field** | Goldilocks | N/A (RISC-V) | N/A (RISC-V) | N/A (RISC-V) |
| **Poseidon2** | Manual | Software | Software | Software |
| **SHA-256** | N/A | Accelerated | Accelerated | N/A |
| **Proof System** | STARK (native) | STARK | STARK + Groth16 | STARK |

---

## 4. Theoretical Challenges

### 4.1 Field Arithmetic in RISC-V

KoalaBear field (p = 2^31 - 2^24 + 1) presents challenges on 32-bit RISC-V:

```
Multiplication overflow:
  a * b where a, b < 2^31
  → result up to 2^62, requires 64-bit intermediate
  → RISC-V rv32: use mulhu/mulhsu for high bits
  → 3-4 instructions per field multiplication
```

**Impact**: Each Poseidon2 round requires ~64 field multiplications, explaining the high cycle count.

### 4.2 Poseidon2 State Size

Poseidon2-16 uses 16 field elements (512 bits of state):

```
State layout:
  [s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15]

Round structure (28 total):
  4 external rounds (full S-box) + 20 internal rounds (partial) + 4 external

Each external round:
  - 16 S-box operations (x³)
  - 16×16 matrix multiplication
  - 16 constant additions
```

**Impact**: Each permutation requires hundreds of field operations, dominating execution time.

### 4.3 Memory vs. Computation Trade-off

| Approach | Memory | Cycles | Use Case |
|----------|--------|--------|----------|
| Precompute constants | High (148 × 4 bytes) | Lower | Miden (memory is cheap) |
| Compute constants | Low | Higher | Constrained environments |
| Hardcode constants | Code size increase | Lowest | When code space is available |

### 4.4 Cycle Count Scaling

For full XMSS verification:

```
Components:
  1. TargetSum decoding: O(message_bits) = negligible
  2. Chain verification: 155 chains × 2 steps × Poseidon2 = 310 permutations
  3. Merkle verification: 18 levels × Poseidon2 = 18 permutations

Total: ~328 Poseidon2 permutations per signature
```

At ~30K-50K cycles per Poseidon2 (software), this explains:
- Zisk: 158K cycles (optimized execution)
- RISC Zero: 11M cycles (general-purpose overhead)
- Miden: 15.5M cycles (field mismatch overhead)

---

## 5. Technical Barriers and Solutions

### 5.1 Barrier: `no_std` Compatibility

**Problem**: leanSig depends on `std` features (`rayon`, `dashmap`, `OnceLock`, etc.)

| Dependency | Usage | Solution |
|------------|-------|----------|
| `rayon` | Parallel signature generation | Not used in verification → remove |
| `dashmap` | Concurrent map | Not used in verification → remove |
| `OnceLock` | Lazy initialization | Replace with direct initialization |
| `std::vec` | Dynamic arrays | Replace with `alloc::vec` |

```rust
// Before (std)
use std::sync::OnceLock;
static CONSTANTS: OnceLock<Vec<F>> = OnceLock::new();

// After (no_std)
#![no_std]
extern crate alloc;
use alloc::vec::Vec;
// Direct initialization or const fn
```

### 5.2 Barrier: Field Arithmetic Overhead

**Problem**: KoalaBear (p = 2^31 - 2^24 + 1) on 32-bit RISC-V

```
Multiplication issue:
  a × b (each 31-bit) → up to 62-bit result
  → Requires 64-bit intermediate
  → RISC-V rv32 needs mulhu/mulhsu instructions
  → 1 field multiplication = 3-4 instructions
```

| Approach | Implementation | Trade-off |
|----------|----------------|-----------|
| Montgomery multiplication | Plonky3 implementation | Conversion cost |
| Barrett reduction | General-purpose | Extra division |
| Dedicated instructions | Miden Assembly | No portability |

### 5.3 Barrier: No Poseidon2 Precompile

**Problem**: Most zkVMs lack Poseidon2-KoalaBear hardware acceleration

**Impact**:
```
Per-signature computation:
  155 chains × 2 steps = 310 Poseidon2 permutations
  18 Merkle levels = 18 Poseidon2 permutations
  Total: ~328 Poseidon2 permutations

Software implementation: 30K-50K cycles/permutation
→ ~10-16M cycles/signature
```

| zkVM | Status |
|------|--------|
| Miden | Native Rescue-Prime (RPO) available, but breaks leanSig compatibility |
| RISC Zero | SHA-256 = 68 cycles; Poseidon2 request pending |
| OpenVM | SHA-256 accelerated (for commitment hashing) |
| Zisk | None (but efficient execution engine) |

### 5.4 Barrier: Host/Guest Data Serialization

**Problem**: Passing leanSig internal types to zkVM guest

**Solution**:
```
┌─────────────────────────────────────────────────────┐
│  HOST                                               │
│  leanSig::PublicKey → export() → SerializedPK      │
│  leanSig::Signature → export() → SerializedSig     │
└─────────────────────────┬───────────────────────────┘
                          │ JSON / bincode / raw bytes
                          ▼
┌─────────────────────────────────────────────────────┐
│  GUEST                                              │
│  deserialize() → VerificationInput                 │
│  verify_signature(input) → bool                    │
└─────────────────────────────────────────────────────┘
```

| Format | Size | Parse Speed | Used By |
|--------|------|-------------|---------|
| JSON | Large | Slow | OpenVM |
| bincode | Medium | Fast | (unused) |
| Raw bytes | Minimal | Fastest | Zisk |

### 5.5 Barrier: Round Constant Memory Management

**Problem**: Poseidon2 requires 148 round constants

**Miden challenge**:
```masm
# Constant loading: ~300 mem_store operations
push.0x7F000001
mem_store.0x100
push.0x12345678
mem_store.0x101
# ... repeat 148 times
```

| Method | Memory | Cycles | Use Case |
|--------|--------|--------|----------|
| Load at startup | 592 bytes | High (initialization) | Miden |
| Hardcode in code | 0 (in code) | Low | Under consideration |
| Generate on demand | 0 | Medium (compute each time) | Small scale |

### 5.6 Barrier: Proof Generation Performance

**Problem**: CPU proving takes impractical time

| zkVM | Cycles | CPU Proving Time |
|------|--------|------------------|
| Zisk | 158K | ~26 min (macOS) |
| RISC Zero | 11M | >10 min (timeout) |
| Miden | 15.5M | OOM (killed after 11 min) |

**Solutions**:

```
1. GPU Acceleration
   RISC0_PROVER=cuda  # RISC Zero
   OPENVM_GUEST_FEATURES=cuda  # OpenVM

2. Cloud Proving Services
   Bonsai (RISC Zero)
   Aggregation server (Zisk -f flag)

3. Run on Linux
   macOS: No AVX2/AVX-512 → slow
   Linux: 5-10x speedup expected

4. Parallelization
   Prove multiple signatures in parallel
   Recursive proof composition
```

### 5.7 Solution Summary

| Barrier | Recommended Solution |
|---------|---------------------|
| no_std compatibility | Use Plonky3 crates (already no_std compatible) |
| Field arithmetic | Montgomery multiplication + Plonky3 implementation |
| No Poseidon2 precompile | Accept software implementation; wait for future precompiles |
| Serialization | Binary format + shared types crate |
| Memory management | Load constants at startup |
| Proving speed | GPU + Linux + cloud services |

---

## 6. Recommendations

### 6.1 For New Integrations

1. **Start with Plonky3**: Use `p3-koala-bear` and `p3-poseidon2` as reference implementations
2. **Separate concerns**: Host handles leanSig linking; guest does verification only
3. **Binary serialization**: Prefer compact formats over JSON for guest input
4. **Profile early**: Identify Poseidon2 as the bottleneck from the start

### 6.2 For Performance Optimization

1. **Precompile request**: Advocate for Poseidon2-KoalaBear precompiles in zkVMs
2. **Batch verification**: Amortize setup costs across multiple signatures
3. **Hardware acceleration**: Use GPU proving where available
4. **Platform selection**: Linux with AVX2/AVX-512 for STARK proving

### 6.3 For Production Use

Consider leanMultisig (the specialized zkVM built for leanSig) if:
- Proving time is critical
- You don't need general-purpose computation
- Poseidon2 dominates your workload

Use general-purpose zkVMs (RISC Zero, Zisk, OpenVM) if:
- Integration with existing infrastructure is needed
- General computation is required alongside verification
- Proof composition with other programs is valuable

---

## References

- [leanSig Repository](https://github.com/geometryresearch/leanSig)
- [Poseidon2 Paper](https://eprint.iacr.org/2023/323)
- [XMSS RFC 8391](https://datatracker.ietf.org/doc/html/rfc8391)
- [Plonky3 Repository](https://github.com/Plonky3/Plonky3)
- [Miden VM Documentation](https://0xpolygonmiden.github.io/miden-vm/)
- [OpenVM Documentation](https://docs.openvm.dev)
- [RISC Zero Documentation](https://dev.risczero.com)
- [Zisk Documentation](https://docs.zisk.io)

---

*Last updated: 2025-12-29*
