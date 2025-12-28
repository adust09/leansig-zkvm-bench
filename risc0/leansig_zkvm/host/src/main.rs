//! RISC Zero host program for leanSig XMSS signature verification
//!
//! This program prepares inputs, runs the prover, and verifies the receipt.

use std::time::Instant;

use leansig_core::{EncodingRandomness, Hash, MerklePath, Parameter, PublicKey, Signature};
use methods::{LEANSIG_VERIFY_ELF, LEANSIG_VERIFY_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProveInfo};
use serde::{Deserialize, Serialize};

/// Input structure for signature verification (mirrors guest)
#[derive(Serialize)]
struct VerifyInput {
    public_key: PublicKey,
    epoch: u32,
    message: [u8; 32],
    signature: Signature,
    chain_length: usize,
}

/// Output structure from the guest journal
#[derive(Deserialize, Debug)]
struct VerifyOutput {
    is_valid: bool,
    message_hash: [u8; 32],
    epoch: u32,
}

/// Create test data for signature verification
///
/// In a real application, this would come from actual leanSig key generation
/// and signing. For now, we create minimal test data.
fn create_test_data() -> VerifyInput {
    // Create a simple public key with default parameter and a test root
    let public_key = PublicKey {
        parameter: Parameter::default(),
        root: Hash::default(),
    };

    // Test message
    let message = [0x42u8; 32];

    // Create a minimal signature with empty authentication path
    // This will fail verification, but allows us to measure performance
    let signature = Signature {
        path: MerklePath {
            co_path: vec![], // Empty path for leaf at root
        },
        rho: EncodingRandomness::default(),
        hashes: vec![Hash::default(); 32], // 32 chains
    };

    VerifyInput {
        public_key,
        epoch: 0,
        message,
        signature,
        chain_length: 16, // Standard chain length
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     leanSig XMSS Verification in RISC Zero zkVM              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Create test input data
    println!("Creating test signature data...");
    let input = create_test_data();
    println!("  - Message: 0x42424242... (32 bytes)");
    println!("  - Epoch: {}", input.epoch);
    println!("  - Chain length: {}", input.chain_length);
    println!("  - Number of chains: {}", input.signature.hashes.len());
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
