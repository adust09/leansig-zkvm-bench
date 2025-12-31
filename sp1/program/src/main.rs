//! SP1 guest program for leanSig XMSS signature verification.
//!
//! This program runs inside the SP1 zkVM and verifies an XMSS signature,
//! producing a zero-knowledge proof of the verification.

#![no_main]
sp1_zkvm::entrypoint!(main);

use leansig_shared::{VerifyInput, verify_signature};

pub fn main() {
    // Read the verification input from the host
    let input: VerifyInput = sp1_zkvm::io::read();

    // Perform signature verification
    let is_valid = verify_signature(
        &input.public_key,
        input.epoch,
        &input.message,
        &input.signature,
    );

    // Commit the verification result as public output
    // 1 = valid, 0 = invalid
    sp1_zkvm::io::commit(&(if is_valid { 1u32 } else { 0u32 }));

    // Also commit the message hash for binding
    sp1_zkvm::io::commit(&input.message);
}
