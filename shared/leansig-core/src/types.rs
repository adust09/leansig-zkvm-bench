//! Core types for signature verification.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::{F, MESSAGE_LENGTH, HASH_LEN, PARAMETER_LEN, RANDOMNESS_LEN};

/// Public key containing Merkle root and hash parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKey {
    /// Merkle tree root (7 field elements)
    pub root: [F; HASH_LEN],
    /// Tweakable hash parameter (5 field elements)
    pub parameter: [F; PARAMETER_LEN],
}

/// XMSS signature containing authentication path and chain data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature {
    /// Merkle authentication path (TREE_HEIGHT sibling hashes, each HASH_LEN elements)
    pub path: Vec<[F; HASH_LEN]>,
    /// Encoding randomness (rho) - 6 field elements
    pub rho: [F; RANDOMNESS_LEN],
    /// Hash chain starting points (NUM_CHAINS hashes, each HASH_LEN elements)
    pub hashes: Vec<[F; HASH_LEN]>,
    /// Leaf index (epoch)
    pub leaf_index: u32,
}

/// Complete input for verification (to be deserialized from zkVM input).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyInput {
    pub public_key: PublicKey,
    pub epoch: u32,
    pub message: [u8; MESSAGE_LENGTH],
    pub signature: Signature,
}

impl VerifyInput {
    /// Deserialize from bytes (postcard format).
    ///
    /// Postcard is a no_std-friendly compact binary format.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        postcard::from_bytes(bytes).map_err(|_| ())
    }

    /// Serialize to bytes (postcard format).
    pub fn to_bytes(&self) -> Result<Vec<u8>, ()> {
        postcard::to_allocvec(self).map_err(|_| ())
    }
}
