//! Minimal no_std implementation of leanSig XMSS signature verification.
//!
//! This crate extracts only the verification-related code from leanSig,
//! making it suitable for use in zkVM environments like Zisk.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod types;
pub mod poseidon;
pub mod encoding;
pub mod verify;

pub use types::{PublicKey, Signature, VerifyInput};
pub use verify::verify_signature;

/// Message length in bytes (fixed at 32 for leanSig)
pub const MESSAGE_LENGTH: usize = 32;

/// Field type used throughout (KoalaBear from Plonky3)
pub type F = p3_koala_bear::KoalaBear;
