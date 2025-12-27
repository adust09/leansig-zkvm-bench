//! XMSS signature verification.

use alloc::vec::Vec;

use crate::{
    types::{PublicKey, Signature, VerifyInput},
    poseidon::{chain_walk, verify_merkle_path},
    encoding::encode,
    MESSAGE_LENGTH, F,
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
    // Step 1: Re-encode the message using the provided randomness
    let encoded = match encode(epoch, &sig.rho, message) {
        Ok(chunks) => chunks,
        Err(_) => return false,
    };

    // Step 2: Reconstruct chain endpoints by walking from signature hashes
    // Each encoded chunk value tells us how many more steps to walk
    let chain_endpoints = reconstruct_chain_endpoints(
        &pk.parameter,
        epoch,
        &encoded,
        &sig.hashes,
    );

    // Step 3: Combine chain endpoints into a leaf value
    let leaf = combine_chain_endpoints(&pk.parameter, &chain_endpoints);

    // Step 4: Verify the Merkle authentication path
    let leaf_index = epoch as u64; // Epoch determines leaf position
    verify_merkle_path(&pk.parameter, &leaf, &sig.path, leaf_index, &pk.root)
}

/// Reconstruct chain endpoints from signature hashes and encoded values.
fn reconstruct_chain_endpoints(
    parameter: &[F],
    epoch: u32,
    encoded: &[u8],
    sig_hashes: &[Vec<F>],
) -> Vec<Vec<F>> {
    // TODO: Implement proper chain endpoint reconstruction
    // For each chain i:
    //   1. Start from sig_hashes[i] (value at step `encoded[i]`)
    //   2. Walk remaining steps: BASE - 1 - encoded[i]
    //   3. Result is the chain endpoint

    let mut endpoints = Vec::with_capacity(encoded.len());

    for (i, &steps_done) in encoded.iter().enumerate() {
        let remaining_steps = (crate::encoding::BASE - 1) as u32 - steps_done as u32;

        let endpoint = chain_walk(
            parameter,
            epoch,
            i as u32,
            steps_done as u32,
            remaining_steps,
            &sig_hashes[i],
        );

        endpoints.push(endpoint);
    }

    endpoints
}

/// Combine chain endpoints into a single leaf value using sponge construction.
///
/// For XMSS, this takes all chain endpoints and combines them into a single
/// leaf hash using a sponge-based approach.
fn combine_chain_endpoints(
    parameter: &[F],
    endpoints: &[Vec<F>],
) -> Vec<F> {
    use crate::poseidon::{HASH_LEN, PARAMETER_LEN, WIDTH_24};
    use p3_koala_bear::{KoalaBear, default_koalabear_poseidon2_24};
    use p3_symmetric::Permutation;

    // Sponge parameters
    const RATE: usize = WIDTH_24 - HASH_LEN; // Rate = WIDTH - Capacity
    const CAPACITY: usize = HASH_LEN;

    // Flatten all endpoints into a single input vector
    let mut input: Vec<F> = Vec::with_capacity(
        PARAMETER_LEN + endpoints.len() * HASH_LEN
    );

    // Add parameter
    for i in 0..PARAMETER_LEN.min(parameter.len()) {
        input.push(parameter[i]);
    }

    // Add all endpoints
    for endpoint in endpoints {
        input.extend(endpoint.iter().cloned());
    }

    // Initialize sponge state
    // State = [rate portion (zeros) | capacity portion (domain separator)]
    let mut state = [KoalaBear::new(0); WIDTH_24];

    // Set domain separator in capacity portion
    // Encode: num_endpoints, hash_len as simple domain separation
    state[RATE] = KoalaBear::new(endpoints.len() as u32);
    state[RATE + 1] = KoalaBear::new(HASH_LEN as u32);

    let perm = default_koalabear_poseidon2_24();

    // Absorb phase: process input in chunks of RATE
    let mut offset = 0;
    while offset < input.len() {
        // XOR input chunk into rate portion
        for i in 0..RATE {
            if offset + i < input.len() {
                state[i] = state[i] + input[offset + i];
            }
        }
        offset += RATE;

        // Apply permutation
        perm.permute_mut(&mut state);
    }

    // Squeeze phase: extract HASH_LEN elements from rate portion
    state[..HASH_LEN].to_vec()
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
