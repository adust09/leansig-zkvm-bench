# Miden VM - LeanSig Benchmark

## Overview

XMSS signature verification in Miden VM. Full re-implementation in Miden Assembly (MASM) including Poseidon2-KoalaBear and 18-level Merkle tree verification.

## Quick Start

```bash
# Run tests
miden-vm run tests/poseidon2_full_test.masm

# Generate proof (small tests only)
miden-vm prove tests/minimal_prove_test.masm
```

## Benchmark Results

| Metric | Value |
|--------|-------|
| VM Cycles | 15,552,770 (~15.5M) |
| Execution Time | 16 s |
| Proving Time | OOM (killed after 11+ min) |
| Status | Implementation complete, proof blocked |

## Notes

- Full implementation complete but STARK proof generation exceeds memory limits
- Miden's native field is Goldilocks, not KoalaBear (field mismatch overhead)
- Small tests (41 cycles) successfully generate proofs in 31ms
- Proof generation requires more powerful hardware or algorithm optimization
