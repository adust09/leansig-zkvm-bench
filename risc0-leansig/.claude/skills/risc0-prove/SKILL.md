---
name: RISC Zero Prove
description: This skill should be used when the user asks to "generate proof", "prove risc0 app", "create zkvm receipt", "run prover", "generate zk proof", "prove execution", or mentions generating zero-knowledge proofs with RISC Zero.
version: 0.1.0
---

# RISC Zero Proof Generation

Generate and verify zero-knowledge proofs with RISC Zero zkVM.

## Purpose

Provide guidance for generating cryptographic proofs of program execution, including different proving modes, receipt types, and verification methods.

## Proof Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Host Program                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │   Input     │→ │  Executor   │→ │   Prover    │ │
│  └─────────────┘  └─────────────┘  └─────────────┘ │
│         ↓               ↓                 ↓         │
│     [bytes]         [Session]         [Receipt]    │
└─────────────────────────────────────────────────────┘
```

## Proving Modes

### Development Mode (Fast, No Real Proofs)

```bash
RISC0_DEV_MODE=1 cargo run
```

Skip proof generation for rapid development. Not for production.

### Local Proving (Real Proofs)

```bash
RISC0_DEV_MODE=0 cargo run --release
```

Generate real cryptographic proofs locally. Requires 16GB+ RAM.

### GPU-Accelerated Proving

```bash
RISC0_CUDA=1 cargo run --release
```

Use NVIDIA CUDA for faster proving.

### Remote Proving (Boundless)

```rust
use risc0_zkvm::BoundlessProver;

let prover = BoundlessProver::new(api_key)?;
let receipt = prover.prove(env, elf).await?;
```

Offload heavy computation to Boundless service.

## Basic Proof Generation

```rust
use risc0_zkvm::{default_prover, ExecutorEnv};
use methods::{GUEST_ELF, GUEST_ID};

fn generate_proof(input: &[u8]) -> anyhow::Result<Receipt> {
    // 1. Build execution environment
    let env = ExecutorEnv::builder()
        .write_slice(input)
        .build()?;

    // 2. Get the prover
    let prover = default_prover();

    // 3. Generate proof (Receipt)
    let receipt = prover.prove(env, GUEST_ELF)?;

    // 4. Verify locally
    receipt.verify(GUEST_ID)?;

    Ok(receipt)
}
```

## Receipt Types

| Type | Size | Use Case |
|------|------|----------|
| Composite | Large | Development |
| Succinct | ~200KB | Off-chain verification |
| Groth16 | ~300B | Ethereum on-chain |

### Receipt Compression

```rust
// Compress to Succinct
let succinct = receipt.compress()?;

// Convert to Groth16 for Ethereum
let groth16 = succinct.groth16()?;
```

## Verification

### Off-Chain Verification

```rust
use methods::GUEST_ID;

receipt.verify(GUEST_ID)?;
let output: OutputType = receipt.journal.decode()?;
```

### On-Chain Verification (Solidity)

```solidity
interface IRiscZeroVerifier {
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalHash
    ) external view;
}
```

## Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| Out of Memory | Large computation | Use Boundless or add RAM |
| Execution Failed | Guest program error | Debug guest logic |
| Verification Failed | ImageID mismatch | Check GUEST_ID |

## Additional Resources

### Reference Files

- **`references/receipt-types.md`** - Detailed receipt format documentation
- **`references/verification.md`** - On-chain verification guide

### Examples

- **`examples/basic-proof.rs`** - Simple proof generation
- **`examples/groth16-proof.rs`** - Ethereum-ready proof
