//! leanSig Core Types for RISC Zero
//!
//! This crate provides no_std compatible types for XMSS signature verification.
//! It is designed to be used in both the RISC Zero guest (no_std) and host (std).

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod field;
pub mod poseidon;
pub mod tweak_hash;
pub mod types;
pub mod verify;

// Re-export main types for convenience
pub use types::{EncodingRandomness, Hash, MerklePath, Parameter, PublicKey, Signature, VerificationInput};
pub use verify::verify_signature;
