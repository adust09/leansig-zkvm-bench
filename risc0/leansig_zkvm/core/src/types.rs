//! Core types for XMSS signature verification
//!
//! These types are designed to be serialized/deserialized for RISC Zero guest input.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::field::{FieldArray, MESSAGE_LENGTH};

/// Hash length in field elements (7 elements = 28 bytes equivalent security)
pub const HASH_LEN: usize = 7;

/// Parameter length in field elements
pub const PARAMETER_LEN: usize = 5;

/// Randomness length in field elements (rho for encoding)
pub const RANDOMNESS_LEN: usize = 6;

/// Number of chains for TargetSum W=1 encoding
pub const NUM_CHAINS: usize = 155;

/// Tree height (2^18 = 262,144 signatures per key)
pub const TREE_HEIGHT: usize = 18;

/// Base for TargetSum W=1 (binary: 0 or 1)
pub const BASE: usize = 2;

/// Message hash length in field elements
pub const MSG_HASH_LEN: usize = 5;

/// Message length in field elements (for sponge input)
pub const MSG_LEN_FE: usize = 9;

/// Hash type alias
pub type Hash = FieldArray<HASH_LEN>;

/// Parameter type alias
pub type Parameter = FieldArray<PARAMETER_LEN>;

/// Public key for XMSS signature scheme
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKey {
    /// Merkle tree root
    pub root: Hash,
    /// Hash function parameter
    pub parameter: Parameter,
}

/// Encoding randomness (matches TargetSum W=1 encoding)
pub type EncodingRandomness = FieldArray<RANDOMNESS_LEN>;

/// XMSS Signature
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature {
    /// Merkle authentication path (TREE_HEIGHT siblings)
    pub path: Vec<Hash>,
    /// Encoding randomness
    pub rho: EncodingRandomness,
    /// Chain hash values (one per chain, NUM_CHAINS total)
    pub hashes: Vec<Hash>,
    /// Leaf index (equals epoch)
    pub leaf_index: u32,
}

/// Input structure for verification in RISC Zero guest
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationInput {
    /// Public key
    pub public_key: PublicKey,
    /// Epoch (time period)
    pub epoch: u32,
    /// Message to verify (32 bytes)
    pub message: [u8; MESSAGE_LENGTH],
    /// Signature to verify
    pub signature: Signature,
}

/// Output from verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationOutput {
    /// Whether the signature is valid
    pub is_valid: bool,
    /// Epoch that was verified
    pub epoch: u32,
    /// Hash of the message that was verified
    pub message_hash: [u8; 32],
}
