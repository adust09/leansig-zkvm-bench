//! Zisk guest program for leanSig XMSS signature verification.
//!
//! This program runs inside the Zisk zkVM and verifies an XMSS signature,
//! producing a zero-knowledge proof of the verification.

#![no_main]

ziskos::entrypoint!(main);

use leansig_minimal::{VerifyInput, verify_signature};

fn main() {
    // Read input from Zisk host
    let input_bytes = ziskos::read_input_slice();

    // Deserialize the verification input
    let input = match VerifyInput::from_bytes(&input_bytes) {
        Ok(input) => input,
        Err(_) => {
            // Output 0 for deserialization failure (output ID 0, value 0)
            ziskos::set_output(0, 0);
            return;
        }
    };

    // Perform signature verification
    let is_valid = verify_signature(
        &input.public_key,
        input.epoch,
        &input.message,
        &input.signature,
    );

    // Output result: output ID 0, value 1 = valid, 0 = invalid
    ziskos::set_output(0, if is_valid { 1 } else { 0 });
}
