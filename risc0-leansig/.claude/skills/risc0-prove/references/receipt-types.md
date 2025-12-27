# RISC Zero Receipt Types

Detailed documentation on receipt formats and when to use each.

## Receipt Overview

A Receipt is the cryptographic proof that a guest program executed correctly. It contains:

1. **Seal**: The cryptographic proof itself
2. **Journal**: Public outputs committed during execution
3. **Metadata**: ImageID and other verification data

## Receipt Types Comparison

### Composite Receipt

The default receipt type from local proving.

**Characteristics:**
- Large size (varies by computation)
- Fast to generate locally
- Contains full proof data
- Can be converted to other types

**Use Cases:**
- Development and testing
- Off-chain verification
- Base for compression

### Succinct Receipt

Compressed version optimized for size.

**Characteristics:**
- Fixed ~200KB size
- Additional compression step required
- Efficient for storage and transmission

**Generation:**
```rust
let succinct_receipt = receipt.compress()?;
```

**Use Cases:**
- Off-chain verification
- Cross-chain communication
- Storage in databases

### Groth16 Receipt

Minimal proof for Ethereum verification.

**Characteristics:**
- Fixed ~300 bytes size
- Requires Succinct receipt first
- Verifiable in Solidity

**Generation:**
```rust
let succinct = receipt.compress()?;
let groth16 = succinct.groth16()?;
```

**Use Cases:**
- Ethereum smart contracts
- Other EVM chains
- Cost-sensitive on-chain verification

## Receipt Components

### Journal

Public output committed by guest program:

```rust
// Guest: Commit output
env::commit(&output);

// Host: Read journal
let output: MyType = receipt.journal.decode()?;
```

### ImageID

Cryptographic identifier of guest ELF binary:

```rust
// Generated at build time
use methods::GUEST_ID;

// Verify against expected
receipt.verify(GUEST_ID)?;
```

### Seal

Cryptographic proof data. Format depends on receipt type:
- Composite: STARK proof segments
- Succinct: Compressed STARK
- Groth16: zk-SNARK proof

## Serialization

```rust
// Serialize receipt
let bytes = bincode::serialize(&receipt)?;

// Deserialize
let receipt: Receipt = bincode::deserialize(&bytes)?;
```

## Best Practices

1. **Development**: Use Composite with dev mode
2. **Off-chain**: Convert to Succinct for transfer
3. **On-chain**: Convert to Groth16 for Ethereum
4. **Always verify locally** before storing/sending
