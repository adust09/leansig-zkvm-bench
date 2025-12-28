//! Core types for signature verification.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::{F, MESSAGE_LENGTH};

/// Public key containing Merkle root and hash parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKey {
    /// Merkle tree root (as field elements)
    pub root: Vec<F>,
    /// Tweakable hash parameter
    pub parameter: Vec<F>,
}

/// XMSS signature containing authentication path and chain data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature {
    /// Merkle authentication path (list of sibling hashes)
    pub path: Vec<Vec<F>>,
    /// Encoding randomness (rho)
    pub rho: Vec<u8>,
    /// Hash chain endpoint values
    pub hashes: Vec<Vec<F>>,
}

/// Complete input for verification (to be deserialized from Zisk input).
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
