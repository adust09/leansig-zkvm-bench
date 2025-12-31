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

// Re-export all constants from shared constants crate
pub use leansig_constants::*;

/// Field type used throughout (KoalaBear)
pub type F = KoalaBear;
