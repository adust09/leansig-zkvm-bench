# Poseidon2 Implementation Requirements for RISC Zero

## Overview

leanSig uses Poseidon2 hash function over the KoalaBear field (prime 2^31 - 2^24 + 1) via Plonky3.

**Good News**: Plonky3's `p3-field` and `p3-koala-bear` are already `#![no_std]` compatible!

---

## Core Functions Used in Verification

### 1. `poseidon_compress<WIDTH, OUT_LEN>()`

**Location**: `leanSig/src/symmetric/tweak_hash/poseidon.rs:102-134`

**Purpose**: Computes `Truncate(Permute(input) + input)` (feed-forward compression)

**Algorithm**:
```
1. Pad input to WIDTH with zeros
2. state = input
3. Permute(state)
4. state += input (element-wise)
5. return state[0..OUT_LEN]
```

**Used with**:
- WIDTH=16 for single-message hashing (chains)
- WIDTH=24 for two-message hashing (tree nodes)

---

### 2. `poseidon_sponge<WIDTH, OUT_LEN>()`

**Location**: `leanSig/src/symmetric/tweak_hash/poseidon.rs:187-232`

**Purpose**: Absorb variable-length input, squeeze output

**Algorithm**:
```
1. rate = WIDTH - capacity_value.len()
2. Pad input to multiple of rate
3. Initialize state = [0...0 | capacity_value]
4. For each chunk:
   - state[0..rate] += chunk
   - Permute(state)
5. Squeeze output by permuting and extracting rate elements
```

---

### 3. `TweakableHash::apply()`

**Location**: `leanSig/src/symmetric/tweak_hash/poseidon.rs:284-357`

**Purpose**: Domain-separated hashing with tweak

**Three modes**:

1. **Single message** (chain hashing):
   ```rust
   input = [parameter | tweak | message]
   poseidon_compress<16, HASH_LEN>(input)
   ```

2. **Two messages** (tree node):
   ```rust
   input = [parameter | tweak | left | right]
   poseidon_compress<24, HASH_LEN>(input)
   ```

3. **Many messages** (chain ends → leaf):
   ```rust
   input = [parameter | tweak | message[0] | ... | message[n]]
   capacity = poseidon_safe_domain_separator(lengths)
   poseidon_sponge<24, HASH_LEN>(capacity, input)
   ```

---

## Dependencies from Plonky3

### Required Crates (all no_std compatible)

```toml
[dependencies]
p3-field = "0.2"         # Field arithmetic
p3-koala-bear = "0.2"    # KoalaBear field implementation
p3-symmetric = "0.2"     # CryptographicPermutation trait
```

### Key Types

```rust
use p3_koala_bear::KoalaBear;                    // The field F
use p3_koala_bear::Poseidon2KoalaBear;           // The permutation
use p3_symmetric::CryptographicPermutation;       // Trait for permutation

type F = KoalaBear;  // Prime field 2^31 - 2^24 + 1
```

### Poseidon2 Permutation Instantiation

```rust
use p3_koala_bear::{
    default_koalabear_poseidon2_16,
    default_koalabear_poseidon2_24,
};

// These functions create the permutation objects with correct round constants
fn get_poseidon2_16() -> Poseidon2KoalaBear<16> {
    default_koalabear_poseidon2_16()
}

fn get_poseidon2_24() -> Poseidon2KoalaBear<24> {
    default_koalabear_poseidon2_24()
}
```

---

## no_std Changes Required

### 1. Remove OnceLock Caching

**Current** (leanSig/src/lib.rs):
```rust
use std::sync::OnceLock;
static POSEIDON2_24: OnceLock<Poseidon2KoalaBear<24>> = OnceLock::new();

pub(crate) fn poseidon2_24() -> Poseidon2KoalaBear<24> {
    POSEIDON2_24
        .get_or_init(default_koalabear_poseidon2_24)
        .clone()
}
```

**no_std version**:
```rust
// No caching needed - just create directly
pub fn poseidon2_24() -> Poseidon2KoalaBear<24> {
    default_koalabear_poseidon2_24()
}
```

### 2. Use `core::array::from_fn` instead of `std::array::from_fn`

Already available in `core` - just change import.

### 3. Use `alloc::vec::Vec`

```rust
extern crate alloc;
use alloc::vec::Vec;
```

---

## RISC Zero Considerations

### No SHA-256 Precompile Benefit

RISC Zero has accelerated SHA-256 (68 cycles/64-byte block), but Poseidon2 is a different hash function.
The Poseidon2 operations will run at normal RISC-V speed, which means:
- Each field multiplication is ~10-20 cycles
- Each Poseidon2 permutation (8 full rounds + 13 partial rounds) ≈ 1000-2000 cycles
- This is significantly slower than SHA-256 precompile

### Performance Estimate

For one signature verification:
- ~256 chain hash operations (assuming 256 chains)
- ~log2(lifetime) tree hash operations (e.g., 10-20 for lifetime 2^10-2^20)
- Each hash involves 1-2 Poseidon2 permutations

**Rough estimate**: 300-500 Poseidon2 permutations per verification
**Cycle count**: 300,000 - 1,000,000 cycles (need to benchmark)

---

## Minimal Port Strategy

1. **Copy these files** (with modifications):
   - `poseidon_compress()` function
   - `poseidon_sponge()` function
   - `PoseidonTweak` enum
   - `TweakableHash` implementation for `PoseidonTweakHash`

2. **Remove**:
   - All SIMD optimizations (`compute_tree_layer` override)
   - `rayon` parallel iterators
   - `OnceLock` caching

3. **Keep using Plonky3** for:
   - Field arithmetic
   - Poseidon2 permutation (includes round constants)

---

## Type Aliases for RISC Zero Guest

```rust
#![no_std]
extern crate alloc;

use p3_koala_bear::KoalaBear;

// Field type
pub type F = KoalaBear;

// Hash output (7 field elements = 28 bytes)
pub type Hash = [F; 7];

// Parameter (5 field elements = 20 bytes)
pub type Parameter = [F; 5];

// Tweak (2 field elements)
pub type Tweak = [F; 2];
```

---

## File Summary

| Original File | Size | Port Complexity |
|--------------|------|-----------------|
| `poseidon.rs` | 1670 lines | Medium - remove SIMD/rayon |
| `tweak_hash.rs` | 272 lines | Low - mostly traits |
| `tweak_hash_tree.rs` | ~800 lines | Low - just `hash_tree_verify` |
| Plonky3 deps | External | None - already no_std |

**Total estimated lines for no_std port**: ~500 lines
