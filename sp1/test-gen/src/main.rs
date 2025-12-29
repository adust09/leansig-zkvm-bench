//! Test vector generator for SP1 verification.
//!
//! This program generates test vectors in the format expected by leansig-minimal.
//! For now, it generates synthetic test data for benchmarking the verification circuit.

use leansig_minimal::types::{PublicKey, Signature, VerifyInput};
use leansig_minimal::poseidon::{HASH_LEN, PARAMETER_LEN};
use leansig_minimal::encoding::DIMENSION;
use leansig_minimal::MESSAGE_LENGTH;
use leansig_minimal::KoalaBear;
use std::fs::File;
use std::io::Write;

/// Generate a test VerifyInput for benchmarking.
///
/// This creates synthetic data in the correct format.
/// The signature won't verify correctly, but it exercises the verification code path.
fn generate_test_input() -> VerifyInput {
    // Create mock public key
    let public_key = PublicKey {
        root: (0..HASH_LEN).map(|i| KoalaBear::new(i as u32 + 100)).collect(),
        parameter: (0..PARAMETER_LEN).map(|i| KoalaBear::new(i as u32 + 200)).collect(),
    };

    // Test message
    let message: [u8; MESSAGE_LENGTH] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];

    // Create mock signature
    // For a real signature, this would come from leanSig
    let tree_height = 18; // LOG_LIFETIME for lifetime 2^18
    let path: Vec<Vec<_>> = (0..tree_height)
        .map(|level| {
            (0..HASH_LEN)
                .map(|i| KoalaBear::new((level * HASH_LEN + i) as u32 + 1000))
                .collect()
        })
        .collect();

    // Randomness for encoding (rho)
    let rho: Vec<u8> = (0..32).map(|i| i as u8).collect();

    // Hash chain starting points (one per dimension)
    let hashes: Vec<Vec<_>> = (0..DIMENSION)
        .map(|chain_idx| {
            (0..HASH_LEN)
                .map(|i| KoalaBear::new((chain_idx * HASH_LEN + i) as u32 + 5000))
                .collect()
        })
        .collect();

    let signature = Signature { path, rho, hashes };

    VerifyInput {
        public_key,
        epoch: 0,
        message,
        signature,
    }
}

fn main() {
    println!("=== leanSig Test Vector Generator (SP1) ===");
    println!("Generating synthetic test input for benchmarking...\n");

    let input = generate_test_input();

    println!("Generated VerifyInput:");
    println!("  - Public key root: {} field elements", input.public_key.root.len());
    println!("  - Public key parameter: {} field elements", input.public_key.parameter.len());
    println!("  - Epoch: {}", input.epoch);
    println!("  - Message length: {} bytes", input.message.len());
    println!("  - Merkle path depth: {} levels", input.signature.path.len());
    println!("  - Randomness (rho): {} bytes", input.signature.rho.len());
    println!("  - Hash chains: {} chains", input.signature.hashes.len());

    // Serialize using postcard
    println!("\nSerializing with postcard...");
    let serialized = input.to_bytes().expect("Serialization failed");
    println!("Serialized size: {} bytes", serialized.len());

    // Create output directory
    std::fs::create_dir_all("../data").expect("Failed to create data directory");

    // Write to file
    let mut file = File::create("../data/input.bin").expect("Failed to create input.bin");
    file.write_all(&serialized).expect("Failed to write input.bin");

    println!("\nTest vector written to data/input.bin");

    // Verify roundtrip
    println!("\nVerifying roundtrip deserialization...");
    let deserialized = VerifyInput::from_bytes(&serialized).expect("Deserialization failed");
    assert_eq!(deserialized.epoch, input.epoch);
    assert_eq!(deserialized.message, input.message);
    assert_eq!(deserialized.public_key.root.len(), input.public_key.root.len());
    assert_eq!(deserialized.signature.path.len(), input.signature.path.len());
    println!("Roundtrip verification passed!");

    println!("\n=== Done ===");
    println!("Note: This is synthetic test data for benchmarking.");
    println!("      The signature will not verify correctly,");
    println!("      but it exercises the full verification code path.");
}
