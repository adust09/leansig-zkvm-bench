//! Minimal no_std implementation of leanSig XMSS signature verification.
//!
//! This crate extracts only the verification-related code from leanSig,
//! making it suitable for use in zkVM environments like SP1.
//!
//! Note: This version uses a custom KoalaBear implementation since SP1's
//! Plonky3 fork does not include p3-koala-bear.

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

/// Field type used throughout (KoalaBear)
pub type F = KoalaBear;
