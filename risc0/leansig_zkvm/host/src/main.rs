//! RISC Zero host program for leanSig XMSS signature verification
//!
//! This program prepares inputs, runs the prover, and verifies the receipt.

use std::time::Instant;

use leansig_core::{
    Hash, PublicKey, Signature, VerificationInput,
    NUM_CHAINS, TREE_HEIGHT, HASH_LEN, PARAMETER_LEN, RANDOMNESS_LEN, F, FieldArray,
};
use methods::{LEANSIG_VERIFY_ELF, LEANSIG_VERIFY_ID};
use p3_field::PrimeCharacteristicRing;
use risc0_zkvm::{default_prover, ExecutorEnv, ProveInfo};
use serde::Deserialize;

/// Output structure from the guest journal
#[derive(Deserialize, Debug)]
struct VerifyOutput {
    is_valid: bool,
    message_hash: [u8; 32],
    epoch: u32,
}

/// Create test data for signature verification
///
/// This creates synthetic test data with the correct structure for TargetSum W=1.
/// The signature won't verify correctly, but it exercises the verification code path.
fn create_test_data() -> VerificationInput {
    // Create mock public key
    let mut root_arr = [F::ZERO; HASH_LEN];
    for i in 0..HASH_LEN {
        root_arr[i] = F::new(i as u32 + 100);
    }

    let mut param_arr = [F::ZERO; PARAMETER_LEN];
    for i in 0..PARAMETER_LEN {
        param_arr[i] = F::new(i as u32 + 200);
    }

    let public_key = PublicKey {
        root: FieldArray::new(root_arr),
        parameter: FieldArray::new(param_arr),
    };

    // Test message
    let message: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];

    // Create mock signature
    // Merkle authentication path (TREE_HEIGHT siblings)
    let path: Vec<Hash> = (0..TREE_HEIGHT)
        .map(|level| {
            let mut hash_arr = [F::ZERO; HASH_LEN];
            for i in 0..HASH_LEN {
                hash_arr[i] = F::new((level * HASH_LEN + i) as u32 + 1000);
            }
            FieldArray::new(hash_arr)
        })
        .collect();

    // Randomness for encoding (rho) - 6 field elements
    let mut rho_arr = [F::ZERO; RANDOMNESS_LEN];
    for i in 0..RANDOMNESS_LEN {
        rho_arr[i] = F::new(i as u32 + 300);
    }
    let rho = FieldArray::new(rho_arr);

    // Hash chain starting points (NUM_CHAINS = 155 chains)
    let hashes: Vec<Hash> = (0..NUM_CHAINS)
        .map(|chain_idx| {
            let mut hash_arr = [F::ZERO; HASH_LEN];
            for i in 0..HASH_LEN {
                hash_arr[i] = F::new((chain_idx * HASH_LEN + i) as u32 + 5000);
            }
            FieldArray::new(hash_arr)
        })
        .collect();

    let epoch = 0u32;
    let signature = Signature {
        path,
        rho,
        hashes,
        leaf_index: epoch,
    };

    VerificationInput {
        public_key,
        epoch,
        message,
        signature,
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     leanSig XMSS Verification in RISC Zero zkVM              ║");
    println!("║     (TargetSum W=1, {} chains, tree height {})             ║", NUM_CHAINS, TREE_HEIGHT);
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Create test input data
    println!("Creating test signature data...");
    let input = create_test_data();
    println!("  - Public key root: {} field elements", HASH_LEN);
    println!("  - Public key parameter: {} field elements", PARAMETER_LEN);
    println!("  - Epoch: {}", input.epoch);
    println!("  - Message length: {} bytes", input.message.len());
    println!("  - Merkle path depth: {} levels", input.signature.path.len());
    println!("  - Randomness (rho): {} field elements", RANDOMNESS_LEN);
    println!("  - Hash chains: {} chains", input.signature.hashes.len());
    println!();

    // Build the executor environment with our input
    println!("Building executor environment...");
    let env = ExecutorEnv::builder()
        .write(&input)
        .expect("Failed to write input")
        .build()
        .expect("Failed to build executor environment");

    // Get the prover
    println!("Initializing prover...");
    let prover = default_prover();

    // Prove execution
    println!("Starting proof generation...");
    let start = Instant::now();

    let prove_info: ProveInfo = prover
        .prove(env, LEANSIG_VERIFY_ELF)
        .expect("Proving failed");

    let proving_time = start.elapsed();
    println!();
    println!("════════════════════════════════════════════════════════════════");
    println!("                        BENCHMARK RESULTS                        ");
    println!("════════════════════════════════════════════════════════════════");
    println!();
    println!("Execution Statistics:");
    println!("  - Total cycles:    {:>12}", prove_info.stats.total_cycles);
    println!("  - User cycles:     {:>12}", prove_info.stats.user_cycles);
    println!("  - Proving time:    {:>12.2?}", proving_time);
    println!();

    // Extract the receipt
    let receipt = prove_info.receipt;

    // Decode the journal output
    let output: VerifyOutput = receipt
        .journal
        .decode()
        .expect("Failed to decode journal");

    println!("Verification Result:");
    println!("  - Signature valid: {}", output.is_valid);
    println!("  - Epoch:           {}", output.epoch);
    println!("  - Message hash:    0x{}...",
        hex::encode(&output.message_hash[..8]));
    println!();

    // Verify the receipt
    println!("Verifying receipt...");
    let verify_start = Instant::now();
    receipt
        .verify(LEANSIG_VERIFY_ID)
        .expect("Receipt verification failed");
    let verify_time = verify_start.elapsed();
    println!("  - Receipt verified successfully!");
    println!("  - Verification time: {:?}", verify_time);
    println!();

    // Print receipt size
    let receipt_bytes = bincode::serialize(&receipt).expect("Failed to serialize receipt");
    println!("Receipt Size:");
    println!("  - Composite receipt: {} bytes ({:.2} KB)",
        receipt_bytes.len(),
        receipt_bytes.len() as f64 / 1024.0);
    println!();

    println!("════════════════════════════════════════════════════════════════");
    if output.is_valid {
        println!("✓ Signature verification PASSED in zkVM");
    } else {
        println!("✗ Signature verification FAILED (expected with test data)");
    }
    println!("════════════════════════════════════════════════════════════════");
}
