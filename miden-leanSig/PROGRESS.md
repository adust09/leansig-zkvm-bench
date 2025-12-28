# leanSig XMSS on Miden VM - Progress Memo

## Project Overview

leanSig XMSS signature verification implementation in Miden Assembly with STARK proof generation.

**Goal**: Generate STARK proofs for XMSS signature verification compatible with leanSig library.

## Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Field | KoalaBear | p = 0x7F000001 = 2,130,706,433 |
| Hash | Poseidon2-16 | 4+20+4 rounds, S-box = x³ |
| W (width) | 1 | leanSig parameter |
| V (chains) | 155 | Number of hash chains |
| CHAIN_LENGTH | 2 | 2^W steps |
| Tree Height | 18 | Merkle tree levels |

## Completed Work

### Phase 1-2: Foundation
- [x] KoalaBear field operations (`kb_add`, `kb_sub`, `kb_mul`, `kb_pow3`)
- [x] Poseidon2-16 permutation (full implementation)
- [x] Round constants extraction from Plonky3
- [x] Test vector verification

### Phase 3-4: XMSS Implementation
- [x] Chain hash function
- [x] Chain walk verification (single chain)
- [x] 155-chain verification loop
- [x] 18-level Merkle tree verification
- [x] Full signature verification integration

### Phase 5: STARK Proof Generation (In Progress)
- [x] Minimal test (push.42) - STARK proof successful
- [x] 50-chain test created
- [ ] Micro test (5 chains, 3 levels) - needs kb_sub fix
- [ ] Find maximum provable scale

## Benchmark Results

| Test | VM Cycles | Execution | STARK Proof |
|------|-----------|-----------|-------------|
| minimal (push.42) | 41 | 61ms | **31ms** |
| full 155 chains | 15,552,770 | 16s | OOM (killed after 11min) |
| 50 chains | 15,494,579 | 14s | Timeout (10min) |

## Key Findings

### 1. Chain Count Reduction Has Minimal Effect
Reducing chains from 155 to 50 barely changes VM cycles (15.5M → 15.5M).

**Root Cause**: The dominant costs are:
- Loading 148 round constants (~300 mem_store operations)
- 18-level Merkle tree verification (18 × Poseidon2 permutation)
- Poseidon2 itself (28 rounds per permutation)

### 2. Proof Generation Memory Limit
Current machine (MacBook Air M2?) cannot generate proofs for 15M+ cycles.

**Estimated limit**: ~1M cycles or less for successful proof generation.

### 3. Miden Stack Convention
- VM starts with 16 zeros on stack
- Programs must end with ≤16 elements
- Use `swap drop` to remove initial zeros

## File Structure

```
miden-leansig/
├── PROGRESS.md              # This file
├── tests/
│   ├── xmss_verify_test.masm        # Full 155-chain test (works)
│   ├── xmss_50chain_prove_test.masm # 50-chain test (proof timeout)
│   ├── xmss_micro_prove_test.masm   # 5-chain, 3-level (needs fix)
│   ├── minimal_prove_test.masm      # Minimal test (proof works)
│   └── ...
└── masm/
    └── ... (implementation files)
```

## Next Steps (Priority Order)

1. **Fix micro test**: Add missing `kb_sub` and run STARK proof
2. **Binary search for limit**: Find max provable VM cycle count
3. **Optimization ideas**:
   - Hardcode constants instead of mem_store
   - Reduce Merkle levels for testing
   - Batch operations where possible
   - Consider native RPO hash for Merkle (breaks leanSig compat)

## Technical Notes

### Miden Syntax Quirks
```masm
# Correct
proc my_proc
    ...
end

# Wrong (no dot)
proc.my_proc  # Invalid!

# Constants must be inline
push.0x7F000001  # Correct
const.KB_PRIME=0x7F000001  # Invalid!
```

### Memory Layout
```
0x0010-0x001F: Poseidon2 state (16 elements)
0x0100-0x0193: Round constants (148 elements)
0x03E8-0x047B: Signature values (155 elements @ 1000+)
0x04B0-0x0543: Public key values (155 elements @ 1200+)
0x0578-0x060B: Step counts (155 elements @ 1400+)
0x07D0-0x07E1: Merkle path (18 elements @ 2000+)
```

## References

- [leanSig Repository](https://github.com/geometryresearch/leanSig)
- [Miden VM Documentation](https://0xpolygonmiden.github.io/miden-vm/)
- [Poseidon2 Paper](https://eprint.iacr.org/2023/323)

---
Last updated: 2025-12-28
