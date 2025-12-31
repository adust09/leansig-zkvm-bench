//! Test vector generator for SP1 verification.
//!
//! This program generates test vectors in the format expected by leansig-minimal.
//! For now, it generates synthetic test data for benchmarking the verification circuit.

use leansig_minimal::types::{PublicKey, Signature, VerifyInput};
use leansig_minimal::{
    HASH_LEN, PARAMETER_LEN, RANDOMNESS_LEN, NUM_CHAINS, TREE_HEIGHT, MESSAGE_LENGTH,
};
use leansig_minimal::KoalaBear;
use std::fs::File;
use std::io::Write;

/// Generate a test VerifyInput for benchmarking.
///
/// This creates synthetic data in the correct format.
/// The signature won't verify correctly, but it exercises the verification code path.
fn generate_test_input() -> VerifyInput {
    // Create mock public key
    let mut root = [KoalaBear::ZERO; HASH_LEN];
    for i in 0..HASH_LEN {
        root[i] = KoalaBear::new(i as u32 + 100);
    }

    let mut parameter = [KoalaBear::ZERO; PARAMETER_LEN];
    for i in 0..PARAMETER_LEN {
        parameter[i] = KoalaBear::new(i as u32 + 200);
    }

    let public_key = PublicKey { root, parameter };

    // Test message
    let message: [u8; MESSAGE_LENGTH] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];

    // Create mock signature
    // Merkle authentication path (TREE_HEIGHT siblings)
    let path: Vec<[KoalaBear; HASH_LEN]> = (0..TREE_HEIGHT)
        .map(|level| {
            let mut hash = [KoalaBear::ZERO; HASH_LEN];
            for i in 0..HASH_LEN {
                hash[i] = KoalaBear::new((level * HASH_LEN + i) as u32 + 1000);
            }
            hash
        })
        .collect();

    // Randomness for encoding (rho) - 6 field elements
    let mut rho = [KoalaBear::ZERO; RANDOMNESS_LEN];
    for i in 0..RANDOMNESS_LEN {
        rho[i] = KoalaBear::new(i as u32 + 300);
    }

    // Hash chain starting points (NUM_CHAINS = 155 chains)
    let hashes: Vec<[KoalaBear; HASH_LEN]> = (0..NUM_CHAINS)
        .map(|chain_idx| {
            let mut hash = [KoalaBear::ZERO; HASH_LEN];
            for i in 0..HASH_LEN {
                hash[i] = KoalaBear::new((chain_idx * HASH_LEN + i) as u32 + 5000);
            }
            hash
        })
        .collect();

    let epoch = 0u32;
    let signature = Signature {
        path,
        rho,
        hashes,
        leaf_index: epoch,
    };

    VerifyInput {
        public_key,
        epoch,
        message,
        signature,
    }
}

fn main() {
    println!("=== leanSig Test Vector Generator (SP1) ===");
    println!("Parameters: TargetSum W=1, {} chains, tree height {}", NUM_CHAINS, TREE_HEIGHT);
    println!("Generating synthetic test input for benchmarking...\n");

    let input = generate_test_input();

    println!("Generated VerifyInput:");
    println!("  - Public key root: {} field elements", HASH_LEN);
    println!("  - Public key parameter: {} field elements", PARAMETER_LEN);
    println!("  - Epoch: {}", input.epoch);
    println!("  - Message length: {} bytes", input.message.len());
    println!("  - Merkle path depth: {} levels", input.signature.path.len());
    println!("  - Randomness (rho): {} field elements", RANDOMNESS_LEN);
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
    assert_eq!(deserialized.signature.path.len(), input.signature.path.len());
    assert_eq!(deserialized.signature.hashes.len(), NUM_CHAINS);
    println!("Roundtrip verification passed!");

    println!("\n=== Done ===");
    println!("Note: This is synthetic test data for benchmarking.");
    println!("      The signature will not verify correctly,");
    println!("      but it exercises the full verification code path.");
}
