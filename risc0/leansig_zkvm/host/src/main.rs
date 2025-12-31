//! RISC Zero host program for leanSig XMSS signature verification
//!
//! This program prepares inputs, runs the prover, and verifies the receipt.

use std::time::Instant;

use leansig_core::{
    Hash, PublicKey, Signature, VerificationInput, FieldArray,
    NUM_CHAINS, TREE_HEIGHT, HASH_LEN, PARAMETER_LEN, RANDOMNESS_LEN, F,
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

/// Convert shared library's VerifyInput to RISC Zero's VerificationInput.
///
/// The shared input.bin uses the shared library's types (raw arrays),
/// while RISC Zero uses FieldArray wrappers. This function performs the conversion.
fn convert_shared_to_risc0(shared: leansig_shared::VerifyInput) -> VerificationInput {
    // Convert public key
    let mut root_arr = [F::ZERO; HASH_LEN];
    for (i, &val) in shared.public_key.root.iter().enumerate() {
        root_arr[i] = F::new(val.value());
    }

    let mut param_arr = [F::ZERO; PARAMETER_LEN];
    for (i, &val) in shared.public_key.parameter.iter().enumerate() {
        param_arr[i] = F::new(val.value());
    }

    let public_key = PublicKey {
        root: FieldArray::new(root_arr),
        parameter: FieldArray::new(param_arr),
    };

    // Convert signature path
    let path: Vec<Hash> = shared.signature.path.iter().map(|hash| {
        let mut arr = [F::ZERO; HASH_LEN];
        for (i, &val) in hash.iter().enumerate() {
            arr[i] = F::new(val.value());
        }
        FieldArray::new(arr)
    }).collect();

    // Convert rho
    let mut rho_arr = [F::ZERO; RANDOMNESS_LEN];
    for (i, &val) in shared.signature.rho.iter().enumerate() {
        rho_arr[i] = F::new(val.value());
    }

    // Convert hashes
    let hashes: Vec<Hash> = shared.signature.hashes.iter().map(|hash| {
        let mut arr = [F::ZERO; HASH_LEN];
        for (i, &val) in hash.iter().enumerate() {
            arr[i] = F::new(val.value());
        }
        FieldArray::new(arr)
    }).collect();

    let signature = Signature {
        path,
        rho: FieldArray::new(rho_arr),
        hashes,
        leaf_index: shared.signature.leaf_index,
    };

    VerificationInput {
        public_key,
        epoch: shared.epoch,
        message: shared.message,
        signature,
    }
}

/// Load test data from the shared input.bin file.
///
/// This reads the pre-generated test data that is shared across all zkVM benchmarks,
/// ensuring fair comparison of performance metrics.
fn load_test_data(path: &str) -> VerificationInput {
    let bytes = std::fs::read(path).expect("Failed to read input file");
    let shared_input = leansig_shared::VerifyInput::from_bytes(&bytes)
        .expect("Failed to deserialize input data");
    convert_shared_to_risc0(shared_input)
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

    // Load test input data from shared file
    let input_path = "../../data/input.bin";
    println!("Loading test signature data from {}...", input_path);
    let input = load_test_data(input_path);
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
