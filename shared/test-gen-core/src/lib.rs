//! Shared test vector generation for leanSig XMSS verification benchmarks.
//!
//! This library generates synthetic test data in the format expected by leansig-core.
//! The generated signature won't verify correctly, but it exercises the full verification code path.

use leansig_shared::types::{PublicKey, Signature, VerifyInput};
use leansig_shared::{
    HASH_LEN, PARAMETER_LEN, RANDOMNESS_LEN, NUM_CHAINS, TREE_HEIGHT, MESSAGE_LENGTH,
};
use leansig_shared::KoalaBear;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Generate a test VerifyInput for benchmarking.
///
/// This creates synthetic data in the correct format.
/// The signature won't verify correctly, but it exercises the verification code path.
pub fn generate_test_input() -> VerifyInput {
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

/// Generate and write test vector to the specified output path.
///
/// Returns the serialized bytes on success.
pub fn generate_and_write<P: AsRef<Path>>(output_path: P) -> std::io::Result<Vec<u8>> {
    let input = generate_test_input();

    // Serialize using postcard
    let serialized = input.to_bytes().expect("Serialization failed");

    // Create output directory if needed
    if let Some(parent) = output_path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write to file
    let mut file = File::create(&output_path)?;
    file.write_all(&serialized)?;

    Ok(serialized)
}

/// Print generation summary to stdout.
pub fn print_summary(input: &VerifyInput, serialized_size: usize) {
    println!("Generated VerifyInput:");
    println!("  - Public key root: {} field elements", HASH_LEN);
    println!("  - Public key parameter: {} field elements", PARAMETER_LEN);
    println!("  - Epoch: {}", input.epoch);
    println!("  - Message length: {} bytes", input.message.len());
    println!("  - Merkle path depth: {} levels", input.signature.path.len());
    println!("  - Randomness (rho): {} field elements", RANDOMNESS_LEN);
    println!("  - Hash chains: {} chains", input.signature.hashes.len());
    println!("\nSerialized size: {} bytes", serialized_size);
}

/// Verify roundtrip serialization.
pub fn verify_roundtrip(original: &VerifyInput, serialized: &[u8]) -> bool {
    let deserialized = match VerifyInput::from_bytes(serialized) {
        Ok(d) => d,
        Err(_) => return false,
    };

    deserialized.epoch == original.epoch
        && deserialized.message == original.message
        && deserialized.signature.path.len() == original.signature.path.len()
        && deserialized.signature.hashes.len() == NUM_CHAINS
}

