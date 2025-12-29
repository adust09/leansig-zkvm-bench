//! Poseidon2 tweakable hash implementation.
//!
//! This module provides the core hashing primitives needed for XMSS verification:
//! - Hash chain walking
//! - Merkle path verification
//!
//! Uses a custom Poseidon2 implementation for KoalaBear field.

use alloc::vec::Vec;
use crate::{KoalaBear, F};

/// Poseidon2 permutation width for compression (single message)
pub const WIDTH_16: usize = 16;
/// Poseidon2 permutation width for tree operations (two messages)
pub const WIDTH_24: usize = 24;
/// Hash output length in field elements
pub const HASH_LEN: usize = 4;
/// Parameter length in field elements
pub const PARAMETER_LEN: usize = 4;
/// Tweak length in field elements
pub const TWEAK_LEN: usize = 3;

/// Number of full rounds in Poseidon2
const FULL_ROUNDS: usize = 8;
/// Number of partial rounds in Poseidon2 for width 16
const PARTIAL_ROUNDS_16: usize = 13;
/// Number of partial rounds in Poseidon2 for width 24
const PARTIAL_ROUNDS_24: usize = 21;

/// Tweak type for domain separation in hash operations.
#[derive(Clone, Debug)]
pub enum Tweak {
    /// Tree node at (level, position)
    Tree { level: u32, position: u64 },
    /// Chain step at (epoch, chain_index, step)
    Chain { epoch: u32, chain_index: u32, step: u32 },
}

impl Tweak {
    /// Convert tweak to field elements for hashing (TWEAK_LEN elements).
    pub fn to_field_elements(&self) -> [F; TWEAK_LEN] {
        match self {
            Tweak::Tree { level, position } => [
                // Domain separator 0x01 for tree
                KoalaBear::new(0x01),
                KoalaBear::new(*level),
                KoalaBear::new(*position as u32),
            ],
            Tweak::Chain { epoch, chain_index, step } => [
                // Domain separator 0x02 for chain
                // Pack epoch and chain_index, then step
                KoalaBear::new(0x02_0000 | (*epoch & 0xFFFF)),
                KoalaBear::new(*chain_index),
                KoalaBear::new(*step),
            ],
        }
    }
}

/// S-box for Poseidon2: x^7 (degree 7)
#[inline]
fn sbox(x: F) -> F {
    let x2 = x * x;
    let x4 = x2 * x2;
    let x3 = x2 * x;
    x4 * x3  // x^7
}

/// Generate round constant for Poseidon2
/// Uses a simple deterministic PRNG based on round and position
#[inline]
fn round_constant(round: usize, pos: usize, width: usize) -> F {
    // Simple deterministic constant generation
    // In production, these should be properly generated constants
    let seed = ((round * width + pos) * 0x9E3779B9) as u32;
    KoalaBear::new(seed % crate::koalabear::P)
}

/// Internal linear layer for Poseidon2 (M4 matrix)
/// Uses a 4x4 circulant matrix with coefficients [5, 7, 1, 3]
fn m4_multiply(state: &mut [F; 4]) {
    let t0 = state[0] + state[1];
    let t1 = state[2] + state[3];
    let t2 = state[1] + state[1] + t1;  // 2*x1 + x2 + x3
    let t3 = state[3] + state[3] + t0;  // x0 + x1 + 2*x3

    state[3] = t0 + t1 + t1 + t1 + state[3];  // t0 + 3*t1 + x3
    state[1] = t0 + t0 + t0 + t1 + state[1];  // 3*t0 + t1 + x1
    state[0] = t2 + t3;
    state[2] = t2 + t2 + t3;
}

/// External linear layer for Poseidon2
fn external_linear_layer<const WIDTH: usize>(state: &mut [F; WIDTH]) {
    // Apply M4 to each 4-element chunk
    let num_chunks = WIDTH / 4;

    // First, apply M4 to each chunk
    for chunk_idx in 0..num_chunks {
        let offset = chunk_idx * 4;
        let mut chunk = [
            state[offset],
            state[offset + 1],
            state[offset + 2],
            state[offset + 3],
        ];
        m4_multiply(&mut chunk);
        state[offset] = chunk[0];
        state[offset + 1] = chunk[1];
        state[offset + 2] = chunk[2];
        state[offset + 3] = chunk[3];
    }

    // Then add sums across chunks
    if num_chunks > 1 {
        // Compute sum of all elements
        let mut sums = [KoalaBear::ZERO; 4];
        for chunk_idx in 0..num_chunks {
            let offset = chunk_idx * 4;
            for j in 0..4 {
                sums[j] = sums[j] + state[offset + j];
            }
        }
        // Add sums back to each element
        for chunk_idx in 0..num_chunks {
            let offset = chunk_idx * 4;
            for j in 0..4 {
                state[offset + j] = state[offset + j] + sums[j];
            }
        }
    }
}

/// Internal linear layer for Poseidon2 (diffusion)
fn internal_linear_layer<const WIDTH: usize>(state: &mut [F; WIDTH]) {
    // Simple diffusion: rotate and add
    let sum: F = state.iter().fold(KoalaBear::ZERO, |acc, &x| acc + x);
    for i in 0..WIDTH {
        state[i] = state[i] + sum;
    }
}

/// Poseidon2 permutation for width 16
fn poseidon2_permute_16(state: &mut [F; WIDTH_16]) {
    let half_full = FULL_ROUNDS / 2;

    // Initial external linear layer
    external_linear_layer(state);

    // First half of full rounds
    for round in 0..half_full {
        // Add round constants and apply S-box to all elements
        for i in 0..WIDTH_16 {
            state[i] = state[i] + round_constant(round, i, WIDTH_16);
            state[i] = sbox(state[i]);
        }
        external_linear_layer(state);
    }

    // Partial rounds (S-box only on first element)
    for round in 0..PARTIAL_ROUNDS_16 {
        state[0] = state[0] + round_constant(half_full + round, 0, WIDTH_16);
        state[0] = sbox(state[0]);
        internal_linear_layer(state);
    }

    // Second half of full rounds
    for round in 0..half_full {
        for i in 0..WIDTH_16 {
            state[i] = state[i] + round_constant(half_full + PARTIAL_ROUNDS_16 + round, i, WIDTH_16);
            state[i] = sbox(state[i]);
        }
        external_linear_layer(state);
    }
}

/// Poseidon2 permutation for width 24 (public for sponge construction)
pub fn poseidon2_permute_24(state: &mut [F; WIDTH_24]) {
    let half_full = FULL_ROUNDS / 2;

    // Initial external linear layer
    external_linear_layer(state);

    // First half of full rounds
    for round in 0..half_full {
        for i in 0..WIDTH_24 {
            state[i] = state[i] + round_constant(round, i, WIDTH_24);
            state[i] = sbox(state[i]);
        }
        external_linear_layer(state);
    }

    // Partial rounds
    for round in 0..PARTIAL_ROUNDS_24 {
        state[0] = state[0] + round_constant(half_full + round, 0, WIDTH_24);
        state[0] = sbox(state[0]);
        internal_linear_layer(state);
    }

    // Second half of full rounds
    for round in 0..half_full {
        for i in 0..WIDTH_24 {
            state[i] = state[i] + round_constant(half_full + PARTIAL_ROUNDS_24 + round, i, WIDTH_24);
            state[i] = sbox(state[i]);
        }
        external_linear_layer(state);
    }
}

/// Apply Poseidon2 compression with feed-forward.
///
/// Constructs state from parameter || tweak || message, applies permutation,
/// then XORs back the original input (feed-forward) and returns first HASH_LEN elements.
fn poseidon_compress_16(input: &[F; WIDTH_16]) -> [F; HASH_LEN] {
    let mut state = *input;
    poseidon2_permute_16(&mut state);

    // Feed-forward: add original input to permuted state
    for i in 0..WIDTH_16 {
        state[i] = state[i] + input[i];
    }

    // Return first HASH_LEN elements
    let mut output = [KoalaBear::ZERO; HASH_LEN];
    output.copy_from_slice(&state[..HASH_LEN]);
    output
}

/// Apply Poseidon2 compression with width 24 (for tree operations).
fn poseidon_compress_24(input: &[F; WIDTH_24]) -> [F; HASH_LEN] {
    let mut state = *input;
    poseidon2_permute_24(&mut state);

    // Feed-forward: add original input to permuted state
    for i in 0..WIDTH_24 {
        state[i] = state[i] + input[i];
    }

    // Return first HASH_LEN elements
    let mut output = [KoalaBear::ZERO; HASH_LEN];
    output.copy_from_slice(&state[..HASH_LEN]);
    output
}

/// Apply tweakable hash function.
///
/// This is the core primitive for both chain walking and Merkle verification.
/// Uses width-16 for single message, width-24 for two messages.
pub fn apply_hash(
    parameter: &[F],
    tweak: &Tweak,
    input: &[F],
) -> Vec<F> {
    let tweak_elems = tweak.to_field_elements();

    // Determine which permutation width to use based on input size
    // Single message (HASH_LEN elements) -> width 16
    // Two messages (2 * HASH_LEN elements) -> width 24

    if input.len() <= HASH_LEN {
        // Width-16 compression for single message
        let mut state = [KoalaBear::ZERO; WIDTH_16];

        // Fill state: parameter || tweak || message || padding
        let mut idx = 0;
        for i in 0..PARAMETER_LEN.min(parameter.len()) {
            state[idx] = parameter[i];
            idx += 1;
        }
        idx = PARAMETER_LEN; // Ensure we start tweak at correct position

        for i in 0..TWEAK_LEN {
            state[idx] = tweak_elems[i];
            idx += 1;
        }
        idx = PARAMETER_LEN + TWEAK_LEN;

        for i in 0..input.len() {
            state[idx] = input[i];
            idx += 1;
        }
        // Remaining positions are zero (padding)

        let output = poseidon_compress_16(&state);
        output.to_vec()
    } else {
        // Width-24 compression for two messages (tree merge)
        let mut state = [KoalaBear::ZERO; WIDTH_24];

        // Fill state: parameter || tweak || message || padding
        let mut idx = 0;
        for i in 0..PARAMETER_LEN.min(parameter.len()) {
            state[idx] = parameter[i];
            idx += 1;
        }
        idx = PARAMETER_LEN;

        for i in 0..TWEAK_LEN {
            state[idx] = tweak_elems[i];
            idx += 1;
        }
        idx = PARAMETER_LEN + TWEAK_LEN;

        for i in 0..input.len() {
            if idx < WIDTH_24 {
                state[idx] = input[i];
                idx += 1;
            }
        }

        let output = poseidon_compress_24(&state);
        output.to_vec()
    }
}

/// Walk a hash chain from starting point for given number of steps.
pub fn chain_walk(
    parameter: &[F],
    epoch: u32,
    chain_index: u32,
    start_step: u32,
    steps: u32,
    start_value: &[F],
) -> Vec<F> {
    let mut current = start_value.to_vec();

    for i in 0..steps {
        let tweak = Tweak::Chain {
            epoch,
            chain_index,
            step: start_step + i,
        };
        current = apply_hash(parameter, &tweak, &current);
    }

    current
}

/// Verify Merkle authentication path.
pub fn verify_merkle_path(
    parameter: &[F],
    leaf: &[F],
    path: &[Vec<F>],
    leaf_index: u64,
    expected_root: &[F],
) -> bool {
    let mut current = leaf.to_vec();
    let mut index = leaf_index;

    for (level, sibling) in path.iter().enumerate() {
        let tweak = Tweak::Tree {
            level: level as u32,
            position: index / 2,
        };

        // Determine ordering based on index parity
        let (left, right) = if index % 2 == 0 {
            (&current, sibling)
        } else {
            (sibling, &current)
        };

        // Hash pair to get parent
        let mut combined = left.clone();
        combined.extend(right.iter());
        current = apply_hash(parameter, &tweak, &combined);

        index /= 2;
    }

    current == expected_root
}
