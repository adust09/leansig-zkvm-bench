//! Poseidon2 tweakable hash implementation for KoalaBear field.
//!
//! This module provides the core hashing primitives for XMSS verification:
//! - Hash chain walking
//! - Merkle path verification
//! - Message hashing (for encoding)
//!
//! Uses Poseidon2 with width 16 (single input) and width 24 (pair/sponge).

use alloc::vec::Vec;
use crate::{KoalaBear, F, HASH_LEN, PARAMETER_LEN, TWEAK_LEN};
use crate::koalabear::P;

/// Poseidon2 permutation width for compression (single message)
pub const WIDTH_16: usize = 16;
/// Poseidon2 permutation width for tree operations (two messages) and sponge
pub const WIDTH_24: usize = 24;

/// Number of full rounds in Poseidon2
const FULL_ROUNDS: usize = 8;
/// Number of partial rounds for width 16
const PARTIAL_ROUNDS_16: usize = 13;
/// Number of partial rounds for width 24
const PARTIAL_ROUNDS_24: usize = 21;

/// Tweak separator constants
const TWEAK_SEPARATOR_FOR_TREE_HASH: u8 = 0x01;
const TWEAK_SEPARATOR_FOR_CHAIN_HASH: u8 = 0x00;

/// Tweak type for domain separation in hash operations.
#[derive(Clone, Copy, Debug)]
pub enum PoseidonTweak {
    /// Tree node: (level, position_in_level)
    Tree { level: u8, pos_in_level: u32 },
    /// Chain step: (epoch, chain_index, position_in_chain)
    Chain { epoch: u32, chain_index: u8, pos_in_chain: u8 },
}

impl PoseidonTweak {
    pub fn tree(level: u8, pos_in_level: u32) -> Self {
        PoseidonTweak::Tree { level, pos_in_level }
    }

    pub fn chain(epoch: u32, chain_index: u8, pos_in_chain: u8) -> Self {
        PoseidonTweak::Chain { epoch, chain_index, pos_in_chain }
    }

    /// Convert tweak to TWEAK_LEN field elements (matching OpenVM encoding).
    pub fn to_field_elements(&self) -> [F; TWEAK_LEN] {
        let mut acc: u128 = match self {
            PoseidonTweak::Tree { level, pos_in_level } => {
                ((*level as u128) << 40)
                    | ((*pos_in_level as u128) << 8)
                    | (TWEAK_SEPARATOR_FOR_TREE_HASH as u128)
            }
            PoseidonTweak::Chain { epoch, chain_index, pos_in_chain } => {
                ((*epoch as u128) << 24)
                    | ((*chain_index as u128) << 16)
                    | ((*pos_in_chain as u128) << 8)
                    | (TWEAK_SEPARATOR_FOR_CHAIN_HASH as u128)
            }
        };

        let mut out = [KoalaBear::ZERO; TWEAK_LEN];
        for digit in &mut out {
            let value = (acc % P as u128) as u32;
            acc /= P as u128;
            *digit = KoalaBear::new(value);
        }
        out
    }
}

/// S-box for Poseidon2: x^7 (degree 7)
#[inline]
fn sbox(x: F) -> F {
    let x2 = x * x;
    let x4 = x2 * x2;
    let x3 = x2 * x;
    x4 * x3
}

/// Generate round constant for Poseidon2.
#[inline]
fn round_constant(round: usize, pos: usize, width: usize) -> F {
    let seed = ((round * width + pos) * 0x9E3779B9) as u32;
    KoalaBear::new(seed % P)
}

/// Internal linear layer for Poseidon2 (M4 matrix).
fn m4_multiply(state: &mut [F; 4]) {
    let t0 = state[0] + state[1];
    let t1 = state[2] + state[3];
    let t2 = state[1] + state[1] + t1;
    let t3 = state[3] + state[3] + t0;

    state[3] = t0 + t1 + t1 + t1 + state[3];
    state[1] = t0 + t0 + t0 + t1 + state[1];
    state[0] = t2 + t3;
    state[2] = t2 + t2 + t3;
}

/// External linear layer for Poseidon2.
fn external_linear_layer<const WIDTH: usize>(state: &mut [F; WIDTH]) {
    let num_chunks = WIDTH / 4;

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

    if num_chunks > 1 {
        let mut sums = [KoalaBear::ZERO; 4];
        for chunk_idx in 0..num_chunks {
            let offset = chunk_idx * 4;
            for j in 0..4 {
                sums[j] = sums[j] + state[offset + j];
            }
        }
        for chunk_idx in 0..num_chunks {
            let offset = chunk_idx * 4;
            for j in 0..4 {
                state[offset + j] = state[offset + j] + sums[j];
            }
        }
    }
}

/// Internal linear layer for Poseidon2 (diffusion).
fn internal_linear_layer<const WIDTH: usize>(state: &mut [F; WIDTH]) {
    let sum: F = state.iter().fold(KoalaBear::ZERO, |acc, &x| acc + x);
    for i in 0..WIDTH {
        state[i] = state[i] + sum;
    }
}

/// Poseidon2 permutation for width 16.
fn poseidon2_permute_16(state: &mut [F; WIDTH_16]) {
    let half_full = FULL_ROUNDS / 2;

    external_linear_layer(state);

    for round in 0..half_full {
        for i in 0..WIDTH_16 {
            state[i] = state[i] + round_constant(round, i, WIDTH_16);
            state[i] = sbox(state[i]);
        }
        external_linear_layer(state);
    }

    for round in 0..PARTIAL_ROUNDS_16 {
        state[0] = state[0] + round_constant(half_full + round, 0, WIDTH_16);
        state[0] = sbox(state[0]);
        internal_linear_layer(state);
    }

    for round in 0..half_full {
        for i in 0..WIDTH_16 {
            state[i] = state[i] + round_constant(half_full + PARTIAL_ROUNDS_16 + round, i, WIDTH_16);
            state[i] = sbox(state[i]);
        }
        external_linear_layer(state);
    }
}

/// Poseidon2 permutation for width 24.
pub fn poseidon2_permute_24(state: &mut [F; WIDTH_24]) {
    let half_full = FULL_ROUNDS / 2;

    external_linear_layer(state);

    for round in 0..half_full {
        for i in 0..WIDTH_24 {
            state[i] = state[i] + round_constant(round, i, WIDTH_24);
            state[i] = sbox(state[i]);
        }
        external_linear_layer(state);
    }

    for round in 0..PARTIAL_ROUNDS_24 {
        state[0] = state[0] + round_constant(half_full + round, 0, WIDTH_24);
        state[0] = sbox(state[0]);
        internal_linear_layer(state);
    }

    for round in 0..half_full {
        for i in 0..WIDTH_24 {
            state[i] = state[i] + round_constant(half_full + PARTIAL_ROUNDS_24 + round, i, WIDTH_24);
            state[i] = sbox(state[i]);
        }
        external_linear_layer(state);
    }
}

/// Poseidon2 compression with feed-forward for width 16.
fn poseidon_compress_16_internal(input: &[F; WIDTH_16]) -> [F; HASH_LEN] {
    let mut state = *input;
    poseidon2_permute_16(&mut state);

    for i in 0..WIDTH_16 {
        state[i] = state[i] + input[i];
    }

    let mut output = [KoalaBear::ZERO; HASH_LEN];
    output.copy_from_slice(&state[..HASH_LEN]);
    output
}

/// Poseidon2 compression with feed-forward for width 24.
/// Returns first OUT_LEN elements.
pub fn poseidon_compress_24<const OUT_LEN: usize>(input: &[F]) -> [F; OUT_LEN] {
    assert!(input.len() >= OUT_LEN);
    let mut padded = [KoalaBear::ZERO; WIDTH_24];
    let copy_len = input.len().min(WIDTH_24);
    padded[..copy_len].copy_from_slice(&input[..copy_len]);

    let mut state = padded;
    poseidon2_permute_24(&mut state);

    for i in 0..copy_len {
        state[i] = state[i] + padded[i];
    }

    let mut output = [KoalaBear::ZERO; OUT_LEN];
    output.copy_from_slice(&state[..OUT_LEN]);
    output
}

/// Apply tweakable hash function.
///
/// Uses width-16 for single hash input, width-24 for two hash inputs.
pub fn poseidon_apply(
    parameter: &[F; PARAMETER_LEN],
    tweak: &PoseidonTweak,
    messages: &[[F; HASH_LEN]],
) -> [F; HASH_LEN] {
    let tweak_fe = tweak.to_field_elements();

    match messages.len() {
        1 => {
            let mut input = [KoalaBear::ZERO; WIDTH_16];
            let mut idx = 0;

            for i in 0..PARAMETER_LEN {
                input[idx] = parameter[i];
                idx += 1;
            }
            for i in 0..TWEAK_LEN {
                input[idx] = tweak_fe[i];
                idx += 1;
            }
            for i in 0..HASH_LEN {
                input[idx] = messages[0][i];
                idx += 1;
            }

            poseidon_compress_16_internal(&input)
        }
        2 => {
            let mut input = [KoalaBear::ZERO; WIDTH_24];
            let mut idx = 0;

            for i in 0..PARAMETER_LEN {
                input[idx] = parameter[i];
                idx += 1;
            }
            for i in 0..TWEAK_LEN {
                input[idx] = tweak_fe[i];
                idx += 1;
            }
            for i in 0..HASH_LEN {
                input[idx] = messages[0][i];
                idx += 1;
            }
            for i in 0..HASH_LEN {
                input[idx] = messages[1][i];
                idx += 1;
            }

            poseidon_compress_24::<HASH_LEN>(&input)
        }
        _ => {
            // Sponge construction for more than 2 messages
            poseidon_sponge(parameter, &tweak_fe, messages)
        }
    }
}

/// Sponge construction for hashing multiple messages.
fn poseidon_sponge(
    parameter: &[F; PARAMETER_LEN],
    tweak: &[F; TWEAK_LEN],
    messages: &[[F; HASH_LEN]],
) -> [F; HASH_LEN] {
    const CAPACITY: usize = 9;
    const RATE: usize = WIDTH_24 - CAPACITY;

    // Build input: parameter || tweak || all messages
    let mut input = Vec::with_capacity(PARAMETER_LEN + TWEAK_LEN + messages.len() * HASH_LEN);
    input.extend_from_slice(parameter);
    input.extend_from_slice(tweak);
    for msg in messages {
        input.extend_from_slice(msg);
    }

    // Initialize state with domain separator in capacity
    let mut state = [KoalaBear::ZERO; WIDTH_24];
    state[RATE] = KoalaBear::new(PARAMETER_LEN as u32);
    state[RATE + 1] = KoalaBear::new(TWEAK_LEN as u32);
    state[RATE + 2] = KoalaBear::new(messages.len() as u32);
    state[RATE + 3] = KoalaBear::new(HASH_LEN as u32);

    // Absorb
    let mut idx = 0;
    while idx < input.len() {
        let chunk_len = RATE.min(input.len() - idx);
        for i in 0..chunk_len {
            state[i] = state[i] + input[idx + i];
        }
        poseidon2_permute_24(&mut state);
        idx += chunk_len;
    }

    // Squeeze
    let mut output = [KoalaBear::ZERO; HASH_LEN];
    output.copy_from_slice(&state[..HASH_LEN]);
    output
}

/// Walk a hash chain from starting point for given number of steps.
pub fn chain_walk(
    parameter: &[F; PARAMETER_LEN],
    epoch: u32,
    chain_index: u8,
    start_pos: u8,
    steps: usize,
    start_value: &[F; HASH_LEN],
) -> [F; HASH_LEN] {
    let mut current = *start_value;

    if steps == 0 {
        return current;
    }

    for offset in 0..steps {
        let tweak = PoseidonTweak::chain(epoch, chain_index, start_pos + offset as u8 + 1);
        current = poseidon_apply(parameter, &tweak, &[current]);
    }

    current
}

/// Verify Merkle authentication path.
pub fn verify_merkle_path(
    parameter: &[F; PARAMETER_LEN],
    root: &[F; HASH_LEN],
    position: u32,
    leaf_hashes: &[[F; HASH_LEN]],
    path: &[[F; HASH_LEN]],
    tree_height: usize,
) -> bool {
    if path.len() != tree_height {
        return false;
    }

    // Hash all chain ends to get leaf value
    let tweak = PoseidonTweak::tree(0, position);
    let mut current = poseidon_apply(parameter, &tweak, leaf_hashes);

    let mut idx = position;
    for (level, sibling) in path.iter().enumerate() {
        let children = if idx & 1 == 0 {
            [current, *sibling]
        } else {
            [*sibling, current]
        };
        idx >>= 1;

        let tweak = PoseidonTweak::tree(level as u8 + 1, idx);
        current = poseidon_apply(parameter, &tweak, &children);
    }

    current == *root
}
