//! Shared constants for leanSig XMSS signature verification.
//!
//! These constants are used across all zkVM implementations to ensure
//! consistent cryptographic parameters for benchmarking.

#![no_std]

/// Message length in bytes (fixed at 32 for leanSig)
pub const MESSAGE_LENGTH: usize = 32;

/// Hash output length in field elements (7 elements for Poseidon2 over KoalaBear)
pub const HASH_LEN: usize = 7;

/// Parameter length in field elements
pub const PARAMETER_LEN: usize = 5;

/// Randomness (rho) length in field elements
pub const RANDOMNESS_LEN: usize = 6;

/// Number of chains for TargetSum W=1 encoding
pub const NUM_CHAINS: usize = 155;

/// Tree height (2^18 = 262,144 signatures per key)
pub const TREE_HEIGHT: usize = 18;

/// Base for chain positions (W=1 means binary: 0 or 1)
pub const BASE: usize = 2;

/// Message hash output length in field elements
pub const MSG_HASH_LEN: usize = 5;

/// Message encoding length in field elements (for sponge input)
pub const MSG_LEN_FE: usize = 9;

/// Tweak length in field elements
pub const TWEAK_LEN: usize = 2;
