//! SP1 host script for leanSig XMSS verification benchmarking.
//!
//! This program orchestrates proof generation and verification for the
//! leanSig XMSS signature verification guest program.

use clap::Parser;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::time::Instant;

/// The ELF binary for the leanSig verification program.
pub const LEANSIG_ELF: &[u8] = include_elf!("leansig-program");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run execution only (no proof generation)
    #[arg(long)]
    execute: bool,

    /// Generate a proof
    #[arg(long)]
    prove: bool,

    /// Path to input file (postcard serialized VerifyInput)
    #[arg(short, long, default_value = "../data/input.bin")]
    input: String,
}

fn main() {
    // Initialize logging
    sp1_sdk::utils::setup_logger();

    let args = Args::parse();

    // Load input data
    println!("=== SP1 leanSig XMSS Verification Benchmark ===\n");

    let input_path = std::path::Path::new(&args.input);
    if !input_path.exists() {
        eprintln!("Error: Input file not found: {}", args.input);
        eprintln!("Run 'cargo run -p test-gen' first to generate test data.");
        std::process::exit(1);
    }

    let input_bytes = std::fs::read(input_path).expect("Failed to read input file");
    println!("Input file: {}", args.input);
    println!("Input size: {} bytes\n", input_bytes.len());

    // Deserialize to verify it's valid
    let verify_input: leansig_shared::VerifyInput =
        leansig_shared::VerifyInput::from_bytes(&input_bytes)
            .expect("Failed to deserialize input");

    println!("Verification Input:");
    println!("  - Epoch: {}", verify_input.epoch);
    println!("  - Message: {} bytes", verify_input.message.len());
    println!("  - Merkle path depth: {} levels", verify_input.signature.path.len());
    println!("  - Hash chains: {}", verify_input.signature.hashes.len());
    println!();

    // Initialize prover client
    let client = ProverClient::from_env();

    // Prepare stdin
    let mut stdin = SP1Stdin::new();
    stdin.write(&verify_input);

    if args.execute {
        // Execute only (no proof)
        println!("--- Execution Mode (no proof) ---\n");

        let start = Instant::now();
        let (output, report) = client
            .execute(LEANSIG_ELF, &stdin)
            .run()
            .expect("Execution failed");
        let exec_time = start.elapsed();

        // Read the result
        let result: u32 = output.as_slice().first().copied().unwrap_or(0).into();

        println!("Execution Results:");
        println!("  - Total cycles: {}", report.total_instruction_count());
        println!("  - Execution time: {:?}", exec_time);
        println!("  - Verification result: {} ({})",
            result,
            if result == 1 { "VALID" } else { "INVALID (expected for synthetic data)" }
        );
        println!();

        // Print cycle breakdown if available
        println!("Cycle Breakdown:");
        println!("  - Total instruction count: {}", report.total_instruction_count());
    }

    if args.prove {
        // Generate proof
        println!("--- Proving Mode ---\n");

        // Setup
        println!("Setting up proving/verifying keys...");
        let start = Instant::now();
        let (pk, vk) = client.setup(LEANSIG_ELF);
        let setup_time = start.elapsed();
        println!("Setup time: {:?}\n", setup_time);

        // Generate proof
        println!("Generating proof...");
        let start = Instant::now();
        let proof = client
            .prove(&pk, &stdin)
            .compressed()
            .run()
            .expect("Proof generation failed");
        let prove_time = start.elapsed();

        // Serialize proof to get size
        let proof_bytes = bincode::serialize(&proof).expect("Failed to serialize proof");

        println!("Proving Results:");
        println!("  - Proving time: {:?}", prove_time);
        println!("  - Proof size: {} bytes ({:.2} KB)", proof_bytes.len(), proof_bytes.len() as f64 / 1024.0);
        println!();

        // Verify proof
        println!("Verifying proof...");
        let start = Instant::now();
        client
            .verify(&proof, &vk)
            .expect("Proof verification failed");
        let verify_time = start.elapsed();

        println!("  - Verification time: {:?}", verify_time);
        println!();

        // Read public values
        let result: u32 = proof.public_values.as_slice().first().copied().unwrap_or(0).into();
        println!("Public Output:");
        println!("  - Verification result: {} ({})",
            result,
            if result == 1 { "VALID" } else { "INVALID (expected for synthetic data)" }
        );

        println!("\n=== Benchmark Complete ===");
        println!("Summary:");
        println!("  - Setup: {:?}", setup_time);
        println!("  - Proving: {:?}", prove_time);
        println!("  - Verification: {:?}", verify_time);
        println!("  - Total: {:?}", setup_time + prove_time + verify_time);
    }

    if !args.execute && !args.prove {
        println!("No action specified. Use --execute or --prove.");
        println!("  --execute  Run execution only (measure cycles, no proof)");
        println!("  --prove    Generate and verify a compressed proof");
    }
}
