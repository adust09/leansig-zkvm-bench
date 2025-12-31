//! XMSS signature verification using TargetSum W=1 encoding.
//!
//! This module provides the core verification functions for XMSS signatures,
//! including Merkle tree authentication path verification.

use alloc::vec::Vec;

use crate::encoding::compute_codeword;
use crate::tweak_hash::{apply, chain, Tweak};
use crate::types::{
    Hash, Parameter, PublicKey, Signature,
    NUM_CHAINS, TREE_HEIGHT, BASE,
};

/// Verify an XMSS signature.
///
/// This function:
/// 1. Validates signature structure
/// 2. Encodes the message to get chain positions (codeword)
/// 3. Walks the chains from signature hash values to chain ends
/// 4. Verifies the Merkle authentication path
///
/// # Arguments
/// * `public_key` - The public key containing root and parameter
/// * `epoch` - The epoch (time period) of the signature
/// * `message` - The message that was signed (32 bytes)
/// * `signature` - The signature to verify
///
/// # Returns
/// `true` if the signature is valid
pub fn verify_signature(
    public_key: &PublicKey,
    epoch: u32,
    message: &[u8; 32],
    signature: &Signature,
) -> bool {
    // Validate signature structure
    if signature.hashes.len() != NUM_CHAINS {
        return false;
    }
    if signature.path.len() != TREE_HEIGHT {
        return false;
    }
    if signature.leaf_index != epoch {
        return false;
    }

    // Step 1: Compute the codeword (chain positions) from the message
    let codeword = compute_codeword(
        &public_key.parameter,
        epoch,
        &signature.rho,
        message,
    );

    if codeword.len() != NUM_CHAINS {
        return false;
    }

    // Step 2: Reconstruct chain endpoints by walking from signature hashes
    let chain_ends = reconstruct_chain_endpoints(
        &public_key.parameter,
        epoch,
        &codeword,
        &signature.hashes,
    );

    if chain_ends.is_empty() {
        return false;
    }

    // Step 3: Verify the Merkle authentication path
    verify_merkle_path(
        &public_key.parameter,
        &public_key.root,
        epoch,
        &chain_ends,
        &signature.path,
    )
}

/// Reconstruct chain endpoints from signature hashes and codeword values.
///
/// For TargetSum W=1 (BASE=2), each codeword value is 0 or 1:
/// - If codeword[i] = 0: walk 1 step from sig.hashes[i] to reach endpoint
/// - If codeword[i] = 1: sig.hashes[i] is already at the endpoint (walk 0 steps)
fn reconstruct_chain_endpoints(
    parameter: &Parameter,
    epoch: u32,
    codeword: &[u8],
    sig_hashes: &[Hash],
) -> Vec<Hash> {
    let mut endpoints = Vec::with_capacity(NUM_CHAINS);

    for (chain_index, (&steps_seen, start_hash)) in codeword
        .iter()
        .zip(sig_hashes.iter())
        .enumerate()
    {
        let start_pos = steps_seen;

        // Validate chain position is within bounds
        if steps_seen as usize >= BASE {
            return Vec::new();
        }

        // Calculate remaining steps to reach endpoint
        // For BASE=2: if steps_seen=0, walk 1 step; if steps_seen=1, walk 0 steps
        let remaining = (BASE - 1) as u8 - start_pos;

        let endpoint = chain(
            parameter,
            epoch,
            chain_index as u8,
            start_pos,
            remaining as usize,
            start_hash,
        );

        endpoints.push(endpoint);
    }

    endpoints
}

/// Verify Merkle authentication path.
fn verify_merkle_path(
    parameter: &Parameter,
    root: &Hash,
    position: u32,
    leaf_hashes: &[Hash],
    path: &[Hash],
) -> bool {
    if path.len() != TREE_HEIGHT {
        return false;
    }

    // Hash all chain ends to get leaf value
    let tweak = Tweak::tree_tweak(0, position);
    let mut current = apply(parameter, &tweak, leaf_hashes);

    let mut idx = position;
    for (level, sibling) in path.iter().enumerate() {
        let children = if idx & 1 == 0 {
            [current, *sibling]
        } else {
            [*sibling, current]
        };
        idx >>= 1;

        let tweak = Tweak::tree_tweak(level as u8 + 1, idx);
        current = apply(parameter, &tweak, &children);
    }

    current == *root
}

/// Convenience function to verify from serialized input.
pub fn verify_from_input(input: &crate::types::VerificationInput) -> bool {
    verify_signature(
        &input.public_key,
        input.epoch,
        &input.message,
        &input.signature,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_path_depth_validation() {
        let param = Parameter::default();
        let root = Hash::default();
        let leaf = vec![Hash::default()];

        // Empty path should fail (we expect TREE_HEIGHT levels)
        let result = verify_merkle_path(&param, &root, 0, &leaf, &[]);
        assert!(!result);
    }
}
