//! XMSS signature verification logic
//!
//! This module provides the core verification functions for XMSS signatures,
//! including Merkle tree authentication path verification.

use alloc::vec::Vec;

use crate::tweak_hash::{apply, chain, Tweak};
use crate::types::{Hash, Parameter, PublicKey, Signature};

/// Verify a Merkle tree authentication path
///
/// Given a leaf (represented as chain ends), its position, and a co-path,
/// recompute the root and check if it matches the expected root.
///
/// # Arguments
/// * `parameter` - Public parameter for the hash function
/// * `root` - Expected Merkle tree root
/// * `position` - Position (index) of the leaf in the tree
/// * `leaf` - Vector of hash values representing the leaf content (chain ends)
/// * `co_path` - Authentication path with sibling hashes and direction flags
///
/// # Returns
/// `true` if the computed root matches the expected root
pub fn hash_tree_verify(
    parameter: &Parameter,
    root: &Hash,
    position: u32,
    leaf: &[Hash],
    co_path: &[(Hash, bool)],
) -> bool {
    let depth = co_path.len();
    let num_leafs: u64 = 1 << depth;

    // Sanity checks
    if depth > 32 {
        return false;
    }

    if (position as u64) >= num_leafs {
        return false;
    }

    // First hash the leaf to get the node in the bottom layer
    let tweak = Tweak::tree_tweak(0, position);
    let mut current_node = apply(parameter, &tweak, leaf);

    // Reconstruct the root using the co-path
    let mut current_position = position;
    for l in 0..depth {
        // Distinguish if current is a left child or right child
        let children = if current_position % 2 == 0 {
            // Left child, so co-path contains the right sibling
            [current_node, co_path[l].0]
        } else {
            // Right child, so co-path contains the left sibling
            [co_path[l].0, current_node]
        };

        // Update position (parent's position)
        current_position >>= 1;

        // Hash to get the parent
        let tweak = Tweak::tree_tweak((l + 1) as u8, current_position);
        current_node = apply(parameter, &tweak, &children);
    }

    // Check that recomputed root matches given root
    current_node == *root
}

/// Verify an XMSS signature
///
/// This is a simplified verification function that:
/// 1. Encodes the message to get chain positions
/// 2. Walks the chains from signature hash values to chain ends
/// 3. Verifies the Merkle authentication path
///
/// # Arguments
/// * `public_key` - The public key containing root and parameter
/// * `epoch` - The epoch (time period) of the signature
/// * `message` - The message that was signed (32 bytes)
/// * `signature` - The signature to verify
/// * `chain_length` - The total length of each chain (BASE from encoding)
///
/// # Returns
/// `true` if the signature is valid
pub fn verify_signature(
    public_key: &PublicKey,
    epoch: u32,
    message: &[u8; 32],
    signature: &Signature,
    chain_length: usize,
) -> bool {
    // Step 1: Encode the message to get chain values
    // For now, we use a simplified encoding that converts message bytes to positions
    let encoded = encode_message_simple(message, chain_length, signature.hashes.len());

    // Step 2: Walk each chain from the signature hash to the chain end
    let mut chain_ends: Vec<Hash> = Vec::with_capacity(signature.hashes.len());

    for (chain_index, &encoded_value) in encoded.iter().enumerate() {
        // The signature provides the hash at position `encoded_value`
        // We need to walk from there to the end of the chain
        let start_pos_in_chain = encoded_value as u8;
        let steps = (chain_length - 1) - encoded_value;

        let end = chain(
            &public_key.parameter,
            epoch,
            chain_index as u8,
            start_pos_in_chain,
            steps,
            &signature.hashes[chain_index],
        );
        chain_ends.push(end);
    }

    // Step 3: Verify the Merkle authentication path
    let co_path: Vec<(Hash, bool)> = signature.path.co_path.clone();

    hash_tree_verify(
        &public_key.parameter,
        &public_key.root,
        epoch,
        &chain_ends,
        &co_path,
    )
}

/// Simple message encoding for testing
///
/// This is a simplified version of the incomparable encoding used in leanSig.
/// For a full implementation, this would need to match the TargetSum encoding.
///
/// # Arguments
/// * `message` - 32-byte message
/// * `chain_length` - Length of each chain
/// * `num_chains` - Number of chains
///
/// # Returns
/// Vector of encoded values (one per chain)
fn encode_message_simple(message: &[u8; 32], chain_length: usize, num_chains: usize) -> Vec<usize> {
    // Simple encoding: spread message bytes across chains
    // Each chain position is derived from message bytes
    let max_val = chain_length - 1;

    message
        .iter()
        .cycle()
        .take(num_chains)
        .map(|&b| (b as usize) % (max_val + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_tree_verify_empty_path() {
        // A tree with depth 0 means just the root is the leaf
        let param = Parameter::default();
        let leaf = vec![Hash::default()];

        // Hash the leaf to get what should be the root
        let tweak = Tweak::tree_tweak(0, 0);
        let expected_root = apply(&param, &tweak, &leaf);

        // Verify with empty co-path
        let result = hash_tree_verify(&param, &expected_root, 0, &leaf, &[]);
        assert!(result);
    }

    #[test]
    fn test_encode_message_simple() {
        let message = [0u8; 32];
        let chain_length = 16;
        let num_chains = 128;

        let encoded = encode_message_simple(&message, chain_length, num_chains);

        assert_eq!(encoded.len(), num_chains);
        for &val in &encoded {
            assert!(val < chain_length);
        }
    }

    #[test]
    fn test_encode_message_deterministic() {
        let message = [42u8; 32];
        let chain_length = 16;
        let num_chains = 64;

        let encoded1 = encode_message_simple(&message, chain_length, num_chains);
        let encoded2 = encode_message_simple(&message, chain_length, num_chains);

        assert_eq!(encoded1, encoded2);
    }
}
