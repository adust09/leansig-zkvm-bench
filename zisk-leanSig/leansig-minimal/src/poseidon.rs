//! Poseidon2 tweakable hash implementation.
//!
//! This module provides the core hashing primitives needed for XMSS verification:
//! - Hash chain walking
//! - Merkle path verification

use alloc::vec::Vec;
use p3_koala_bear::{KoalaBear, default_koalabear_poseidon2_16, default_koalabear_poseidon2_24};
use p3_symmetric::Permutation;

use crate::F;

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

/// Apply Poseidon2 compression with feed-forward.
///
/// Constructs state from parameter || tweak || message, applies permutation,
/// then XORs back the original input (feed-forward) and returns first HASH_LEN elements.
fn poseidon_compress_16(input: &[F; WIDTH_16]) -> [F; HASH_LEN] {
    let perm = default_koalabear_poseidon2_16();
    let mut state = *input;
    perm.permute_mut(&mut state);

    // Feed-forward: add original input to permuted state
    for i in 0..WIDTH_16 {
        state[i] = state[i] + input[i];
    }

    // Return first HASH_LEN elements
    let mut output = [KoalaBear::new(0); HASH_LEN];
    output.copy_from_slice(&state[..HASH_LEN]);
    output
}

/// Apply Poseidon2 compression with width 24 (for tree operations).
fn poseidon_compress_24(input: &[F; WIDTH_24]) -> [F; HASH_LEN] {
    let perm = default_koalabear_poseidon2_24();
    let mut state = *input;
    perm.permute_mut(&mut state);

    // Feed-forward: add original input to permuted state
    for i in 0..WIDTH_24 {
        state[i] = state[i] + input[i];
    }

    // Return first HASH_LEN elements
    let mut output = [KoalaBear::new(0); HASH_LEN];
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
        let mut state = [KoalaBear::new(0); WIDTH_16];

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
        let mut state = [KoalaBear::new(0); WIDTH_24];

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
