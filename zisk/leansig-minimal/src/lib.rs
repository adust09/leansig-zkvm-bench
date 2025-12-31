//! Minimal no_std implementation of leanSig XMSS signature verification.
//!
//! This crate implements XMSS verification using TargetSum W=1 encoding
//! with Poseidon2 over KoalaBear field, matching the leanSig specification.
//!
//! Parameters:
//! - Tree height: 18
//! - Chains: 155 (TargetSum W=1, no checksum)
//! - Hash: Poseidon2 over KoalaBear (p = 2^31 - 2^24 + 1)
//! - Hash output: 7 field elements
//! - Parameter: 5 field elements
//! - Randomness (rho): 6 field elements

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod koalabear;
pub mod types;
pub mod poseidon;
pub mod encoding;
pub mod verify;

pub use koalabear::KoalaBear;
pub use types::{PublicKey, Signature, VerifyInput};
pub use verify::verify_signature;

/// Message length in bytes (fixed at 32 for leanSig)
pub const MESSAGE_LENGTH: usize = 32;

/// Hash output length in field elements
pub const HASH_LEN: usize = 7;

/// Parameter length in field elements
pub const PARAMETER_LEN: usize = 5;

/// Randomness (rho) length in field elements
pub const RANDOMNESS_LEN: usize = 6;

/// Number of chains (TargetSum W=1)
pub const NUM_CHAINS: usize = 155;

/// Tree height (2^18 = 262,144 signatures per key)
pub const TREE_HEIGHT: usize = 18;

/// Base for chain positions (W=1 means binary: 0 or 1)
pub const BASE: usize = 2;

/// Message hash output length in field elements
pub const MSG_HASH_LEN: usize = 5;

/// Tweak length in field elements
pub const TWEAK_LEN: usize = 2;

/// Message encoding length in field elements
pub const MSG_LEN_FE: usize = 9;

/// Field type used throughout (KoalaBear)
pub type F = KoalaBear;
