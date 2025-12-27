//! Tweakable hash function implementation for XMSS
//!
//! This module provides the Poseidon2-based tweakable hash functions
//! used in chain and tree operations.

use alloc::vec::Vec;

use crate::field::{
    F, TWEAK_SEPARATOR_FOR_CHAIN_HASH, TWEAK_SEPARATOR_FOR_TREE_HASH,
};
use crate::poseidon::{
    poseidon_compress, poseidon_sponge, poseidon_safe_domain_separator,
    poseidon2_16, poseidon2_24, CHAIN_COMPRESSION_WIDTH, MERGE_COMPRESSION_WIDTH,
};
use crate::types::{Hash, Parameter, HASH_LEN, PARAMETER_LEN};
use p3_field::{PrimeCharacteristicRing, PrimeField64};

/// Tweak length in field elements
pub const TWEAK_LEN: usize = 2;

/// Capacity for sponge construction
pub const CAPACITY: usize = 9;

/// Domain parameters length for sponge separator
const DOMAIN_PARAMETERS_LENGTH: usize = 4;

/// Tweak enum for different hash contexts
#[derive(Debug, Clone)]
pub enum Tweak {
    /// Tweak for Merkle tree hashing
    TreeTweak {
        level: u8,
        pos_in_level: u32,
    },
    /// Tweak for chain hashing
    ChainTweak {
        epoch: u32,
        chain_index: u8,
        pos_in_chain: u8,
    },
}

impl Tweak {
    /// Convert tweak to field elements using base-p decomposition
    pub fn to_field_elements(&self) -> [F; TWEAK_LEN] {
        // Represent the entire tweak as one big integer
        let mut acc = match self {
            Self::TreeTweak {
                level,
                pos_in_level,
            } => {
                ((*level as u128) << 40)
                    | ((*pos_in_level as u128) << 8)
                    | (TWEAK_SEPARATOR_FOR_TREE_HASH as u128)
            }
            Self::ChainTweak {
                epoch,
                chain_index,
                pos_in_chain,
            } => {
                ((*epoch as u128) << 24)
                    | ((*chain_index as u128) << 16)
                    | ((*pos_in_chain as u128) << 8)
                    | (TWEAK_SEPARATOR_FOR_CHAIN_HASH as u128)
            }
        };

        // Interpret this integer in base-p to get field elements
        core::array::from_fn(|_| {
            let digit = (acc % (F::ORDER_U64 as u128)) as u64;
            acc /= F::ORDER_U64 as u128;
            F::from_u64(digit)
        })
    }

    /// Create a tree tweak
    pub fn tree_tweak(level: u8, pos_in_level: u32) -> Self {
        Self::TreeTweak { level, pos_in_level }
    }

    /// Create a chain tweak
    pub fn chain_tweak(epoch: u32, chain_index: u8, pos_in_chain: u8) -> Self {
        Self::ChainTweak {
            epoch,
            chain_index,
            pos_in_chain,
        }
    }
}

/// Apply the tweakable hash to parameter, tweak, and message
///
/// Three cases:
/// 1. Single hash input (chain): compression mode with width 16
/// 2. Two hash inputs (tree merge): compression mode with width 24
/// 3. Many hash inputs (leaf): sponge mode
pub fn apply(parameter: &Parameter, tweak: &Tweak, message: &[Hash]) -> Hash {
    let tweak_fe = tweak.to_field_elements();

    match message.len() {
        1 => {
            // Chain: compress parameter || tweak || single_hash
            let perm = poseidon2_16();
            let combined_input: Vec<F> = parameter
                .iter()
                .chain(tweak_fe.iter())
                .chain(message[0].iter())
                .copied()
                .collect();
            Hash::new(
                poseidon_compress::<F, _, CHAIN_COMPRESSION_WIDTH, HASH_LEN>(
                    &perm,
                    &combined_input,
                ),
            )
        }

        2 => {
            // Tree merge: compress parameter || tweak || left || right
            let perm = poseidon2_24();
            let combined_input: Vec<F> = parameter
                .iter()
                .chain(tweak_fe.iter())
                .chain(message[0].iter())
                .chain(message[1].iter())
                .copied()
                .collect();
            Hash::new(
                poseidon_compress::<F, _, MERGE_COMPRESSION_WIDTH, HASH_LEN>(
                    &perm,
                    &combined_input,
                ),
            )
        }

        n if n > 2 => {
            // Leaf hashing: sponge over many chain ends
            let perm = poseidon2_24();
            let combined_input: Vec<F> = parameter
                .iter()
                .chain(tweak_fe.iter())
                .chain(message.iter().flat_map(|h| h.iter()))
                .copied()
                .collect();

            let lengths: [u32; DOMAIN_PARAMETERS_LENGTH] = [
                PARAMETER_LEN as u32,
                TWEAK_LEN as u32,
                n as u32,
                HASH_LEN as u32,
            ];
            let capacity_value = poseidon_safe_domain_separator::<CAPACITY>(&perm, &lengths);
            Hash::new(poseidon_sponge::<HASH_LEN>(
                &perm,
                &capacity_value,
                &combined_input,
            ))
        }

        _ => {
            // Empty message - return identity (shouldn't happen in practice)
            Hash::default()
        }
    }
}

/// Walk a hash chain from start position for a number of steps
///
/// This is the core chain operation used in XMSS verification.
/// Each step applies: current = TH(parameter, chain_tweak, current)
pub fn chain(
    parameter: &Parameter,
    epoch: u32,
    chain_index: u8,
    start_pos_in_chain: u8,
    steps: usize,
    start: &Hash,
) -> Hash {
    let mut current = *start;

    for j in 0..steps {
        let tweak = Tweak::chain_tweak(epoch, chain_index, start_pos_in_chain + (j as u8) + 1);
        current = apply(parameter, &tweak, &[current]);
    }

    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tweak_encoding() {
        // Test that tree and chain tweaks produce different field elements
        let tree_tweak = Tweak::tree_tweak(0, 0);
        let chain_tweak = Tweak::chain_tweak(0, 0, 0);

        let tree_fe = tree_tweak.to_field_elements();
        let chain_fe = chain_tweak.to_field_elements();

        // They should be different due to different separators
        assert_ne!(tree_fe, chain_fe);
    }

    #[test]
    fn test_chain_deterministic() {
        let param = Parameter::default();
        let start = Hash::default();

        let result1 = chain(&param, 1, 0, 0, 5, &start);
        let result2 = chain(&param, 1, 0, 0, 5, &start);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_apply_single_hash() {
        let param = Parameter::default();
        let tweak = Tweak::chain_tweak(1, 2, 3);
        let msg = Hash::default();

        let result = apply(&param, &tweak, &[msg]);

        // Result should be a valid hash (non-zero with overwhelming probability)
        // Just check it doesn't panic
        assert_eq!(result.inner().len(), HASH_LEN);
    }
}
