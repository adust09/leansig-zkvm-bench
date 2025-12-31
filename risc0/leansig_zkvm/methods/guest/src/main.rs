//! RISC Zero guest program for leanSig XMSS signature verification
//!
//! This program runs inside the zkVM and verifies an XMSS signature,
//! producing a proof that the verification was performed correctly.

#![no_main]
#![no_std]

use leansig_core::{VerificationInput, verify_signature};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

/// Output structure committed to the journal
#[derive(serde::Serialize)]
struct VerifyOutput {
    /// Whether the signature is valid
    is_valid: bool,
    /// Hash of the message (for binding)
    message_hash: [u8; 32],
    /// Epoch of the verified signature
    epoch: u32,
}

fn main() {
    // Read the verification input from the host
    let input: VerificationInput = env::read();

    // Perform XMSS signature verification (TargetSum W=1, 155 chains)
    let is_valid = verify_signature(
        &input.public_key,
        input.epoch,
        &input.message,
        &input.signature,
    );

    // Commit the verification result to the public journal
    let output = VerifyOutput {
        is_valid,
        message_hash: input.message,
        epoch: input.epoch,
    };

    env::commit(&output);
}
