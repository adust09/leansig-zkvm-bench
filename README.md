# LeanSig zkVM Benchmarks

Benchmarking XMSS signature verification from [leanSig](https://github.com/geometryxyz/leanSig) across multiple zero-knowledge virtual machines.

## Benchmark Summary

![Benchmark Comparison](charts/benchmark_comparison.png)

| zkVM | Cycles | Proving Time | Status |
|------|--------|--------------|--------|
| [SP1](sp1/README.md) | 136K | 71 s | Done |
| [Zisk](zisk/README.md) | 158K | ~21 min | Done |
| [RISC Zero](risc0/README.md) | 6.3M | ~31 min | Done |
| [OpenVM](openvm/README.md) | - | ~5 min | Done |
| [Miden](miden/README.md) | 15.5M | OOM | WIP |

*Proving times measured on macOS (Apple Silicon). Linux with AVX2/AVX-512 expected 5-10x faster.*

## Configuration

| Parameter | Value |
|-----------|-------|
| Signature Scheme | XMSS (eXtended Merkle Signature Scheme) |
| Tree Height | 18 (2^18 = 262,144 signatures) |
| Hash Function | Poseidon2 (KoalaBear field) |
| Encoding | TargetSum W=1 (155 chains) |

## References

- [LeanSig Paper](https://eprint.iacr.org/2024/1205)
- [SP1 Docs](https://docs.succinct.xyz)
- [Zisk Docs](https://docs.zisk.io)
- [RISC Zero Docs](https://dev.risczero.com)
- [OpenVM Docs](https://docs.openvm.dev)
- [Miden VM Docs](https://docs.polygon.technology/miden)
