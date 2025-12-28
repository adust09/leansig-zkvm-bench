# no_std Blockers for leanSig Verification Port

## Critical Blockers (Must Fix)

### 1. `std::sync::OnceLock` - Poseidon2 Caching

**Location**: `leanSig/src/lib.rs:31-48`

```rust
// BLOCKER CODE:
static POSEIDON2_24: OnceLock<Poseidon2KoalaBear<24>> = OnceLock::new();
static POSEIDON2_16: OnceLock<Poseidon2KoalaBear<16>> = OnceLock::new();

pub(crate) fn poseidon2_24() -> Poseidon2KoalaBear<24> {
    POSEIDON2_24
        .get_or_init(default_koalabear_poseidon2_24)
        .clone()
}
```

**Fix Options**:
1. **Remove caching entirely** (recommended for RISC Zero):
   ```rust
   pub(crate) fn poseidon2_24() -> Poseidon2KoalaBear<24> {
       default_koalabear_poseidon2_24()
   }
   ```
2. Use `once_cell::race::OnceBox` (requires `once_cell` with `race` feature)

**Impact**: Low - Slight performance cost, but acceptable for zkVM

---

### 2. `ethereum_ssz` - Serialization

**Location**: `leanSig/src/serialization.rs:4` and throughout

```rust
// BLOCKER CODE:
use ssz::{Decode, DecodeError, Encode};
```

**Fix Options**:
1. **Use `ssz_rs`** - Has explicit no_std support
2. **Manual serialization** - Implement custom serialize/deserialize for RISC Zero inputs
3. **Use `serde` with `postcard`** - Binary format with no_std support

**Recommended**: Manual serialization for guest inputs (simpler, smaller code)

---

### 3. `std::vec::Vec` - Heap Allocation

**Location**: Multiple files throughout verification path

**Fix**:
```rust
// Add to lib.rs
#![no_std]
extern crate alloc;
use alloc::vec::Vec;
```

**Impact**: Trivial - Just import path change

---

## Minor Blockers (Easy to Fix)

### 4. `std::marker::PhantomData`

**Location**: Multiple files

**Fix**: Change `std::marker::PhantomData` → `core::marker::PhantomData`

---

### 5. `std::ops::Range`

**Location**: `generalized_xmss.rs:498-510`

**Fix**: Change `std::ops::Range` → `core::ops::Range`

---

### 6. `std::array::from_fn`

**Location**: Test code only, not in verification path

**Impact**: None for production code

---

## NOT Blockers (Verification Path Doesn't Use)

| Component | Reason |
|-----------|--------|
| `rayon` | Only in `sign()` and `key_gen()` |
| `dashmap` | Not used in leanSig |
| `std::collections::HashMap` | Not in verify path |
| `std::io` | Not in verify path |
| `std::fs` | Not in verify path |
| `std::thread` | Not in verify path |

---

## Verification Path Code Changes Summary

### Files to Modify

| File | Changes Required |
|------|------------------|
| `lib.rs` | Add `#![no_std]`, `extern crate alloc`, remove OnceLock |
| `serialization.rs` | Replace ethereum_ssz or remove |
| `signature/generalized_xmss.rs` | `std::marker` → `core::marker` |
| `symmetric/tweak_hash.rs` | `std::vec` → `alloc::vec` |
| `symmetric/tweak_hash_tree.rs` | `std::vec` → `alloc::vec` |
| `symmetric/tweak_hash/poseidon.rs` | Remove OnceLock dependency |
| `inc_encoding/target_sum.rs` | `std::marker` → `core::marker` |

---

## Minimum Changes for RISC Zero Guest

For a minimal port, only these changes are needed:

1. **Replace OnceLock with direct creation**:
   ```rust
   fn poseidon2_24() -> Poseidon2KoalaBear<24> {
       default_koalabear_poseidon2_24()
   }
   ```

2. **Add no_std preamble**:
   ```rust
   #![no_std]
   extern crate alloc;
   use alloc::vec::Vec;
   use core::marker::PhantomData;
   ```

3. **Custom input serialization** (bypass SSZ entirely):
   - Host serializes inputs using RISC Zero's `env::write()`
   - Guest deserializes using `env::read()`

---

## Estimated Fix Time

| Blocker | Time |
|---------|------|
| OnceLock removal | 30 min |
| no_std preamble | 15 min |
| Custom serialization | 2-3 hours |
| Testing | 1-2 hours |
| **Total** | **4-6 hours** |
