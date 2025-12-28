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

/// Merkle tree authentication path
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerklePath {
    /// Co-path nodes with direction (true = right sibling, false = left sibling)
    pub co_path: Vec<(Hash, bool)>,
}

/// Encoding randomness (matches MH::Randomness)
pub type EncodingRandomness = FieldArray<4>;

/// XMSS Signature
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature {
    /// Merkle authentication path
    pub path: MerklePath,
    /// Encoding randomness
    pub rho: EncodingRandomness,
    /// Chain hash values (one per chain)
    pub hashes: Vec<Hash>,
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
