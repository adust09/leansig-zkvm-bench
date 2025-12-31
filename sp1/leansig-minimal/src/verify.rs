//! XMSS signature verification using TargetSum W=1 encoding.

use alloc::vec::Vec;

use crate::{
    types::{PublicKey, Signature, VerifyInput},
    poseidon::{chain_walk, verify_merkle_path},
    encoding::compute_codeword,
    MESSAGE_LENGTH, F, HASH_LEN, NUM_CHAINS, BASE, TREE_HEIGHT, PARAMETER_LEN,
};

/// Verify an XMSS signature.
///
/// # Arguments
/// * `pk` - Public key (Merkle root + parameters)
/// * `epoch` - Epoch number for this signature
/// * `message` - 32-byte message that was signed
/// * `sig` - The signature to verify
///
/// # Returns
/// `true` if the signature is valid, `false` otherwise.
pub fn verify_signature(
    pk: &PublicKey,
    epoch: u32,
    message: &[u8; MESSAGE_LENGTH],
    sig: &Signature,
) -> bool {
    // Validate signature structure
    if sig.hashes.len() != NUM_CHAINS {
        return false;
    }
    if sig.path.len() != TREE_HEIGHT {
        return false;
    }
    if sig.leaf_index != epoch {
        return false;
    }

    // Step 1: Compute the codeword (chain positions) from the message
    let codeword = compute_codeword(
        &pk.parameter,
        epoch,
        &sig.rho,
        message,
    );

    if codeword.len() != NUM_CHAINS {
        return false;
    }

    // Step 2: Reconstruct chain endpoints by walking from signature hashes
    let chain_ends = reconstruct_chain_endpoints(
        &pk.parameter,
        epoch,
        &codeword,
        &sig.hashes,
    );

    // Step 3: Verify the Merkle authentication path
    verify_merkle_path(
        &pk.parameter,
        &pk.root,
        epoch,
        &chain_ends,
        &sig.path,
        TREE_HEIGHT,
    )
}

/// Reconstruct chain endpoints from signature hashes and codeword values.
///
/// For TargetSum W=1 (BASE=2), each codeword value is 0 or 1:
/// - If codeword[i] = 0: walk 1 step from sig.hashes[i] to reach endpoint
/// - If codeword[i] = 1: sig.hashes[i] is already at the endpoint (walk 0 steps)
fn reconstruct_chain_endpoints(
    parameter: &[F; PARAMETER_LEN],
    epoch: u32,
    codeword: &[u8],
    sig_hashes: &[[F; HASH_LEN]],
) -> Vec<[F; HASH_LEN]> {
    let mut endpoints = Vec::with_capacity(NUM_CHAINS);

    for (chain_index, (&steps_seen, start_hash)) in codeword
        .iter()
        .zip(sig_hashes.iter())
        .enumerate()
    {
        let start_pos = steps_seen;

        // Validate chain position is within bounds
        if steps_seen as usize >= BASE {
            // Invalid codeword value, return empty (will fail verification)
            return Vec::new();
        }

        // Calculate remaining steps to reach endpoint
        // For BASE=2: if steps_seen=0, walk 1 step; if steps_seen=1, walk 0 steps
        let remaining = (BASE - 1) as u8 - start_pos;

        let endpoint = chain_walk(
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

/// Convenience function to verify from serialized input.
pub fn verify_from_input(input: &VerifyInput) -> bool {
    verify_signature(
        &input.public_key,
        input.epoch,
        &input.message,
        &input.signature,
    )
}
