# leanSig Verification Path Dependency Analysis

## Executive Summary

**Key Finding**: The `verify()` function does NOT use `rayon`! This makes porting to no_std significantly easier than initially expected.

The verification path only uses:
- `chain()` - Pure sequential for-loop (no parallelism)
- `hash_tree_verify()` - Pure sequential for-loop (no parallelism)
- `IE::encode()` - Simple hash + sum check

---

## Verification Call Path

```
verify() [generalized_xmss.rs:856-909]
├── IE::encode() [target_sum.rs:54-72]
│   └── MH::apply() - Message hash application
├── chain::<TH>() [tweak_hash.rs:120-139]
│   └── Pure for-loop, NO rayon
└── hash_tree_verify() [tweak_hash_tree.rs:597-647]
    └── Pure for-loop, NO rayon
```

---

## no_std Compatibility Analysis

### Already no_std Compatible

| Component | Status | Notes |
|-----------|--------|-------|
| `p3-field` | ✅ Compatible | Has `#![no_std]` with `extern crate alloc` |
| `p3-koala-bear` | ✅ Compatible | Uses no_std p3-field |
| `chain()` | ✅ Compatible | Pure for-loop, no std dependencies |
| `hash_tree_verify()` | ✅ Compatible | Pure for-loop, no std dependencies |
| `serde` | ✅ Compatible | With `alloc` feature, no `std` feature |
| `PhantomData` | ✅ Compatible | Change `std::marker` → `core::marker` |

### Requires Modification

| Component | Issue | Solution |
|-----------|-------|----------|
| `std::sync::OnceLock` | Used in lib.rs for caching | Use `once_cell::race::OnceBox` or remove caching |
| `ethereum_ssz` | May not support no_std | Switch to `ssz_rs` or manual serialization |
| `Vec<T>` allocations | Requires allocator | Use `extern crate alloc; use alloc::vec::Vec;` |
| `std::ops::Range` | Used in SecretKey | Change to `core::ops::Range` |

### NOT Used in Verification (Safe to Ignore)

| Component | Where Used | Impact |
|-----------|------------|--------|
| `rayon` | `sign()`, `key_gen()` only | Not needed for verify |
| `dashmap` | Not in leanSig | N/A |
| Parallel iterators | `sign()`, `key_gen()` only | Not needed for verify |

---

## Types Required for Verification

### Input Types (to be serialized from host)

```rust
// Public Key
pub struct GeneralizedXMSSPublicKey<TH: TweakableHash> {
    root: TH::Domain,        // FieldArray<N> of KoalaBear elements
    parameter: TH::Parameter, // FieldArray<M> of KoalaBear elements
}

// Signature
pub struct GeneralizedXMSSSignature<IE, TH> {
    path: HashTreeOpening<TH>,  // Merkle authentication path
    rho: IE::Randomness,        // Encoding randomness
    hashes: Vec<TH::Domain>,    // Chain hashes
}

// Merkle Path
pub struct HashTreeOpening<TH> {
    co_path: Vec<(TH::Domain, bool)>,  // Sibling nodes + direction
}
```

### Constants Needed

```rust
const MESSAGE_LENGTH: usize = 32;
const TWEAK_SEPARATOR_FOR_CHAIN_HASH: u8 = 0x00;
const TWEAK_SEPARATOR_FOR_TREE_HASH: u8 = 0x01;
```

---

## Estimated Port Complexity

| Task | Complexity | Time Estimate |
|------|------------|---------------|
| Port field types | Low | Already no_std |
| Port TweakableHash trait | Low | Pure trait, minimal changes |
| Port chain() function | Low | Already sequential |
| Port hash_tree_verify() | Low | Already sequential |
| Port Poseidon2 compress/sponge | Medium | Remove OnceLock caching |
| Port IncomparableEncoding | Low | Pure trait |
| Port MessageHash | Low | Uses Poseidon internally |
| Serialization (SSZ → manual) | Medium | Or use ssz_rs |

**Total Estimate**: 2-3 days for minimal viable port

---

## Recommended Approach

1. **Create `core/` crate** with:
   - Field type wrappers (re-export from p3-field)
   - TweakableHash trait (no_std version)
   - Poseidon2 implementation (inline, no caching)
   - verify() logic

2. **Simplifications for RISC Zero**:
   - Remove OnceLock caching (just create Poseidon2 directly)
   - Use `alloc::vec::Vec` instead of `std::vec::Vec`
   - Use manual serialization or ssz_rs

3. **Guest Program**:
   ```rust
   #![no_std]
   #![no_main]

   extern crate alloc;

   risc0_zkvm::guest::entry!(main);

   fn main() {
       // Deserialize inputs
       // Call verify()
       // Commit result
   }
   ```

---

## References

- [Plonky3 GitHub](https://github.com/Plonky3/Plonky3) - Already no_std compatible
- [ssz_rs](https://github.com/ralexstokes/ssz-rs) - no_std SSZ alternative
- [leanSig source](https://github.com/leanEthereum/leanSig)

---

## Source Code Locations

| Function | File | Lines |
|----------|------|-------|
| `verify()` | `src/signature/generalized_xmss.rs` | 856-909 |
| `chain()` | `src/symmetric/tweak_hash.rs` | 120-139 |
| `hash_tree_verify()` | `src/symmetric/tweak_hash_tree.rs` | 597-647 |
| `IE::encode()` | `src/inc_encoding/target_sum.rs` | 54-72 |
| `Poseidon2 permutation` | `src/symmetric/tweak_hash/poseidon.rs` | Full file |
| `OnceLock cache` | `src/lib.rs` | 31-48 |
