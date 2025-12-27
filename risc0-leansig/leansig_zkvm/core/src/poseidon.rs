//! Poseidon2 hash function implementation for no_std
//!
//! This module provides the core Poseidon2 compression and sponge functions
//! used in XMSS signature verification.

use alloc::vec::Vec;
use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::{
    default_koalabear_poseidon2_16, default_koalabear_poseidon2_24, Poseidon2KoalaBear,
};
use p3_symmetric::Permutation;

use crate::field::F;

/// Width for chain compression (single hash input)
pub const CHAIN_COMPRESSION_WIDTH: usize = 16;

/// Width for tree/sponge operations (two hash inputs)
pub const MERGE_COMPRESSION_WIDTH: usize = 24;

/// Get Poseidon2 permutation with width 16 (for chains)
///
/// Note: In no_std, we create fresh each time instead of caching.
/// This has minimal overhead since the round constants are compile-time.
pub fn poseidon2_16() -> Poseidon2KoalaBear<16> {
    default_koalabear_poseidon2_16()
}

/// Get Poseidon2 permutation with width 24 (for tree nodes)
pub fn poseidon2_24() -> Poseidon2KoalaBear<24> {
    default_koalabear_poseidon2_24()
}

/// Poseidon Compression Function
///
/// Computes: PoseidonCompress(x) = Truncate(PoseidonPermute(x) + x)
///
/// # Type Parameters
/// - `WIDTH`: total state width (input length to permutation)
/// - `OUT_LEN`: number of output elements to return
///
/// # Arguments
/// - `perm`: Poseidon permutation
/// - `input`: slice of input values (will be zero-padded to WIDTH)
///
/// # Returns
/// First `OUT_LEN` elements of the compressed state
pub fn poseidon_compress<R, P, const WIDTH: usize, const OUT_LEN: usize>(
    perm: &P,
    input: &[R],
) -> [R; OUT_LEN]
where
    R: PrimeCharacteristicRing + Copy,
    P: Permutation<[R; WIDTH]>,
{
    assert!(
        input.len() >= OUT_LEN,
        "Poseidon Compression: Input length must be at least output length."
    );

    // Zero-pad input to WIDTH
    let mut padded_input = [R::ZERO; WIDTH];
    padded_input[..input.len()].copy_from_slice(input);

    // Apply permutation - use permute() since it takes by value
    let permuted = perm.permute(padded_input);

    // Feed-forward: add input back
    let mut state = permuted;
    for i in 0..WIDTH {
        state[i] += padded_input[i];
    }

    // Truncate to OUT_LEN
    let mut result = [R::ZERO; OUT_LEN];
    result.copy_from_slice(&state[..OUT_LEN]);
    result
}

/// Poseidon Sponge Hash Function
///
/// Absorbs arbitrary-length input using sponge construction.
/// Uses fixed width 24 for KoalaBear (as used in leanSig for sponge mode).
///
/// # Type Parameters
/// - `OUT_LEN`: number of output elements
///
/// # Arguments
/// - `perm`: Poseidon2 permutation (width 24)
/// - `capacity_value`: values for capacity (domain separation)
/// - `input`: message to hash
pub fn poseidon_sponge<const OUT_LEN: usize>(
    perm: &Poseidon2KoalaBear<MERGE_COMPRESSION_WIDTH>,
    capacity_value: &[F],
    input: &[F],
) -> [F; OUT_LEN] {
    assert!(
        capacity_value.len() < MERGE_COMPRESSION_WIDTH,
        "Capacity length must be smaller than state width."
    );

    let rate = MERGE_COMPRESSION_WIDTH - capacity_value.len();

    // Pad input to multiple of rate
    let extra_elements = (rate - (input.len() % rate)) % rate;
    let mut input_vector = input.to_vec();
    input_vector.resize(input.len() + extra_elements, F::ZERO);

    // Initialize state with capacity
    let mut state = [F::ZERO; MERGE_COMPRESSION_WIDTH];
    state[rate..].copy_from_slice(capacity_value);

    // Absorb phase
    for chunk in input_vector.chunks(rate) {
        for i in 0..chunk.len() {
            state[i] += chunk[i];
        }
        state = perm.permute(state);
    }

    // Squeeze phase
    let mut out: Vec<F> = Vec::new();
    while out.len() < OUT_LEN {
        out.extend_from_slice(&state[..rate]);
        state = perm.permute(state);
    }

    let mut result = [F::ZERO; OUT_LEN];
    result.copy_from_slice(&out[0..OUT_LEN]);
    result
}

/// Domain separator computation for sponge
pub fn poseidon_safe_domain_separator<const OUT_LEN: usize>(
    perm: &Poseidon2KoalaBear<MERGE_COMPRESSION_WIDTH>,
    params: &[u32; 4],
) -> [F; OUT_LEN] {
    // Combine params into single number in base 2^32
    let mut acc: u128 = 0;
    for &param in params {
        acc = (acc << 32) | (param as u128);
    }

    // Base-p decomposition
    let input: [F; MERGE_COMPRESSION_WIDTH] = core::array::from_fn(|_| {
        let digit = (acc % (F::ORDER_U64 as u128)) as u64;
        acc /= F::ORDER_U64 as u128;
        F::from_u64(digit)
    });

    poseidon_compress::<F, _, MERGE_COMPRESSION_WIDTH, OUT_LEN>(perm, &input)
}

use p3_field::PrimeField64;
